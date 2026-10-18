use std::env;
use tweaks::{disable_hpet, get_hpet_status, rollback_hpet, HpetStatus, HpetTweak, Tweak};

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Desativar HPET (useplatformclock false): cargo run -p tweaks --example test_hpet -- --disable");
    println!("  Rollback (deletevalue useplatformclock): cargo run -p tweaks --example test_hpet -- --rollback");
    println!("  Consultar status no BCD:                 cargo run -p tweaks --example test_hpet -- --status");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let tweak = HpetTweak;

    println!("=== Teste: Temporizador de Eventos de Alta Precisão (HPET) ===");
    println!("Nome:         {}", tweak.name());
    println!("ID:           {}", tweak.id());
    println!("Descrição:    {}", tweak.description());
    println!("Requer Admin: {}", tweak.requires_admin());

    match action {
        Some("--disable") => {
            println!("\nDesativando HPET via BCD (bcdedit /set useplatformclock false)...");
            match disable_hpet() {
                Ok(_) => {
                    println!("[SUCESSO] HPET desativado com sucesso no BCD!");
                    println!("O Windows passará a utilizar os contadores TSC de baixa latência da CPU.");
                    println!("\nStatus atualizado:");
                    match get_hpet_status() {
                        Ok(st) => println!("  {}", st),
                        Err(e) => eprintln!("  Erro ao ler status: {e}"),
                    }
                    println!("\nNota: Para surtir efeito, reinicie o computador.");
                }
                Err(err) => eprintln!("[FALHA] Erro ao desativar HPET: {err}"),
            }
            print_usage();
        }
        Some("--rollback") => {
            println!("\nExecutando rollback do HPET (bcdedit /deletevalue useplatformclock)...");
            match rollback_hpet() {
                Ok(_) => {
                    println!("[SUCESSO] Rollback executado! A configuração useplatformclock foi removida do BCD.");
                    println!("O sistema retornou ao comportamento padrão do Windows.");
                    println!("\nStatus atualizado:");
                    match get_hpet_status() {
                        Ok(st) => println!("  {}", st),
                        Err(e) => eprintln!("  Erro ao ler status: {e}"),
                    }
                    println!("\nNota: Para surtir efeito, reinicie o computador.");
                }
                Err(err) => eprintln!("[FALHA] Erro ao executar rollback do HPET: {err}"),
            }
            print_usage();
        }
        _ => {
            println!("\nConsultando status do HPET no BCD...");
            match get_hpet_status() {
                Ok(st) => {
                    println!("Status Atual: {}", st);
                    match st {
                        HpetStatus::Disabled => {
                            println!("\n[OK] HPET desativado: Menor latência e sem sobrecarga do barramento da placa-mãe.");
                        }
                        HpetStatus::Enabled => {
                            println!("\n[AVISO] HPET está ativado forçadamente: Pode causar micro-stutters e maior input lag.");
                        }
                        HpetStatus::Default => {
                            println!("\n[PADRÃO] useplatformclock não está presente no BCD. O Windows gerencia o relógio nativamente.");
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Erro ao consultar status: {err}");
                    eprintln!("Dica: Execute o terminal como Administrador para consultar ou modificar o BCD.");
                }
            }
            print_usage();
        }
    }
}
