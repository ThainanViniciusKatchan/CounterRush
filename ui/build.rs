fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_bundled_translations("../locale")
        // Os catálogos em ../locale foram escritos sem msgctxt (sem contexto por
        // componente), então o contexto padrão precisa ser desativado para que os
        // textos de @tr(...) sejam encontrados nos arquivos .po.
        .with_default_translation_context(slint_build::DefaultTranslationContext::None);
    slint_build::compile_with_config("main.slint", config).expect("Falha ao compilar arquivos Slint");

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icons/app.ico");
        res.set_manifest_file("counterrush.exe.manifest");
        res.compile().expect("Falha ao compilar recurso de ícone e manifesto do Windows");
    }
}
