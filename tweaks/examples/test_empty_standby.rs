use tweaks::{EmptyStandbyListTweak, Tweak};

fn main() {
    let tweak = EmptyStandbyListTweak;

    println!("=== Teste do Tweak: Empty Standby List ===");
    println!("Nome:        {}", tweak.name());
    println!("ID:          {}", tweak.id());
    println!("Descrição:   {}", tweak.description());
    println!("Requer Admin: {}", tweak.requires_admin());

    println!("\nIniciando processo de limpeza da Standby List...");
    match tweak.apply() {
        Ok(_) => {
            println!("\n[SUCESSO] Standby List esvaziada com êxito!");
            println!("A memória RAM ocupada por páginas de cache foi purgada pelo kernel.");
            println!("A memória liberada agora está disponível para o sistema e para o jogo.");
        }
        Err(tweaks::TweakError::WindowsApi(windows_api::WindowsApiError::AccessDenied)) => {
            eprintln!("\n[ERRO DE PERMISSÃO] Acesso negado!");
            eprintln!("A limpeza da Standby List exige privilégios de Administrador (SeProfileSingleProcessPrivilege).");
            eprintln!("Por favor, abra o terminal PowerShell ou Prompt de Comando como Administrador e execute novamente:");
            eprintln!("  cargo run -p tweaks --example test_empty_standby");
        }
        Err(err) => {
            eprintln!("\n[FALHA] Erro ao tentar limpar a Standby List: {}", err);
        }
    }
}
