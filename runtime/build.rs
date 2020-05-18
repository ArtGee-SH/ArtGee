use wasm_builder_runner::WasmBuilder;

fn main() {
    WasmBuilder::new()
        .with_current_project()
        .with_wasm_builder_from_git(
            "https://github.com/paritytech/substrate.git",
            "5a7600912b7fd50091990899e8ad93ad618f40c4",
        )
        .export_heap_base()
        .import_memory()
        .build()
}
