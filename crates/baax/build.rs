fn main() {
    println!("cargo:rustc-env=BAAX_TARGET={}", std::env::var("TARGET").expect("TARGET"));
}
