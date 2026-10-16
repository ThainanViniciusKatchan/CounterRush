use std::env;
use tweaks::{
    launch_cs2_with_dpi_tweak, rollback_cs2_dpi_tweak, Cs2DpiOptimizationTweak, Tweak,
};

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Aplicar e abrir o jogo:  cargo run -p tweaks --example test_dpi_optimization -- --apply");
    println!("  Executar fallback:       cargo run -p tweaks --example test_dpi_optimization -- --rollback");
    println!("  Consultar status:        cargo run -p tweaks --example test_dpi_optimization -- --status");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let tweak = Cs2DpiOptimizationTweak;

    println!("=== Teste: Otimização de DPI e Modo Janela do CS2 ===");
    println!("Nome:        {}", tweak.name());
    println!("ID:          {}", tweak.id());
    println!("Descrição:   {}", tweak.description());

    match action {
        Some("--apply") => {
            println!("\nAplicando otimização de DPI no Registro e disparando o CS2...");
            match launch_cs2_with_dpi_tweak() {
                Ok(_) => {
                    println!("\n[SUCESSO] Otimização de DPI aplicada com êxito no Registro!");
                    println!("[SUCESSO] Comando de inicialização do CS2 enviado à Steam.");
                }
                Err(err) => {
                    eprintln!("\n[FALHA] Erro ao aplicar e abrir o jogo: {err}");
                }
            }
            print_usage();
        }
        Some("--rollback") => {
            println!("\nExecutando fallback (removendo chave de compatibilidade de DPI do Registro)...");
            match rollback_cs2_dpi_tweak() {
                Ok(_) => {
                    println!("\n[SUCESSO] Fallback concluído! Configuração padrão do Windows restaurada.");
                }
                Err(err) => {
                    eprintln!("\n[FALHA] Erro ao executar fallback: {err}");
                }
            }
            print_usage();
        }
        Some("--status") | None => {
            match tweak.status() {
                Ok(status) => println!("\nStatus atual no Registro: {:?}", status),
                Err(err) => eprintln!("\nErro ao consultar status no Registro: {err}"),
            }
            print_usage();
        }
        Some(other) => {
            eprintln!("\nOpção desconhecida: '{}'", other);
            print_usage();
        }
    }
}
