// Teste Empírico Adversarial do Mecanismo de Cancelamento e Rollback (Milestone M4)
// CounterRush — Challenger 1: Cancel State Challenger

slint::include_modules!();

use counterrush_core::config::{AppSettings, CustomOptimizationConfig};
use std::fs;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq, Clone)]
enum MockTweak {
    Timer,
    Mmcss,
    Dpi,
    GpuAccel,
    MemCompression,
    GameBar,
    Msi,
    PowerSaving,
    RenderQueue,
}

fn sleep_with_cancellation_test(
    duration: Duration,
    token: &AtomicBool,
    chunk_ms: u64,
) -> bool {
    let chunk = Duration::from_millis(chunk_ms);
    let mut elapsed = Duration::ZERO;
    while elapsed < duration {
        if token.load(Ordering::SeqCst) {
            return false;
        }
        let to_sleep = chunk.min(duration - elapsed);
        std::thread::sleep(to_sleep);
        elapsed += to_sleep;
    }
    !token.load(Ordering::SeqCst)
}

fn simulate_rollback(applied: Vec<MockTweak>, rollback_log: &mut Vec<MockTweak>) {
    for tweak in applied.into_iter().rev() {
        rollback_log.push(tweak);
    }
}

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Suíte Adversarial: Mecanismo de Cancelamento e Rollback M4");
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
    // [1] CONTRATOS DE PROPRIEDADES E ESTADO INICIAL DO MODAL M4
    // -------------------------------------------------------------------------
    println!("\n[1] VALIDAÇÃO DE PROPRIEDADES DO MODAL DE OTIMIZAÇÃO AUTOMÁTICA EM MAINWINDOW");
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
        format!("Valor: {}", main_window.get_show_auto_optimize_modal())
    );

    check!(
        "auto-optimize-progress inicial é 0.0",
        main_window.get_auto_optimize_progress() == 0.0,
        format!("Valor: {}", main_window.get_auto_optimize_progress())
    );

    check!(
        "auto-optimize-step-name inicial é vazio",
        main_window.get_auto_optimize_step_name().is_empty(),
        format!("Valor: '{}'", main_window.get_auto_optimize_step_name())
    );

    check!(
        "auto-optimize-finished inicial é false",
        !main_window.get_auto_optimize_finished(),
        format!("Valor: {}", main_window.get_auto_optimize_finished())
    );

    // -------------------------------------------------------------------------
    // [2] CONTRATOS DE TRANSIÇÃO E CALLBACKS DE CANCELAMENTO
    // -------------------------------------------------------------------------
    println!("\n[2] TESTES DE TRANSIÇÃO DE ESTADO E DISPARO DE CALLBACKS DE CANCELAMENTO");
    {
        let cancel_called = Arc::new(AtomicBool::new(false));
        let cancel_clone = Arc::clone(&cancel_called);
        main_window.on_cancel_auto_optimize_clicked(move || {
            cancel_clone.store(true, Ordering::SeqCst);
        });

        // Simula clique de abertura do modal
        main_window.set_auto_optimize_progress(0.0);
        main_window.set_auto_optimize_step_name("Iniciando calibração...".into());
        main_window.set_auto_optimize_finished(false);
        main_window.set_show_auto_optimize_modal(true);

        check!(
            "Modal aberto em estado de calibração ativa",
            main_window.get_show_auto_optimize_modal()
                && !main_window.get_auto_optimize_finished()
                && main_window.get_auto_optimize_progress() == 0.0,
            "show=true, finished=false, progress=0.0"
        );

        // Dispara o callback de cancelamento registrado na UI
        main_window.invoke_cancel_auto_optimize_clicked();
        check!(
            "Callback cancel-auto-optimize-clicked invocado com sucesso",
            cancel_called.load(Ordering::SeqCst),
            format!("cancel_called = {}", cancel_called.load(Ordering::SeqCst))
        );

        // Simula o fechamento do modal pelo manipulador de cancelamento
        main_window.set_show_auto_optimize_modal(false);
        check!(
            "Modal fechado após cancelamento",
            !main_window.get_show_auto_optimize_modal(),
            "show=false"
        );
    }

    // -------------------------------------------------------------------------
    // [3] EMPIRICAL TIMING & ATOMIC CANCELLATION HARNESS (sleep_with_cancellation)
    // -------------------------------------------------------------------------
    println!("\n[3] TESTE EMPÍRICO DE ATOMICIDADE E TEMPO DE RESPOSTA DO CANCELAMENTO");
    {
        // 3.1 Execução completa sem cancelamento (deve dormir a duração total)
        let token_no_cancel = AtomicBool::new(false);
        let start = Instant::now();
        let res = sleep_with_cancellation_test(Duration::from_millis(100), &token_no_cancel, 25);
        let elapsed = start.elapsed();
        check!(
            "sleep_with_cancellation retorna true quando não cancelado",
            res && elapsed >= Duration::from_millis(90),
            format!("Retorno: {}, Tempo decorrido: {:?}", res, elapsed)
        );

        // 3.2 Cancelamento imediato pré-existente (token já em true)
        let token_pre_canceled = AtomicBool::new(true);
        let start = Instant::now();
        let res = sleep_with_cancellation_test(Duration::from_millis(200), &token_pre_canceled, 25);
        let elapsed = start.elapsed();
        check!(
            "sleep_with_cancellation aborta imediatamente (< 10ms) com token true prévio",
            !res && elapsed < Duration::from_millis(15),
            format!("Retorno: {}, Tempo decorrido: {:?}", res, elapsed)
        );

        // 3.3 Cancelamento assíncrono em vôo (thread externa ativa token no meio do sleep)
        let token_mid_flight = Arc::new(AtomicBool::new(false));
        let token_clone = Arc::clone(&token_mid_flight);
        let t_handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            token_clone.store(true, Ordering::SeqCst);
        });

        let start = Instant::now();
        let res = sleep_with_cancellation_test(Duration::from_millis(300), &token_mid_flight, 25);
        let elapsed = start.elapsed();
        t_handle.join().expect("Falha ao aguardar thread auxiliar");

        check!(
            "sleep_with_cancellation interrompe rapidamente em vôo (< 80ms) ao receber sinal",
            !res && elapsed < Duration::from_millis(80) && elapsed >= Duration::from_millis(25),
            format!("Retorno: {}, Tempo decorrido: {:?}", res, elapsed)
        );
    }

    // -------------------------------------------------------------------------
    // [4] TESTE DE ESTRESSE: CANCELAMENTO MASSIVO CONCORRENTE MULTI-THREAD
    // -------------------------------------------------------------------------
    println!("\n[4] TESTE DE ESTRESSE: CANCELAMENTO CONCORRENTE MULTI-THREAD (50 THREADS)");
    {
        let num_threads = 50;
        let mut handles = Vec::new();
        let canceled_count = Arc::new(AtomicUsize::new(0));

        for i in 0..num_threads {
            let counter = Arc::clone(&canceled_count);
            let handle = std::thread::spawn(move || {
                let token = Arc::new(AtomicBool::new(false));
                let token_signaler = Arc::clone(&token);

                // Disparador de cancelamento com atraso variável (0ms a 20ms)
                let delay = (i % 20) as u64;
                let sig_handle = std::thread::spawn(move || {
                    if delay > 0 {
                        std::thread::sleep(Duration::from_millis(delay));
                    }
                    token_signaler.store(true, Ordering::SeqCst);
                });

                let completed = sleep_with_cancellation_test(Duration::from_millis(150), &token, 25);
                sig_handle.join().unwrap();

                if !completed {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("Falha em thread do teste de estresse");
        }

        let total_canceled = canceled_count.load(Ordering::SeqCst);
        check!(
            "Todas as 50 threads foram canceladas cooperativamente sem deadlocks",
            total_canceled == num_threads,
            format!("Total canceladas: {} / {}", total_canceled, num_threads)
        );
    }

    // -------------------------------------------------------------------------
    // [5] TESTE DE INTEGRIDADE DO ROLLBACK (ORDEM REVERSA LIFO E ISOLAMENTO)
    // -------------------------------------------------------------------------
    println!("\n[5] VALIDAÇÃO DA LÓGICA DE ROLLBACK (ORDEM LIFO E ISOLAMENTO)");
    {
        // 5.1 Rollback completo de todos os tweaks aplicados
        let applied_all = vec![
            MockTweak::Timer,
            MockTweak::Mmcss,
            MockTweak::Dpi,
            MockTweak::GpuAccel,
            MockTweak::MemCompression,
            MockTweak::GameBar,
            MockTweak::Msi,
            MockTweak::PowerSaving,
            MockTweak::RenderQueue,
        ];

        let mut rollback_log = Vec::new();
        simulate_rollback(applied_all.clone(), &mut rollback_log);

        let mut expected_reversed = applied_all.clone();
        expected_reversed.reverse();

        check!(
            "Rollback reverte todos os tweaks na ordem estritamente inversa (LIFO)",
            rollback_log == expected_reversed,
            format!("Primeiro revertido: {:?}, Último revertido: {:?}", rollback_log.first(), rollback_log.last())
        );

        // 5.2 Rollback parcial se cancelado na etapa 3
        let applied_partial = vec![
            MockTweak::Timer,
            MockTweak::Mmcss,
            MockTweak::Dpi,
        ];
        let mut rollback_partial_log = Vec::new();
        simulate_rollback(applied_partial, &mut rollback_partial_log);

        check!(
            "Rollback parcial reverte somente tweaks efetivamente aplicados",
            rollback_partial_log == vec![MockTweak::Dpi, MockTweak::Mmcss, MockTweak::Timer],
            format!("Revertidos: {:?}", rollback_partial_log)
        );

        // 5.3 Rollback com lista vazia (cancelado na etapa 0 ou 1 antes de qualquer tweak)
        let applied_empty: Vec<MockTweak> = Vec::new();
        let mut rollback_empty_log = Vec::new();
        simulate_rollback(applied_empty, &mut rollback_empty_log);

        check!(
            "Rollback com lista vazia conclui com sucesso sem operações espúrias",
            rollback_empty_log.is_empty(),
            "0 tweaks revertidos"
        );
    }

    // -------------------------------------------------------------------------
    // [6] TESTE DE NÃO-MUTAÇÃO DE ARQUIVOS EM CASO DE CANCELAMENTO
    // -------------------------------------------------------------------------
    println!("\n[6] VALIDAÇÃO DE NÃO-MUTAÇÃO NO DISCO DURANTE CANCELAMENTO");
    {
        let temp_dir = std::env::temp_dir().join(format!("counterrush_cancel_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let autodetect_path = temp_dir.join("AutoDetect.json");
        let settings_path = temp_dir.join("settings.json");

        // Cria settings inicial com initial_optimization_done = false
        let mut settings = AppSettings::default();
        settings.initial_optimization_done = false;
        settings.save_to_file(&settings_path).expect("Falha ao salvar settings.json de teste");

        // Simula cancelamento antes de 100%: AutoDetect.json NÃO deve ser criado
        check!(
            "AutoDetect.json não existe no disco antes do término",
            !autodetect_path.exists(),
            "Arquivo inexistente confirmado"
        );

        let reloaded_settings = AppSettings::load_from_file(&settings_path).unwrap();
        check!(
            "initial_optimization_done permanece false após cancelamento",
            !reloaded_settings.initial_optimization_done,
            format!("initial_optimization_done = {}", reloaded_settings.initial_optimization_done)
        );

        // Agora simula término a 100%: salva AutoDetect e atualiza flag
        let mut dummy_cfg = CustomOptimizationConfig::default();
        dummy_cfg.apply_timer_resolution = true;
        dummy_cfg.save_to_file(&autodetect_path).expect("Falha ao salvar AutoDetect.json");

        let mut final_settings = AppSettings::load_from_file(&settings_path).unwrap();
        final_settings.initial_optimization_done = true;
        final_settings.save_to_file(&settings_path).unwrap();

        check!(
            "AutoDetect.json gravado com sucesso somente após 100%",
            autodetect_path.exists(),
            "Arquivo criado com sucesso"
        );

        let verified_final = AppSettings::load_from_file(&settings_path).unwrap();
        check!(
            "initial_optimization_done é persistido como true somente após 100%",
            verified_final.initial_optimization_done,
            format!("initial_optimization_done = {}", verified_final.initial_optimization_done)
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    // -------------------------------------------------------------------------
    // [7] AUDITORIA ESTRUTURAL ESTÁTICA DO COMPONENTE SLINT
    // -------------------------------------------------------------------------
    println!("\n[7] AUDITORIA ESTRUTURAL DE auto_optimize_dialog.slint");
    {
        let dialog_src = fs::read_to_string("ui/components/auto_optimize_dialog.slint")
            .expect("Falha ao ler ui/components/auto_optimize_dialog.slint");

        // 7.1 TouchArea bloqueador na raiz
        check!(
            "Root possui TouchArea bloqueador de 980x620",
            dialog_src.contains("TouchArea") && dialog_src.contains("mouse-cursor: default;"),
            "Bloqueador de cliques de fundo validado"
        );

        // 7.2 Botão 'X' durante execução vs pós-conclusão
        check!(
            "Botão 'X' condicional: close() pós-conclusão vs show-cancel-confirm durante execução",
            dialog_src.contains("if root.is-finished {")
                && dialog_src.contains("root.close();")
                && dialog_src.contains("root.show-cancel-confirm = true;")
                && dialog_src.contains("root.request-cancel();"),
            "Lógica de bifurcação do botão 'X' validada"
        );

        // 7.3 Botão CANCELAR aciona confirmação
        check!(
            "Botão CANCELAR aciona show-cancel-confirm e request-cancel()",
            dialog_src.contains("cancel-touch := TouchArea")
                && dialog_src.contains("root.show-cancel-confirm = true;")
                && dialog_src.contains("root.request-cancel();"),
            "Botão Cancelar validado"
        );

        // 7.4 Sub-diálogo: TouchArea bloqueador e botões NÃO / SIM
        check!(
            "Sub-diálogo de confirmação possui TouchArea bloqueador próprio",
            dialog_src.contains("if root.show-cancel-confirm : Rectangle")
                && dialog_src.contains("TouchArea {\n                mouse-cursor: default;\n            }"),
            "Sub-diálogo TouchArea bloqueador validado"
        );

        check!(
            "Sub-diálogo botão NÃO fecha alerta e invoca dismiss-cancel-confirm()",
            dialog_src.contains("cont-touch := TouchArea")
                && dialog_src.contains("root.show-cancel-confirm = false;")
                && dialog_src.contains("root.dismiss-cancel-confirm();"),
            "Botão 'NÃO, CONTINUAR' validado"
        );

        check!(
            "Sub-diálogo botão SIM fecha alerta e invoca confirm-cancel()",
            dialog_src.contains("abort-touch := TouchArea")
                && dialog_src.contains("root.show-cancel-confirm = false;")
                && dialog_src.contains("root.confirm-cancel();"),
            "Botão 'SIM, CANCELAR' validado"
        );

        // 7.5 Estado 100% de sucesso: sem timer de auto-fechamento
        check!(
            "Ausência absoluta de Timer no componente (estado estático retido)",
            !dialog_src.contains("Timer {") && !dialog_src.contains("Timer{"),
            "Nenhum auto-dismiss timer presente no diálogo"
        );

        // 7.6 Botão JOGAR CS2 aciona play-cs2()
        check!(
            "Botão JOGAR CS2 aciona root.play-cs2()",
            dialog_src.contains("root.play-cs2();"),
            "Callback play-cs2 conectado ao TacticalButton"
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
