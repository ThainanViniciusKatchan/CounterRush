use std::env;
use tweaks::{
    enable_all_msi, enable_gpu_msi, enable_network_msi, enable_usb_msi, get_msi_status,
    is_all_msi_enabled, rollback_all_msi, rollback_gpu_msi, rollback_network_msi, rollback_usb_msi,
    MsiDeviceStatus, MsiModeTweak, Tweak,
};

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Ativar em tudo (GPU + Rede + USB): cargo run -p tweaks --example test_msi -- --enable");
    println!("  Ativar apenas na GPU:             cargo run -p tweaks --example test_msi -- --enable-gpu");
    println!("  Ativar apenas na Placa de Rede:   cargo run -p tweaks --example test_msi -- --enable-network");
    println!("  Ativar apenas nas Controladoras:  cargo run -p tweaks --example test_msi -- --enable-usb");
    println!("  Rollback completo (restaurar):    cargo run -p tweaks --example test_msi -- --rollback");
    println!("  Rollback apenas GPU:              cargo run -p tweaks --example test_msi -- --rollback-gpu");
    println!("  Rollback apenas Placa de Rede:    cargo run -p tweaks --example test_msi -- --rollback-network");
    println!("  Rollback apenas Controladoras:    cargo run -p tweaks --example test_msi -- --rollback-usb");
    println!("  Consultar status dos dispositivos:cargo run -p tweaks --example test_msi -- --status");
}

fn print_status_table(devices: &[MsiDeviceStatus]) {
    println!("\nDispositivos PCI Detectados:");
    println!("{:-<105}", "");
    println!(
        "  {:<9} | {:<48} | {:<16} | {:<15}",
        "Categoria", "Nome do Dispositivo", "Modo MSI", "Prioridade GPU"
    );
    println!("{:-<105}", "");

    for dev in devices {
        let msi_str = match dev.msi_supported {
            Some(1) => "ATIVO (1)",
            Some(0) => "DESATIVADO (0)",
            _ => "NÃO CONFIGURADO",
        };

        let prio_str = match dev.device_priority {
            Some(3) => "ALTA (3)".to_string(),
            Some(2) => "NORMAL (2)".to_string(),
            Some(1) => "BAIXA (1)".to_string(),
            Some(v) => format!("PERSONALIZADA ({v})"),
            None => {
                if dev.category == "GPU" {
                    "PADRÃO".to_string()
                } else {
                    "-".to_string()
                }
            }
        };

        println!(
            "  {:<9} | {:<48} | {:<16} | {:<15}",
            dev.category,
            if dev.name.len() > 48 {
                format!("{}...", &dev.name[..45])
            } else {
                dev.name.clone()
            },
            msi_str,
            prio_str
        );
    }
    println!("{:-<105}", "");

    let all_enabled = is_all_msi_enabled().unwrap_or(false);
    if all_enabled && !devices.is_empty() {
        println!("Conformidade Geral: [TODOS OS DISPOSITIVOS ESTÃO COM MODO MSI ATIVO E OTIMIZADO]");
    } else {
        println!("Conformidade Geral: [PARCIALMENTE ATIVADO OU PADRÃO DO WINDOWS]");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let tweak = MsiModeTweak;

    println!("=== Teste: Modo MSI (Message Signaled-Based Interrupts) ===");
    println!("Nome:         {}", tweak.name());
    println!("ID:           {}", tweak.id());
    println!("Descrição:    {}", tweak.description());
    println!("Requer Admin: {}", tweak.requires_admin());

    match action {
        Some("--enable") => {
            println!("\nAtivando Modo MSI em todos os dispositivos PCI (GPU, Rede, USB)...");
            match enable_all_msi() {
                Ok(_) => {
                    println!("[SUCESSO] Modo MSI ativado em todos os dispositivos!");
                    if let Ok(devices) = get_msi_status() {
                        print_status_table(&devices);
                    }
                    println!("\nNota: Para surtir efeito pleno nas interrupções, reinicie o computador.");
                }
                Err(err) => eprintln!("[FALHA] Erro ao ativar Modo MSI: {err}"),
            }
            print_usage();
        }
        Some("--enable-gpu") => {
            println!("\nAtivando Modo MSI com Prioridade Alta na GPU...");
            match enable_gpu_msi() {
                Ok(_) => {
                    println!("[SUCESSO] Modo MSI ativado na GPU!");
                    if let Ok(devices) = get_msi_status() {
                        print_status_table(&devices);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro ao ativar MSI na GPU: {err}"),
            }
            print_usage();
        }
        Some("--enable-network") => {
            println!("\nAtivando Modo MSI na Placa de Rede...");
            match enable_network_msi() {
                Ok(_) => {
                    println!("[SUCESSO] Modo MSI ativado na Placa de Rede!");
                    if let Ok(devices) = get_msi_status() {
                        print_status_table(&devices);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro ao ativar MSI na Placa de Rede: {err}"),
            }
            print_usage();
        }
        Some("--enable-usb") => {
            println!("\nAtivando Modo MSI nas Controladoras Host USB...");
            match enable_usb_msi() {
                Ok(_) => {
                    println!("[SUCESSO] Modo MSI ativado nas Controladoras USB!");
                    if let Ok(devices) = get_msi_status() {
                        print_status_table(&devices);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro ao ativar MSI no USB: {err}"),
            }
            print_usage();
        }
        Some("--rollback") => {
            println!("\nExecutando rollback do Modo MSI em todos os dispositivos...");
            match rollback_all_msi() {
                Ok(_) => {
                    println!("[SUCESSO] Rollback executado! Configurações de MSI removidas.");
                    if let Ok(devices) = get_msi_status() {
                        print_status_table(&devices);
                    }
                    println!("\nNota: Reinicie o computador para restaurar os modos de interrupção padrão.");
                }
                Err(err) => eprintln!("[FALHA] Erro no rollback: {err}"),
            }
            print_usage();
        }
        Some("--rollback-gpu") => {
            println!("\nExecutando rollback do Modo MSI na GPU...");
            match rollback_gpu_msi() {
                Ok(_) => println!("[SUCESSO] MSI e Affinity Policy removidos da GPU!"),
                Err(err) => eprintln!("[FALHA] Erro: {err}"),
            }
            if let Ok(devices) = get_msi_status() {
                print_status_table(&devices);
            }
            print_usage();
        }
        Some("--rollback-network") => {
            println!("\nExecutando rollback do Modo MSI na Placa de Rede...");
            match rollback_network_msi() {
                Ok(_) => println!("[SUCESSO] MSI removido da Placa de Rede!"),
                Err(err) => eprintln!("[FALHA] Erro: {err}"),
            }
            if let Ok(devices) = get_msi_status() {
                print_status_table(&devices);
            }
            print_usage();
        }
        Some("--rollback-usb") => {
            println!("\nExecutando rollback do Modo MSI nas Controladoras USB...");
            match rollback_usb_msi() {
                Ok(_) => println!("[SUCESSO] MSI removido das Controladoras USB!"),
                Err(err) => eprintln!("[FALHA] Erro: {err}"),
            }
            if let Ok(devices) = get_msi_status() {
                print_status_table(&devices);
            }
            print_usage();
        }
        _ => {
            println!("\nConsultando status atual do Modo MSI no Registro...");
            match get_msi_status() {
                Ok(devices) => print_status_table(&devices),
                Err(err) => eprintln!("Erro ao consultar status: {err}"),
            }
            print_usage();
        }
    }
}
