fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_bundled_translations("../locale");
    slint_build::compile_with_config("main.slint", config).expect("Falha ao compilar arquivos Slint");
}
