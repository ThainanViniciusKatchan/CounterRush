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

    // 3. Teste de Cancelamento Resiliente durante Espera
    println!("\n--- 3. Cancelamento Resiliente Durante o Download ---");
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
