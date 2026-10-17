use std::env;
use tweaks::{
    apply_timer_resolution, is_precision_reduced, is_timer_registry_applied,
    rollback_timer_all, rollback_timer_registry, rollback_timer_resolution,
    test_timer_precision, TimerResolutionTweak, Tweak,
};

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Aplicar timer 0.5ms e testar: cargo run -p tweaks --example test_timer_resolution -- --apply");
    println!("  Apenas testar pausas (10x):   cargo run -p tweaks --example test_timer_resolution -- --test");
    println!("  Rollback completo (timer+reg):cargo run -p tweaks --example test_timer_resolution -- --rollback");
    println!("  Rollback apenas timer:        cargo run -p tweaks --example test_timer_resolution -- --rollback-timer");
    println!("  Rollback apenas registro:     cargo run -p tweaks --example test_timer_resolution -- --rollback-reg");
    println!("  Consultar status:             cargo run -p tweaks --example test_timer_resolution -- --status");
}

fn print_pauses(pauses: &[f64]) {
    for (i, p) in pauses.iter().enumerate() {
        println!("  Pausa {:2} : {:.4} ms", i + 1, p);
    }
    if !pauses.is_empty() {
        let relevant = if pauses.len() > 2 { &pauses[2..] } else { pauses };
        let avg = relevant.iter().sum::<f64>() / relevant.len() as f64;
        println!("  Média das pausas (estáveis): {:.4} ms", avg);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let tweak = TimerResolutionTweak;

    println!("=== Teste: Resolução de Temporizador do Sistema (0.5ms) ===");
    println!("Nome:        {}", tweak.name());
    println!("ID:          {}", tweak.id());
    println!("Descrição:   {}", tweak.description());
    println!("Requer Admin: {}", tweak.requires_admin());

    let reg_status = is_timer_registry_applied().unwrap_or(false);
    println!(
        "Registro GlobalTimerResolutionRequests: {}",
        if reg_status { "ATIVO (0x1)" } else { "NÃO CONFIGURADO / PADRÃO" }
    );

    match action {
        Some("--apply") => {
            println!("\nAplicando configuração no Registro e executando teste de timer...");
            println!("Passo 1: Tentando Primeiro Comando (Timer05) e medindo precisão...");
            match apply_timer_resolution() {
                Ok(res) => {
                    if res.command_used == 1 {
                        println!("[OK] Primeiro Comando (Timer05) aplicado com sucesso!");
                    } else {
                        println!("[OK] Primeiro Comando não reduziu a precisão suficiente; Segundo Comando (KernelTimer) aplicado com sucesso!");
                    }

                    if res.registry_applied {
                        println!("[OK] Registro GlobalTimerResolutionRequests configurado como 1.");
                    } else {
                        println!("[AVISO] Falha ao configurar chave no Registro (execute como Administrador).");
                    }

                    println!("\nResultados do teste de precisão (10 pausas de 0.5ms):");
                    print_pauses(&res.pauses_ms);

                    if is_precision_reduced(&res.pauses_ms) {
                        println!("\n[SUCESSO] Resolução de temporizador cravada e latência reduzida para ~0.5ms!");
                    } else {
                        println!("\n[AVISO] O temporizador foi configurado, mas as pausas ainda estão acima do esperado.");
                    }
                }
                Err(err) => {
                    eprintln!("\n[FALHA] Erro ao aplicar resolução do temporizador: {err}");
                }
            }
            print_usage();
        }
        Some("--test") => {
            println!("\nExecutando teste de precisão de temporizador (10 pausas de 0.5ms via NtDelayExecution)...");
            match test_timer_precision() {
                Ok(pauses) => {
                    print_pauses(&pauses);
                    if is_precision_reduced(&pauses) {
                        println!("\n[OK] O sistema está operando com alta precisão (~0.5ms a 1ms por interrupção).");
                    } else {
                        println!("\n[PADRÃO] O sistema está operando na resolução padrão do Windows (~15.6ms).");
                    }
                }
                Err(err) => eprintln!("\n[FALHA] Erro ao executar teste de temporizador: {err}"),
            }
            print_usage();
        }
        Some("--rollback") => {
            println!("\nExecutando rollback completo (timer e registro)...");
            match rollback_timer_all() {
                Ok(_) => println!("[SUCESSO] Rollback completo realizado com sucesso! Timer restaurado e registro limpo."),
                Err(err) => eprintln!("[FALHA] Erro no rollback completo: {err}"),
            }
            print_usage();
        }
        Some("--rollback-timer") => {
            println!("\nExecutando rollback apenas do timer (NtSetTimerResolution Set=false)...");
            match rollback_timer_resolution() {
                Ok(restored) => println!("[SUCESSO] Rollback do timer realizado! Timer voltou para: {:.4} ms", restored),
                Err(err) => eprintln!("[FALHA] Erro no rollback do timer: {err}"),
            }
            print_usage();
        }
        Some("--rollback-reg") => {
            println!("\nExecutando rollback apenas do registro (reg delete GlobalTimerResolutionRequests)...");
            match rollback_timer_registry() {
                Ok(_) => println!("[SUCESSO] Chave de registro removida com sucesso!"),
                Err(err) => eprintln!("[FALHA] Erro ao remover chave de registro: {err}"),
            }
            print_usage();
        }
        _ => {
            print_usage();
        }
    }
}
