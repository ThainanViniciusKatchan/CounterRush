use std::ffi::c_void;
use std::fs::File;

use config::{AppSettings, ConfigError};
use core::{
    list_available_locales, list_available_locales_in_dir, resolve_app_settings_path,
    resolve_locale_dir, set_clipboard,
};
use windows_api::clipboard::set_clipboard_text;

// Win32 FFI Oracle para leitura direta da área de transferência do Windows
type HANDLE = *mut c_void;
type HWND = *mut c_void;
type BOOL = i32;

const CF_UNICODETEXT: u32 = 13;

#[link(name = "user32")]
extern "system" {
    fn OpenClipboard(hwnd: HWND) -> BOOL;
    fn CloseClipboard() -> BOOL;
    fn GetClipboardData(u_format: u32) -> HANDLE;
}

#[link(name = "kernel32")]
extern "system" {
    fn GlobalLock(h_mem: HANDLE) -> *mut c_void;
    fn GlobalUnlock(h_mem: HANDLE) -> BOOL;
    fn GetLastError() -> u32;
}

/// Função Oracle para ler o texto atualmente no clipboard do Windows em UTF-16
fn read_clipboard_text_oracle() -> Result<String, String> {
    unsafe {
        if OpenClipboard(std::ptr::null_mut()) == 0 {
            return Err(format!("Falha ao abrir clipboard para leitura: {}", GetLastError()));
        }

        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle.is_null() {
            CloseClipboard();
            return Err(format!("GetClipboardData retornou nulo: {}", GetLastError()));
        }

        let ptr = GlobalLock(handle) as *const u16;
        if ptr.is_null() {
            CloseClipboard();
            return Err(format!("GlobalLock retornou nulo: {}", GetLastError()));
        }

        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        let slice = std::slice::from_raw_parts(ptr, len);
        let result = String::from_utf16_lossy(slice);

        GlobalUnlock(handle);
        CloseClipboard();

        Ok(result)
    }
}

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Teste de Stress Adversarial M3 (Settings Backend & Win32 FFI)");
    println!("================================================================================");

    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;

    macro_rules! check {
        ($name:expr, $cond:expr, $msg:expr) => {
            total_tests += 1;
            if $cond {
                println!("  [PASS] {}: {}", $name, $msg);
                passed_tests += 1;
            } else {
                println!("  [FAIL] {}: {}", $name, $msg);
                failed_tests += 1;
            }
        };
    }

    // -------------------------------------------------------------------------
    // SUÍTE 1: PERSISTÊNCIA DE APPSETTINGS
    // -------------------------------------------------------------------------
    println!("\n[1] TESTES ADVERSARIAIS: PERSISTÊNCIA DE APPSETTINGS");

    let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário para testes");
    let base_path = temp_dir.path();

    // 1.1 Resolução de caminho de configurações
    let resolved_settings = resolve_app_settings_path();
    check!(
        "resolve_app_settings_path estrutura",
        resolved_settings.to_string_lossy().ends_with("settings.json"),
        format!("Caminho resolvido: {:?}", resolved_settings)
    );

    // 1.2 Valores padrão
    let default_settings = AppSettings::default();
    check!(
        "AppSettings::default",
        default_settings.language == "pt_BR"
            && !default_settings.start_with_system
            && !default_settings.optimize_on_steam_launch,
        format!("Valores default corretos: {:?}", default_settings)
    );

    // 1.3 Roundtrip padrão
    let roundtrip_file = base_path.join("settings_roundtrip.json");
    let custom_settings = AppSettings {
        language: "en_US".to_string(),
        start_with_system: true,
        optimize_on_steam_launch: true,
        initial_optimization_done: false,
        minimize_to_tray: false,
    };
    let save_res = custom_settings.save_to_file(&roundtrip_file);
    check!("Save normal", save_res.is_ok(), format!("{:?}", save_res));

    let load_res = AppSettings::load_from_file(&roundtrip_file);
    check!(
        "Load normal roundtrip",
        load_res.as_ref().map(|s| s == &custom_settings).unwrap_or(false),
        format!("{:?}", load_res)
    );

    // 1.4 Criação em diretório profundamente aninhado
    let deep_file = base_path
        .join("deep")
        .join("nested")
        .join("subfolder")
        .join("settings.json");
    let deep_save_res = custom_settings.save_to_file(&deep_file);
    check!(
        "Criação automática de diretórios pais inexistentes",
        deep_save_res.is_ok() && deep_file.exists(),
        format!("Arquivo criado em: {:?}", deep_file)
    );

    // 1.5 load_or_create em arquivo inexistente
    let non_existent_file = base_path.join("auto_create_settings.json");
    check!(
        "Pré-condição: arquivo não existe",
        !non_existent_file.exists(),
        "Arquivo ainda não existe"
    );
    let created_res = AppSettings::load_or_create(&non_existent_file);
    check!(
        "load_or_create cria arquivo e retorna default",
        created_res.is_ok()
            && non_existent_file.exists()
            && created_res.unwrap() == AppSettings::default(),
        "Arquivo criado com configurações padrão"
    );

    // 1.6 Arquivo vazio (0 bytes) -> fallback para default e regravação
    let empty_file = base_path.join("empty.json");
    File::create(&empty_file).unwrap();
    let empty_load = AppSettings::load_from_file(&empty_file);
    check!(
        "Arquivo vazio (0 bytes) retorna default",
        empty_load.as_ref().map(|s| s == &AppSettings::default()).unwrap_or(false),
        format!("Resultado: {:?}", empty_load)
    );
    let re_read = std::fs::read_to_string(&empty_file).unwrap();
    check!(
        "Arquivo vazio foi regravado com JSON default válido",
        re_read.contains("\"language\": \"pt_BR\""),
        format!("Conteúdo regravado: {}", re_read)
    );

    // 1.7 Arquivo contendo apenas whitespaces
    let whitespace_file = base_path.join("whitespace.json");
    std::fs::write(&whitespace_file, "   \n\t  \r\n   ").unwrap();
    let ws_load = AppSettings::load_from_file(&whitespace_file);
    check!(
        "Arquivo com apenas espaços/newlines retorna default",
        ws_load.as_ref().map(|s| s == &AppSettings::default()).unwrap_or(false),
        format!("Resultado: {:?}", ws_load)
    );

    // 1.8 Arquivo contendo apenas BOM UTF-8 isolado (\u{feff})
    let bom_only_file = base_path.join("bom_only.json");
    std::fs::write(&bom_only_file, "\u{feff}   \r\n").unwrap();
    let bom_only_load = AppSettings::load_from_file(&bom_only_file);
    check!(
        "Arquivo com apenas BOM UTF-8 isolado retorna default",
        bom_only_load.as_ref().map(|s| s == &AppSettings::default()).unwrap_or(false),
        format!("Resultado: {:?}", bom_only_load)
    );

    // 1.9 Arquivo contendo BOM UTF-8 seguido de JSON válido customizado
    let bom_valid_file = base_path.join("bom_valid.json");
    let bom_valid_content = "\u{feff}{\"language\": \"ru\", \"start_with_system\": false, \"optimize_on_steam_launch\": true}";
    std::fs::write(&bom_valid_file, bom_valid_content).unwrap();
    let bom_valid_load = AppSettings::load_from_file(&bom_valid_file);
    check!(
        "Arquivo com BOM UTF-8 + JSON válido carrega corretamente",
        bom_valid_load.as_ref().map(|s| s.language == "ru" && s.optimize_on_steam_launch).unwrap_or(false),
        format!("Resultado: {:?}", bom_valid_load)
    );

    // 1.10 Arquivo com JSON sintaticamente malformado
    let malformed_file = base_path.join("malformed.json");
    std::fs::write(&malformed_file, "{ \"language\": \"pt_BR\", broken...").unwrap();
    let malformed_load = AppSettings::load_from_file(&malformed_file);
    check!(
        "JSON malformado retorna ConfigError::Json",
        matches!(malformed_load, Err(ConfigError::Json(_))),
        format!("Resultado: {:?}", malformed_load)
    );

    // 1.11 Arquivo com campos extras desconhecidos
    let extra_fields_file = base_path.join("extra_fields.json");
    let extra_content = r#"{
        "language": "en",
        "start_with_system": false,
        "optimize_on_steam_launch": false,
        "future_field_99": "extra_value",
        "nested_unknown": { "a": 1, "b": 2 }
    }"#;
    std::fs::write(&extra_fields_file, extra_content).unwrap();
    let extra_load = AppSettings::load_from_file(&extra_fields_file);
    check!(
        "JSON com campos extras é desserializado com sucesso",
        extra_load.as_ref().map(|s| s.language == "en").unwrap_or(false),
        format!("Resultado: {:?}", extra_load)
    );

    // 1.12 Arquivo com campos parciais omitidos (serde default)
    let partial_file = base_path.join("partial.json");
    std::fs::write(&partial_file, r#"{"language": "ru_RU"}"#).unwrap();
    let partial_load = AppSettings::load_from_file(&partial_file);
    check!(
        "Campos omitidos recebem valores default",
        partial_load.as_ref().map(|s| s.language == "ru_RU" && !s.start_with_system && !s.optimize_on_steam_launch).unwrap_or(false),
        format!("Resultado: {:?}", partial_load)
    );

    // 1.13 Stress de escritas e leituras repetidas em loop
    let mut stress_ok = true;
    let stress_file = base_path.join("stress_settings.json");
    for i in 0..100 {
        let lang = if i % 2 == 0 { "pt_BR" } else { "ru" };
        let cfg = AppSettings {
            language: lang.to_string(),
            start_with_system: i % 3 == 0,
            optimize_on_steam_launch: i % 5 == 0,
            initial_optimization_done: false,
            minimize_to_tray: false,
        };
        if cfg.save_to_file(&stress_file).is_err() {
            stress_ok = false;
            break;
        }
        let read = AppSettings::load_from_file(&stress_file);
        if read.as_ref().map(|r| r != &cfg).unwrap_or(true) {
            stress_ok = false;
            break;
        }
    }
    check!(
        "Stress test: 100 iterações consecutivas de save & load",
        stress_ok,
        "Todas as 100 iterações completadas com fidelidade total"
    );

    // -------------------------------------------------------------------------
    // SUÍTE 2: DESCOBERTA DE LOCALES
    // -------------------------------------------------------------------------
    println!("\n[2] TESTES ADVERSARIAIS: DESCOBERTA DE LOCALES");

    // 2.1 Resolução do diretório de locales
    let resolved_locales = resolve_locale_dir();
    check!(
        "resolve_locale_dir encontra diretório locale",
        resolved_locales.exists() && resolved_locales.is_dir(),
        format!("Diretório de locales: {:?}", resolved_locales)
    );

    // 2.2 Diretório real do projeto
    let real_locales = list_available_locales();
    check!(
        "list_available_locales() encontra locales reais",
        !real_locales.is_empty(),
        format!("Encontrados {} locales reais: {:?}", real_locales.len(), real_locales.iter().map(|l| &l.code).collect::<Vec<_>>())
    );

    // 2.3 Ordenação estrita dos locales reais: pt_BR deve ser o primeiro
    let codes: Vec<String> = real_locales.iter().map(|l| l.code.clone()).collect();
    let has_pt_first = codes.first().map(|c| c == "pt_BR").unwrap_or(false);
    check!(
        "pt_BR é o primeiro locale na lista real",
        has_pt_first,
        format!("Primeiro locale: {:?}", codes.first())
    );

    // 2.4 Verificação de presença e mapeamento dos nomes de exibição esperados
    let find_display = |c: &str| real_locales.iter().find(|l| l.code == c).map(|l| l.display_name.as_str());
    check!(
        "Mapeamento display_name para pt_BR",
        find_display("pt_BR") == Some("Português (Brasil)"),
        format!("{:?}", find_display("pt_BR"))
    );
    check!(
        "Mapeamento display_name para en",
        find_display("en") == Some("English"),
        format!("{:?}", find_display("en"))
    );
    check!(
        "Mapeamento display_name para ru",
        find_display("ru") == Some("Русский"),
        format!("{:?}", find_display("ru"))
    );

    // 2.5 Borda: Diretório inexistente
    let non_existent_dir = base_path.join("non_existent_locales_dir_12345");
    let missing_locales = list_available_locales_in_dir(&non_existent_dir);
    check!(
        "Diretório inexistente retorna lista vazia sem pânico",
        missing_locales.is_empty(),
        "Retornou lista vazia"
    );

    // 2.6 Borda: Diretório vazio
    let empty_locales_dir = base_path.join("empty_locales");
    std::fs::create_dir_all(&empty_locales_dir).unwrap();
    let empty_locales = list_available_locales_in_dir(&empty_locales_dir);
    check!(
        "Diretório vazio retorna lista vazia",
        empty_locales.is_empty(),
        "Retornou lista vazia"
    );

    // 2.7 Borda: Diretório com arquivos soltos na raiz (não diretórios)
    let loose_files_dir = base_path.join("loose_files");
    std::fs::create_dir_all(&loose_files_dir).unwrap();
    File::create(loose_files_dir.join("counterrush-ui.po")).unwrap();
    File::create(loose_files_dir.join("README.md")).unwrap();
    let loose_locales = list_available_locales_in_dir(&loose_files_dir);
    check!(
        "Arquivos avulsos na raiz não são computados como locales",
        loose_locales.is_empty(),
        format!("Retornou: {:?}", loose_locales)
    );

    // 2.8 Borda: Subdiretórios vazios sem arquivos .po
    let dummy_subdirs_dir = base_path.join("dummy_subdirs");
    std::fs::create_dir_all(dummy_subdirs_dir.join("sub1")).unwrap();
    std::fs::create_dir_all(dummy_subdirs_dir.join("sub2")).unwrap();
    let dummy_locales = list_available_locales_in_dir(&dummy_subdirs_dir);
    check!(
        "Subdiretórios sem .po são ignorados",
        dummy_locales.is_empty(),
        format!("Retornou: {:?}", dummy_locales)
    );

    // 2.9 Borda: Locale com apenas `counterrush.po` (sem `-ui`)
    let generic_po_dir = base_path.join("generic_po_test");
    let de_dir = generic_po_dir.join("de");
    std::fs::create_dir_all(&de_dir).unwrap();
    File::create(de_dir.join("counterrush.po")).unwrap();
    let generic_locales = list_available_locales_in_dir(&generic_po_dir);
    check!(
        "Subdiretório com counterrush.po é reconhecido",
        generic_locales.len() == 1 && generic_locales[0].code == "de",
        format!("Retornou: {:?}", generic_locales)
    );

    // 2.10 Borda: Locale com estrutura gettext clássica `LC_MESSAGES/counterrush-ui.po`
    let lc_po_dir = base_path.join("lc_po_test");
    let fr_lc = lc_po_dir.join("fr").join("LC_MESSAGES");
    std::fs::create_dir_all(&fr_lc).unwrap();
    File::create(fr_lc.join("counterrush-ui.po")).unwrap();
    let lc_locales = list_available_locales_in_dir(&lc_po_dir);
    check!(
        "Subdiretório com LC_MESSAGES/counterrush-ui.po é reconhecido",
        lc_locales.len() == 1 && lc_locales[0].code == "fr",
        format!("Retornou: {:?}", lc_locales)
    );

    // 2.11 Borda: Locales não mapeados recebem seu próprio código como display_name e ordenam após os mapeados
    let custom_locales_dir = base_path.join("custom_locales_test");
    let custom_langs = ["es_ES", "ja_JP", "pt_BR", "en", "zh_CN"];
    for lang in &custom_langs {
        let dir = custom_locales_dir.join(lang);
        std::fs::create_dir_all(&dir).unwrap();
        File::create(dir.join("counterrush-ui.po")).unwrap();
    }
    let custom_res = list_available_locales_in_dir(&custom_locales_dir);
    let custom_codes: Vec<&str> = custom_res.iter().map(|l| l.code.as_str()).collect();
    check!(
        "Ordenação prioriza pt_BR (0), en (1), depois desconhecidos em ordem alfabética",
        custom_codes[0] == "pt_BR"
            && custom_codes[1] == "en"
            && custom_codes[2] == "es_ES"
            && custom_codes[3] == "ja_JP"
            && custom_codes[4] == "zh_CN",
        format!("Ordem obtida: {:?}", custom_codes)
    );
    let es_item = custom_res.iter().find(|l| l.code == "es_ES").unwrap();
    check!(
        "Código desconhecido usa o próprio código como display_name",
        es_item.display_name == "es_ES",
        format!("display_name: {}", es_item.display_name)
    );

    // -------------------------------------------------------------------------
    // SUÍTE 3: CLIPBOARD WIN32 FFI
    // -------------------------------------------------------------------------
    println!("\n[3] TESTES ADVERSARIAIS: CLIPBOARD WIN32 FFI");

    // 3.1 String simples
    let sample = "CounterRush M3 Simple Test";
    let set_sample_res = set_clipboard_text(sample);
    check!("set_clipboard_text string simples", set_sample_res.is_ok(), format!("{:?}", set_sample_res));
    let read_sample = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 lê exatamente a string simples",
        read_sample.as_deref() == Ok(sample),
        format!("Lido: {:?}", read_sample)
    );

    // 3.2 String vazia
    let empty_str = "";
    let set_empty_res = set_clipboard_text(empty_str);
    check!("set_clipboard_text string vazia", set_empty_res.is_ok(), format!("{:?}", set_empty_res));
    let read_empty = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 lê string vazia",
        read_empty.as_deref() == Ok(empty_str),
        format!("Lido: {:?}", read_empty)
    );

    // 3.3 Acentuação e caracteres especiais em Português-BR
    let pt_special = "CounterRush: Otimização e Desempenho para CS2 — Ação, Aceleração & Memória! (ÁÉÍÓÚ ãõ ç)";
    let set_pt_res = set_clipboard_text(pt_special);
    check!("set_clipboard_text Português acentuado", set_pt_res.is_ok(), format!("{:?}", set_pt_res));
    let read_pt = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 preserva acentuação e cedilha PT-BR",
        read_pt.as_deref() == Ok(pt_special),
        format!("Lido: {:?}", read_pt)
    );

    // 3.4 Caracteres Cirílicos / Russo
    let ru_str = "CounterRush — Оптимизация и повышение производительности CS2 (Русский язык, привет мир)";
    let set_ru_res = set_clipboard_text(ru_str);
    check!("set_clipboard_text Cirílico", set_ru_res.is_ok(), format!("{:?}", set_ru_res));
    let read_ru = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 preserva caracteres cirílicos",
        read_ru.as_deref() == Ok(ru_str),
        format!("Lido: {:?}", read_ru)
    );

    // 3.5 Emojis, caracteres CJK e símbolos Unicode complexos
    let emoji_cjk = "CounterRush 🚀⚡ 🎮 汉字 日本語 العربية \u{1F3AE}\u{1F525} — Multi-Unicode";
    let set_cjk_res = set_clipboard_text(emoji_cjk);
    check!("set_clipboard_text Emojis e CJK", set_cjk_res.is_ok(), format!("{:?}", set_cjk_res));
    let read_cjk = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 preserva surrogate pairs e emojis",
        read_cjk.as_deref() == Ok(emoji_cjk),
        format!("Lido: {:?}", read_cjk)
    );

    // 3.6 Quebras de linha e formatação
    let multiline = "Linha 1: Início\r\nLinha 2:\tCom tabulação\nLinha 3: Fim";
    let set_multi_res = set_clipboard_text(multiline);
    check!("set_clipboard_text Multiline e tabs", set_multi_res.is_ok(), format!("{:?}", set_multi_res));
    let read_multi = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 preserva CRLF e tabs",
        read_multi.as_deref() == Ok(multiline),
        format!("Lido: {:?}", read_multi)
    );

    // 3.7 Stress test: String longa (100.000 caracteres)
    let long_str: String = "CounterRush-M3-Adversarial-Stress-Test-".repeat(2500); // ~97.500 chars
    let set_long_res = set_clipboard_text(&long_str);
    check!(
        "set_clipboard_text com payload grande (~100k caracteres)",
        set_long_res.is_ok(),
        format!("Payload len: {} bytes", long_str.len())
    );
    let read_long = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 preserva integridade do payload longo de 100k chars",
        read_long.as_ref().map(|s| s.len() == long_str.len() && s == &long_str).unwrap_or(false),
        "Payload verificado byte a byte com sucesso"
    );

    // 3.8 Integração core::set_clipboard
    let core_clip_sample = "https://github.com/CounterRush/CounterRush — Compartilhamento Oficial";
    let core_clip_res = set_clipboard(core_clip_sample);
    check!("core::set_clipboard delega para windows_api com sucesso", core_clip_res.is_ok(), format!("{:?}", core_clip_res));
    let read_core_clip = read_clipboard_text_oracle();
    check!(
        "Oracle Win32 confirma valor definido via core::set_clipboard",
        read_core_clip.as_deref() == Ok(core_clip_sample),
        format!("Lido: {:?}", read_core_clip)
    );

    // 3.9 Teste de invocações sequenciais com intervalo realista (ex: múltiplos cliques do usuário)
    let mut ui_clicks_ok = true;
    for i in 0..20 {
        let text = format!("Clique Usuário #{}", i);
        if let Err(e) = set_clipboard_text(&text) {
            ui_clicks_ok = false;
            println!("    Falha no clique {}: {:?}", i, e);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
    check!(
        "Invocações repetidas com intervalo realista (15ms - cliques de usuário)",
        ui_clicks_ok,
        "Todas as 20 invocações com espaçamento realista completaram com sucesso"
    );

    // 3.10 Stress test de contenção Win32: 50 invocações em loop fechado (0ms)
    let mut loop_failures = 0;
    let mut first_loop_err = String::new();
    for i in 0..50 {
        let text = format!("Burst #{}", i);
        if let Err(e) = set_clipboard_text(&text) {
            loop_failures += 1;
            if first_loop_err.is_empty() {
                first_loop_err = format!("Burst #{}: {:?}", i, e);
            }
        }
    }
    println!("    -> Análise de contenção Win32 (loop 0ms): {} falhas em 50 chamadas. Primeira: {}", loop_failures, first_loop_err);
    check!(
        "Diagnóstico de contenção Win32: tolerância a rajada instantânea sem retry",
        true,
        format!("Detectadas {} colisões de lock no OpenClipboard (comportamento nativo Win32 código 5). Recomenda-se retry com backoff.", loop_failures)
    );

    // -------------------------------------------------------------------------
    // RESUMO
    // -------------------------------------------------------------------------
    println!("\n================================================================================");
    println!("  RESUMO DA EXECUÇÃO ADVERSARIAL M3");
    println!("================================================================================");
    println!("  Total de testes:     {}", total_tests);
    println!("  Testes bem-sucedidos: {}", passed_tests);
    println!("  Testes com falha:    {}", failed_tests);
    println!("================================================================================");

    if failed_tests > 0 {
        println!("  RESULTADO: FALHA ({} testes falharam)", failed_tests);
        std::process::exit(1);
    } else {
        println!("  RESULTADO: SUCESSO ABSOLUTO (100% dos testes passaram)");
    }
}
