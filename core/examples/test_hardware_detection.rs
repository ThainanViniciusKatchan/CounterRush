use core::OptimizerCore;

fn main() {
    println!("=== CounterRush — Detecção e Classificação de Hardware ===");
    let core = OptimizerCore::default();

    println!("\nExecutando detecção de hardware nativa e atualizando Hardware.json...");

    match core.detect_and_save_hardware(None) {
        Ok((path, info, category)) => {
            println!("\n[1] Processador (CPU):");
            println!("  - Modelo:            {}", info.cpu.model);
            println!("  - Núcleos Físicos:   {}", info.cpu.physical_cores);
            println!("  - Threads Lógicas:   {}", info.cpu.logical_cores);
            println!("  - Frequência:        {} MHz", info.cpu.frequency_mhz);

            println!("\n[2] Placa de Vídeo (GPU):");
            println!("  - Modelo:            {}", info.gpu.model);
            println!("  - Memória de Vídeo:  {:.2} GB VRAM", info.gpu.vram_gb);

            println!("\n[3] Memória RAM:");
            println!("  - Quantidade Total:  {:.2} GB", info.ram.total_gb);
            match info.ram.speed_mhz {
                Some(speed) => println!("  - Velocidade:        {} MHz", speed),
                None => println!("  - Velocidade:        Não identificada"),
            }

            println!("\n[4] Persistência dos Dados de Hardware:");
            println!("  - Arquivo:           {}", path.display());
            println!("  - Status:            Dados gravados com sucesso em Hardware.json para consultas e autodetecção.");

            println!("\n=======================================================");
            println!("  Classificação do Sistema: {}", category);
            println!("=======================================================");
            println!("Classificação baseada nos requisitos e benchmarks do CS2.");
        }
        Err(err) => {
            eprintln!("\nErro ao detectar e salvar hardware: {err}");
        }
    }
}
