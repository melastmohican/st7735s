use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Get the target architecture from the environment
    let target = env::var("TARGET").expect("TARGET was not set");

    // Determine the appropriate memory.x file based on the target architecture
    let memory_file = match target.as_str() {
        "thumbv7em-none-eabihf" => "examples/stm32h7-example/memory.x", // Example for ARM Cortex-M4
        "thumbv6m-none-eabi" => "examples/pico-example/memory.x",    // Example for ARM Cortex-M0
        _ => panic!("Unsupported target architecture: {}", target),
    };

    // Get output directory from cargo
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir);

    // Create the memory.x in the output directory by copying the appropriate file
    let dest_path = out_path.join("memory.x");
    fs::copy(memory_file, &dest_path).expect("Failed to copy memory.x file");

    // Tell cargo to re-run the build script if the memory file changes
    println!("cargo:rerun-if-changed={}", memory_file);

    // Tell cargo to look for the linker script in the output directory
    println!("cargo:rustc-link-search={}", out_path.display());

    // Ensure the memory file is propagated to the linker
    //println!("cargo:rustc-link-arg=-Tmemory.x");
}
