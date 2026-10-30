use std::path::Path;
use std::time::Duration;
use core::{OptimizerCore, resolve_custom_config_path};

fn main() {
    println!("=== CounterRush — Inicialização com Otimizações Personalizadas (JSON) ===");

    let args: Vec<String> = std::env::args().collect();
    let mut chosen_path: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--config" || arg == "-c" {
            if i + 1 < args.len() {
                chosen_path = Some(args[i + 1].clone());
                i += 1;
            }
        } else if !arg.starts_with('-') {
            chosen_path = Some(arg.clone());
        }
        i += 1;
    }

    let input_path = match chosen_path {
        Some(p) => p,
        None => {
            eprintln!("\n[ERRO DE USO] Nenhum arquivo .json de configuração foi especificado!");
            eprintln!("Você deve indicar qual arquivo de configuração o custom_launch deve seguir.\n");
            eprintln!("Sintaxe:");
            eprintln!("  cargo run -p core --example test_custom_launch -- <arquivo.json>");
            eprintln!("  cargo run -p core --example test_custom_launch -- --config <arquivo.json>\n");
            eprintln!("Exemplos:");
            eprintln!("  cargo run -p core --example test_custom_launch -- AutoDetect.json");
            eprintln!("  cargo run -p core --example test_custom_launch -- config/optimzations/AutoDetect.json");
            eprintln!("  cargo run -p core --example test_custom_launch -- config/optimzations/custom_optimization.json");

            let search_dirs = [
                Path::new("config/optimzations"),
                Path::new("config/optimizations"),
                Path::new("config"),
            ];
            for dir in &search_dirs {
                if dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        let json_files: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .map(|e| e.path())
                            .filter(|p| p.extension().map_or(false, |ext| ext == "json"))
                            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                            .collect();
                        if !json_files.is_empty() {
                            eprintln!("\nArquivos disponíveis encontrados em '{}':", dir.display());
                            for f in json_files {
                                eprintln!("  - {}", f);
                            }
                            break;
                        }
                    }
                }
            }
            std::process::exit(1);
        }
    };

    let resolved_path = resolve_custom_config_path(Some(Path::new(&input_path)));
    println!("\n1. Arquivo de configuração especificado:");
    println!("   - Informado:   {}", input_path);
    println!("   - Localizado:  {}", resolved_path.display());

    let was_created = !resolved_path.exists();
    let core = OptimizerCore::default();
    let (resolved, config) = match core.load_or_create_custom_config(Some(&resolved_path)) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("\n[ERRO ESTRITO] Falha ao carregar ou criar configuração: {err}");
            std::process::exit(1);
        }
    };

    if was_created {
        println!("   - Status:      Arquivo não existia e foi gerado com os parâmetros recomendados do seu hardware.");
    } else {
        println!("   - Status:      Carregado com sucesso.");
    }

    println!("\n2. Parâmetros e Tweaks definidos no JSON:");
    println!("   - Arquivo: {}", resolved.display());
    println!("   - Launch Args: {:?}", config.launch_args);
    println!("   - Standby List: {}", config.empty_standby_list);
    println!("   - Prioridade RealTime: {}", config.cs2_priority);
    println!("   - Afinidade (Sem Core 0): {}", config.cs2_affinity);
    println!("   - Desativar Aceleração GPU apps: {}", config.disable_gpu_acceleration);
    println!("   - Otimização DPI: {}", config.apply_dpi_optimization);
    println!("   - Desativar Compressão RAM: {}", config.disable_memory_compression);
    println!("   - Timer Resolution 0.5ms: {}", config.apply_timer_resolution);
    println!("   - MMCSS Jogos: {}", config.apply_mmcss);
    println!("   - Desativar HPET: {}", config.disable_hpet);
    println!("   - Desativar Game Bar: {}", config.disable_game_bar);
    println!("   - Ativar MSI Mode: {}", config.enable_msi);
    println!("   - Desativar Economia Energia: {}", config.disable_power_saving);
    println!("   - Render Queue: {}", config.apply_render_queue);

    println!("\n3. Validando restrições de hardware, aplicando tweaks e iniciando CS2 via Steam...");
    match core.launch_custom(Some(&resolved), Duration::from_secs(90)) {
        Ok(report) => {
            println!("\n4. Resumo da Execução de Tweaks:");
            if !report.skipped_tweaks.is_empty() {
                println!("   Tweaks pulados ou bloqueados pelo sistema:");
                for tweak in &report.skipped_tweaks {
                    println!("     [PULADO] {tweak}");
                }
            } else {
                println!("   Nenhum tweak precisou ser pulado.");
            }

            if !report.applied_tweaks.is_empty() {
                println!("\n   Tweaks aplicados com sucesso nesta sessão:");
                for tweak in &report.applied_tweaks {
                    println!("     [APLICADO] {tweak}");
                }
            }

            println!("\n[SUCESSO] CS2 iniciado seguindo a configuração '{}'!", resolved.display());
        }
        Err(err) => {
            eprintln!("\n[ERRO ESTRITO] Processo interrompido devido a falha: {err}");
            std::process::exit(1);
        }
    }
}
