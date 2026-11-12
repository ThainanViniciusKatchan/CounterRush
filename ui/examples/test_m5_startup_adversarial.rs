// Teste Adversarial M5: Slint UI, Callback de Startup, Reconciliação e Manifesto PE
slint::include_modules!();

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Teste Adversarial M5: Slint UI & Startup Contracts");
    println!("================================================================================");

    let mut total_passed = 0;
    let mut total_failed = 0;

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
    // 1. Instanciação e Contratos de Propriedade
    // =========================================================================
    println!("\n--- 1. Instanciação e Contratos de Propriedade ---");
    let main_window_res = MainWindow::new();
    run_assert!("MainWindow instanciada com sucesso", main_window_res.is_ok());
    let main_window = main_window_res.unwrap();

    run_assert!(
        "start-with-system inicial default é false",
        !main_window.get_start_with_system()
    );

    main_window.set_start_with_system(true);
    run_assert!(
        "set_start_with_system(true) reflete no getter",
        main_window.get_start_with_system()
    );

    main_window.set_start_with_system(false);
    run_assert!(
        "set_start_with_system(false) reflete no getter",
        !main_window.get_start_with_system()
    );

    // =========================================================================
    // 2. Teste de Callback e Chaveamento Rápido na UI
    // =========================================================================
    println!("\n--- 2. Callback e Chaveamento Rápido na UI ---");
    let toggle_count = Arc::new(AtomicUsize::new(0));
    let last_state = Arc::new(AtomicBool::new(false));

    let count_clone = Arc::clone(&toggle_count);
    let state_clone = Arc::clone(&last_state);

    main_window.on_start_with_system_toggled(move |enabled| {
        count_clone.fetch_add(1, Ordering::SeqCst);
        state_clone.store(enabled, Ordering::SeqCst);
    });

    // Dispara 50 alternâncias rápidas
    for i in 0..50 {
        let want = (i % 2) == 0;
        main_window.invoke_start_with_system_toggled(want);
    }

    run_assert!(
        "Callback start-with-system-toggled foi invocado 50 vezes",
        toggle_count.load(Ordering::SeqCst) == 50
    );
    run_assert!(
        "Último estado recebido pelo callback é false (50a alternância, i=49)",
        !last_state.load(Ordering::SeqCst)
    );

    // =========================================================================
    // 3. Simulação de Rollback em Falha do Backend
    // =========================================================================
    println!("\n--- 3. Simulação de Rollback em Falha ---");
    // Usuário liga a opção
    main_window.set_start_with_system(true);
    run_assert!("Estado antes do erro: true", main_window.get_start_with_system());

    // Backend falha e executa rollback: ui.set_start_with_system(!enabled) onde enabled=true
    let failed_attempt_enabled = true;
    main_window.set_start_with_system(!failed_attempt_enabled);
    run_assert!(
        "Estado após rollback defensivo: revertido para false",
        !main_window.get_start_with_system()
    );

    // =========================================================================
    // 4. Auditoria do Arquivo de Manifesto UAC (ui/counterrush.exe.manifest)
    // =========================================================================
    println!("\n--- 4. Auditoria Forense do Manifesto XML UAC ---");
    let manifest_path = Path::new("ui/counterrush.exe.manifest");
    run_assert!("Arquivo ui/counterrush.exe.manifest existe", manifest_path.exists());

    if let Ok(content) = fs::read_to_string(manifest_path) {
        let has_admin = content.contains(r#"level="requireAdministrator""#);
        let has_ui_access = content.contains(r#"uiAccess="false""#);
        let has_win10_compat = content.contains(r#"{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"#);
        let has_dpi_permonitor2 = content.contains("permonitorv2");
        let has_common_controls = content.contains("Microsoft.Windows.Common-Controls");

        run_assert!("Manifesto contém requireAdministrator", has_admin);
        run_assert!("Manifesto contém uiAccess=\"false\"", has_ui_access);
        run_assert!("Manifesto contém GUID compatível com Win10/11", has_win10_compat);
        run_assert!("Manifesto contém Per-Monitor DPI v2", has_dpi_permonitor2);
        run_assert!("Manifesto contém dependência do Common-Controls 6.0", has_common_controls);
    } else {
        run_assert!("Leitura do arquivo de manifesto", false);
    }

    // =========================================================================
    // 5. Auditoria de Integração em ui/build.rs
    // =========================================================================
    println!("\n--- 5. Auditoria do Script de Compilação ui/build.rs ---");
    let build_rs_path = Path::new("ui/build.rs");
    if let Ok(build_content) = fs::read_to_string(build_rs_path) {
        let sets_manifest = build_content.contains(r#"set_manifest_file("counterrush.exe.manifest")"#);
        let sets_icon = build_content.contains("set_icon");
        run_assert!("ui/build.rs configura set_manifest_file", sets_manifest);
        run_assert!("ui/build.rs preserva set_icon", sets_icon);
    } else {
        run_assert!("Leitura do ui/build.rs", false);
    }

    // =========================================================================
    // 6. Verificação de Metadados PE no Binário Compilado (se disponível)
    // =========================================================================
    println!("\n--- 6. Verificação de Metadados PE no Binário ---");
    let exe_path = Path::new(r"C:\cargo-target\CounterRush\debug\counterrush-ui.exe");
    if exe_path.exists() {
        if let Ok(binary_bytes) = fs::read(exe_path) {
            let binary_str = String::from_utf8_lossy(&binary_bytes);
            let has_req_admin = binary_str.contains("requireAdministrator");
            let has_dpi = binary_str.contains("permonitorv2");
            run_assert!("Binário PE contém string 'requireAdministrator' incorporada", has_req_admin);
            run_assert!("Binário PE contém 'permonitorv2' incorporado", has_dpi);
        } else {
            run_assert!("Leitura do binário counterrush-ui.exe", false);
        }
    } else {
        println!("  [SKIP] counterrush-ui.exe não encontrado em C:\\cargo-target\\CounterRush\\debug\\ (teste com binário compilado executável)");
    }

    // =========================================================================
    // RESUMO
    // =========================================================================
    println!("\n================================================================================");
    println!("  Resultado Final dos Testes Adversariais de UI:");
    println!("  Total de Asserções Aprovadas: {}", total_passed);
    println!("  Total de Falhas:             {}", total_failed);
    println!("================================================================================");

    if total_failed > 0 {
        std::process::exit(1);
    }
}
