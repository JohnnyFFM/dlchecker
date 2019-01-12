extern crate cc;

fn main() {
    let mut shared_config = cc::Build::new();

    #[cfg(target_env = "msvc")]
    shared_config
        .flag("/O2")
        .flag("/Oi")
        .flag("/Ot")
        .flag("/Oy")
        .flag("/GT")
        .flag("/GL");

    #[cfg(not(target_env = "msvc"))]
    shared_config.flag("-std=c99").flag("-mtune=native");

    let mut config = shared_config.clone();

    config
        .file("src/c/sph_shabal.c")
        .file("src/c/noncegen_32.c")
        .file("src/c/common.c")
        .compile("shabal");
}
