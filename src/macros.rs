#[macro_export]
macro_rules! asset {
    ($path:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), $path))
    };
}
