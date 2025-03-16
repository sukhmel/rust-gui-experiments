fn main() {
    #[cfg(feature = "slint")]
    slint_build::compile("ui/main.slint").unwrap()
}
