use tweaks::{Cs2AffinityTweak, Cs2PriorityTweak, Tweak};

fn main() {
    let priority_tweak = Cs2PriorityTweak;
    let affinity_tweak = Cs2AffinityTweak;
    let physical_cores = windows_api::get_cpu_physical_cores();

    println!("=== Teste dos Tweaks de Processo CS2 ===");
    println!("Núcleos físicos da CPU detectados: {}", physical_cores);

    println!("\n1. Tweak: {} ({})", priority_tweak.name(), priority_tweak.id());
    println!("   Descrição: {}", priority_tweak.description());
    match priority_tweak.status() {
        Ok(status) => println!("   Status atual: {:?}", status),
        Err(err) => eprintln!("   Erro ao consultar status: {}", err),
    }

    println!("\n2. Tweak: {} ({})", affinity_tweak.name(), affinity_tweak.id());
    println!("   Descrição: {}", affinity_tweak.description());
    if physical_cores <= 4 {
        println!("   [AVISO] CPU possui <= 4 núcleos. O tweak será ignorado para preservar as threads do jogo.");
    } else {
        println!("   [OK] CPU possui > 4 núcleos ({}). Elegível para isolamento do Core 0.", physical_cores);
    }
    match affinity_tweak.status() {
        Ok(status) => println!("   Status atual: {:?}", status),
        Err(err) => eprintln!("   Erro ao consultar status: {}", err),
    }

    println!("\nTentando aplicar tweaks (requer cs2.exe em execução)...");
    match priority_tweak.apply() {
        Ok(_) => println!("   Prioridade RealTime aplicada com sucesso!"),
        Err(err) => println!("   Prioridade: {}", err),
    }

    match affinity_tweak.apply() {
        Ok(_) => {
            if physical_cores <= 4 {
                println!("   Afinidade: ignorada propositalmente (CPU com <= 4 núcleos).");
            } else {
                println!("   Afinidade (sem Core 0) aplicada com sucesso!");
            }
        }
        Err(err) => println!("   Afinidade: {}", err),
    }
}
