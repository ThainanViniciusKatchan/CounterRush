use std::env;
use tweaks::render_queue::{
    apply_amd_flip_queue, apply_nvidia_persistence_mode, apply_render_queue,
    get_render_queue_status, rollback_amd_flip_queue, rollback_nvidia_persistence_mode,
    rollback_render_queue, RenderQueueStatus,
};

fn print_status(status: &RenderQueueStatus) {
    println!("\n================================================================================");
    println!("     STATUS DA FILA DE QUADROS / RENDER QUEUE (FLIP QUEUE) - COUNTER RUSH");
    println!("================================================================================\n");

    println!("1. Placas de Vídeo AMD (FlipQueueSize no Registro):");
    if status.amd_gpus.is_empty() {
        println!("   Nenhum adaptador de vídeo AMD detectado na classe Display do Registro.");
    } else {
        println!("{:<8} | {:<30} | {:<16} | {:<12} | {:<20}", "Chave", "Modelo (DriverDesc)", "FlipQueueSize", "Bytes", "Status");
        println!("{:-<8}-+-{:-<30}-+-{:-<16}-+-{:-<12}-+-{:-<20}", "", "", "", "", "");
        for gpu in &status.amd_gpus {
            let fq = gpu.flip_queue_size.as_deref().unwrap_or("Não definido");
            let raw = gpu.raw_bytes.as_deref().unwrap_or("-");
            let st = if gpu.is_optimized {
                "[OK] OTIMIZADO (1)"
            } else {
                "[AVISO] Padrão driver"
            };
            println!("{:<8} | {:<30} | {:<16} | {:<12} | {:<20}", gpu.registry_key, gpu.driver_desc, fq, raw, st);
        }
        println!("\n   - Conformidade AMD: {}", if status.is_amd_optimized() {
            "[OK] TODAS AS GPUS AMD ESTÃO COM FlipQueueSize = 1"
        } else {
            "[AVISO] Alguma GPU AMD ainda está no padrão de fábrica"
        });
    }

    println!("\n2. Placas de Vídeo NVIDIA (Modo de Persistência / nvidia-smi):");
    if status.nvidia_gpus.is_empty() {
        println!("   Nenhuma GPU NVIDIA ativa ou utilitário nvidia-smi disponível no sistema.");
    } else {
        println!("{:<35} | {:<20} | {:<20}", "Modelo da GPU", "Persistence Mode", "Status");
        println!("{:-<35}-+-{:-<20}-+-{:-<20}", "", "", "");
        for gpu in &status.nvidia_gpus {
            let mode = match gpu.persistence_mode {
                Some(true) => "Enabled",
                Some(false) => "Disabled",
                None => "Desconhecido",
            };
            let st = if gpu.is_optimized {
                "[OK] OTIMIZADO (Ativo)"
            } else {
                "[AVISO] Desativado"
            };
            println!("{:<35} | {:<20} | {:<20}", gpu.gpu_name, mode, st);
        }
        println!("\n   - Conformidade NVIDIA: {}", if status.is_nvidia_optimized() {
            "[OK] TODAS AS GPUS NVIDIA ESTÃO COM PERSISTENCE MODE ATIVO"
        } else {
            "[AVISO] Modo de persistência desativado"
        });
    }

    println!("\n--------------------------------------------------------------------------------");
    println!("Conformidade Geral do Sistema: {}", if status.is_all_optimized() {
        "FILA DE QUADROS OTIMIZADA PARA RESPOSTA INSTANTÂNEA NO CS2!"
    } else {
        "OTIMIZAÇÕES DE FILA DE QUADROS AINDA NÃO FORAM APLICADAS EM TODAS AS GPUS."
    });
    println!("--------------------------------------------------------------------------------\n");
}

fn print_usage() {
    println!("Uso: cargo run -p tweaks --example test_render_queue -- [OPÇÃO]\n");
    println!("Opções:");
    println!("  --apply            Aplica otimização em todas as GPUs detectadas (AMD e/ou NVIDIA)");
    println!("  --apply-amd        Aplica FlipQueueSize=1 apenas para GPUs AMD");
    println!("  --apply-nvidia     Aplica nvidia-smi -pm 1 apenas para GPUs NVIDIA");
    println!("  --rollback         Restaura padrão de fábrica em todas as GPUs detectadas");
    println!("  --rollback-amd     Remove FlipQueueSize das GPUs AMD");
    println!("  --rollback-nvidia  Desativa Persistence Mode nas GPUs NVIDIA (nvidia-smi -pm 0)");
    println!("  --status           Exibe status detalhado atual de todas as GPUs");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str()).unwrap_or("--status");

    match action {
        "--apply" => {
            println!("Aplicando otimização na Fila de Quadros (Render Queue / Flip Queue)...");
            match apply_render_queue() {
                Ok(_) => {
                    println!("[SUCESSO] Otimização aplicada com sucesso!");
                    if let Ok(st) = get_render_queue_status() {
                        print_status(&st);
                    }
                }
                Err(e) => eprintln!("[ERRO] Falha ao aplicar otimização: {}", e),
            }
        }
        "--apply-amd" => {
            println!("Definindo FlipQueueSize = 1 em adaptadores de vídeo AMD...");
            match apply_amd_flip_queue() {
                Ok(_) => println!("[SUCESSO] FlipQueueSize definido como 1 (formato binário 0x31, 0x00) com sucesso!"),
                Err(e) => eprintln!("[ERRO] Falha ao configurar FlipQueueSize: {}", e),
            }
        }
        "--apply-nvidia" => {
            println!("Configurando driver NVIDIA para persistência e resposta rápida (nvidia-smi -pm 1)...");
            match apply_nvidia_persistence_mode() {
                Ok(_) => println!("[SUCESSO] Driver NVIDIA configurado para persistência e resposta rápida!"),
                Err(e) => eprintln!("[ERRO] Falha ao configurar NVIDIA: {}", e),
            }
        }
        "--rollback" => {
            println!("Restaurando padrão de fábrica da Fila de Quadros...");
            match rollback_render_queue() {
                Ok(_) => {
                    println!("[SUCESSO] Padrão de fábrica restaurado com sucesso!");
                    if let Ok(st) = get_render_queue_status() {
                        print_status(&st);
                    }
                }
                Err(e) => eprintln!("[ERRO] Falha ao restaurar padrão: {}", e),
            }
        }
        "--rollback-amd" => {
            println!("Removendo FlipQueueSize dos adaptadores AMD...");
            match rollback_amd_flip_queue() {
                Ok(_) => println!("[SUCESSO] FlipQueueSize removido com sucesso. Driver AMD voltou ao padrão de fábrica!"),
                Err(e) => eprintln!("[ERRO] Falha ao remover FlipQueueSize: {}", e),
            }
        }
        "--rollback-nvidia" => {
            println!("Desativando Modo de Persistência no driver NVIDIA (nvidia-smi -pm 0)...");
            match rollback_nvidia_persistence_mode() {
                Ok(_) => println!("[SUCESSO] Modo de persistência desativado (Padrão NVIDIA restaurado)!"),
                Err(e) => eprintln!("[ERRO] Falha ao desativar persistência NVIDIA: {}", e),
            }
        }
        "--status" => match get_render_queue_status() {
            Ok(status) => print_status(&status),
            Err(e) => eprintln!("[ERRO] Falha ao consultar status da Render Queue: {}", e),
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
