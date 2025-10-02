fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .embed_resources(slint_build::EmbedResourcesKind::EmbedForSoftwareRenderer);
    slint_build::compile_with_config("src/ui/main.slint", config).unwrap();
    slint_build::print_rustc_flags().unwrap();

    // Build libcrypt0 for simulation
    #[cfg(feature = "minifb")]
    {
        let mut build = cc::Build::new();
        
        // Add libcrypt0 source files
        build.file("src/crypto/libcrypt0/src/crypt0.c")
             .file("src/crypto/libcrypt0/src/crypt0_aes_ccm.c")
             .file("src/crypto/libcrypt0/src/crypt0_bech32.c")
             .file("src/crypto/libcrypt0/src/crypt0_bip32.c")
             .file("src/crypto/libcrypt0/src/crypt0_bip39.c")
             .file("src/crypto/libcrypt0/src/crypt0_bip39_english.c")
             .file("src/crypto/libcrypt0/src/crypt0_crc.c")
             .file("src/crypto/libcrypt0/src/crypt0_ed25519.c")
             .file("src/crypto/libcrypt0/src/crypt0_hmac.c")
             .file("src/crypto/libcrypt0/src/crypt0_hw_keys.c")
             .file("src/crypto/libcrypt0/src/crypt0_key.c")
             .file("src/crypto/libcrypt0/src/crypt0_log.c")
             .file("src/crypto/libcrypt0/src/crypt0_pbkdf2.c")
             .file("src/crypto/libcrypt0/src/crypt0_ripemd160.c")
             .file("src/crypto/libcrypt0/src/crypt0_rlp.c")
             .file("src/crypto/libcrypt0/src/crypt0_rng.c")
             .file("src/crypto/libcrypt0/src/crypt0_secp256k1.c")
             .file("src/crypto/libcrypt0/src/crypt0_sha.c")
             .file("src/crypto/libcrypt0/src/intc_impl.c")
             .file("src/crypto/crc16_ccitt/crc16_ccitt.c");

        // Add include directories
        build.include("src/crypto/libcrypt0/include")
             .include("src/crypto/libcrypt0/lib/secp256k1/src")
             .include("src/crypto/libcrypt0/lib/secp256k1/include")
             .include("src/crypto/libcrypt0/lib/ripemd160")
             .include("src/crypto/libcrypt0/lib/SHA3IUF")
             .include("src/crypto/libcrypt0/lib/base58")
             .include("src/crypto/libcrypt0/lib/bech32")
             .include("src/crypto/libcrypt0/lib/intc")
             .include("src/crypto/crc16_ccitt");

        // Set compiler flags
        build.flag("-DCRYPT0_SIMULATION")
             .flag("-DHITO_PLATFORM_DESKTOP")
             .flag("-DCRYPT0_USE_OPENSSL");
             
        // Add OpenSSL include paths for macOS (common locations)
        #[cfg(target_os = "macos")]
        {
            // Try Homebrew OpenSSL first
            if std::path::Path::new("/opt/homebrew/include/openssl").exists() {
                build.include("/opt/homebrew/include");
            } else if std::path::Path::new("/usr/local/include/openssl").exists() {
                build.include("/usr/local/include");
            }
        }

        build.compile("crypt0");

        println!("cargo:rustc-link-lib=static=crypt0");
        
        // Link OpenSSL for crypto functions on desktop
        #[cfg(target_os = "macos")]
        {
            println!("cargo:rustc-link-lib=dylib=crypto");
            println!("cargo:rustc-link-lib=dylib=ssl");
        }
        
        #[cfg(target_os = "linux")]
        {
            println!("cargo:rustc-link-lib=dylib=crypto");
            println!("cargo:rustc-link-lib=dylib=ssl");
        }
        
        #[cfg(target_os = "windows")]
        {
            println!("cargo:rustc-link-lib=dylib=libcrypto");
            println!("cargo:rustc-link-lib=dylib=libssl");
        }
    }
}
