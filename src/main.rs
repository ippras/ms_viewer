#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use ms_viewer::App;

// Native
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    // std::env::set_var("POLARS_FMT_TABLE_CELL_LIST_LEN", "2");
    // std::env::set_var("TRIM_DECIMAL_ZEROS", "256");

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let native_options = Default::default();
    eframe::run_native(
        "MS viewer",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// Web
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();
    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let start_result = eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        let loading_text = web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("loading_text"));
        if let Some(loading_text) = loading_text {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(error) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {error:?}");
                }
            }
        }
    });
}
