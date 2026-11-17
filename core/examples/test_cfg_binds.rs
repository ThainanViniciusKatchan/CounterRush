use core::{
    manage_cs2_bind_from_args, resolve_cfg_backup_dir, resolve_cs2_cfg_dir, resolve_local_cfg_dir,
    BindOperationOutcome,
};

fn print_usage() {
    println!("Uso:");
    println!("  Modo padrão (criar ou atualizar preservando o restante):");
    println!("    cargo run -p core --example test_cfg_binds -- <nome_config> <tecla> <comando1> [comando2 ...]");
    println!("    Exemplo (comando único):");
    println!("      cargo run -p core --example test_cfg_binds -- autoexec.cfg f3 \"toggle cl_mute_enemy_team 0 1\"");
    println!("    Exemplo (múltiplos comandos separados por ';'):");
    println!("      cargo run -p core --example test_cfg_binds -- autoexec mouse4 \"+jump\" \"-attack\"");
    println!();
    println!("  Criar nova config do zero (sobrescreve após backup):");
    println!("    cargo run -p core --example test_cfg_binds -- <nome_config> <tecla> <comando1> [comando2 ...] --new_config");
    println!();
    println!("  Restaurar último backup de config\\cfg\\bkp:");
    println!("    cargo run -p core --example test_cfg_binds -- <nome_config> --restore");
}

fn main() {
    println!("=== CounterRush — Gerenciador de Binds do Counter-Strike 2 (.cfg) ===");

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("\n[ERRO DE USO] Parâmetros obrigatórios não informados.\n");
        print_usage();
        std::process::exit(1);
    }

    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!();
        print_usage();
        return;
    }

    println!("\nDiretórios de trabalho:");
    println!("  - Local (software): {}", resolve_local_cfg_dir().display());
    println!("  - Backup automático: {}", resolve_cfg_backup_dir().display());
    println!("  - Global (CS2):     {}", resolve_cs2_cfg_dir().display());
    println!();

    match manage_cs2_bind_from_args(&args) {
        Ok(BindOperationOutcome::Created {
            file_name,
            bind_line,
            local_path,
            cs2_path,
            ..
        }) => {
            println!("[SUCESSO] Arquivo '{file_name}' criado com a bind:");
            println!("  -> {bind_line}");
            println!("  - Salvo em (local): {}", local_path.display());
            println!("  - Salvo em (CS2):   {}", cs2_path.display());
        }
        Ok(BindOperationOutcome::Updated {
            file_name,
            bind_line,
            replaced_existing_key,
            local_path,
            cs2_path,
            backup_path,
        }) => {
            if replaced_existing_key {
                println!("[SUCESSO] Bind existente substituída em '{file_name}':");
            } else {
                println!("[SUCESSO] Arquivo '{file_name}' atualizado com a bind:");
            }
            println!("  -> {bind_line}");
            if let Some(bkp) = backup_path {
                println!("  - Backup salvo em:  {}", bkp.display());
            }
            println!("  - Salvo em (local): {}", local_path.display());
            println!("  - Salvo em (CS2):   {}", cs2_path.display());
        }
        Ok(BindOperationOutcome::Restored {
            file_name,
            backup_path,
            local_path,
            cs2_path,
        }) => {
            println!("[SUCESSO] Backup de '{file_name}' restaurado:");
            println!("  - Origem (bkp):     {}", backup_path.display());
            println!("  - Restaurado local: {}", local_path.display());
            println!("  - Restaurado CS2:   {}", cs2_path.display());
        }
        Ok(BindOperationOutcome::CancelledByUser(conflict)) => {
            println!(
                "[CANCELADO] Substituição da tecla '{}' recusada pelo usuário em '{}'. Nenhuma alteração foi gravada.",
                conflict.key,
                conflict.file_path.display()
            );
        }
        Err(err) => {
            eprintln!("[ERRO] Falha ao gerenciar bind do CS2: {err}");
            std::process::exit(1);
        }
    }
}
