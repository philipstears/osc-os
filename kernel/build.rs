use std::env;

fn main() {
    let target = env::var("TARGET").expect("TARGET missing");
    println!("cargo:rerun-if-changed=../Makefile");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=arch/{}.json", target);
    println!("cargo:rerun-if-changed=arch/{}.ld", target);
}
