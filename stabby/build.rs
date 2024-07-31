fn main() {
    println!(r#"cargo:rustc-check-cfg=cfg(stabby_unsafe_wakers, values(none(), "true", "false"))"#);
}
