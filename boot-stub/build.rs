fn main() {
    println!("cargo:rerun-if-changed=../Makefile");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
