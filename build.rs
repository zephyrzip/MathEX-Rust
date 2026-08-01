fn main() {
    // Tell Cargo to re-run this build script only if build.rs itself changes
    println!("cargo:rerun-if-changed=build.rs");
}
