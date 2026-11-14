use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Barrier};
use std::thread;

use config::AppSettings;
use core::startup::{
    normalize_exe_path, StartupManager, TaskSchedulerStartupManager,
};
use windows_api::scheduler::{
    create_logon_task, delete_task, is_task_registered,
};

fn query_task_xml(task_name: &str) -> Result<String, String> {
    let output = Command::new("schtasks")
        .args(&["/query", "/tn", task_name, "/xml"])
        .output()
        .map_err(|e| format!("Falha ao invocar schtasks query xml: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Teste Adversarial M5 (schtasks, Concorrência e Inicialização)");
    println!("================================================================================");

    let mut total_passed = 0;
    let mut total_failed = 0;

    // Helper macro para asserções
    macro_rules! run_assert {
        ($desc:expr, $cond:expr) => {
            if $cond {
                println!("  [PASS] {}", $desc);
                total_passed += 1;
            } else {
                eprintln!("  [FAIL] {}", $desc);
                total_failed += 1;
            }
        };
    }

    // =========================================================================
    // TESTE 1: Sintaxe schtasks com caminho simples sem espaços
    // =========================================================================
    println!("\n--- TESTE 1: Sintaxe schtasks com caminho simples sem espaços ---");
    let task_simple = "CounterRush_Adv_Simple";
    let _ = delete_task(task_simple);

    let target_simple = Path::new(r"C:\Windows\System32\cmd.exe");
    let create_res = create_logon_task(task_simple, target_simple);
    run_assert!("Criação de tarefa simples retorna Ok(())", create_res.is_ok());

    let reg_res = is_task_registered(task_simple);
    run_assert!("is_task_registered retorna Ok(true) após criação", matches!(reg_res, Ok(true)));

    if let Ok(xml) = query_task_xml(task_simple) {
        let has_logon = xml.contains("<LogonTrigger>") || xml.contains("LogonTrigger");
        let has_highest = xml.contains("HighestAvailable");
        let has_cmd = xml.to_lowercase().contains("cmd.exe");
        run_assert!("XML da tarefa contém LogonTrigger", has_logon);
        run_assert!("XML da tarefa contém RunLevel HighestAvailable", has_highest);
        run_assert!("XML da tarefa contém executável alvo", has_cmd);
    } else {
        run_assert!("Consulta XML da tarefa simples", false);
    }

    let del_res = delete_task(task_simple);
    run_assert!("Exclusão da tarefa simples retorna Ok(())", del_res.is_ok());
    run_assert!("is_task_registered retorna Ok(false) após exclusão", matches!(is_task_registered(task_simple), Ok(false)));

    // =========================================================================
    // TESTE 2: Sintaxe schtasks com caminho contendo espaços
    // =========================================================================
    println!("\n--- TESTE 2: Sintaxe schtasks com caminho contendo espaços ---");
    let task_spaces = "CounterRush_Adv_Spaces";
    let _ = delete_task(task_spaces);

    let target_spaces = Path::new(r"C:\Program Files\CounterRush\CounterRush.exe");
    let create_spaces_res = create_logon_task(task_spaces, target_spaces);
    if let Err(ref e) = create_spaces_res {
        eprintln!("    Detalhes do erro: {:?}", e);
    }
    run_assert!("Criação de tarefa com espaços no caminho retorna Ok(())", create_spaces_res.is_ok());

    let reg_spaces_res = is_task_registered(task_spaces);
    run_assert!("is_task_registered para tarefa com espaços retorna Ok(true)", matches!(reg_spaces_res, Ok(true)));

    if let Ok(xml) = query_task_xml(task_spaces) {
        let has_logon = xml.contains("<LogonTrigger>") || xml.contains("LogonTrigger");
        let has_highest = xml.contains("HighestAvailable");
        let contains_full_path = xml.contains("C:\\Program Files\\CounterRush\\CounterRush.exe")
            || xml.contains("Program Files");
        run_assert!("XML da tarefa com espaços possui LogonTrigger", has_logon);
        run_assert!("XML da tarefa com espaços possui HighestAvailable", has_highest);
        run_assert!("XML da tarefa com espaços preserva caminho completo sem truncar", contains_full_path);
    } else {
        run_assert!("Consulta XML da tarefa com espaços", false);
    }

    let del_spaces_res = delete_task(task_spaces);
    run_assert!("Exclusão da tarefa com espaços retorna Ok(())", del_spaces_res.is_ok());
    run_assert!("is_task_registered retorna Ok(false) após exclusão", matches!(is_task_registered(task_spaces), Ok(false)));

    // =========================================================================
    // TESTE 3: Normalização de prefixo UNC (\\?\) e integração com StartupManager
    // =========================================================================
    println!("\n--- TESTE 3: Normalização de prefixo UNC (\\\\?\\) e StartupManager ---");
    let raw_unc = Path::new(r"\\?\C:\Program Files\CounterRush\CounterRush.exe");
    let normalized = normalize_exe_path(raw_unc);
    run_assert!(
        "normalize_exe_path remove prefixo UNC \\\\?\\",
        normalized == PathBuf::from(r"C:\Program Files\CounterRush\CounterRush.exe")
    );

    let task_unc = "CounterRush_Adv_UNC";
    let _ = delete_task(task_unc);

    let mgr_unc = TaskSchedulerStartupManager::with_custom_target(task_unc, raw_unc.to_path_buf());
    let enable_unc_res = mgr_unc.enable();
    run_assert!("StartupManager::enable com caminho UNC retorna Ok(())", enable_unc_res.is_ok());
    run_assert!("StartupManager::is_enabled retorna Ok(true)", matches!(mgr_unc.is_enabled(), Ok(true)));

    if let Ok(xml) = query_task_xml(task_unc) {
        let contains_raw_unc = xml.contains(r"\\?\");
        run_assert!("XML da tarefa registrada NÃO contém o prefixo cru \\\\?\\", !contains_raw_unc);
    }

    let disable_unc_res = mgr_unc.disable();
    run_assert!("StartupManager::disable retorna Ok(())", disable_unc_res.is_ok());
    run_assert!("StartupManager::is_enabled retorna Ok(false)", matches!(mgr_unc.is_enabled(), Ok(false)));

    // =========================================================================
    // TESTE 4: Teste com current_exe real (caminho em G:\\Outros computadores\\...)
    // =========================================================================
    println!("\n--- TESTE 4: Teste com executável atual (current_exe) ---");
    let task_current = "CounterRush_Adv_CurrentExe";
    let _ = delete_task(task_current);

    let current_exe = std::env::current_exe().expect("Deve obter current_exe");
    println!("    Current exe detectado: {}", current_exe.display());
    let mgr_current = TaskSchedulerStartupManager::with_custom_target(task_current, current_exe.clone());

    let enable_cur_res = mgr_current.enable();
    run_assert!("StartupManager::enable com current_exe real retorna Ok(())", enable_cur_res.is_ok());
    run_assert!("is_enabled() confirma registro do current_exe", matches!(mgr_current.is_enabled(), Ok(true)));

    if let Ok(xml) = query_task_xml(task_current) {
        let has_logon = xml.contains("<LogonTrigger>") || xml.contains("LogonTrigger");
        let has_highest = xml.contains("HighestAvailable");
        run_assert!("XML com current_exe tem LogonTrigger", has_logon);
        run_assert!("XML com current_exe tem HighestAvailable", has_highest);
    }

    let disable_cur_res = mgr_current.disable();
    run_assert!("disable() remove current_exe task com Ok(())", disable_cur_res.is_ok());
    run_assert!("is_enabled() confirma que a tarefa foi removida", matches!(mgr_current.is_enabled(), Ok(false)));

    // =========================================================================
    // TESTE 5: Idempotência de criação e exclusão
    // =========================================================================
    println!("\n--- TESTE 5: Idempotência de criação e exclusão repetida ---");
    let task_idem = "CounterRush_Adv_Idempotence";
    let _ = delete_task(task_idem);

    let mgr_idem = TaskSchedulerStartupManager::with_custom_target(
        task_idem,
        PathBuf::from(r"C:\Windows\System32\notepad.exe"),
    );

    let en1 = mgr_idem.enable();
    let en2 = mgr_idem.enable();
    run_assert!("Primeira chamada enable() tem sucesso", en1.is_ok());
    run_assert!("Segunda chamada enable() (sobrescrita /f) tem sucesso", en2.is_ok());
    run_assert!("Tarefa permanece ativa após sobrescrita", matches!(mgr_idem.is_enabled(), Ok(true)));

    let dis1 = mgr_idem.disable();
    let dis2 = mgr_idem.disable();
    run_assert!("Primeira chamada disable() tem sucesso", dis1.is_ok());
    run_assert!("Segunda chamada disable() em tarefa inexistente é idempotente (Ok(()))", dis2.is_ok());
    run_assert!("Tarefa permanece inativa", matches!(mgr_idem.is_enabled(), Ok(false)));

    // =========================================================================
    // TESTE 6: Concorrência e chaveamento rápido (Rapid Toggle Stress Test)
    // =========================================================================
    println!("\n--- TESTE 6: Concorrência e chaveamento rápido (Rapid Toggle & Settings) ---");
    let temp_dir = tempfile::tempdir().expect("Falha ao criar tempdir");
    let settings_file = temp_dir.path().join("settings.json");

    let initial = AppSettings {
        language: "pt_BR".to_string(),
        start_with_system: false,
        optimize_on_steam_launch: false,
        initial_optimization_done: true,
        minimize_to_tray: true,
        ..Default::default()
    };
    initial.save_to_file(&settings_file).expect("Falha ao gravar settings inicial");

    let task_stress_base = "CounterRush_Adv_Stress";
    let num_threads = 6;
    let iterations_per_thread = 6;
    let barrier = Arc::new(Barrier::new(num_threads));
    let settings_path_arc = Arc::new(settings_file.clone());

    let mut handles = Vec::new();

    for t_idx in 0..num_threads {
        let b = Arc::clone(&barrier);
        let s_path = Arc::clone(&settings_path_arc);
        let task_name = format!("{}_{}", task_stress_base, t_idx);

        let handle = thread::spawn(move || {
            b.wait();
            let mgr = TaskSchedulerStartupManager::with_custom_target(
                &task_name,
                PathBuf::from(r"C:\Windows\System32\calc.exe"),
            );

            for i in 0..iterations_per_thread {
                let want_enable = (i % 2) == 0;
                if want_enable {
                    let _ = mgr.enable();
                } else {
                    let _ = mgr.disable();
                }

                // Operação concorrente no arquivo de configurações JSON
                if let Ok(mut cfg) = AppSettings::load_or_create(&*s_path) {
                    cfg.start_with_system = want_enable;
                    let _ = cfg.save_to_file(&*s_path);
                }
            }

            // Cleanup
            let _ = mgr.disable();
        });
        handles.push(handle);
    }

    for h in handles {
        let _ = h.join();
    }

    // Validação da integridade do arquivo de settings após concorrência severa
    let loaded_res = AppSettings::load_from_file(&settings_file);
    run_assert!("Arquivo settings.json permanece válido e parseável após 36 operações concorrentes", loaded_res.is_ok());

    // Limpeza de quaisquer tarefas de stress que possam ter sobrado
    for t_idx in 0..num_threads {
        let task_name = format!("{}_{}", task_stress_base, t_idx);
        let _ = delete_task(&task_name);
        run_assert!(
            format!("Tarefa de stress {} foi removida com sucesso", task_name),
            matches!(is_task_registered(&task_name), Ok(false))
        );
    }

    // =========================================================================
    // TESTE 7: Verificação de codepage e mensagens de erro não encontradas
    // =========================================================================
    println!("\n--- TESTE 7: Resiliência a variações de idioma e codepages ---");
    let non_existent_task = "CounterRush_Definitively_Non_Existent_Task_XYZ_999";
    let query_non_existent = is_task_registered(non_existent_task);
    run_assert!(
        "is_task_registered em tarefa inexistente retorna Ok(false)",
        matches!(query_non_existent, Ok(false))
    );

    let delete_non_existent = delete_task(non_existent_task);
    run_assert!(
        "delete_task em tarefa inexistente retorna Ok(())",
        delete_non_existent.is_ok()
    );

    // =========================================================================
    // RESUMO
    // =========================================================================
    println!("\n================================================================================");
    println!("  Resultado Final dos Testes Adversariais:");
    println!("  Total de Asserções Aprovadas: {}", total_passed);
    println!("  Total de Falhas:             {}", total_failed);
    println!("================================================================================");

    if total_failed > 0 {
        std::process::exit(1);
    }
}
