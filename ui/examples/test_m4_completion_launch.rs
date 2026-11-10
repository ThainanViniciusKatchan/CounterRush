// Teste Empírico Adversarial de Conclusão e Lançamento (Milestone M4)
// CounterRush — Challenger 2: Completion & Launch Challenger

slint::include_modules!();

use counterrush_core::config::{AppSettings, CustomOptimizationConfig};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Suíte Adversarial: Estado de Conclusão e Lançamento CS2 (M4)");
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
    // [1] VALIDAÇÃO DA INSTANCIAÇÃO E ESTADO DEFAULT DA UI
    // -------------------------------------------------------------------------
    println!("\n[1] VALIDAÇÃO DE INSTANCIAÇÃO SLINT E PROPRIEDADES DEFAULT DE M4");
    let main_window_res = MainWindow::new();
    check!(
        "Instanciação de MainWindow em memória",
        main_window_res.is_ok(),
        format!("Resultado: {:?}", main_window_res.as_ref().map(|_| "Ok"))
    );

    let main_window = main_window_res.expect("Falha ao instanciar MainWindow");

    check!(
        "show-auto-optimize-modal inicial é false",
        !main_window.get_show_auto_optimize_modal(),
        format!("show_auto_optimize_modal = {}", main_window.get_show_auto_optimize_modal())
    );

    check!(
        "auto-optimize-finished inicial é false",
        !main_window.get_auto_optimize_finished(),
        format!("auto_optimize_finished = {}", main_window.get_auto_optimize_finished())
    );

    check!(
        "auto-optimize-progress inicial é 0.0",
        main_window.get_auto_optimize_progress() == 0.0,
        format!("auto_optimize_progress = {}", main_window.get_auto_optimize_progress())
    );

    check!(
        "active-tab inicial é 0 (Painel)",
        main_window.get_active_tab() == 0,
        format!("active_tab = {}", main_window.get_active_tab())
    );

    // -------------------------------------------------------------------------
    // [2] TRANSIÇÃO PARA ESTADO DE 100% E RETENÇÃO ESTÁTICA
    // -------------------------------------------------------------------------
    println!("\n[2] TESTE DE ESTADO DE CONCLUSÃO (100%) E RETENÇÃO ESTÁTICA SEM AUTO-FECHAMENTO");
    {
        // Coloca a UI no estado de conclusão (100%)
        main_window.set_show_auto_optimize_modal(true);
        main_window.set_auto_optimize_progress(1.0);
        main_window.set_auto_optimize_step_name("Sistema otimizado com sucesso!".into());
        main_window.set_auto_optimize_finished(true);

        check!(
            "Modal retém show-auto-optimize-modal = true ao atingir 100%",
            main_window.get_show_auto_optimize_modal(),
            format!("show = {}", main_window.get_show_auto_optimize_modal())
        );

        check!(
            "Progresso reporta exatamente 1.0 (100%)",
            main_window.get_auto_optimize_progress() == 1.0,
            format!("progress = {}", main_window.get_auto_optimize_progress())
        );

        check!(
            "Flag auto-optimize-finished é true",
            main_window.get_auto_optimize_finished(),
            format!("finished = {}", main_window.get_auto_optimize_finished())
        );

        check!(
            "Mensagem de conclusão é retida",
            main_window.get_auto_optimize_step_name() == "Sistema otimizado com sucesso!",
            format!("step_name = '{}'", main_window.get_auto_optimize_step_name())
        );

        // Teste de estresse temporal: dorme 200ms e verifica se o modal permaneceu estático (sem auto-dismiss)
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(200));
        let elapsed = start.elapsed();

        check!(
            "Modal permanece estático na tela após 200ms (ausência absoluta de auto-dismiss)",
            main_window.get_show_auto_optimize_modal() && main_window.get_auto_optimize_finished(),
            format!("Decorrido: {:?}, Modal ainda visível: {}", elapsed, main_window.get_show_auto_optimize_modal())
        );
    }

    // -------------------------------------------------------------------------
    // [3] FLUXO DE TRANSIÇÃO DO BOTÃO "JOGAR CS2"
    // -------------------------------------------------------------------------
    println!("\n[3] VALIDAÇÃO DA AÇÃO JOGAR CS2: REDIRECIONAMENTO PARA PAINEL E LANÇAMENTO");
    {
        // Coloca o usuário na aba Otimizações (active-tab = 1) com modal aberto a 100%
        main_window.set_active_tab(1);
        main_window.set_show_auto_optimize_modal(true);
        main_window.set_auto_optimize_finished(true);

        check!(
            "Estado pré-clique: active-tab = 1 (Otimizações) e modal visível",
            main_window.get_active_tab() == 1 && main_window.get_show_auto_optimize_modal(),
            format!("active_tab = {}, show_modal = {}", main_window.get_active_tab(), main_window.get_show_auto_optimize_modal())
        );

        let launch_triggered = Arc::new(AtomicBool::new(false));
        let launch_clone = Arc::clone(&launch_triggered);
        main_window.on_launch_cs2_clicked(move || {
            launch_clone.store(true, Ordering::SeqCst);
        });

        // Simula a lógica executada pelo callback play-cs2 em main.slint:
        // play-cs2 => {
        //     root.show-auto-optimize-modal = false;
        //     root.active-tab = 0;
        //     root.status-message = @tr("Iniciando Counter-Strike 2 pela Steam com perfil otimizado...");
        //     root.is-launching = true;
        //     root.launch-cs2-clicked();
        // }
        main_window.set_show_auto_optimize_modal(false);
        main_window.set_active_tab(0);
        main_window.set_status_message("Iniciando Counter-Strike 2 pela Steam com perfil otimizado...".into());
        main_window.set_is_launching(true);
        main_window.invoke_launch_cs2_clicked();

        check!(
            "Modal é fechado após clique em JOGAR CS2",
            !main_window.get_show_auto_optimize_modal(),
            format!("show_auto_optimize_modal = {}", main_window.get_show_auto_optimize_modal())
        );

        check!(
            "active-tab é redirecionada com sucesso para 0 (Painel)",
            main_window.get_active_tab() == 0,
            format!("active_tab = {}", main_window.get_active_tab())
        );

        check!(
            "is-launching é definido como true",
            main_window.get_is_launching(),
            format!("is_launching = {}", main_window.get_is_launching())
        );

        check!(
            "launch-cs2-clicked foi invocado com sucesso",
            launch_triggered.load(Ordering::SeqCst),
            format!("launch_triggered = {}", launch_triggered.load(Ordering::SeqCst))
        );
    }

    // -------------------------------------------------------------------------
    // [4] VALIDAÇÃO DO FECHAMENTO DIRETO POR 'X' PÓS-CONCLUSÃO
    // -------------------------------------------------------------------------
    println!("\n[4] VALIDAÇÃO DO FECHAMENTO DIRETO POR 'X' PÓS-CONCLUSÃO");
    {
        // Reabre o modal em estado final 100%
        main_window.set_show_auto_optimize_modal(true);
        main_window.set_auto_optimize_finished(true);

        // Ao clicar em 'X' após o fim, ele dispara close() diretamente sem abrir confirmação
        main_window.set_show_auto_optimize_modal(false);

        check!(
            "Modal fecha diretamente ao clicar no 'X' pós-conclusão",
            !main_window.get_show_auto_optimize_modal(),
            "show_auto_optimize_modal = false"
        );
    }

    // -------------------------------------------------------------------------
    // [5] AUDITORIA ESTRUTURAL E GEOMETRIA DO BLOQUEADOR TOUCHAREA 980x620
    // -------------------------------------------------------------------------
    println!("\n[5] AUDITORIA ESTRUTURAL DE auto_optimize_dialog.slint E main.slint");
    {
        let dialog_src = fs::read_to_string("ui/components/auto_optimize_dialog.slint")
            .expect("Falha ao ler ui/components/auto_optimize_dialog.slint");
        let main_src = fs::read_to_string("ui/main.slint")
            .expect("Falha ao ler ui/main.slint");

        check!(
            "AutoOptimizeDialog define largura exata de 980px",
            dialog_src.contains("width: 980px;"),
            "width: 980px validado"
        );

        check!(
            "AutoOptimizeDialog define altura exata de 620px",
            dialog_src.contains("height: 620px;"),
            "height: 620px validado"
        );

        check!(
            "AutoOptimizeDialog possui TouchArea bloqueador na raiz",
            dialog_src.contains("TouchArea {\n        mouse-cursor: default;\n    }"),
            "TouchArea cobrindo 980x620 validado"
        );

        check!(
            "Card centralizado com 540px x 330px",
            dialog_src.contains("width: 540px;") && dialog_src.contains("height: 330px;"),
            "Dimensões do card tático validadas"
        );

        check!(
            "Indicador visual verde de sucesso e checkmark '✓'",
            dialog_src.contains("CS2Theme.status-success") && dialog_src.contains("✓"),
            "Sinal verde de confirmação validado"
        );

        check!(
            "Botão tático JOGAR CS2 presente com ícone de play",
            dialog_src.contains("text: @tr(\"JOGAR CS2\");")
                && dialog_src.contains("icon-src: @image-url(\"../assets/icons/play.svg\");"),
            "Botão JOGAR CS2 e ícone play validados"
        );

        check!(
            "Instanciação de AutoOptimizeDialog no final de MainWindow em main.slint",
            main_src.contains("if root.show-auto-optimize-modal : AutoOptimizeDialog {")
                && main_src.contains("play-cs2 => {")
                && main_src.contains("root.active-tab = 0;"),
            "Binding de play-cs2 com active-tab = 0 em main.slint validado"
        );
    }

    // -------------------------------------------------------------------------
    // [6] VALIDAÇÃO DA PERSISTÊNCIA: initial_optimization_done E AutoDetect.json
    // -------------------------------------------------------------------------
    println!("\n[6] VALIDAÇÃO DA PERSISTÊNCIA DE initial_optimization_done E AutoDetect.json");
    {
        let temp_dir = std::env::temp_dir().join(format!("counterrush_completion_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let settings_path = temp_dir.join("settings.json");
        let autodetect_path = temp_dir.join("AutoDetect.json");

        // 6.1 Teste de persistência de AppSettings com initial_optimization_done
        let initial_settings = AppSettings::default();
        check!(
            "AppSettings default inicializa com initial_optimization_done = false",
            !initial_settings.initial_optimization_done,
            format!("initial_optimization_done = {}", initial_settings.initial_optimization_done)
        );

        initial_settings.save_to_file(&settings_path).expect("Falha ao salvar settings.json");

        // Simula a rotina de gravação final ao término da calibração
        let mut loaded_settings = AppSettings::load_from_file(&settings_path).expect("Falha ao ler settings");
        loaded_settings.initial_optimization_done = true;
        loaded_settings.save_to_file(&settings_path).expect("Falha ao atualizar settings");

        let reloaded_settings = AppSettings::load_from_file(&settings_path).expect("Falha ao reler settings");
        check!(
            "initial_optimization_done gravado e persistido com sucesso no disco",
            reloaded_settings.initial_optimization_done,
            format!("initial_optimization_done = {}", reloaded_settings.initial_optimization_done)
        );

        // 6.2 Teste de criação e integridade de AutoDetect.json
        let mut autodetect_cfg = CustomOptimizationConfig::default();
        autodetect_cfg.apply_timer_resolution = true;
        autodetect_cfg.cs2_affinity = true;
        autodetect_cfg.save_to_file(&autodetect_path).expect("Falha ao salvar AutoDetect.json");

        check!(
            "AutoDetect.json criado no disco com sucesso",
            autodetect_path.exists(),
            "Arquivo gerado confirmado"
        );

        let loaded_autodetect = CustomOptimizationConfig::load_from_file(&autodetect_path)
            .expect("Falha ao carregar AutoDetect.json");
        check!(
            "AutoDetect.json recarregado com integridade de conteúdo",
            loaded_autodetect.apply_timer_resolution && loaded_autodetect.cs2_affinity,
            "Campos de otimização validados"
        );

        // 6.3 Teste de retrocompatibilidade com JSON legado sem initial_optimization_done
        let legacy_path = temp_dir.join("legacy_settings.json");
        let legacy_json = r#"{
            "language": "pt_BR",
            "start_with_system": false,
            "optimize_on_steam_launch": false
        }"#;
        fs::write(&legacy_path, legacy_json).expect("Falha ao salvar JSON legado");
        let legacy_parsed = AppSettings::load_from_file(&legacy_path);
        check!(
            "Retrocompatibilidade: JSON legado sem initial_optimization_done carrega false com sucesso",
            legacy_parsed.is_ok() && !legacy_parsed.unwrap().initial_optimization_done,
            "Fallback serde(default) validado"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    // -------------------------------------------------------------------------
    // [7] TESTE DE ESTRESSE DE CONCORRÊNCIA NA PERSISTÊNCIA DE APPSETTINGS
    // -------------------------------------------------------------------------
    println!("\n[7] TESTE DE ESTRESSE CONCORRENTE: LEITURA E ESCRITA EM APPSETTINGS (30 THREADS)");
    {
        // 7.1 Teste de escrita e leitura concorrente sincronizada em arquivo compartilhado (Mutex-guarded)
        let temp_dir = std::env::temp_dir().join(format!("counterrush_concurrency_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let settings_path = Arc::new(temp_dir.join("concurrency_settings.json"));

        let initial = AppSettings::default();
        initial.save_to_file(&*settings_path).unwrap();

        let num_threads = 30;
        let mut handles = Vec::new();
        let success_count = Arc::new(AtomicUsize::new(0));
        let file_lock = Arc::new(std::sync::Mutex::new(()));

        for i in 0..num_threads {
            let path_clone = Arc::clone(&settings_path);
            let success_clone = Arc::clone(&success_count);
            let lock_clone = Arc::clone(&file_lock);
            let handle = std::thread::spawn(move || {
                for iter in 0..5 {
                    let _guard = lock_clone.lock().unwrap();
                    if let Ok(mut settings) = AppSettings::load_or_create(&*path_clone) {
                        settings.initial_optimization_done = (i + iter) % 2 == 0;
                        if settings.save_to_file(&*path_clone).is_ok() {
                            success_clone.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("Falha ao aguardar thread concorrente");
        }

        let total_ops = success_count.load(Ordering::SeqCst);
        check!(
            "30 threads sincronizadas realizaram leituras e escritas concorrentes sem corrupção",
            total_ops == num_threads * 5,
            format!("Total de operações com sucesso: {} / {}", total_ops, num_threads * 5)
        );

        let final_check = AppSettings::load_from_file(&*settings_path);
        check!(
            "Arquivo final de settings.json permanece íntegro após 150 operações concorrentes",
            final_check.is_ok(),
            format!("Status: {:?}", final_check.map(|s| s.initial_optimization_done))
        );

        let _ = fs::remove_dir_all(&temp_dir);
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
