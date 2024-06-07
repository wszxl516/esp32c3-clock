fn main()  -> anyhow::Result<()> {
    embuild::espidf::sysenv::output();
    for file in std::fs::read_dir("ui")? {
        println!("cargo:rerun-if-changed={}", file?.path().display());
    }
    slint_build::compile_with_config(
        "ui/main.slint",
        slint_build::CompilerConfiguration::new()
            .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer)
            .with_style("native".into())
    )?;
    Ok(())
}
