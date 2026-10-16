use std::env;
use std::time::Duration;
use tweaks::{Cs2AffinityTweak, Cs2PriorityTweak, Tweak};

fn ensure_game_running() -> Result<(), Box<dyn std::error::Error>> {
    if !windows_api::is_process_running("cs2").unwrap_or(false) {
        println!("\nCS2 não está em execução. Disparando inicialização via Steam...");
        windows_api::launch_cs2()?;
        println!("Aguardando até 60 segundos para o processo 'cs2.exe' ser instanciado...");
        windows_api::wait_for_process("cs2", Duration::from_secs(60))?;
        println!("[OK] Processo 'cs2.exe' detectado em execução!");
    } else {
        println!("\n[OK] Processo 'cs2.exe' já está em execução!");
    }
    Ok(())
}

fn apply_priority(tweak: &Cs2PriorityTweak) {
    println!("\n--- Tweak: Prioridade de Processo ---");
    println!("Nome:        {}", tweak.name());
    println!("ID:          {}", tweak.id());
    println!("Descrição:   {}", tweak.description());
    match tweak.status() {
        Ok(status) => println!("Status atual: {:?}", status),
        Err(err) => eprintln!("Erro ao consultar status: {}", err),
    }

    println!("\nAplicando prioridade RealTime ao cs2.exe...");
    match tweak.apply() {
        Ok(_) => println!("[OK] Prioridade RealTime aplicada com sucesso!"),
        Err(err) => eprintln!("[FALHA] Prioridade: {}", err),
    }
}

fn apply_affinity(tweak: &Cs2AffinityTweak) {
    let physical_cores = windows_api::get_cpu_physical_cores();
    println!("\n--- Tweak: Afinidade de CPU ---");
    println!("Nome:        {}", tweak.name());
    println!("ID:          {}", tweak.id());
    println!("Descrição:   {}", tweak.description());
    println!("Núcleos físicos detectados: {}", physical_cores);

    if physical_cores <= 4 {
        println!("[AVISO] CPU possui <= 4 núcleos. O tweak é ignorado propositalmente para preservar as threads do jogo.");
    } else {
        println!("[OK] CPU possui > 4 núcleos ({}). Elegível para isolamento do Core 0.", physical_cores);
    }

    match tweak.status() {
        Ok(status) => println!("Status atual: {:?}", status),
        Err(err) => eprintln!("Erro ao consultar status: {}", err),
    }

    println!("\nAplicando afinidade de CPU ao cs2.exe...");
    match tweak.apply() {
        Ok(_) => {
            if physical_cores <= 4 {
                println!("[AVISO] Afinidade ignorada propositalmente (CPU <= 4 núcleos).");
            } else {
                println!("[OK] Afinidade (sem Core 0) aplicada com sucesso!");
            }
        }
        Err(err) => eprintln!("[FALHA] Afinidade: {}", err),
    }
}

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Apenas Prioridade:  cargo run -p tweaks --example test_process_tweaks -- --priority");
    println!("  Apenas Afinidade:   cargo run -p tweaks --example test_process_tweaks -- --affinity");
    println!("  Ambos em conjunto:  cargo run -p tweaks --example test_process_tweaks -- --all");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let priority_tweak = Cs2PriorityTweak;
    let affinity_tweak = Cs2AffinityTweak;

    println!("=== Teste dos Tweaks de Processo CS2 ===");

    match action {
        Some("--priority") => {
            if let Err(err) = ensure_game_running() {
                eprintln!("\n[ERRO] Falha ao verificar/iniciar o CS2: {}", err);
                return;
            }
            apply_priority(&priority_tweak);
            print_usage();
        }
        Some("--affinity") => {
            if let Err(err) = ensure_game_running() {
                eprintln!("\n[ERRO] Falha ao verificar/iniciar o CS2: {}", err);
                return;
            }
            apply_affinity(&affinity_tweak);
            print_usage();
        }
        Some("--all") | None => {
            if let Err(err) = ensure_game_running() {
                eprintln!("\n[ERRO] Falha ao verificar/iniciar o CS2: {}", err);
                return;
            }
            apply_priority(&priority_tweak);
            apply_affinity(&affinity_tweak);
            print_usage();
        }
        Some(other) => {
            eprintln!("\nOpção inválida: '{}'", other);
            print_usage();
        }
    }
}
