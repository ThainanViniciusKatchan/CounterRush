use std::env;
use tweaks::power_saving::{
    disable_all_power_saving, disable_pcie_power_saving, disable_usb_hubs_power_saving,
    disable_usb_selective_suspend, get_power_saving_status, rollback_all_power_saving,
    rollback_pcie_power_saving, rollback_usb_hubs_power_saving, rollback_usb_selective_suspend,
    PowerSavingStatus,
};

fn format_aspm(val: Option<u32>) -> String {
    match val {
        Some(0) => "0 (Desligado - Otimizado para Jogos)".to_string(),
        Some(1) => "1 (Economia de Energia Moderada - Padrão Windows)".to_string(),
        Some(2) => "2 (Economia de Energia Máxima)".to_string(),
        Some(n) => format!("{} (Personalizado)", n),
        None => "Não configurado / Não detectado".to_string(),
    }
}

fn format_usb_suspend(val: Option<u32>) -> String {
    match val {
        Some(0) => "0 (Desabilitado - Otimizado para Jogos)".to_string(),
        Some(1) => "1 (Habilitado - Padrão Windows)".to_string(),
        Some(n) => format!("{} (Personalizado)", n),
        None => "Não configurado / Não detectado".to_string(),
    }
}

fn print_status(status: &PowerSavingStatus) {
    println!("\n================================================================================");
    println!("        STATUS DA ECONOMIA DE ENERGIA (PCIe & USB) - COUNTER RUSH");
    println!("================================================================================\n");

    println!("1. PCI Express (Link State Power Management - ASPM):");
    println!("   - Na Tomada (AC): {}", format_aspm(status.pcie_ac_setting));
    println!("   - Na Bateria (DC): {}", format_aspm(status.pcie_dc_setting));
    println!("   - Status: {}", if status.is_pcie_optimized() { "[OK] OTIMIZADO (ASPM Desligado)" } else { "[AVISO] Economia de energia ativa" });

    println!("\n2. Suspensão Seletiva USB (Plano de Energia):");
    println!("   - Na Tomada (AC): {}", format_usb_suspend(status.usb_suspend_ac_setting));
    println!("   - Na Bateria (DC): {}", format_usb_suspend(status.usb_suspend_dc_setting));
    println!("   - Status: {}", if status.is_usb_suspend_optimized() { "[OK] OTIMIZADO (Suspensão Desabilitada)" } else { "[AVISO] Economia de energia ativa" });

    println!("\n3. Hubs e Controladoras USB (Gerenciador de Dispositivos / WMI):");
    if status.usb_devices.is_empty() {
        println!("   Nenhum dispositivo USB gerenciado encontrado no namespace WMI.");
    } else {
        println!("{:<50} | {:<25}", "Dispositivo (InstanceName)", "Economia de Energia");
        println!("{:-<50}-+-{:-<25}", "", "");
        for dev in &status.usb_devices {
            let state = if dev.power_saving_enabled {
                "[AVISO] Ativada (Padrão Windows)"
            } else {
                "[OK] Desativada (Otimizado)"
            };
            println!("{:<50} | {:<25}", dev.instance_name, state);
        }
        println!("\n   - Status Geral USB: {}", if status.is_usb_hubs_optimized() { "[OK] OTIMIZADO (Economia desmarcada em todos)" } else { "[AVISO] Economia permitida em dispositivos" });
    }

    println!("\n--------------------------------------------------------------------------------");
    println!("Conformidade Geral do Sistema: {}", if status.is_all_optimized() {
        "TODAS AS OTIMIZAÇÕES DE ENERGIA ESTÃO ATIVADAS! (Latência mínima garantida)"
    } else {
        "ALGUMAS ECONOMIAS DE ENERGIA AINDA ESTÃO ATIVAS NO WINDOWS."
    });
    println!("--------------------------------------------------------------------------------\n");
}

