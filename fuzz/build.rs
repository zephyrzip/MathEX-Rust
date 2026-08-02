fn main() {
    // Compile the renamed C original to link alongside our Rust code
    cc::Build::new()
        .file("c_src/expression.c")
        .compile("original_mathex");
}