fn main() {
    println!("cargo:rerun-if-changed=../Makefile");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=x86_64-unknown-none-gnu.json");
    println!("cargo:rerun-if-changed=x86_64-unknown-none-gnu.ld");
}