fn print_usage() {
    println!("Uso: cargo run -p tweaks --example test_power_saving -- [OPÇÃO]\n");
    println!("Opções:");
    println!("  --disable              Desativa economia PCIe (ASPM), Suspensão Seletiva USB e Hubs USB");
    println!("  --disable-pcie         Desativa apenas economia PCIe (Link State Power Management)");
    println!("  --disable-usb-suspend  Desativa apenas Suspensão Seletiva USB no plano de energia");
    println!("  --disable-usb-hubs     Desativa apenas economia em Hubs USB (Gerenciador de Dispositivos)");
    println!("  --rollback             Restaura todos para o padrão de fábrica do Windows");
    println!("  --rollback-pcie        Restaura apenas economia PCIe (ASPM)");
    println!("  --rollback-usb-suspend Restaura apenas Suspensão Seletiva USB");
    println!("  --rollback-usb-hubs    Restaura apenas economia em Hubs USB");
    println!("  --status               Exibe status detalhado atual das configurações");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str()).unwrap_or("--status");

    match action {
        "--disable" => {
            println!("Desativando economia de energia PCIe, Suspensão Seletiva USB e Hubs USB...");
            match disable_all_power_saving() {
                Ok(_) => {
                    println!("[SUCESSO] Todas as economias de energia PCIe e USB foram desativadas!");
                    if let Ok(st) = get_power_saving_status() {
                        print_status(&st);
                    }
                }
                Err(e) => eprintln!("[ERRO] Falha ao desativar economia de energia: {}", e),
            }
        }
        "--disable-pcie" => {
            println!("Desativando economia de energia PCIe (ASPM)...");
            match disable_pcie_power_saving() {
                Ok(_) => println!("[SUCESSO] PCIe Link State Power Management desativado!"),
                Err(e) => eprintln!("[ERRO] Falha ao desativar economia PCIe: {}", e),
            }
        }
        "--disable-usb-suspend" => {
            println!("Desativando Suspensão Seletiva USB...");
            match disable_usb_selective_suspend() {
                Ok(_) => println!("[SUCESSO] Suspensão Seletiva USB desativada!"),
                Err(e) => eprintln!("[ERRO] Falha ao desativar Suspensão Seletiva USB: {}", e),
            }
        }
        "--disable-usb-hubs" => {
            println!("Desmarcando permissão de economia de energia nos Hubs USB...");
            match disable_usb_hubs_power_saving() {
                Ok(_) => println!("[SUCESSO] Economia de energia nos Hubs USB desativada!"),
                Err(e) => eprintln!("[ERRO] Falha ao desativar economia nos Hubs USB: {}", e),
            }
        }
        "--rollback" => {
            println!("Restaurando economia de energia PCIe e USB para o padrão do Windows...");
            match rollback_all_power_saving() {
                Ok(_) => {
                    println!("[SUCESSO] Todas as configurações foram restauradas para o padrão do Windows!");
                    if let Ok(st) = get_power_saving_status() {
                        print_status(&st);
                    }
                }
                Err(e) => eprintln!("[ERRO] Falha ao realizar rollback: {}", e),
            }
        }
        "--rollback-pcie" => {
            println!("Restaurando economia de energia PCIe para o padrão moderado...");
            match rollback_pcie_power_saving() {
                Ok(_) => println!("[SUCESSO] PCIe restaurado para Economia Moderada!"),
                Err(e) => eprintln!("[ERRO] Falha ao restaurar PCIe: {}", e),
            }
        }
        "--rollback-usb-suspend" => {
            println!("Restaurando Suspensão Seletiva USB para o padrão habilitado...");
            match rollback_usb_selective_suspend() {
                Ok(_) => println!("[SUCESSO] Suspensão Seletiva USB restaurada para o padrão!"),
                Err(e) => eprintln!("[ERRO] Falha ao restaurar Suspensão Seletiva USB: {}", e),
            }
        }
        "--rollback-usb-hubs" => {
            println!("Restaurando permissão de economia de energia nos Hubs USB...");
            match rollback_usb_hubs_power_saving() {
                Ok(_) => println!("[SUCESSO] Hubs USB restaurados para o padrão do Windows!"),
                Err(e) => eprintln!("[ERRO] Falha ao restaurar Hubs USB: {}", e),
            }
        }
        "--status" => match get_power_saving_status() {
            Ok(status) => print_status(&status),
            Err(e) => eprintln!("[ERRO] Falha ao consultar status de economia de energia: {}", e),
        },
        "--help" | "-h" => {
            print_usage();
        }
        other => {
            eprintln!("Opção desconhecida: {}", other);
            print_usage();
        }
    }
}
