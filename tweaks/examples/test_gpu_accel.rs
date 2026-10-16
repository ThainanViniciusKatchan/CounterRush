use std::env;
use tweaks::{
    disable_all_gpu_accel, disable_chromium_gpu_accel, disable_discord_gpu_accel,
    disable_firefox_gpu_accel, disable_steam_gpu_accel, enable_all_gpu_accel,
    enable_chromium_gpu_accel, enable_discord_gpu_accel, enable_firefox_gpu_accel,
    enable_steam_gpu_accel, ChromiumGpuAccelTweak, DiscordGpuAccelTweak, FirefoxGpuAccelTweak,
    SteamGpuAccelTweak, Tweak,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = args.get(1).map(|s| s.as_str());

    println!("=== Teste de Aceleração de GPU por Aplicativo ===");

    match action {
        Some("--disable-discord") => {
            println!("Desativando aceleração de GPU no Discord...");
            match disable_discord_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU desativada no Discord!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--enable-discord") => {
            println!("Reativando aceleração de GPU no Discord (Rollback)...");
            match enable_discord_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU reativada no Discord!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--disable-steam") => {
            println!("Desativando aceleração de GPU na Steam...");
            match disable_steam_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU desativada na Steam!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--enable-steam") => {
            println!("Reativando aceleração de GPU na Steam (Rollback)...");
            match enable_steam_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU reativada na Steam!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--disable-chromium") => {
            println!("Desativando aceleração de GPU nos Navegadores Chromium...");
            match disable_chromium_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU desativada no Chrome/Edge/Brave!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--enable-chromium") => {
            println!("Reativando aceleração de GPU nos Navegadores Chromium (Rollback)...");
            match enable_chromium_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU reativada no Chrome/Edge/Brave!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--disable-firefox") => {
            println!("Desativando aceleração de GPU no Firefox...");
            match disable_firefox_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU desativada no Firefox!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--enable-firefox") => {
            println!("Reativando aceleração de GPU no Firefox (Rollback)...");
            match enable_firefox_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU reativada no Firefox!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--disable-all") => {
            println!("Desativando aceleração de GPU em todos os aplicativos...");
            match disable_all_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU desativada em todos!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        Some("--enable-all") => {
            println!("Reativando aceleração de GPU em todos os aplicativos (Rollback Geral)...");
            match enable_all_gpu_accel() {
                Ok(_) => println!("Sucesso: Aceleração de GPU reativada em todos!"),
                Err(e) => eprintln!("Erro: {e}"),
            }
        }
        _ => {
            println!("\nConsultando status atual da aceleração de GPU:\n");

            let discord = DiscordGpuAccelTweak;
            let steam = SteamGpuAccelTweak;
            let chromium = ChromiumGpuAccelTweak;
            let firefox = FirefoxGpuAccelTweak;

            println!("1. {}: {:?}", discord.name(), discord.status().unwrap_or(tweaks::TweakStatus::Unknown));
            println!("2. {}: {:?}", steam.name(), steam.status().unwrap_or(tweaks::TweakStatus::Unknown));
            println!("3. {}: {:?}", chromium.name(), chromium.status().unwrap_or(tweaks::TweakStatus::Unknown));
            println!("4. {}: {:?}", firefox.name(), firefox.status().unwrap_or(tweaks::TweakStatus::Unknown));

            println!("\nPara testar ações diretas com rollback opcional:");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --disable-discord");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --enable-discord");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --disable-steam");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --enable-steam");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --disable-chromium");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --enable-chromium");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --disable-firefox");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --enable-firefox");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --disable-all");
            println!("  cargo run -p tweaks --example test_gpu_accel -- --enable-all");
        }
    }
}
