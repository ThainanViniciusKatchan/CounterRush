use core::OptimizerCore;

fn main() {
    println!("=== Teste de Inicialização do CS2 por Perfil de Hardware ===");
    let core = OptimizerCore::default();

    match core.get_hardware_profile() {
        Ok((info, category)) => {
            println!("\nHardware Detectado:");
            println!("  - CPU:  {}", info.cpu.model);
            println!("  - GPU:  {} ({:.2} GB VRAM)", info.gpu.model, info.gpu.vram_gb);
            println!("  - RAM:  {:.2} GB", info.ram.total_gb);
            println!("  - Perfil atribuído: {}", category);

            println!("\nDisparando o jogo com os parâmetros específicos do perfil {}...", category);
            match core.launch_by_category(category) {
                Ok(_) => {
                    println!("\nSucesso! CS2 iniciado via Steam com os parâmetros configurados em Parameters.md.");
                }
                Err(err) => {
                    eprintln!("\nErro ao iniciar o jogo: {err}");
                }
            }
        }
        Err(err) => {
            eprintln!("Erro ao detectar perfil de hardware: {err}");
        }
    }
}
