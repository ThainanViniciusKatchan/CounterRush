use std::env;
use tweaks::{
    disable_memory_compression, enable_memory_compression, get_memory_compression_status,
    MemoryCompressionTweak, Tweak,
};

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Desativar compactação:  cargo run -p tweaks --example test_memory_compression -- --disable");
    println!("  Reativar (Rollback):    cargo run -p tweaks --example test_memory_compression -- --enable");
    println!("  Consultar status:       cargo run -p tweaks --example test_memory_compression -- --status");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let tweak = MemoryCompressionTweak;

    println!("=== Teste: Compactação de Memória do Windows (MMAgent) ===");
    println!("Nome:        {}", tweak.name());
    println!("ID:          {}", tweak.id());
    println!("Descrição:   {}", tweak.description());
    println!("Requer Admin: {}", tweak.requires_admin());

    let ram_info = windows_api::detect_hardware().map(|h| h.ram.total_gb).unwrap_or(0.0);
    println!("RAM Total Detectada: {:.2} GB", ram_info);

    if ram_info <= 16.0 {
        println!("  [AVISO] Memória RAM <= 16 GB. A desativação não é permitida por segurança.");
    } else {
        println!("  [OK] Memória RAM > 16 GB ({:.2} GB). Elegível para desativação da compactação.", ram_info);
    }

    match action {
        Some("--disable") => {
            println!("\nVerificando status prévio com Get-MMAgent...");
            match get_memory_compression_status() {
                Ok(false) => {
                    println!("A compactação de memória já está desativada no sistema.");
                }
                Ok(true) => {
                    println!("Compactação de memória está ativa. Executando Disable-MMAgent -mc...");
                    match disable_memory_compression() {
                        Ok(_) => {
                            println!("\n[SUCESSO] Compactação de memória desativada com êxito!");
                            println!("As páginas de RAM não serão mais compactadas pela CPU, reduzindo micro-stutters.");
                        }
                        Err(err) => {
                            eprintln!("\n[FALHA] Erro ao desativar compactação de memória: {err}");
                        }
                    }
                }
                Err(err) => eprintln!("Erro ao consultar status prévio: {err}"),
            }
            print_usage();
        }
        Some("--enable") => {
            println!("\nReativando compactação de memória (Rollback com Enable-MMAgent -mc)...");
            match enable_memory_compression() {
                Ok(_) => {
                    println!("\n[SUCESSO] Compactação de memória reativada com êxito!");
                    println!("Padrão do Windows restaurado.");
                }
                Err(err) => {
                    eprintln!("\n[FALHA] Erro ao reativar compactação de memória: {err}");
                }
            }
            print_usage();
        }
        Some("--status") | None => {
            println!("\nConsultando status atual com Get-MMAgent...");
            match get_memory_compression_status() {
                Ok(true) => println!("Status atual: ATIVA (Padrão do Windows - MemoryCompression = True)"),
                Ok(false) => println!("Status atual: DESATIVADA (Otimizado para >16GB - MemoryCompression = False)"),
                Err(err) => eprintln!("Erro ao consultar status: {err}"),
            }
            print_usage();
        }
        Some(other) => {
            eprintln!("\nOpção desconhecida: '{}'", other);
            print_usage();
        }
    }
}
