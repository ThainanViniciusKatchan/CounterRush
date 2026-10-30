use config::CustomOptimizationConfig;
use core::{list_optimization_configs_in_dir, validate_profile_name};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    println!("================================================================================");
    println!("     CounterRush — Suíte Adversarial de Desafio Empírico (Milestone 2)");
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

    println!("\n[1] TESTES EXAUSTIVOS DE CASE-INSENSITIVITY DA EXTENSÃO .JSON");

    // Todas as 16 permutações binárias de maiúsculas e minúsculas para "json"
    let mut json_permutations = Vec::new();
    for i in 0..16 {
        let chars = ['j', 's', 'o', 'n'];
        let mut perm = String::new();
        for (bit, &c) in chars.iter().enumerate() {
            if (i & (1 << bit)) != 0 {
                perm.push(c.to_ascii_uppercase());
            } else {
                perm.push(c);
            }
        }
        json_permutations.push(perm);
    }

    for perm in &json_permutations {
        // Teste de nome válido com sufixo
        let input = format!("perfil_teste.{}", perm);
        let res = validate_profile_name(&input);
        check!(
            &format!("Remoção de sufixo .{}", perm),
            res == Ok("perfil_teste".to_string()),
            format!("input: '{}' -> {:?}", input, res)
        );

        // Teste com espaços antes e depois
        let input_spaces = format!("   perfil_espacado.{}   ", perm);
        let res_spaces = validate_profile_name(&input_spaces);
        check!(
            &format!("Trim + remoção de sufixo .{}", perm),
            res_spaces == Ok("perfil_espacado".to_string()),
            format!("input: '{}' -> {:?}", input_spaces, res_spaces)
        );

        // Teste de sufixo isolado sem nome (deve falhar)
        let bare_ext = format!(".{}", perm);
        let res_bare = validate_profile_name(&bare_ext);
        check!(
            &format!("Rejeição de extensão pura .{}", perm),
            res_bare.is_err(),
            format!("input: '{}' -> {:?}", bare_ext, res_bare)
        );

        let bare_ext_spaces = format!("  .{}  ", perm);
        let res_bare_spaces = validate_profile_name(&bare_ext_spaces);
        check!(
            &format!("Rejeição de extensão pura com espaços ' .{} '", perm),
            res_bare_spaces.is_err(),
            format!("input: '{}' -> {:?}", bare_ext_spaces, res_bare_spaces)
        );
    }

    println!("\n[2] TESTES DE CASOS DE BORDA E NOMES MALFORMADOS");

    // Nomes com .json no meio do nome
    let res = validate_profile_name("meu.json.perfil");
    check!(
        "Extensão .json no meio da string",
        res == Ok("meu.json.perfil".to_string()),
        format!("{:?}", res)
    );

    let res = validate_profile_name("meu.JSON.perfil");
    check!(
        "Extensão .JSON no meio da string",
        res == Ok("meu.JSON.perfil".to_string()),
        format!("{:?}", res)
    );

    // Múltiplos pontos
    let res = validate_profile_name("perfil.v1.0.beta.json");
    check!(
        "Múltiplos pontos no nome com .json final",
        res == Ok("perfil.v1.0.beta".to_string()),
        format!("{:?}", res)
    );

    // Sufixos duplicados
    let res = validate_profile_name("perfil.json.JSON");
    check!(
        "Sufixo duplicado .json.JSON",
        res == Ok("perfil.json".to_string()),
        format!("{:?}", res)
    );

    // Nomes vazios e espaços
    check!("String vazia", validate_profile_name("").is_err(), "Rejeitado");
    check!("Apenas espaços", validate_profile_name("     ").is_err(), "Rejeitado");
    check!("Apenas tabs e quebras", validate_profile_name("\t\r\n").is_err(), "Rejeitado");

    // Todos os 9 caracteres proibidos pelo NTFS/Windows
    let illegal_chars = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
    for c in illegal_chars {
        let name = format!("teste{}invalido", c);
        let res = validate_profile_name(&name);
        check!(
            &format!("Caractere NTFS proibido '{}'", c),
            res.is_err(),
            format!("{:?}", res)
        );
    }

    // Nomes reservados de dispositivos do Windows
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    for name in reserved {
        check!(
            &format!("Dispositivo reservado maiúsculo {}", name),
            validate_profile_name(name).is_err(),
            "Rejeitado"
        );
        check!(
            &format!("Dispositivo reservado minúsculo {}", name.to_lowercase()),
            validate_profile_name(&name.to_lowercase()).is_err(),
            "Rejeitado"
        );
        check!(
            &format!("Dispositivo reservado com .json '{}.json'", name),
            validate_profile_name(&format!("{}.json", name)).is_err(),
            "Rejeitado"
        );
        check!(
            &format!("Dispositivo reservado com .JSON '{}.JSON'", name),
            validate_profile_name(&format!("{}.JSON", name)).is_err(),
            "Rejeitado"
        );
    }

    println!("\n[3] TESTES DE ESTRESSE DE CONCORRÊNCIA E RACE CONDITIONS");

    let num_threads = 16;
    let iterations_per_thread = 50;
    let success_counter = Arc::new(AtomicUsize::new(0));
    let failure_counter = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for thread_idx in 0..num_threads {
        let successes = Arc::clone(&success_counter);
        let failures = Arc::clone(&failure_counter);

        let handle = thread::spawn(move || {
            let temp_dir = match tempfile::tempdir() {
                Ok(td) => td,
                Err(_) => {
                    failures.fetch_add(1, Ordering::SeqCst);
                    return;
                }
            };

            for iter in 0..iterations_per_thread {
                let profile_raw_name = format!("thread_{}_iter_{}.JSON", thread_idx, iter);
                let valid_name = match validate_profile_name(&profile_raw_name) {
                    Ok(n) => n,
                    Err(_) => {
                        failures.fetch_add(1, Ordering::SeqCst);
                        continue;
                    }
                };

                let file_path = temp_dir.path().join(format!("{}.json", valid_name));

                let mut config = CustomOptimizationConfig::all_disabled();
                config.launch_args = vec![format!("-thread_{}", thread_idx), format!("-iter_{}", iter)];
                config.apply_timer_resolution = (iter % 2) == 0;
                config.disable_hpet = (iter % 3) == 0;
                config.cs2_priority = true;

                if config.save_to_file(&file_path).is_err() {
                    failures.fetch_add(1, Ordering::SeqCst);
                    continue;
                }

                let loaded = match CustomOptimizationConfig::load_from_file(&file_path) {
                    Ok(l) => l,
                    Err(_) => {
                        failures.fetch_add(1, Ordering::SeqCst);
                        continue;
                    }
                };

                if loaded != config {
                    failures.fetch_add(1, Ordering::SeqCst);
                    continue;
                }

                let listed = list_optimization_configs_in_dir(temp_dir.path());
                if !listed.contains(&file_path) {
                    failures.fetch_add(1, Ordering::SeqCst);
                    continue;
                }

                successes.fetch_add(1, Ordering::SeqCst);
            }
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().expect("Falha ao aguardar thread concorrente");
    }

    let total_concurrent_ops = num_threads * iterations_per_thread;
    let actual_successes = success_counter.load(Ordering::SeqCst);
    let actual_failures = failure_counter.load(Ordering::SeqCst);

    check!(
        "Concorrência multi-thread (16 threads x 50 iterações)",
        actual_failures == 0 && actual_successes == total_concurrent_ops,
        format!(
            "Total esperado: {} | Sucessos: {} | Falhas: {}",
            total_concurrent_ops, actual_successes, actual_failures
        )
    );

    println!("\n[4] TESTE DE ISOLAMENTO E NÃO-POLUIÇÃO EM DISCO");

    // Verificar se config/optimzations contém apenas os 2 arquivos legítimos
    let real_opt_dir = std::path::Path::new("config/optimzations");
    if real_opt_dir.exists() {
        let files = list_optimization_configs_in_dir(real_opt_dir);
        let names: Vec<String> = files
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()))
            .collect();
        println!("  [INFO-DISK] Arquivos no diretório real de otimizações: {:?}", names);
        check!(
            "Diretório de produção intocado",
            !names.iter().any(|n| n.starts_with("perfil_") || n.starts_with("thread_") || n.starts_with("ultra_fps")),
            "Nenhum arquivo de teste vazou para config/optimzations"
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
