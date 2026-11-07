fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_bundled_translations("../locale")
        // Os catálogos em ../locale foram escritos sem msgctxt (sem contexto por
        // componente), então o contexto padrão precisa ser desativado para que os
        // textos de @tr(...) sejam encontrados nos arquivos .po.
        .with_default_translation_context(slint_build::DefaultTranslationContext::None);
    slint_build::compile_with_config("main.slint", config).expect("Falha ao compilar arquivos Slint");
}
