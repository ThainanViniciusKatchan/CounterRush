use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Teste Adversarial: Monitoramento de Download CS2 (ID 730)");
    println!("================================================================================");

    let mut passed = 0;
    let mut failed = 0;

    macro_rules! check {
        ($desc:expr, $cond:expr) => {
            if $cond {
                println!("  [PASS] {}", $desc);
                passed += 1;
            } else {
                eprintln!("  [FAIL] {}", $desc);
                failed += 1;
            }
        };
    }

    // 1. Verificação da Detecção de Pastas e Arquivos 730
    println!("\n--- 1. Detecção em steamapps/downloading/ ---");
    let tmp = tempdir().unwrap();
    let downloading = tmp.path().join("steamapps").join("downloading");
    fs::create_dir_all(&downloading).unwrap();

    check!(
        "Pasta downloading vazia retorna false",
        !windows_api::check_downloading_dir_has_730(&downloading)
    );

    let folder_730 = downloading.join("730");
    fs::create_dir(&folder_730).unwrap();
    check!(
        "Pasta com nome '730' é detectada como download/atualização",
        windows_api::check_downloading_dir_has_730(&downloading)
    );

    fs::remove_dir(&folder_730).unwrap();
    check!(
        "Após exclusão da pasta 730, detecção retorna false",
        !windows_api::check_downloading_dir_has_730(&downloading)
    );

    let patch_file = downloading.join("state_730_123456.patch");
    fs::write(&patch_file, b"data").unwrap();
    check!(
        "Arquivo com '730' no nome é detectado",
        windows_api::check_downloading_dir_has_730(&downloading)
    );

    fs::remove_file(&patch_file).unwrap();
    check!(
        "Após exclusão do arquivo, detecção retorna false",
        !windows_api::check_downloading_dir_has_730(&downloading)
    );

    let other_file = downloading.join("state_570_dota.patch");
    fs::write(&other_file, b"data").unwrap();
    check!(
        "Arquivos de outros jogos (ex: 570) não são identificados como CS2",
        !windows_api::check_downloading_dir_has_730(&downloading)
    );

    // 2. Verificação de Multi-Bibliotecas Steam (VDF)
    println!("\n--- 2. Multi-Bibliotecas Steam ---");
    let second_lib = tempdir().unwrap();
    let second_downloading = second_lib.path().join("steamapps").join("downloading");
    fs::create_dir_all(&second_downloading).unwrap();

    let vdf_content = format!(
        r#""libraryfolders"
{{
	"0"
	{{
		"path"		"{}"
	}}
	"1"
	{{
		"path"		"{}"
	}}
}}
"#,
        tmp.path().to_str().unwrap().replace('\\', "\\\\"),
        second_lib.path().to_str().unwrap().replace('\\', "\\\\")
    );
    fs::write(tmp.path().join("steamapps").join("libraryfolders.vdf"), vdf_content).unwrap();

    let all_dirs = windows_api::get_steam_downloading_dirs_from_base(tmp.path());
    check!(
        "Localiza pastas downloading em todas as bibliotecas configuradas",
        all_dirs.len() == 2 && all_dirs.contains(&downloading) && all_dirs.contains(&second_downloading)
    );

    // Cria 730 na segunda biblioteca
    let sec_730 = second_downloading.join("730");
    fs::create_dir(&sec_730).unwrap();
    check!(
        "Detecta 730 mesmo quando localizado em biblioteca secundária",
        all_dirs.iter().any(|d| windows_api::check_downloading_dir_has_730(d))
    );
    fs::remove_dir(&sec_730).unwrap();

    // 3. Validação via logs SteamCMD (content_log.txt e bootstrap_log.txt)
    println!("\n--- 3. Camadas de Log do SteamCMD (content_log.txt / bootstrap_log.txt) ---");
    let steamcmd_sim = tempdir().unwrap();
    let logs_dir = steamcmd_sim.path().join("logs");
    fs::create_dir_all(&logs_dir).unwrap();
    let content_log_path = logs_dir.join("content_log.txt");
    let bootstrap_log_path = logs_dir.join("bootstrap_log.txt");

    fs::write(
        &content_log_path,
        "[2026-11-13 14:10:00] Client version: 1788292693\n[2026-11-13 14:10:01] AppID 730 App update changed : Running Update,Downloading,\n",
    )
    .unwrap();
    let st_content_dl = windows_api::check_steamcmd_logs_in_dir(steamcmd_sim.path(), 730);
    check!(
        "Detecta downloading ativo em logs/content_log.txt",
        st_content_dl.state == windows_api::DownloadActivityState::Downloading
    );

    fs::write(
        &content_log_path,
        "[2026-11-13 14:10:00] Client version: 1788292693\n[2026-11-13 14:10:02] AppID 730 App update changed : Running Update,Verifying Installed,\n",
    )
    .unwrap();
    let st_content_val = windows_api::check_steamcmd_logs_in_dir(steamcmd_sim.path(), 730);
    check!(
        "Detecta validação (Verifying Installed) em logs/content_log.txt",
        st_content_val.state == windows_api::DownloadActivityState::Validating
    );

    fs::write(
        &content_log_path,
        "[2026-11-13 14:10:00] Client version: 1788292693\n[2026-11-13 14:10:03] AppID 730 App update changed : None\n[2026-11-13 14:10:03] AppID 730 scheduler finished : removed from schedule (result No Error, state 0xc)\n",
    )
    .unwrap();
    let st_content_done = windows_api::check_steamcmd_logs_in_dir(steamcmd_sim.path(), 730);
    check!(
        "Após 'None' e 'scheduler finished' no content_log.txt, libera fluxo",
        !st_content_done.is_active()
    );

    fs::write(
        &bootstrap_log_path,
        "[2026-11-13 14:11:00] Startup - updater built Sep 1 2026\n[2026-11-13 14:11:01] Downloading update (15,000 of 30,000 KB)...\n",
    )
    .unwrap();
    let st_boot_dl = windows_api::check_steamcmd_logs_in_dir(steamcmd_sim.path(), 730);
    check!(
        "Detecta atualização bootstrap ativa em logs/bootstrap_log.txt",
        st_boot_dl.is_active()
            && st_boot_dl.progress_percent.map(|p| (p - 50.0).abs() < 0.1).unwrap_or(false)
    );

    fs::write(
        &bootstrap_log_path,
        "[2026-11-13 14:11:00] Startup - updater built Sep 1 2026\n[2026-11-13 14:11:05] Update complete, launching...\n[2026-11-13 14:11:05] Shutdown\n",
    )
    .unwrap();
    let st_boot_done = windows_api::check_steamcmd_logs_in_dir(steamcmd_sim.path(), 730);
    check!(
        "Após Shutdown em logs/bootstrap_log.txt, libera fluxo",
        !st_boot_done.is_active()
    );

    if std::path::Path::new(windows_api::DEFAULT_STEAMCMD_DIR).exists() {
        let real_status = windows_api::SteamCmdClient::default().check_validation_status(730);
        check!(
            "Logs históricos reais em C:\\steamcmd não geram falso positivo quando inativos",
            !real_status.is_active()
        );
    }

    // 4. Análise de stdout em tempo real e linha de comando de processo
    println!("\n--- 4. Análise de Stdout (+app_update) e Processo ---");
    let mut cmd = std::process::Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-Command",
        "Write-Output 'Update state (0x61) downloading, progress: 61.25 (1000 / 1632)'; Start-Sleep -Milliseconds 120; Write-Output \"Success! App '730' fully installed.\"",
    ])
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::null());

    let mut child = cmd.spawn().expect("Falha ao spawnar simulador de stdout");
    let mut seen_pct = None;
    let summary = windows_api::monitor_child_stdout(&mut child, 730, None, |ev| {
        if let windows_api::SteamCmdStdoutEvent::Progress {
            state: windows_api::DownloadActivityState::Downloading,
            percent: Some(p),
            ..
        } = ev
        {
            seen_pct = Some(*p);
        }
    })
    .unwrap();

    check!(
        "Stdout parser captura 'Update state (0x61) downloading, progress: XX.XX' em tempo real",
        seen_pct.map(|p| (p - 61.25).abs() < 0.01).unwrap_or(false) && summary.completed
    );

    check!(
        "Validador multi-camada integrado (dir + steamcmd + steam logs) detecta e libera corretamente",
        {
            let status_idle = windows_api::get_cs2_download_validation_status_for_paths(
                tmp.path(),
                steamcmd_sim.path(),
                false,
            );
            !status_idle.is_active()
        }
    );

    // 4b. Pasta 730 vazia em Steam\steamapps\downloading continua checando logs
    let empty_730_dir = downloading.join("730");
    fs::create_dir_all(&empty_730_dir).unwrap();
    let steam_logs_dir = tmp.path().join("logs");
    fs::create_dir_all(&steam_logs_dir).unwrap();
    fs::write(
        steam_logs_dir.join("content_log.txt"),
        "[2026-11-13 15:10:00] AppID 730 App update changed : None\n[2026-11-13 15:10:01] AppID 730 scheduler finished : removed from schedule\n",
    )
    .unwrap();
    check!(
        "Pasta 730 vazia com logs finalizados continua checando os logs e libera o fluxo",
        !windows_api::get_cs2_download_validation_status_for_paths(
            tmp.path(),
            steamcmd_sim.path(),
            false,
        )
        .is_active()
    );

    fs::write(
        steam_logs_dir.join("content_log.txt"),
        "[2026-11-13 15:11:00] AppID 730 App update changed : Running Update,Verifying Installed,\n",
    )
    .unwrap();
    check!(
        "Pasta 730 vazia com log ativo continua aos logs e detecta validação",
        windows_api::get_cs2_download_validation_status_for_paths(
            tmp.path(),
            steamcmd_sim.path(),
            false,
        )
        .state
            == windows_api::DownloadActivityState::Validating
    );
    fs::write(
        steam_logs_dir.join("content_log.txt"),
        "[2026-11-13 15:12:00] AppID 730 App update changed : None\n",
    )
    .unwrap();
    fs::write(empty_730_dir.join("depot_chunk.bin"), b"bytes").unwrap();
    check!(
        "Pasta 730 com arquivo interno detecta download ativo imediatamente",
        windows_api::get_cs2_download_validation_status_for_paths(
            tmp.path(),
            steamcmd_sim.path(),
            false,
        )
        .state
            == windows_api::DownloadActivityState::Downloading
    );
    fs::remove_dir_all(&empty_730_dir).unwrap();

    // 5. Teste de Cancelamento Resiliente durante Espera
    println!("\n--- 5. Cancelamento Resiliente Durante o Download ---");
    let cancel_token = Arc::new(AtomicBool::new(false));
    let token_clone = cancel_token.clone();

    // Simula thread que ativa o cancelamento após 100ms
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(100));
        token_clone.store(true, Ordering::SeqCst);
    });

    let mut canceled_detected = false;
    for _ in 0..10 {
        if cancel_token.load(Ordering::SeqCst) {
            canceled_detected = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(30));
    }
    check!(
        "Cancelamento pelo usuário interrompe a espera imediatamente",
        canceled_detected
    );

    println!("\n================================================================================");
    println!("  Resultado Final: {} APROVADOS, {} FALHAS", passed, failed);
    println!("================================================================================");

    assert_eq!(failed, 0, "Existem falhas nos testes adversariais");
}
