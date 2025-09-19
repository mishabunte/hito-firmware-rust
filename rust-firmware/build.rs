fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("src/helloworld.slint", config).unwrap();
    slint_build::print_rustc_flags().unwrap();
}
