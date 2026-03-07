fn main() {
    // Windows icon resource — embed_resource is a no-op on other platforms,
    // but we gate it explicitly for clarity.
    #[cfg(windows)]
    embed_resource::compile("app-icon.rc", embed_resource::NONE);
}
