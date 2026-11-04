// Teste Empírico Adversarial dos Contratos de GUI e Modais (Milestone M3)
// CounterRush — Challenger Settings 2

slint::include_modules!();

use counterrush_core::config::AppSettings;
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Suíte Adversarial de Desafio Empírico: GUI & Settings M3");
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
    // [1] INSTANCIAÇÃO DO COMPONENTE SLINT E VALIDAÇÃO DE PROPRIEDADES DEFAULT
    // -------------------------------------------------------------------------
    println!("\n[1] VALIDAÇÃO DE INSTANCIAÇÃO SLINT E PROPRIEDADES DEFAULT");
    let main_window_res = MainWindow::new();
    check!(
        "Instanciação de MainWindow em memória",
        main_window_res.is_ok(),
        format!("Resultado: {:?}", main_window_res.as_ref().map(|_| "Ok"))
    );

    let main_window = main_window_res.expect("Falha ao instanciar MainWindow");

    // 1.1 Modal de desenvolvimento fechado por padrão
    let initial_modal_visible = main_window.get_show_in_development_modal();
    check!(
        "show-in-development-modal inicial é false",
        !initial_modal_visible,
        format!("show_in_development_modal = {}", initial_modal_visible)
    );

    // 1.2 Nome de recurso em desenvolvimento vazio por padrão
    let initial_feature_name = main_window.get_in_development_feature_name();
    check!(
        "in-development-feature-name inicial é vazio",
        initial_feature_name.is_empty(),
        format!("in_development_feature_name = '{}'", initial_feature_name)
    );

    // 1.3 Idioma inicial default é pt_BR
    let initial_language = main_window.get_active_language();
    check!(
        "active-language default é pt_BR",
        initial_language == "pt_BR",
        format!("active_language = '{}'", initial_language)
    );

    // -------------------------------------------------------------------------
    // [2] CONTRATO REATIVO DO MODAL DE DESENVOLVIMENTO (SETTERS, GETTERS, RESET)
    // -------------------------------------------------------------------------
    println!("\n[2] TESTES DE TRANSIÇÃO DE ESTADO DO MODAL 'EM DESENVOLVIMENTO'");
    {
        // 2.1 Simula abertura para 'Iniciar com o Windows'
        main_window.set_in_development_feature_name("Iniciar com o Windows".into());
        main_window.set_show_in_development_modal(true);
        check!(
            "Transição de estado para abertura de 'Iniciar com o Windows'",
            main_window.get_show_in_development_modal() && main_window.get_in_development_feature_name() == "Iniciar com o Windows",
            format!("show = {}, name = '{}'", main_window.get_show_in_development_modal(), main_window.get_in_development_feature_name())
        );

        // 2.2 Simula fechamento pelo botão 'X' ou 'Entendido'
        main_window.set_show_in_development_modal(false);
        main_window.set_in_development_feature_name("".into());
        check!(
            "Reset do modal fecha exibição e limpa nome da feature",
            !main_window.get_show_in_development_modal() && main_window.get_in_development_feature_name().is_empty(),
            format!("show = {}, name = '{}'", main_window.get_show_in_development_modal(), main_window.get_in_development_feature_name())
        );

        // 2.3 Simula abertura para 'Auto-Otimização Steam'
        main_window.set_in_development_feature_name("Auto-Otimização Steam".into());
        main_window.set_show_in_development_modal(true);
        check!(
            "Transição de estado para abertura de 'Auto-Otimização Steam'",
            main_window.get_show_in_development_modal() && main_window.get_in_development_feature_name() == "Auto-Otimização Steam",
            format!("show = {}, name = '{}'", main_window.get_show_in_development_modal(), main_window.get_in_development_feature_name())
        );

        // Reset final
        main_window.set_show_in_development_modal(false);
        main_window.set_in_development_feature_name("".into());
    }

    // -------------------------------------------------------------------------
    // [3] CONTRATOS DE EVENTOS E CALLBACKS DA GUI
    // -------------------------------------------------------------------------
    println!("\n[3] VALIDAÇÃO DE CONTRATOS DE CALLBACKS (DISPARO E DADOS)");
    {
        // 3.1 Callback language-selected
        let selected_lang_received = Arc::new(Mutex::new(String::new()));
        let lang_clone = Arc::clone(&selected_lang_received);
        main_window.on_language_selected(move |code| {
            if let Ok(mut l) = lang_clone.lock() {
                *l = code.to_string();
            }
        });

        main_window.invoke_language_selected("pt_BR".into());
        check!(
            "Callback language-selected recebe código 'pt_BR'",
            *selected_lang_received.lock().unwrap() == "pt_BR",
            format!("Recebido: '{}'", selected_lang_received.lock().unwrap())
        );

        main_window.invoke_language_selected("ru_RU".into());
        check!(
            "Callback language-selected recebe código 'ru_RU'",
            *selected_lang_received.lock().unwrap() == "ru_RU",
            format!("Recebido: '{}'", selected_lang_received.lock().unwrap())
        );

        main_window.invoke_language_selected("en_US".into());
        check!(
            "Callback language-selected recebe código 'en_US'",
            *selected_lang_received.lock().unwrap() == "en_US",
            format!("Recebido: '{}'", selected_lang_received.lock().unwrap())
        );

        // Validação da API de traduções embutidas do Slint para os 3 idiomas oficiais
        let pt_res = slint::select_bundled_translation("pt_BR");
        check!(
            "slint::select_bundled_translation('pt_BR') executa com sucesso",
            pt_res.is_ok(),
            format!("Resultado: {:?}", pt_res)
        );

        let ru_res = slint::select_bundled_translation("ru_RU");
        check!(
            "slint::select_bundled_translation('ru_RU') executa com sucesso",
            ru_res.is_ok(),
            format!("Resultado: {:?}", ru_res)
        );

        let en_res = slint::select_bundled_translation("en_US");
        check!(
            "slint::select_bundled_translation('en_US') executa com sucesso",
            en_res.is_ok(),
            format!("Resultado: {:?}", en_res)
        );

        // 3.2 Callback share-project-clicked
        let share_clicked = Arc::new(AtomicBool::new(false));
        let share_clone = Arc::clone(&share_clicked);
        main_window.on_share_project_clicked(move || {
            share_clone.store(true, Ordering::SeqCst);
        });

        main_window.invoke_share_project_clicked();
        check!(
            "Callback share-project-clicked disparado com sucesso",
            share_clicked.load(Ordering::SeqCst),
            format!("share_clicked = {}", share_clicked.load(Ordering::SeqCst))
        );

        // 3.3 Callback open-external-url
        let external_url_received = Arc::new(Mutex::new(String::new()));
        let url_clone = Arc::clone(&external_url_received);
        main_window.on_open_external_url(move |url| {
            if let Ok(mut u) = url_clone.lock() {
                *u = url.to_string();
            }
        });

        main_window.invoke_open_external_url("https://github.com/ThainanViniciusKatchan/CounterRush".into());
        check!(
            "Callback open-external-url com URL do GitHub",
            *external_url_received.lock().unwrap() == "https://github.com/ThainanViniciusKatchan/CounterRush",
            format!("Recebido: '{}'", external_url_received.lock().unwrap())
        );

        main_window.invoke_open_external_url("https://feitonobrasil.dev.br".into());
        check!(
            "Callback open-external-url com URL do Website",
            *external_url_received.lock().unwrap() == "https://feitonobrasil.dev.br",
            format!("Recebido: '{}'", external_url_received.lock().unwrap())
        );
    }

    // -------------------------------------------------------------------------
    // [4] VALIDAÇÃO DE NÃO-MUTAÇÃO NO DISCO E INTEGRIDADE DE APPSETTINGS
    // -------------------------------------------------------------------------
    println!("\n[4] TESTE EMPÍRICO DE NÃO-MUTAÇÃO NO DISCO PARA RECURSOS EM DESENVOLVIMENTO");
    {
        let test_dir = std::env::temp_dir().join(format!("counterrush_settings_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&test_dir);
        let settings_path = test_dir.join("settings.json");

        // Cria settings default
        let default_settings = AppSettings::default();
        check!(
            "AppSettings default possui start_with_system = false",
            !default_settings.start_with_system,
            format!("start_with_system = {}", default_settings.start_with_system)
        );
        check!(
            "AppSettings default possui optimize_on_steam_launch = false",
            !default_settings.optimize_on_steam_launch,
            format!("optimize_on_steam_launch = {}", default_settings.optimize_on_steam_launch)
        );

        default_settings.save_to_file(&settings_path).expect("Falha ao salvar settings inicial");

        // Simula disparo repetido do modal (sem alteração em disco)
        for _ in 0..10 {
            main_window.set_in_development_feature_name("Iniciar com o Windows".into());
            main_window.set_show_in_development_modal(true);
            main_window.set_show_in_development_modal(false);
            main_window.set_in_development_feature_name("Auto-Otimização Steam".into());
            main_window.set_show_in_development_modal(true);
            main_window.set_show_in_development_modal(false);
        }

        // Recarrega do disco
        let loaded = AppSettings::load_from_file(&settings_path).expect("Falha ao recarregar settings");
        check!(
            "Disco não sofre mutação em start_with_system após interações de modal",
            !loaded.start_with_system,
            format!("start_with_system = {}", loaded.start_with_system)
        );
        check!(
            "Disco não sofre mutação em optimize_on_steam_launch após interações de modal",
            !loaded.optimize_on_steam_launch,
            format!("optimize_on_steam_launch = {}", loaded.optimize_on_steam_launch)
        );

        // Agora simula troca legítima de idioma: apenas language deve mudar!
        let mut modified = loaded.clone();
        modified.language = "ru".to_string();
        modified.save_to_file(&settings_path).expect("Falha ao salvar idioma alterado");

        let reloaded = AppSettings::load_from_file(&settings_path).expect("Falha ao recarregar após troca de idioma");
        check!(
            "Troca de idioma atualiza apenas 'language' preservando flags de sistema desligadas",
            reloaded.language == "ru" && !reloaded.start_with_system && !reloaded.optimize_on_steam_launch,
            format!("language = '{}', start_with_system = {}, optimize_on_steam = {}", reloaded.language, reloaded.start_with_system, reloaded.optimize_on_steam_launch)
        );

        let _ = fs::remove_dir_all(&test_dir);
    }

    // -------------------------------------------------------------------------
    // [5] VALIDAÇÃO DA ÁREA DE TRANSFERÊNCIA (WIN32 CLIPBOARD FFI)
    // -------------------------------------------------------------------------
    println!("\n[5] TESTES EMPÍRICOS DA ÁREA DE TRANSFERÊNCIA (WIN32 CLIPBOARD)");
    {
        let test_message = "CounterRush - Otimizador de Desempenho e Latência para Counter-Strike 2: https://github.com/ThainanViniciusKatchan/CounterRush";
        let clip_res = counterrush_core::set_clipboard(test_message);
        check!(
            "Cópia da mensagem oficial para a área de transferência do Windows",
            clip_res.is_ok(),
            format!("Resultado FFI: {:?}", clip_res)
        );

        // Teste de estresse com strings variadas e caracteres especiais / UTF-16
        let adversarial_strings = [
            "CounterRush © 2026 — Тестирование производительности CS2!",
            "Área de transferência com acentuação e cedilha: áéíóú çãõ",
            "", // string vazia
            "https://feitonobrasil.dev.br/counterrush?query=test&param=1#fragment",
        ];
        let mut all_copied = true;
        for s in &adversarial_strings {
            if let Err(e) = counterrush_core::set_clipboard(s) {
                println!("    Erro ao copiar '{}': {:?}", s, e);
                all_copied = false;
            }
        }
        check!(
            "Cópia de strings adversariais (UTF-8, caracteres especiais, vazio) via Win32 FFI",
            all_copied,
            "Todas as strings foram aceitas pelo Win32 Clipboard FFI"
        );
    }

    // -------------------------------------------------------------------------
    // [6] VALIDAÇÃO DA DESCOBERTA E INTEGRIDADE DE LOCALES
    // -------------------------------------------------------------------------
    println!("\n[6] TESTES DE DESCOBERTA E ESTRUTURA DOS LOCALES");
    {
        let locales = counterrush_core::list_available_locales();
        check!(
            "Descoberta de pelo menos 5 locales disponíveis",
            locales.len() >= 5,
            format!("Total descoberto: {}", locales.len())
        );

        let codes: Vec<String> = locales.iter().map(|l| l.code.clone()).collect();
        check!(
            "Contém pt_BR, en, en_US, ru, ru_RU",
            codes.contains(&"pt_BR".to_string())
                && codes.contains(&"en".to_string())
                && codes.contains(&"en_US".to_string())
                && codes.contains(&"ru".to_string())
                && codes.contains(&"ru_RU".to_string()),
            format!("Códigos encontrados: {:?}", codes)
        );

        // Validação de nomes legíveis
        for loc in &locales {
            check!(
                format!("Nome legível para '{}'", loc.code),
                !loc.display_name.is_empty() && loc.display_name != loc.code,
                format!("display_name = '{}'", loc.display_name)
            );
        }
    }

    // -------------------------------------------------------------------------
    // [7] VALIDAÇÃO ESTRUTURAL ESTÁTICA DOS ARQUIVOS SLINT
    // -------------------------------------------------------------------------
    println!("\n[7] VALIDAÇÃO ESTRUTURAL ESTÁTICA DE UI E DIMENSÕES DO MODAL");
    {
        // 7.1 in_development_dialog.slint
        let dialog_src = fs::read_to_string("ui/components/in_development_dialog.slint")
            .expect("Falha ao ler in_development_dialog.slint");

        check!(
            "InDevelopmentDialog possui dimensões de tela cheia: width: 980px",
            dialog_src.contains("width: 980px;"),
            "width: 980px presente"
        );
        check!(
            "InDevelopmentDialog possui dimensões de tela cheia: height: 620px",
            dialog_src.contains("height: 620px;"),
            "height: 620px presente"
        );
        check!(
            "InDevelopmentDialog possui background semi-transparente escurecido",
            dialog_src.contains("background: rgba(11, 15, 23, 0.88);"),
            "rgba(11, 15, 23, 0.88) presente"
        );
        check!(
            "InDevelopmentDialog possui TouchArea bloqueador cobrindo o fundo",
            dialog_src.contains("TouchArea") && dialog_src.contains("mouse-cursor: default;"),
            "TouchArea bloqueador de tela cheia presente"
        );
        check!(
            "InDevelopmentDialog possui card centralizado 440px x 180px",
            dialog_src.contains("width: 440px;") && dialog_src.contains("height: 180px;"),
            "Dimensões do card centralizado validadas"
        );
        check!(
            "InDevelopmentDialog possui botão 'X' que invoca root.close()",
            dialog_src.contains("close-touch := TouchArea") && dialog_src.contains("root.close();"),
            "Botão X configurado com root.close()"
        );
        check!(
            "InDevelopmentDialog possui botão 'Entendido' que invoca root.close()",
            dialog_src.contains("ok-touch := TouchArea") && dialog_src.contains("root.close();"),
            "Botão Entendido configurado com root.close()"
        );
        check!(
            "InDevelopmentDialog exibe título tático 'RECURSO EM DESENVOLVIMENTO'",
            dialog_src.contains("@tr(\"RECURSO EM DESENVOLVIMENTO\")"),
            "Título tático validado"
        );
        check!(
            "InDevelopmentDialog exibe mensagem 'Ainda em Desenvolvimento'",
            dialog_src.contains("@tr(\"Ainda em Desenvolvimento\")"),
            "Texto da mensagem validado"
        );

        // 7.2 ui/main.slint
        let main_slint_src = fs::read_to_string("ui/main.slint")
            .expect("Falha ao ler ui/main.slint");

        check!(
            "Linha 'Iniciar com o Windows' dispara in-development-feature-name e modal = true",
            main_slint_src.contains("root.in-development-feature-name = @tr(\"Iniciar com o Windows\");")
                && main_slint_src.contains("root.show-in-development-modal = true;"),
            "Trigger de Iniciar com o Windows validado"
        );

        check!(
            "Linha 'Auto-Otimização Steam' dispara in-development-feature-name e modal = true",
            main_slint_src.contains("root.in-development-feature-name = @tr(\"Auto-Otimização Steam\");")
                && main_slint_src.contains("root.show-in-development-modal = true;"),
            "Trigger de Auto-Otimização Steam validado"
        );

        check!(
            "Chips de idioma disparam root.language-selected(lang.code)",
            main_slint_src.contains("root.language-selected(lang.code);"),
            "Callback de idioma validado no TouchArea dos chips"
        );

        check!(
            "Botão 'Compartilhar Projeto' dispara root.share-project-clicked()",
            main_slint_src.contains("root.share-project-clicked();"),
            "Callback de compartilhamento validado no TouchArea"
        );

        check!(
            "Botão GitHub dispara open-external-url com URL oficial",
            main_slint_src.contains("root.open-external-url(\"https://github.com/ThainanViniciusKatchan/CounterRush\");"),
            "URL do GitHub validada"
        );

        check!(
            "Botão Website dispara open-external-url com URL oficial",
            main_slint_src.contains("root.open-external-url(\"https://feitonobrasil.dev.br\");"),
            "URL do Website validada"
        );

        check!(
            "Overlay InDevelopmentDialog em main.slint limpa modal e feature-name ao fechar",
            main_slint_src.contains("root.show-in-development-modal = false;")
                && main_slint_src.contains("root.in-development-feature-name = \"\";"),
            "Reset do modal validado no close callback em main.slint"
        );

        // 7.3 Verificação de layout limpo sem cálculos manuais frágeis de geometria
        let normalized_main_slint = main_slint_src.replace("\r\n", "\n");
        check!(
            "Indicador de idioma ativo usa VerticalLayout com alignment: center nativo",
            normalized_main_slint.contains("if is-selected : VerticalLayout {\n                                    alignment: center;"),
            "Alinhamento centralizado nativo validado para o indicador de seleção"
        );
    }

    // -------------------------------------------------------------------------
    // [8] TESTE DE ESTRESSE E CONCORRÊNCIA EM APPSETTINGS
    // -------------------------------------------------------------------------
    println!("\n[8] TESTES DE ESTRESSE DE CONCORRÊNCIA EM APPSETTINGS");
    {
        let num_threads = 6;
        let iterations = 20;
        let success_counter = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for thread_id in 0..num_threads {
            let successes = Arc::clone(&success_counter);
            let handle = thread::spawn(move || {
                let test_dir = std::env::temp_dir().join(format!("counterrush_bench_t{}_{}", thread_id, std::process::id()));
                let _ = fs::create_dir_all(&test_dir);
                let path = test_dir.join(format!("settings_t{}.json", thread_id));

                for iter in 0..iterations {
                    let mut s = AppSettings::default();
                    s.language = format!("lang_{}_{}", thread_id, iter);
                    s.save_to_file(&path).unwrap();

                    let loaded = AppSettings::load_from_file(&path).unwrap();
                    assert_eq!(loaded.language, format!("lang_{}_{}", thread_id, iter));
                    assert_eq!(loaded.start_with_system, false);
                    assert_eq!(loaded.optimize_on_steam_launch, false);

                    successes.fetch_add(1, Ordering::SeqCst);
                }
                let _ = fs::remove_dir_all(&test_dir);
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("Thread panicked");
        }

        let total_ops = num_threads * iterations;
        let actual_successes = success_counter.load(Ordering::SeqCst);
        check!(
            "Concorrência multi-thread em AppSettings (6 threads x 20 iter)",
            actual_successes == total_ops,
            format!("Total: {} | Sucessos: {}", total_ops, actual_successes)
        );
    }

    println!("\n================================================================================");
    println!(
        "     RESULTADO FINAL: {} testes executados | {} APROVADOS | {} FALHAS",
        total_tests, passed_tests, failed_tests
    );
    println!("================================================================================");

    if failed_tests > 0 {
        std::process::exit(1);
    }
}
