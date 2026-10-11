use std::time::Duration;
use core::OptimizerCore;

fn main() {
    println!("=== Teste de Inicialização e Otimização do CS2 ===");
    let core = OptimizerCore::default();

    println!("1. Disparando inicialização do CS2 via Steam...");
    println!("2. Aguardando até 60 segundos para o processo 'cs2.exe' ser instanciado...");

    match core.launch_and_optimize(Duration::from_secs(60)) {
        Ok(_) => {
            println!("\nSucesso absoluto!");
            println!("- Jogo iniciado");
            println!("- Processo cs2.exe detectado");
            println!("- Prioridade RealTime aplicada");
            println!("- Afinidade (sem Core 0) aplicada");
        }
        Err(err) => {
            eprintln!("\nFalha na execução: {err}");
        }
    }
}
