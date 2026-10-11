fn main() {
    println!("Disparando inicialização do Counter-Strike 2...");
    match windows_api::launch_cs2() {
        Ok(_) => println!("Sucesso: comando de inicialização enviado à Steam."),
        Err(err) => eprintln!("Erro ao iniciar: {err}"),
    }
}
