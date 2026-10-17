use std::env;
use tweaks::{
    apply_all_mmcss, apply_mmcss_with_tier, get_mmcss_status, is_mmcss_applied_for_tier,
    rollback_all_mmcss, MmcssHardwareTier, MmcssStatus, MmcssTweak, Tweak,
};

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Aplicar automático por hardware: cargo run -p tweaks --example test_mmcss -- --apply");
    println!("  Forçar perfil PC Forte:          cargo run -p tweaks --example test_mmcss -- --apply-forte");
    println!("  Forçar perfil PC Fraco:          cargo run -p tweaks --example test_mmcss -- --apply-fraco");
    println!("  Rollback (restaurar padrão Win): cargo run -p tweaks --example test_mmcss -- --rollback");
    println!("  Consultar status no Registro:    cargo run -p tweaks --example test_mmcss -- --status");
}

fn print_status_table(status: &MmcssStatus) {
    println!("\nValores Atuais no Registro:");
    println!("----------------------------------------------------------------------------------");
    println!(
        "  {:<26} : {}",
        "SystemResponsiveness",
        status
            .system_responsiveness
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string())
    );
    println!(
        "  {:<26} : {}",
        "NetworkThrottlingIndex",
        status
            .network_throttling_index
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string())
    );
    println!(
        "  {:<26} : {}",
        "GPU Priority",
        status
            .gpu_priority
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string())
    );
    println!(
        "  {:<26} : {}",
        "Priority",
        status
            .priority
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string())
    );
    println!(
        "  {:<26} : {}",
        "Scheduling Category",
        status.scheduling_category.as_deref().unwrap_or("Não configurado")
    );
    println!(
        "  {:<26} : {}",
        "SFIO Priority",
        status.sfio_priority.as_deref().unwrap_or("Não configurado")
    );
    println!("----------------------------------------------------------------------------------");

    let is_forte = is_mmcss_applied_for_tier(status, MmcssHardwareTier::HighEnd);
    let is_fraco = is_mmcss_applied_for_tier(status, MmcssHardwareTier::EntryLevel);

    if is_forte {
        println!("Conformidade: [PERFIL PC FORTE APLICADO COM SUCESSO]");
    } else if is_fraco {
        println!("Conformidade: [PERFIL PC FRACO/QUAD-CORE APLICADO COM SUCESSO]");
    } else {
        println!("Conformidade: [PADRÃO DO WINDOWS OU PERSONALIZADO]");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let tweak = MmcssTweak;

    println!("=== Teste: Otimizações de MMCSS (Multimedia Class Scheduler Service) ===");
    println!("Nome:         {}", tweak.name());
    println!("ID:           {}", tweak.id());
    println!("Descrição:    {}", tweak.description());
    println!("Requer Admin: {}", tweak.requires_admin());

    let hw_info = windows_api::detect_hardware();
    if let Ok(ref hw) = hw_info {
        println!(
            "Hardware Detectado: {} núcleos físicos / {:.2} GB RAM",
            hw.cpu.physical_cores, hw.ram.total_gb
        );
    }

    match action {
        Some("--apply") => {
            println!("\nAplicando MMCSS com detecção automática de hardware...");
            match apply_all_mmcss() {
                Ok(tier) => {
                    println!("[SUCESSO] Perfil aplicado: {}", tier);
                    if let Ok(status) = get_mmcss_status() {
                        print_status_table(&status);
                    }
                    println!("\nNota: Para surtir efeito pleno no agendador, reinicie o computador.");
                }
                Err(err) => eprintln!("[FALHA] Erro ao aplicar MMCSS: {err}"),
            }
            print_usage();
        }
        Some("--apply-forte") => {
            println!("\nForçando aplicação do perfil PC Forte (6+ núcleos / 16GB+ RAM)...");
            match apply_mmcss_with_tier(MmcssHardwareTier::HighEnd) {
                Ok(_) => {
                    println!("[SUCESSO] Perfil PC Forte gravado no Registro!");
                    if let Ok(status) = get_mmcss_status() {
                        print_status_table(&status);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro ao aplicar perfil PC Forte: {err}"),
            }
            print_usage();
        }
        Some("--apply-fraco") => {
            println!("\nForçando aplicação do perfil PC Fraco / Quad-Core / 8GB RAM...");
            match apply_mmcss_with_tier(MmcssHardwareTier::EntryLevel) {
                Ok(_) => {
                    println!("[SUCESSO] Perfil PC Fraco gravado no Registro!");
                    if let Ok(status) = get_mmcss_status() {
                        print_status_table(&status);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro ao aplicar perfil PC Fraco: {err}"),
            }
            print_usage();
        }
        Some("--rollback") => {
            println!("\nExecutando rollback para os valores padrão de fábrica do Windows...");
            match rollback_all_mmcss() {
                Ok(_) => {
                    println!("[SUCESSO] Rollback executado com sucesso!");
                    if let Ok(status) = get_mmcss_status() {
                        print_status_table(&status);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro ao executar rollback: {err}"),
            }
            print_usage();
        }
        _ => {
            if let Ok(status) = get_mmcss_status() {
                println!("Perfil Recomendado para este PC: {}", status.tier_detected);
                print_status_table(&status);
            }
            print_usage();
        }
    }
}
