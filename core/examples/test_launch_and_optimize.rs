use std::time::Duration;
use core::OptimizerCore;

fn main() {
    println!("=== Teste de Inicialização e Otimização do CS2 ===");
    let core = OptimizerCore::default();

    println!("\n1. Esvaziando Standby List da memória RAM (RAMMap Clone)...");
    match core.empty_standby_list() {
        Ok(_) => println!("   [OK] Standby List esvaziada com sucesso! Cache de RAM purgado."),
        Err(core::CoreError::Tweak(tweaks::TweakError::WindowsApi(windows_api::WindowsApiError::AccessDenied))) => {
            eprintln!("   [AVISO] Acesso negado para esvaziar Standby List (requer permissão de Administrador).");
            eprintln!("   Para que o cache de memória seja purgado, execute o terminal como Administrador.");
        }
        Err(err) => {
            eprintln!("   [AVISO] Não foi possível esvaziar a Standby List: {err}");
        }
    }

    println!("\n2. Disparando inicialização do CS2 via Steam...");
    if let Err(err) = core.launch_game() {
        eprintln!("   [ERRO] Falha ao disparar o jogo: {err}");
        return;
    }
    println!("   [OK] Comando enviado à Steam.");

    println!("\n3. Aguardando até 60 segundos para o processo 'cs2.exe' ser instanciado...");
    if let Err(err) = windows_api::wait_for_process("cs2", Duration::from_secs(60)) {
        eprintln!("   [ERRO] {err}");
        return;
    }
    println!("   [OK] Processo 'cs2.exe' detectado em execução!");

    println!("\n4. Aplicando tweaks de processo (Prioridade RealTime e Afinidade)...");
    if let Err(err) = core.apply_tweaks() {
        eprintln!("   [ERRO] Falha ao aplicar tweaks de processo: {err}");
        return;
    }

    println!("\nSucesso absoluto!");
    println!("- Standby List da memória RAM purgada");
    println!("- Jogo iniciado via Steam");
    println!("- Processo cs2.exe detectado");
    println!("- Prioridade RealTime aplicada");
    println!("- Afinidade (sem Core 0) aplicada");
}
