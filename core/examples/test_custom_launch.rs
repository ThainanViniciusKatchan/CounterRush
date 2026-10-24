use std::time::Duration;
use core::{OptimizerCore, resolve_custom_config_path};

fn main() {
    println!("=== Teste de Inicialização com Otimizações Personalizadas (JSON) ===");

    let core = OptimizerCore::default();
    let config_path = resolve_custom_config_path(None);

    println!("\n1. Verificando configuração em: {}", config_path.display());
    let was_created = !config_path.exists();

    let (resolved, config) = match core.load_or_create_custom_config(None) {
        Ok(res) => res,
        Err(err) => {
            eprintln!("   [ERRO ESTRITO] Falha ao carregar ou criar configuração: {err}");
            std::process::exit(1);
        }
    };

    if was_created {
        println!("   [CRIADO] Arquivo não existia e foi gerado com parâmetros do seu perfil de hardware!");
    } else {
        println!("   [LIDO] Configuração personalizada existente carregada com sucesso.");
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

    println!("\n3. Verificando status dos tweaks e iniciando CS2 via Steam...");
    match core.launch_custom(Some(&resolved), Duration::from_secs(60)) {
        Ok(report) => {
            println!("\n4. Resumo da Execução de Tweaks:");
            if !report.skipped_tweaks.is_empty() {
                println!("   Tweaks pulados (já estavam aplicados no sistema):");
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

            println!("\n[SUCESSO] CS2 iniciado com todas as otimizações ativas!");
        }
        Err(err) => {
            eprintln!("\n[ERRO ESTRITO] Processo interrompido devido a falha: {err}");
            std::process::exit(1);
        }
    }
}
