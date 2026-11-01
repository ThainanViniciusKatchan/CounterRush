use core::{delete_optimization_profile_in_dir, CoreError};
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    println!("================================================================================");
    println!("  CounterRush — Teste de Stress Adversarial de Exclusão de Perfis (Milestone 1)");
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

    let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário para testes");
    let base_path = temp_dir.path();

    println!("\n[1] TESTES ADVERSARIAIS: PATH TRAVERSAL");
    let traversal_payloads = [
        "../evil",
        "..\\evil",
        "../../escape",
        "..\\..\\escape",
        "foo/bar",
        "foo\\bar",
        "C:\\Windows\\System32\\calc",
        "..\\..\\..\\..\\..\\..\\test",
        "/etc/passwd",
        "sub/../dir",
    ];

    for payload in traversal_payloads {
        let res = delete_optimization_profile_in_dir(base_path, payload);
        check!(
            &format!("Bloqueio Path Traversal: '{}'", payload),
            matches!(res, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::InvalidInput),
            format!("Rejeitado como InvalidInput: {:?}", res)
        );
    }

    println!("\n[2] TESTES ADVERSARIAIS: NOMES DE DISPOSITIVOS RESERVADOS DO WINDOWS");
    let reserved_devices = [
        "CON", "PRN", "AUX", "NUL",
        "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9",
        "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    for dev in reserved_devices {
        // Teste maiúsculo
        let res_upper = delete_optimization_profile_in_dir(base_path, dev);
        check!(
            &format!("Dispositivo reservado maiúsculo: '{}'", dev),
            matches!(res_upper, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::InvalidInput),
            format!("{:?}", res_upper)
        );

        // Teste minúsculo
        let dev_lower = dev.to_lowercase();
        let res_lower = delete_optimization_profile_in_dir(base_path, &dev_lower);
        check!(
            &format!("Dispositivo reservado minúsculo: '{}'", dev_lower),
            matches!(res_lower, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::InvalidInput),
            format!("{:?}", res_lower)
        );

        // Teste com extensão .json
        let dev_ext = format!("{}.json", dev);
        let res_ext = delete_optimization_profile_in_dir(base_path, &dev_ext);
        check!(
            &format!("Dispositivo reservado com .json: '{}'", dev_ext),
            matches!(res_ext, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::InvalidInput),
            format!("{:?}", res_ext)
        );
    }

    println!("\n[3] TESTES ADVERSARIAIS: CARACTERES PROIBIDOS NTFS / WINDOWS");
    let forbidden_chars = ['*', '?', '<', '>', ':', '"', '|', '\\', '/'];
    for c in forbidden_chars {
        let name = format!("perfil{}teste", c);
        let res = delete_optimization_profile_in_dir(base_path, &name);
        check!(
            &format!("Caractere proibido '{}'", c),
            matches!(res, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::InvalidInput),
            format!("{:?}", res)
        );
    }

    println!("\n[4] TESTES DE ENTRADAS VAZIAS OU APENAS ESPAÇOS");
    let blank_inputs = ["", "   ", "\t", "\n", "\r\n", ".json", "  .json  ", ".JSON", "  .JSON  "];
    for blank in blank_inputs {
        let res = delete_optimization_profile_in_dir(base_path, blank);
        check!(
            &format!("Entrada em branco/vazia: '{:?}'", blank),
            matches!(res, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::InvalidInput),
            format!("{:?}", res)
        );
    }

    println!("\n[5] TESTE DE PERFIL INEXISTENTE (NOTFOUND)");
    let res_not_found = delete_optimization_profile_in_dir(base_path, "perfil_fantasma");
    check!(
        "Perfil inexistente sem extensão",
        matches!(res_not_found, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::NotFound),
        format!("{:?}", res_not_found)
    );

    let res_not_found_ext = delete_optimization_profile_in_dir(base_path, "perfil_fantasma.json");
    check!(
        "Perfil inexistente com extensão .json",
        matches!(res_not_found_ext, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::NotFound),
        format!("{:?}", res_not_found_ext)
    );

    println!("\n[6] TESTES DE CICLO DE VIDA COMPLETO DE EXCLUSÃO E INTEGRIDADE DE VIZINHOS");
    // Criação de 3 perfis
    let file1 = base_path.join("perfil_alfa.json");
    let file2 = base_path.join("perfil_beta.json");
    let file3 = base_path.join("perfil_gama.json");

    File::create(&file1).expect("Falha ao criar file1");
    File::create(&file2).expect("Falha ao criar file2");
    File::create(&file3).expect("Falha ao criar file3");

    check!("3 arquivos criados no diretório", base_path.read_dir().unwrap().count() == 3, "OK");

    // Exclui perfil_beta sem extensão
    let del_beta = delete_optimization_profile_in_dir(base_path, "perfil_beta");
    check!("Excluir perfil_beta sem extensão", del_beta.is_ok(), format!("{:?}", del_beta));
    check!("perfil_beta.json foi removido do disco", !file2.exists(), "Não existe");
    check!("perfil_alfa.json permanece intocado", file1.exists(), "Existe");
    check!("perfil_gama.json permanece intocado", file3.exists(), "Existe");
    check!("Contagem de arquivos agora é 2", base_path.read_dir().unwrap().count() == 2, "OK");

    // Tentativa imediata de excluir perfil_beta novamente -> Deve dar NotFound
    let del_beta_again = delete_optimization_profile_in_dir(base_path, "perfil_beta");
    check!(
        "Segunda exclusão de perfil_beta retorna NotFound",
        matches!(del_beta_again, Err(CoreError::Io(ref io_err)) if io_err.kind() == ErrorKind::NotFound),
        format!("{:?}", del_beta_again)
    );

    // Exclui perfil_alfa com .json explícito e espaços
    let del_alfa = delete_optimization_profile_in_dir(base_path, "  perfil_alfa.json  ");
    check!("Excluir perfil_alfa com .json e espaços", del_alfa.is_ok(), format!("{:?}", del_alfa));
    check!("perfil_alfa.json foi removido do disco", !file1.exists(), "Não existe");
    check!("perfil_gama.json permanece intocado", file3.exists(), "Existe");
    check!("Contagem de arquivos agora é 1", base_path.read_dir().unwrap().count() == 1, "OK");

    // Exclui perfil_gama (último perfil restante)
    let del_gama = delete_optimization_profile_in_dir(base_path, "perfil_gama.JSON");
    check!("Excluir perfil_gama com .JSON maiúsculo", del_gama.is_ok(), format!("{:?}", del_gama));
    check!("perfil_gama.json foi removido do disco", !file3.exists(), "Não existe");
    check!("Diretório está agora 100% vazio (0 arquivos)", base_path.read_dir().unwrap().count() == 0, "Vazio");

    println!("\n[7] TESTE COM NOMES VÁLIDOS ESPECIAIS (UNICODE / ACENTOS)");
    let file_unicode = base_path.join("perfil_otimização_extrema.json");
    File::create(&file_unicode).expect("Falha ao criar arquivo unicode");
    check!("Arquivo unicode criado", file_unicode.exists(), "OK");

    let del_unicode = delete_optimization_profile_in_dir(base_path, "perfil_otimização_extrema");
    check!("Exclusão com acentuação UTF-8", del_unicode.is_ok(), format!("{:?}", del_unicode));
    check!("Arquivo unicode removido", !file_unicode.exists(), "Não existe");

    println!("\n================================================================================");
    println!("  RESULTADO FINAL: {} testes | {} APROVADOS | {} FALHAS", total_tests, passed_tests, failed_tests);
    println!("================================================================================");

    if failed_tests > 0 {
        std::process::exit(1);
    }
}
