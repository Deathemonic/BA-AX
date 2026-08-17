use std::env;

fn main() {
    println!("cargo:rustc-env=BAAX_TARGET={}", env::var("TARGET").expect("TARGET"));
}
