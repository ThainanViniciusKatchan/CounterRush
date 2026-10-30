use config::CustomOptimizationConfig;
use core::validate_profile_name;
use std::fs;
use std::path::PathBuf;

fn main() {
    println!("================================================================================");
    println!("     CounterRush — Suíte Adversarial de Verificação Empírica (Milestone 1)");
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

    println!("\n[1] TESTES DE VALIDAÇÃO DE NOME DE PERFIL (`validate_profile_name`)");

    // 1.1 Nomes válidos básicos
    let res = validate_profile_name("meu_perfil");
    check!("Nome simples", res == Ok("meu_perfil".to_string()), format!("{:?}", res));

    let res = validate_profile_name("  com_espacos_laterais  ");
    check!("Espaços laterais", res == Ok("com_espacos_laterais".to_string()), format!("{:?}", res));

    let res = validate_profile_name("Perfil CS2 2026");
    check!("Espaços internos", res == Ok("Perfil CS2 2026".to_string()), format!("{:?}", res));

    // 1.2 Tratamento de extensão .json redundante
    let res = validate_profile_name("config.json");
    check!("Sufixo .json minúsculo", res == Ok("config".to_string()), format!("{:?}", res));

    let res = validate_profile_name("CONFIG.JSON");
    check!("Sufixo .JSON maiúsculo", res == Ok("CONFIG".to_string()), format!("{:?}", res));

    let res = validate_profile_name("  teste.json  ");
    check!("Sufixo .json com trim", res == Ok("teste".to_string()), format!("{:?}", res));

    // 1.3 Nomes vazios ou formados apenas por espaços / extensão
    let res = validate_profile_name("");
    check!("String vazia", res.is_err(), format!("{:?}", res));

    let res = validate_profile_name("     ");
    check!("Apenas espaços", res.is_err(), format!("{:?}", res));

    let res = validate_profile_name(".json");
    check!("Apenas '.json'", res.is_err(), format!("{:?}", res));

    let res = validate_profile_name(".JSON");
    check!("Apenas '.JSON'", res.is_err(), format!("{:?}", res));

    let res = validate_profile_name("   .json   ");
    check!("Espaços com '.json'", res.is_err(), format!("{:?}", res));

    // 1.4 Caracteres proibidos do Windows (\ / : * ? " < > |)
    let illegal_chars = ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];
    for c in illegal_chars {
        let name = format!("perfil{}teste", c);
        let res = validate_profile_name(&name);
        check!(
            &format!("Caractere ilegal '{}'", c),
            res.is_err(),
            format!("Rejeitado adequadamente: {:?}", res)
        );
    }

    // 1.5 Nomes de dispositivos reservados do Windows
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    for name in reserved_names {
        // Teste minúsculo e maiúsculo
        let res_upper = validate_profile_name(name);
        check!(
            &format!("Dispositivo reservado {}", name),
            res_upper.is_err(),
            format!("{:?}", res_upper)
        );

        let res_lower = validate_profile_name(&name.to_lowercase());
        check!(
            &format!("Dispositivo reservado minúsculo {}", name.to_lowercase()),
            res_lower.is_err(),
            format!("{:?}", res_lower)
        );

        // Com extensão .json redundante
        let res_with_ext = validate_profile_name(&format!("{}.json", name));
        check!(
            &format!("Dispositivo reservado com .json '{}.json'", name),
            res_with_ext.is_err(),
            format!("{:?}", res_with_ext)
        );
    }

    // 1.6 Casos limítrofes adversariais específicos
    // Extensão com casing misto (.Json, .jSoN)
    let res = validate_profile_name("perfil.Json");
    println!("  [INFO-ADVERSARIAL] validate_profile_name('perfil.Json') -> {:?}", res);
    let res = validate_profile_name("perfil.jSoN");
    println!("  [INFO-ADVERSARIAL] validate_profile_name('perfil.jSoN') -> {:?}", res);

    // Ponto duplo ou sufixo duplicado
    let res = validate_profile_name("perfil.json.json");
    check!("Sufixo duplicado .json.json", res == Ok("perfil.json".to_string()), format!("{:?}", res));

    println!("\n[2] TESTES DO MODELO `CustomOptimizationConfig::all_disabled()`");

    let disabled = CustomOptimizationConfig::all_disabled();
    check!("launch_args vazio", disabled.launch_args.is_empty(), format!("{:?}", disabled.launch_args));
    check!("empty_standby_list == false", !disabled.empty_standby_list, "Desativado");
    check!("cs2_priority == false", !disabled.cs2_priority, "Desativado");
    check!("cs2_affinity == false", !disabled.cs2_affinity, "Desativado");
    check!("disable_gpu_acceleration == false", !disabled.disable_gpu_acceleration, "Desativado");
    check!("apply_dpi_optimization == false", !disabled.apply_dpi_optimization, "Desativado");
    check!("disable_memory_compression == false", !disabled.disable_memory_compression, "Desativado");
    check!("apply_timer_resolution == false", !disabled.apply_timer_resolution, "Desativado");
    check!("apply_mmcss == false", !disabled.apply_mmcss, "Desativado");
    check!("disable_hpet == false", !disabled.disable_hpet, "Desativado");
    check!("disable_game_bar == false", !disabled.disable_game_bar, "Desativado");
    check!("enable_msi == false", !disabled.enable_msi, "Desativado");
    check!("disable_power_saving == false", !disabled.disable_power_saving, "Desativado");
    check!("apply_render_queue == false", !disabled.apply_render_queue, "Desativado");

    let def = CustomOptimizationConfig::default();
    check!("all_disabled() != default()", disabled != def, "Garantida distinção do padrão ativo");

    println!("\n[3] TESTES DE SERIALIZAÇÃO E DESSERIALIZAÇÃO JSON");

    // 3.1 Roundtrip de all_disabled()
    let json_str = serde_json::to_string_pretty(&disabled).expect("Serialização falhou");
    let deserialized: CustomOptimizationConfig = serde_json::from_str(&json_str).expect("Desserialização falhou");
    check!("Roundtrip all_disabled() PartialEq", disabled == deserialized, "Identidade preservada");

    // 3.2 Desserialização de JSON vazio {}
    let empty_json = "{}";
    let deserialized_empty: Result<CustomOptimizationConfig, _> = serde_json::from_str(empty_json);
    check!("Desserialização de JSON vazio {}", deserialized_empty.is_ok(), format!("{:?}", deserialized_empty));
    if let Ok(cfg) = deserialized_empty {
        check!("JSON vazio aplica serde(default)", cfg == CustomOptimizationConfig::default(), "Padrão aplicado");
    }

    // 3.3 Desserialização de JSON com campos desconhecidos (tolerância de schema)
    let extra_fields_json = r#"{"launch_args":[],"unknown_field_123":true,"cs2_priority":false}"#;
    let res_extra: Result<CustomOptimizationConfig, _> = serde_json::from_str(extra_fields_json);
    check!("Tolerância a campos desconhecidos", res_extra.is_ok(), format!("{:?}", res_extra));

    // 3.4 Desserialização de JSON corrompido / sintaxe inválida
    let corrupted_json = r#"{"launch_args": [incomplete"#;
    let res_corrupted: Result<CustomOptimizationConfig, _> = serde_json::from_str(corrupted_json);
    check!("Rejeição graciosa de JSON corrompido", res_corrupted.is_err(), "Erro retornado sem panic");

    println!("\n[4] TESTES DE OPERAÇÕES EM DISCO E RESPEITO AO R1");

    // 4.1 Teste com diretório temporário isolado
    let temp_dir = std::env::temp_dir().join(format!("counterrush_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir_all(&temp_dir).expect("Falha ao criar diretório temporário de teste");

    // Verificação de que pasta vazia não tem perfis
    let entries = fs::read_dir(&temp_dir).unwrap().count();
    check!("Diretório temporário vazio", entries == 0, "0 arquivos");

    // Gravação e carregamento de novo perfil no diretório temporário
    let test_profile_path = temp_dir.join("perfil_adversarial.json");
    let mut custom_cfg = CustomOptimizationConfig::all_disabled();
    custom_cfg.launch_args = vec!["-novid".to_string(), "-high".to_string()];
    custom_cfg.cs2_priority = true;

    let save_res = custom_cfg.save_to_file(&test_profile_path);
    check!("save_to_file em arquivo novo", save_res.is_ok(), format!("{:?}", save_res));
    check!("Arquivo gravado no disco", test_profile_path.exists(), "Arquivo existe");

    let loaded_res = CustomOptimizationConfig::load_from_file(&test_profile_path);
    check!("load_from_file arquivo salvo", loaded_res.is_ok(), format!("{:?}", loaded_res));
    if let Ok(loaded) = loaded_res {
        check!("Configuração carregada confere", loaded == custom_cfg, "Campos idênticos");
    }

    // Limpeza do diretório temporário
    let _ = fs::remove_dir_all(&temp_dir);

    println!("\n================================================================================");
    println!("     RESULTADO FINAL: {} testes executados | {} APROVADOS | {} FALHAS", total_tests, passed_tests, failed_tests);
    println!("================================================================================");

    if failed_tests > 0 {
        std::process::exit(1);
    }
}
