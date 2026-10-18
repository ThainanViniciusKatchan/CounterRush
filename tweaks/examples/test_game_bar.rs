use std::env;
use tweaks::{
    disable_all_game_bar_features, disable_gamedvr_captures, disable_game_bar_shortcuts,
    disable_game_mode, disable_xbox_mode, get_game_bar_status, rollback_all_game_bar_features,
    GameBarStatus, GameBarTweak, Tweak,
};

fn print_usage() {
    println!("\nComandos disponíveis:");
    println!("  Desativar tudo (Game Mode + Game Bar + Xbox + Capturas): cargo run -p tweaks --example test_game_bar -- --disable");
    println!("  Desativar apenas Modo de Jogo:                           cargo run -p tweaks --example test_game_bar -- --disable-game-mode");
    println!("  Desativar apenas atalhos Game Bar:                       cargo run -p tweaks --example test_game_bar -- --disable-game-bar");
    println!("  Desativar apenas Modo Xbox:                              cargo run -p tweaks --example test_game_bar -- --disable-xbox-mode");
    println!("  Desativar apenas Capturas (GameDVR):                     cargo run -p tweaks --example test_game_bar -- --disable-captures");
    println!("  Rollback (restaurar padrão Windows):                     cargo run -p tweaks --example test_game_bar -- --rollback");
    println!("  Consultar status no Registro:                            cargo run -p tweaks --example test_game_bar -- --status");
}

fn print_status_table(status: &GameBarStatus) {
    println!("\nValores Atuais no Registro:");
    println!("----------------------------------------------------------------------------------");
    println!(
        "  {:<36} : {} [{}]",
        "AutoGameModeEnabled (Game Mode)",
        status
            .auto_game_mode
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string()),
        if status.is_game_mode_disabled() { "DESATIVADO (Otimizado)" } else { "ATIVADO (Padrão)" }
    );
    println!(
        "  {:<36} : {} [{}]",
        "UseNexusForGameBarEnabled (Game Bar)",
        status
            .use_nexus_game_bar
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string()),
        if status.is_game_bar_shortcuts_disabled() { "DESATIVADO (Otimizado)" } else { "ATIVADO (Padrão)" }
    );
    println!(
        "  {:<36} : {} [{}]",
        "GamingHomeApp (Modo Xbox)",
        status
            .gaming_home_app
            .as_deref()
            .map(|s| if s.is_empty() { "\"\" (Vazio)" } else { s })
            .unwrap_or("Não configurado"),
        if status.is_xbox_mode_disabled() { "DESATIVADO (Otimizado)" } else { "ATIVADO (Padrão)" }
    );
    println!(
        "  {:<36} : {} [{}]",
        "GameDVR_Enabled (GameConfigStore)",
        status
            .gamedvr_enabled
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string()),
        if status.gamedvr_enabled == Some(0) { "DESATIVADO (Otimizado)" } else { "ATIVADO (Padrão)" }
    );
    println!(
        "  {:<36} : {} [{}]",
        "AppCaptureEnabled (GameDVR)",
        status
            .app_capture_enabled
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado".to_string()),
        if status.app_capture_enabled == Some(0) { "DESATIVADO (Otimizado)" } else { "ATIVADO (Padrão)" }
    );
    println!(
        "  {:<36} : {} [{}]",
        "AllowGameDVR (Política HKLM)",
        status
            .allow_gamedvr_policy
            .map(|v| format!("{} (0x{:x})", v, v))
            .unwrap_or_else(|| "Não configurado (Padrão)".to_string()),
        if status.allow_gamedvr_policy == Some(0) { "BLOQUEADO (Otimizado)" } else { "PERMITIDO (Padrão)" }
    );
    println!("----------------------------------------------------------------------------------");

    if status.is_all_disabled() {
        println!("Conformidade Geral: [TODOS OS RECURSOS XBOX/GAMEDVR ESTÃO DESATIVADOS (100% OTIMIZADO)]");
    } else {
        println!("Conformidade Geral: [PARCIALMENTE ATIVADO OU PADRÃO DO WINDOWS]");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    let tweak = GameBarTweak;

    println!("=== Teste: Desativação do Xbox Game Bar, Modo de Jogo e Capturas (GameDVR) ===");
    println!("Nome:         {}", tweak.name());
    println!("ID:           {}", tweak.id());
    println!("Descrição:    {}", tweak.description());
    println!("Requer Admin: {}", tweak.requires_admin());

    match action {
        Some("--disable") => {
            println!("\nDesativando todos os recursos (Modo de Jogo, Game Bar, Modo Xbox e Capturas)...");
            match disable_all_game_bar_features() {
                Ok(_) => {
                    println!("[SUCESSO] Todos os recursos foram desativados no Registro!");
                    if let Ok(st) = get_game_bar_status() {
                        print_status_table(&st);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro ao desativar recursos: {err}"),
            }
            print_usage();
        }
        Some("--disable-game-mode") => {
            println!("\nDesativando Modo de Jogo (AutoGameModeEnabled = 0)...");
            match disable_game_mode() {
                Ok(_) => println!("[SUCESSO] Modo de Jogo desativado!"),
                Err(err) => eprintln!("[FALHA] Erro: {err}"),
            }
            if let Ok(st) = get_game_bar_status() {
                print_status_table(&st);
            }
            print_usage();
        }
        Some("--disable-game-bar") => {
            println!("\nDesativando atalhos da Game Bar (UseNexusForGameBarEnabled = 0)...");
            match disable_game_bar_shortcuts() {
                Ok(_) => println!("[SUCESSO] Atalhos da Game Bar desativados!"),
                Err(err) => eprintln!("[FALHA] Erro: {err}"),
            }
            if let Ok(st) = get_game_bar_status() {
                print_status_table(&st);
            }
            print_usage();
        }
        Some("--disable-xbox-mode") => {
            println!("\nDesativando Modo Xbox (GamingHomeApp = \"\")...");
            match disable_xbox_mode() {
                Ok(_) => println!("[SUCESSO] Modo Xbox desativado!"),
                Err(err) => eprintln!("[FALHA] Erro: {err}"),
            }
            if let Ok(st) = get_game_bar_status() {
                print_status_table(&st);
            }
            print_usage();
        }
        Some("--disable-captures") => {
            println!("\nDesativando Capturas e GameDVR (GameDVR_Enabled = 0, AppCaptureEnabled = 0, AllowGameDVR = 0)...");
            match disable_gamedvr_captures() {
                Ok(_) => println!("[SUCESSO] Capturas e GameDVR desativados!"),
                Err(err) => eprintln!("[FALHA] Erro: {err}"),
            }
            if let Ok(st) = get_game_bar_status() {
                print_status_table(&st);
            }
            print_usage();
        }
        Some("--rollback") => {
            println!("\nExecutando rollback para os valores padrão de fábrica do Windows...");
            match rollback_all_game_bar_features() {
                Ok(_) => {
                    println!("[SUCESSO] Rollback executado! Todos os valores retornaram ao padrão.");
                    if let Ok(st) = get_game_bar_status() {
                        print_status_table(&st);
                    }
                }
                Err(err) => eprintln!("[FALHA] Erro no rollback: {err}"),
            }
            print_usage();
        }
        _ => {
            println!("\nConsultando status atual no Registro...");
            match get_game_bar_status() {
                Ok(st) => print_status_table(&st),
                Err(err) => eprintln!("Erro ao consultar status: {err}"),
            }
            print_usage();
        }
    }
}
