fn main() {
    // Link to the crypto library
    println!("cargo:rustc-link-search=native=/app/crypto");
    println!("cargo:rustc-link-search=native=/usr/local/lib");
    println!("cargo:rustc-link-lib=dylib=evoting_crypto");
    println!("cargo:rustc-link-lib=dylib=pbc");
    println!("cargo:rustc-link-lib=dylib=gmp");
}
