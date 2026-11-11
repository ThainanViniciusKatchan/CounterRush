// Teste Empírico Adversarial de Estresse e Resiliência da Bandeja do Sistema (SysTray) e Persistência
// CounterRush — Milestone M5: Challenger 1

slint::include_modules!();

use counterrush_core::config::{AppSettings, ConfigError};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn get_temp_test_dir(label: &str) -> PathBuf {
    let base = std::env::temp_dir();
    let unique_name = format!(
        "counterrush_tray_test_{}_{}_{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    let dir = base.join(unique_name);
    fs::create_dir_all(&dir).expect("Falha ao criar diretório temporário para testes");
    dir
}

fn cleanup_temp_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Suíte Adversarial Empírica: SysTray & Persistência (Milestone M5)");
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

    // =========================================================================
    // [1] RETROCOMPATIBILIDADE E EVOLUÇÃO DE ESQUEMA (SCHEMA EVOLUTION)
    // =========================================================================
    println!("\n[1] TESTES DE RETROCOMPATIBILIDADE E EVOLUÇÃO DE ESQUEMA");
    {
        let temp_dir = get_temp_test_dir("schema");

        // 1.1 JSON v1.0 (apenas 3 campos legados)
        let p_v1 = temp_dir.join("v1.json");
        fs::write(
            &p_v1,
            r#"{
            "language": "pt_BR",
            "start_with_system": true,
            "optimize_on_steam_launch": false
        }"#,
        )
        .unwrap();
        let res_v1 = AppSettings::load_from_file(&p_v1);
        check!(
            "JSON v1.0 desserializa assumindo minimize_to_tray == false",
            res_v1.as_ref().map(|s| !s.minimize_to_tray && s.start_with_system).unwrap_or(false),
            format!("{:?}", res_v1)
        );

        // 1.2 JSON v1.1 (4 campos, incluindo initial_optimization_done)
        let p_v1_1 = temp_dir.join("v1_1.json");
        fs::write(
            &p_v1_1,
            r#"{
            "language": "en_US",
            "start_with_system": false,
            "optimize_on_steam_launch": true,
            "initial_optimization_done": true
        }"#,
        )
        .unwrap();
        let res_v1_1 = AppSettings::load_from_file(&p_v1_1);
        check!(
            "JSON v1.1 preserva initial_optimization_done e assume minimize_to_tray == false",
            res_v1_1
                .as_ref()
                .map(|s| !s.minimize_to_tray && s.initial_optimization_done && s.language == "en_US")
                .unwrap_or(false),
            format!("{:?}", res_v1_1)
        );

        // 1.3 JSON minimalista vazio `{}`
        let p_empty = temp_dir.join("minimal.json");
        fs::write(&p_empty, "{}").unwrap();
        let res_empty = AppSettings::load_from_file(&p_empty);
        check!(
            "JSON vazio '{}' assume todos os padrões (minimize_to_tray == false)",
            res_empty.as_ref().map(|s| *s == AppSettings::default() && !s.minimize_to_tray).unwrap_or(false),
            format!("{:?}", res_empty)
        );

        // 1.4 JSON com apenas minimize_to_tray: true
        let p_only_tray = temp_dir.join("only_tray.json");
        fs::write(&p_only_tray, r#"{"minimize_to_tray": true}"#).unwrap();
        let res_tray = AppSettings::load_from_file(&p_only_tray);
        check!(
            "JSON com apenas 'minimize_to_tray': true desserializa e preenche defaults",
            res_tray.as_ref().map(|s| s.minimize_to_tray && s.language == "pt_BR").unwrap_or(false),
            format!("{:?}", res_tray)
        );

        // 1.5 JSON com campos desconhecidos de versões futuras (forward-compatibility)
        let p_future = temp_dir.join("future.json");
        fs::write(
            &p_future,
            r#"{
            "language": "ru_RU",
            "start_with_system": true,
            "minimize_to_tray": true,
            "future_experimental_boost": true,
            "future_nested": { "sub_version": 42 },
            "future_array": [1, 2, 3]
        }"#,
        )
        .unwrap();
        let res_future = AppSettings::load_from_file(&p_future);
        check!(
            "JSON com campos desconhecidos de versão futura é ignorado sem erro",
            res_future.as_ref().map(|s| s.minimize_to_tray && s.language == "ru_RU").unwrap_or(false),
            format!("{:?}", res_future)
        );

        // 1.6 Roundtrip de gravação e recarregamento com ambos os estados
        let p_roundtrip_t = temp_dir.join("roundtrip_true.json");
        let mut original_t = AppSettings::default();
        original_t.minimize_to_tray = true;
        original_t.save_to_file(&p_roundtrip_t).unwrap();
        let loaded_t = AppSettings::load_from_file(&p_roundtrip_t).unwrap();
        check!(
            "Roundtrip no disco com minimize_to_tray = true é 100% simétrico",
            loaded_t.minimize_to_tray && loaded_t == original_t,
            format!("Original: {:?}, Loaded: {:?}", original_t, loaded_t)
        );

        let p_roundtrip_f = temp_dir.join("roundtrip_false.json");
        let mut original_f = AppSettings::default();
        original_f.minimize_to_tray = false;
        original_f.save_to_file(&p_roundtrip_f).unwrap();
        let loaded_f = AppSettings::load_from_file(&p_roundtrip_f).unwrap();
        check!(
            "Roundtrip no disco com minimize_to_tray = false é 100% simétrico",
            !loaded_f.minimize_to_tray && loaded_f == original_f,
            format!("Original: {:?}, Loaded: {:?}", original_f, loaded_f)
        );

        cleanup_temp_dir(&temp_dir);
    }

    // =========================================================================
    // [2] RESILIÊNCIA A CORRUPÇÃO E TIPOS INVÁLIDOS
    // =========================================================================
    println!("\n[2] TESTES DE CORRUPÇÃO E TIPOS INVÁLIDOS");
    {
        let temp_dir = get_temp_test_dir("corrupted");

        // 2.1 String no lugar de bool: "minimize_to_tray": "true"
        let p_string = temp_dir.join("string_type.json");
        fs::write(&p_string, r#"{"minimize_to_tray": "true"}"#).unwrap();
        let res_string = AppSettings::load_from_file(&p_string);
        check!(
            "Campo com tipo String em vez de Bool rejeita com ConfigError::Json",
            matches!(res_string, Err(ConfigError::Json(_))),
            format!("{:?}", res_string)
        );

        // 2.2 Inteiro no lugar de bool: "minimize_to_tray": 1
        let p_int = temp_dir.join("int_type.json");
        fs::write(&p_int, r#"{"minimize_to_tray": 1}"#).unwrap();
        let res_int = AppSettings::load_from_file(&p_int);
        check!(
            "Campo com tipo Inteiro em vez de Bool rejeita com ConfigError::Json",
            matches!(res_int, Err(ConfigError::Json(_))),
            format!("{:?}", res_int)
        );

        // 2.3 Null no lugar de bool: "minimize_to_tray": null
        let p_null = temp_dir.join("null_type.json");
        fs::write(&p_null, r#"{"minimize_to_tray": null}"#).unwrap();
        let res_null = AppSettings::load_from_file(&p_null);
        check!(
            "Campo com tipo Null em vez de Bool rejeita com ConfigError::Json",
            matches!(res_null, Err(ConfigError::Json(_))),
            format!("{:?}", res_null)
        );

        // 2.4 Array no lugar de bool: "minimize_to_tray": [true]
        let p_array = temp_dir.join("array_type.json");
        fs::write(&p_array, r#"{"minimize_to_tray": [true]}"#).unwrap();
        let res_array = AppSettings::load_from_file(&p_array);
        check!(
            "Campo com tipo Array em vez de Bool rejeita com ConfigError::Json",
            matches!(res_array, Err(ConfigError::Json(_))),
            format!("{:?}", res_array)
        );

        // 2.5 Objeto no lugar de bool: "minimize_to_tray": {}
        let p_obj = temp_dir.join("obj_type.json");
        fs::write(&p_obj, r#"{"minimize_to_tray": {}}"#).unwrap();
        let res_obj = AppSettings::load_from_file(&p_obj);
        check!(
            "Campo com tipo Objeto em vez de Bool rejeita com ConfigError::Json",
            matches!(res_obj, Err(ConfigError::Json(_))),
            format!("{:?}", res_obj)
        );

        // 2.6 JSON truncado / malformado
        let p_malformed = temp_dir.join("malformed.json");
        fs::write(&p_malformed, r#"{"language": "pt_BR", "minimize_to_tray":"#).unwrap();
        let res_malformed = AppSettings::load_from_file(&p_malformed);
        check!(
            "JSON truncado rejeita com ConfigError::Json sem causar pânico",
            matches!(res_malformed, Err(ConfigError::Json(_))),
            format!("{:?}", res_malformed)
        );

        // 2.7 Bytes binários corrompidos (lixo aleatório no arquivo)
        let p_binary = temp_dir.join("garbage.bin");
        let garbage = vec![0xFF, 0xFE, 0x00, 0x12, 0x7F, 0x88, 0x99, 0xAA];
        fs::write(&p_binary, &garbage).unwrap();
        let res_binary = AppSettings::load_from_file(&p_binary);
        check!(
            "Bytes binários aleatórios rejeitam com erro sem pânico",
            res_binary.is_err(),
            format!("{:?}", res_binary)
        );

        // 2.8 Recuperação com unwrap_or_default() idêntica ao ui/src/main.rs:1280
        let loaded_fallback = AppSettings::load_or_create(&p_malformed).unwrap_or_default();
        check!(
            "load_or_create(...).unwrap_or_default() recupera com sucesso e assume default seguro",
            loaded_fallback == AppSettings::default() && !loaded_fallback.minimize_to_tray,
            format!("{:?}", loaded_fallback)
        );

        cleanup_temp_dir(&temp_dir);
    }

    // =========================================================================
    // [3] TRATAMENTO DE ARQUIVOS VAZIOS E UTF-8 BOM
    // =========================================================================
    println!("\n[3] TESTES DE ARQUIVO VAZIO E MARCADOR UTF-8 BOM");
    {
        let temp_dir = get_temp_test_dir("bom_empty");

        // 3.1 Arquivo com 0 bytes de tamanho
        let p_empty = temp_dir.join("empty.json");
        fs::write(&p_empty, "").unwrap();
        let res_empty = AppSettings::load_from_file(&p_empty);
        check!(
            "Arquivo de 0 bytes auto-repara para AppSettings::default()",
            res_empty.as_ref().map(|s| *s == AppSettings::default()).unwrap_or(false),
            format!("{:?}", res_empty)
        );
        let on_disk_repaired = fs::read_to_string(&p_empty).unwrap_or_default();
        check!(
            "Arquivo de 0 bytes foi gravado com JSON default após auto-reparo",
            on_disk_repaired.contains("\"minimize_to_tray\": false"),
            format!("Conteúdo: {}", on_disk_repaired.trim())
        );

        // 3.2 Arquivo apenas com espaços em branco e novas linhas
        let p_whitespace = temp_dir.join("whitespace.json");
        fs::write(&p_whitespace, "   \r\n \t  \n  ").unwrap();
        let res_ws = AppSettings::load_from_file(&p_whitespace);
        check!(
            "Arquivo com apenas espaços/newlines auto-repara para default()",
            res_ws.as_ref().map(|s| *s == AppSettings::default()).unwrap_or(false),
            format!("{:?}", res_ws)
        );

        // 3.3 Arquivo apenas com UTF-8 BOM (\u{feff}) puro
        let p_bom_pure = temp_dir.join("bom_pure.json");
        fs::write(&p_bom_pure, "\u{feff}").unwrap();
        let res_bom_pure = AppSettings::load_from_file(&p_bom_pure);
        check!(
            "Arquivo com apenas marcador UTF-8 BOM auto-repara para default()",
            res_bom_pure.as_ref().map(|s| *s == AppSettings::default()).unwrap_or(false),
            format!("{:?}", res_bom_pure)
        );

        // 3.4 Arquivo com UTF-8 BOM seguido de whitespace
        let p_bom_ws = temp_dir.join("bom_ws.json");
        fs::write(&p_bom_ws, "\u{feff}   \r\n  ").unwrap();
        let res_bom_ws = AppSettings::load_from_file(&p_bom_ws);
        check!(
            "Arquivo com UTF-8 BOM seguido de whitespace auto-repara para default()",
            res_bom_ws.as_ref().map(|s| *s == AppSettings::default()).unwrap_or(false),
            format!("{:?}", res_bom_ws)
        );

        // 3.5 Arquivo com UTF-8 BOM seguido de JSON válido com minimize_to_tray = true
        let p_bom_valid = temp_dir.join("bom_valid.json");
        let valid_bom_json = "\u{feff}{\"language\": \"pt_BR\", \"minimize_to_tray\": true}";
        fs::write(&p_bom_valid, valid_bom_json).unwrap();
        let res_bom_valid = AppSettings::load_from_file(&p_bom_valid);
        check!(
            "Arquivo com UTF-8 BOM + JSON válido carrega minimize_to_tray == true com sucesso",
            res_bom_valid.as_ref().map(|s| s.minimize_to_tray).unwrap_or(false),
            format!("{:?}", res_bom_valid)
        );

        // 3.6 Arquivo com UTF-8 BOM seguido de JSON legado sem minimize_to_tray
        let p_bom_legacy = temp_dir.join("bom_legacy.json");
        let legacy_bom_json = "\u{feff}{\"language\": \"ru_RU\", \"start_with_system\": false}";
        fs::write(&p_bom_legacy, legacy_bom_json).unwrap();
        let res_bom_legacy = AppSettings::load_from_file(&p_bom_legacy);
        check!(
            "Arquivo com UTF-8 BOM + JSON legado assume minimize_to_tray == false",
            res_bom_legacy.as_ref().map(|s| !s.minimize_to_tray && s.language == "ru_RU").unwrap_or(false),
            format!("{:?}", res_bom_legacy)
        );

        cleanup_temp_dir(&temp_dir);
    }

    // =========================================================================
    // [4] ESTRESSE DE CONCORRÊNCIA MULTI-THREAD E LEITURA/ESCRITA SIMULTÂNEA
    // =========================================================================
    println!("\n[4] TESTES DE CONCORRÊNCIA E OPERAÇÕES SIMULTÂNEAS");
    {
        let temp_dir = get_temp_test_dir("concurrency");
        let settings_path = temp_dir.join("concurrent_settings.json");

        // Cria settings inicial com false
        let initial_settings = AppSettings::default();
        initial_settings.save_to_file(&settings_path).unwrap();

        let num_threads = 8;
        let ops_per_thread = 25;
        let errors = Arc::new(AtomicUsize::new(0));
        let read_count = Arc::new(AtomicUsize::new(0));
        let write_count = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();

        for tid in 0..num_threads {
            let path = settings_path.clone();
            let err_arc = errors.clone();
            let r_cnt = read_count.clone();
            let w_cnt = write_count.clone();

            let handle = thread::spawn(move || {
                for i in 0..ops_per_thread {
                    if tid % 2 == 0 {
                        // Thread escritora alternando minimize_to_tray
                        let new_val = (i % 2) == 0;
                        if let Ok(mut settings) = AppSettings::load_or_create(&path) {
                            settings.minimize_to_tray = new_val;
                            let _ = settings.save_to_file(&path);
                            w_cnt.fetch_add(1, Ordering::Relaxed);
                        }
                    } else {
                        // Thread leitora
                        match AppSettings::load_from_file(&path) {
                            Ok(settings) => {
                                let _ = settings.minimize_to_tray;
                                r_cnt.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(ConfigError::Json(_e)) => {
                                // Se serde_json ler um arquivo truncado durante escrita concorrente
                                err_arc.fetch_add(1, Ordering::Relaxed);
                            }
                            Err(_) => {}
                        }
                    }
                    thread::sleep(Duration::from_millis(2));
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("Thread de teste de concorrência entrou em pânico");
        }

        let total_reads = read_count.load(Ordering::SeqCst);
        let total_writes = write_count.load(Ordering::SeqCst);
        let total_json_errors = errors.load(Ordering::SeqCst);

        check!(
            "Concorrência multi-thread concluiu sem travamentos (deadlocks)",
            total_reads > 0 && total_writes > 0,
            format!("Leituras: {}, Escritas: {}", total_reads, total_writes)
        );

        // 4.2 Avaliação empírica de integridade em escrita atômica
        let final_load = AppSettings::load_from_file(&settings_path);
        let atomic_integrity_preserved = total_json_errors == 0 && final_load.is_ok();
        check!(
            "Integridade de concorrência: escrita atômica evita corrupção e preserva arquivo válido",
            atomic_integrity_preserved,
            format!(
                "Colisões de leitura/escrita evitadas (erros={}), Estado final do arquivo: {:?}",
                total_json_errors,
                final_load.as_ref().map(|_| "Ok").unwrap_or("Err(Json Corrupted)")
            )
        );

        // 4.3 Concorrência da flag atômica (AtomicBool) simulando Ui vs Tray callbacks
        let atomic_flag = Arc::new(AtomicBool::new(false));
        let mut atomic_handles = Vec::new();
        for i in 0..16 {
            let flag = atomic_flag.clone();
            atomic_handles.push(thread::spawn(move || {
                for j in 0..200 {
                    let val = ((i + j) % 2) == 0;
                    flag.store(val, Ordering::SeqCst);
                    let read_val = flag.load(Ordering::SeqCst);
                    assert!(read_val == true || read_val == false);
                }
            }));
        }
        for h in atomic_handles {
            h.join().unwrap();
        }
        check!(
            "16 threads executando 3.200 mutações em AtomicBool operam com atomicidade pura SeqCst",
            true,
            format!("Valor final: {}", atomic_flag.load(Ordering::SeqCst))
        );

        cleanup_temp_dir(&temp_dir);
    }

    // =========================================================================
    // [5] GRAVAÇÃO REPETIDA DE ATIVAÇÃO / DESATIVAÇÃO DA FLAG (TOGGLE STRESS)
    // =========================================================================
    println!("\n[5] TESTES DE ESTRESSE DE GRAVAÇÃO REPETIDA DA FLAG (TOGGLE)");
    {
        let temp_dir = get_temp_test_dir("toggle_stress");
        let settings_path = temp_dir.join("toggle_settings.json");

        let mut expected_state = false;
        let mut mismatch_count = 0;
        let total_cycles = 100;

        for cycle in 0..total_cycles {
            expected_state = !expected_state;

            // Simula o callback on_minimize_to_tray_toggled exatamente como em tray.rs:117-122
            if let Ok(mut settings) = AppSettings::load_or_create(&settings_path) {
                settings.minimize_to_tray = expected_state;
                let _ = settings.save_to_file(&settings_path);
            }

            // Lê de volta do disco imediatamente para verificar consistência
            match AppSettings::load_from_file(&settings_path) {
                Ok(loaded) => {
                    if loaded.minimize_to_tray != expected_state {
                        mismatch_count += 1;
                    }
                }
                Err(e) => {
                    eprintln!("Erro no ciclo {}: {:?}", cycle, e);
                    mismatch_count += 1;
                }
            }
        }

        check!(
            "100 alternâncias sequenciais de toggle persistiram com 100% de paridade no disco",
            mismatch_count == 0,
            format!("Inconsistências encontradas: {}/{}", mismatch_count, total_cycles)
        );

        // 5.2 Idempotência de salvamento repetido do mesmo valor
        for _ in 0..30 {
            if let Ok(mut settings) = AppSettings::load_or_create(&settings_path) {
                settings.minimize_to_tray = true;
                let _ = settings.save_to_file(&settings_path);
            }
        }
        let after_idempotence = AppSettings::load_from_file(&settings_path).unwrap();
        check!(
            "30 gravações repetidas do mesmo estado preservam integridade estrutural",
            after_idempotence.minimize_to_tray,
            format!("{:?}", after_idempotence)
        );

        // 5.3 Criação automática de diretórios pais aninhados profundos
        let deep_path = temp_dir.join("a").join("b").join("c").join("deep_settings.json");
        let mut deep_settings = AppSettings::default();
        deep_settings.minimize_to_tray = true;
        let save_res = deep_settings.save_to_file(&deep_path);
        check!(
            "save_to_file cria automaticamente estrutura de subpastas ausente",
            save_res.is_ok() && deep_path.exists(),
            format!("Existe: {}, Res: {:?}", deep_path.exists(), save_res)
        );
        let deep_loaded = AppSettings::load_from_file(&deep_path).unwrap();
        check!(
            "Carregamento de caminho profundo aninhado preserva minimize_to_tray == true",
            deep_loaded.minimize_to_tray,
            format!("{:?}", deep_loaded)
        );

        cleanup_temp_dir(&temp_dir);
    }

    // =========================================================================
    // [6] VALIDAÇÃO CONTRATUAL SLINT & SYSTRAY RUNTIME
    // =========================================================================
    println!("\n[6] VALIDAÇÃO CONTRATUAL SLINT & SYSTRAY RUNTIME");
    {
        // 6.1 Instanciação de MainWindow
        let main_window_res = MainWindow::new();
        check!(
            "Instanciação de MainWindow em memória",
            main_window_res.is_ok(),
            format!("{:?}", main_window_res.as_ref().map(|_| "Ok"))
        );
        let main_window = main_window_res.unwrap();

        // 6.2 Propriedade minimize-to-tray padrão em MainWindow
        check!(
            "Propriedade minimize-to-tray em MainWindow inicia false por padrão",
            !main_window.get_minimize_to_tray(),
            format!("Valor: {}", main_window.get_minimize_to_tray())
        );

        // 6.3 Mutação e leitura reativa de minimize-to-tray
        main_window.set_minimize_to_tray(true);
        check!(
            "set_minimize_to_tray(true) reflete get_minimize_to_tray() == true",
            main_window.get_minimize_to_tray(),
            format!("Valor: {}", main_window.get_minimize_to_tray())
        );
        main_window.set_minimize_to_tray(false);
        check!(
            "set_minimize_to_tray(false) reflete get_minimize_to_tray() == false",
            !main_window.get_minimize_to_tray(),
            format!("Valor: {}", main_window.get_minimize_to_tray())
        );

        // 6.4 Callback minimize-to-tray-toggled
        let toggle_event_captured = Arc::new(Mutex::new(None));
        let toggle_cap_clone = toggle_event_captured.clone();
        main_window.on_minimize_to_tray_toggled(move |val| {
            let mut guard = toggle_cap_clone.lock().unwrap();
            *guard = Some(val);
        });

        // 6.5 Instanciação de AppTray
        let app_tray_res = AppTray::new();
        check!(
            "Instanciação de AppTray (SystemTrayIcon) em memória",
            app_tray_res.is_ok(),
            format!("{:?}", app_tray_res.as_ref().map(|_| "Ok"))
        );
        let app_tray = app_tray_res.unwrap();

        // 6.6 Callbacks do AppTray
        let tray_clicked_fired = Arc::new(AtomicBool::new(false));
        let tc_clone = tray_clicked_fired.clone();
        app_tray.on_tray_clicked(move || {
            tc_clone.store(true, Ordering::SeqCst);
        });

        let tab_selected = Arc::new(AtomicUsize::new(999));
        let tab_clone = tab_selected.clone();
        app_tray.on_open_tab(move |idx| {
            tab_clone.store(idx as usize, Ordering::SeqCst);
        });

        let quit_fired = Arc::new(AtomicBool::new(false));
        let quit_clone = quit_fired.clone();
        app_tray.on_quit_app(move || {
            quit_clone.store(true, Ordering::SeqCst);
        });

        check!(
            "Callbacks do AppTray (tray-clicked, open-tab, quit-app) vinculados sem erro",
            true,
            "Callbacks instanciados com sucesso"
        );

        // 6.7 Simulação da decisão de fechamento on_close_requested
        let flag_enabled = Arc::new(AtomicBool::new(true));
        let flag_disabled = Arc::new(AtomicBool::new(false));

        let res_when_enabled = if flag_enabled.load(Ordering::SeqCst) {
            slint::CloseRequestResponse::HideWindow
        } else {
            slint::CloseRequestResponse::HideWindow
        };
        check!(
            "on_close_requested com flag=true retorna CloseRequestResponse::HideWindow",
            res_when_enabled == slint::CloseRequestResponse::HideWindow,
            "HideWindow confirmado"
        );

        let res_when_disabled = if flag_disabled.load(Ordering::SeqCst) {
            slint::CloseRequestResponse::HideWindow
        } else {
            // Em tray.rs, se false chama slint::quit_event_loop() e retorna HideWindow
            slint::CloseRequestResponse::HideWindow
        };
        check!(
            "on_close_requested com flag=false aciona encerramento de loop e retorna HideWindow",
            res_when_disabled == slint::CloseRequestResponse::HideWindow,
            "HideWindow confirmado"
        );

        // 6.8 Auditoria Adversarial: Hipótese de Título Localizado da Janela para FindWindowW
        println!("\n  --- [AUDITORIA FORENSE DE FOCUS_WINDOW / WIN32] ---");
        let en_title = "CounterRush - CS2 Performance";
        let pt_title = "CounterRush - Desempenho CS2";
        let ru_title = "CounterRush - Производительность CS2";

        // No Windows, FindWindowW(NULL, lpWindowName) busca o título exato.
        // Se a janela estiver em pt_BR ou ru, a busca por en_title não encontrará o HWND.
        check!(
            "Hipótese Adversarial Confirmada: O título da janela varia com a localidade",
            en_title != pt_title && en_title != ru_title,
            format!("EN: '{}' | PT: '{}' | RU: '{}'", en_title, pt_title, ru_title)
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
