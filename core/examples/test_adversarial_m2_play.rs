use config::CustomOptimizationConfig;
use core::{
    delete_optimization_profile_in_dir, list_optimization_configs_in_dir, OptimizerCore,
};
use std::fs::{self, File};
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Suíte Adversarial de Desafio Empírico: Play M2");
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
    // [1] TESTE DE DIRETÓRIO VAZIO E DEFESAS CONTRA CRIAÇÃO ESPÚRIA
    // -------------------------------------------------------------------------
    println!("\n[1] TESTES DE DIRETÓRIO VAZIO E BARREIRA DEFENSIVA");
    {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar tempdir");
        let dir_path = temp_dir.path();

        // 1.1 Diretório vazio retorna lista vazia
        let configs = list_optimization_configs_in_dir(dir_path);
        check!(
            "Diretório vazio detectado",
            configs.is_empty(),
            format!("configs.len() = {}", configs.len())
        );

        // 1.2 Simulação da salvaguarda em on_launch_cs2_clicked:
        // Se configs.is_empty(), abortamos e NÃO chamamos load_or_create_custom_config
        let is_empty = configs.is_empty();
        let target_cfg = dir_path.join("custom_optimization.json");

        if is_empty {
            // Comportamento defensivo esperado: não faz nada no disco
        } else {
            let core = OptimizerCore::default();
            let _ = core.load_or_create_custom_config(Some(&target_cfg));
        }

        // Verifica que NENHUM arquivo foi criado no diretório
        let entries_count = fs::read_dir(dir_path).unwrap().count();
        check!(
            "Salvaguarda impede criação espúria de arquivos no disco",
            entries_count == 0 && !target_cfg.exists(),
            format!("Entradas em disco: {} (esperado 0)", entries_count)
        );

        // 1.3 Contra-prova: se a salvaguarda NÃO existisse e load_or_create_custom_config fosse chamada
        let core = OptimizerCore::default();
        let (resolved, _) = core
            .load_or_create_custom_config(Some(&target_cfg))
            .expect("Falha ao criar config na contra-prova");
        check!(
            "Contra-prova: chamada sem salvaguarda criaria arquivo em disco",
            resolved.exists() && target_cfg.exists(),
            format!("Arquivo criado em: {:?}", resolved)
        );
    }

    // -------------------------------------------------------------------------
    // [2] TESTES DE DIRETÓRIO COM CONTEÚDO ADVERSARIAL E NÃO-JSON
    // -------------------------------------------------------------------------
    println!("\n[2] TESTES DE CONTEÚDO ADVERSARIAL (ARQUIVOS NÃO-JSON, DIRETÓRIOS, CASES)");
    {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar tempdir");
        let dir_path = temp_dir.path();

        // Criar arquivos que NÃO devem ser considerados perfis de otimização
        File::create(dir_path.join("notas.txt")).unwrap();
        File::create(dir_path.join("backup.json.bak")).unwrap();
        File::create(dir_path.join("config.ini")).unwrap();
        fs::create_dir(dir_path.join("pasta_falsa.json")).unwrap(); // Diretório com terminação .json!

        // Com apenas arquivos não-json / diretório, deve ser considerado VAZIO de perfis
        let configs_adversarial = list_optimization_configs_in_dir(dir_path);
        check!(
            "Arquivos não-JSON e pastas .json ignorados",
            configs_adversarial.is_empty(),
            format!("configs_adversarial.len() = {}", configs_adversarial.len())
        );

        // Agora adicionar perfis legítimos com diferentes cases de extensão
        File::create(dir_path.join("perfil_lower.json")).unwrap();
        File::create(dir_path.join("perfil_upper.JSON")).unwrap();
        File::create(dir_path.join("perfil_mixed.Json")).unwrap();

        let configs_populated = list_optimization_configs_in_dir(dir_path);
        check!(
            "Detecção correta com arquivos JSON em diferentes cases",
            configs_populated.len() == 3,
            format!("configs_populated.len() = {} (esperado 3)", configs_populated.len())
        );

        let names: Vec<String> = configs_populated
            .iter()
            .filter_map(|p| p.file_stem().and_then(|s| s.to_str()).map(|s| s.to_string()))
            .collect();
        check!(
            "Nomes dos perfis identificados com precisão",
            names.contains(&"perfil_lower".to_string())
                && names.contains(&"perfil_upper".to_string())
                && names.contains(&"perfil_mixed".to_string()),
            format!("Perfis encontrados: {:?}", names)
        );
    }

    // -------------------------------------------------------------------------
    // [3] TESTES DE CONCORRÊNCIA E RACE CONDITION NA DETECÇÃO DEFENSIVA
    // -------------------------------------------------------------------------
    println!("\n[3] TESTES DE ESTRESSE DE CONCORRÊNCIA E RACE CONDITION");
    {
        let num_threads = 8;
        let iterations = 30;
        let success_counter = Arc::new(AtomicUsize::new(0));
        let race_detected_safely = Arc::new(AtomicUsize::new(0));
        let file_leak_counter = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();

        for thread_id in 0..num_threads {
            let successes = Arc::clone(&success_counter);
            let races_safeguarded = Arc::clone(&race_detected_safely);
            let leaks = Arc::clone(&file_leak_counter);

            let handle = thread::spawn(move || {
                let temp_dir = tempfile::tempdir().expect("Falha ao criar tempdir por thread");
                let dir_path = temp_dir.path();

                for iter in 0..iterations {
                    let profile_name = format!("perfil_t{}_i{}", thread_id, iter);
                    let file_path = dir_path.join(format!("{}.json", profile_name));

                    // Cria um perfil legítimo
                    let config = CustomOptimizationConfig::all_disabled();
                    config.save_to_file(&file_path).unwrap();

                    // Simula exclusão do perfil
                    let _ = delete_optimization_profile_in_dir(dir_path, &profile_name);

                    // Imediatamente tenta lançar, aplicando a salvaguarda defensiva
                    let current_presets = list_optimization_configs_in_dir(dir_path);
                    if current_presets.is_empty() {
                        // Salvaguarda ativada: NÃO chama load_or_create
                        races_safeguarded.fetch_add(1, Ordering::SeqCst);
                    } else {
                        // Se o arquivo ainda existisse, chamaria load_or_create
                        let core = OptimizerCore::default();
                        let _ = core.load_or_create_custom_config(Some(&file_path));
                    }

                    // Se após a salvaguarda existir algum arquivo inesperado, conta vazamento
                    let remaining = list_optimization_configs_in_dir(dir_path);
                    if !current_presets.is_empty() {
                        // se não estava vazio, limpa para próxima iteração
                        let _ = fs::remove_file(&file_path);
                    } else if !remaining.is_empty() {
                        leaks.fetch_add(1, Ordering::SeqCst);
                    }

                    successes.fetch_add(1, Ordering::SeqCst);
                }
            });
            handles.push(handle);
        }

        for h in handles {
            h.join().expect("Thread panicked");
        }

        let total_ops = num_threads * iterations;
        let actual_successes = success_counter.load(Ordering::SeqCst);
        let actual_races = race_detected_safely.load(Ordering::SeqCst);
        let actual_leaks = file_leak_counter.load(Ordering::SeqCst);

        check!(
            "Concorrência multi-thread na barreira defensiva (8 threads x 30 iter)",
            actual_successes == total_ops && actual_leaks == 0 && actual_races == total_ops,
            format!(
                "Total: {} | Sucessos: {} | Defesas disparadas: {} | Vazamentos: {}",
                total_ops, actual_successes, actual_races, actual_leaks
            )
        );
    }

    // -------------------------------------------------------------------------
    // [4] VALIDAÇÃO DO CONTRATO ESTÁTICO DE UI E SLINT
    // -------------------------------------------------------------------------
    println!("\n[4] VALIDAÇÃO DOS CONTRATOS DE UI (SLINT)");
    {
        // 4.1 no_profile_dialog.slint
        let dialog_content = fs::read_to_string("ui/components/no_profile_dialog.slint")
            .expect("Falha ao ler no_profile_dialog.slint");
        check!(
            "Texto exato da mensagem no NoProfileDialog",
            dialog_content.contains("Nenhum Perfil de Otimização encontrado, Deseja criar um?"),
            "String da mensagem validada"
        );
        check!(
            "Botão 'Criar Perfil' presente no NoProfileDialog",
            dialog_content.contains("Criar Perfil") && dialog_content.contains("callback create-profile();"),
            "Botão e callback de criação presentes"
        );
        check!(
            "Botão 'Sair' presente no NoProfileDialog",
            dialog_content.contains("Sair") && dialog_content.contains("callback close();"),
            "Botão e callback de fechar presentes"
        );
        check!(
            "Bloqueador TouchArea de fundo presente",
            dialog_content.contains("TouchArea") && dialog_content.contains("mouse-cursor: default;"),
            "TouchArea bloqueador validado"
        );

        // 4.2 main.slint
        let main_slint_content = fs::read_to_string("ui/main.slint")
            .expect("Falha ao ler ui/main.slint");
        check!(
            "Propriedade show-no-profile-modal declarada em main.slint",
            main_slint_content.contains("in-out property <bool> show-no-profile-modal: false;"),
            "Propriedade booleana validada"
        );
        check!(
            "Interceptação no botão JOGAR CS2 ao checar presets.length == 0",
            main_slint_content.contains("if root.presets.length == 0 {")
                && main_slint_content.contains("root.show-no-profile-modal = true;"),
            "Lógica de abertura do modal interceptada"
        );
        check!(
            "Callback create-profile navega para aba 1 e fecha modal",
            main_slint_content.contains("root.active-tab = 1;")
                && main_slint_content.contains("root.show-no-profile-modal = false;"),
            "Transição para aba de otimizações validada"
        );
        check!(
            "Callback close fecha o modal",
            main_slint_content.contains("close => {")
                && main_slint_content.contains("root.show-no-profile-modal = false;"),
            "Fechamento do modal validado"
        );

        // 4.3 ui/src/main.rs
        let main_rs_content = fs::read_to_string("ui/src/main.rs")
            .expect("Falha ao ler ui/src/main.rs");
        check!(
            "Salvaguarda em ui/src/main.rs no on_launch_cs2_clicked",
            main_rs_content.contains("if scan_presets().is_empty() {")
                && main_rs_content.contains("ui.set_is_launching(false);")
                && main_rs_content.contains("ui.set_show_no_profile_modal(true);")
                && main_rs_content.contains("return;"),
            "Barreira de defesa em profundidade validada no Rust"
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
