fn main() {
    const _: () = {
        let enabled_features = {
            cfg!(feature = "slint") as u32
                + cfg!(feature = "floem") as u32
                + cfg!(feature = "iced") as u32
                + cfg!(feature = "egui") as u32
                + cfg!(feature = "gpui") as u32
                + cfg!(feature = "wasm") as u32
                + cfg!(feature = "xilem") as u32
                + cfg!(feature = "leptos") as u32
                + cfg!(feature = "rui") as u32
                + cfg!(feature = "ratatui") as u32
        };

        match enabled_features {
            0 => panic!("None of the features were enabled, exactly one must be."),
            1 => {}
            2.. => panic!("You can't compile to run multiple GUIs at the same time"),
        }
    };

    #[cfg(feature = "slint")]
    slint_build::compile("ui/main.slint").unwrap();

    // Auto-patch gpui shaders on macOS to fix Metal compilation
    #[cfg(all(target_os = "macos", feature = "gpui"))]
    patch_gpui_shaders();
}

#[cfg(all(target_os = "macos", feature = "gpui"))]
fn patch_gpui_shaders() {
    // Find cargo home directory
    let cargo_home = std::env::var("CARGO_HOME")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.cargo", h)))
        .expect("Cannot find cargo home directory");

    // Build glob pattern to find the shaders.metal file
    let shader_pattern = format!(
        "{cargo_home}/registry/src/index.crates.io-*/gpui-*/src/platform/mac/shaders.metal"
    );

    // Find and patch the file
    match glob::glob(&shader_pattern) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Err(err) = patch_shader_file(&entry) {
                    println!("cargo:warning=Failed to patch {entry:?}: {err}");
                }
            }
        }
        Err(err) => {
            println!("cargo:warning=Failed to glob pattern {shader_pattern}: {err}");
        }
    }
}

/// See https://github.com/zed-industries/zed/discussions/31809
#[cfg(all(target_os = "macos", feature = "gpui"))]
fn patch_shader_file(path: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;

    // Check if the problematic include is present
    if content.contains("#include <simd/simd.h>") {
        // Remove the problematic include
        let fixed = content.replace(
            "#include <metal_stdlib>\n#include <simd/simd.h>\n",
            "#include <metal_stdlib>\n",
        );

        std::fs::write(path, fixed)?;
        println!("cargo:warning=Patched gpui shaders.metal at {path:?}");
        println!("cargo:rerun-if-changed={}", path.display());
    }

    Ok(())
}
