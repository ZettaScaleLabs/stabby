#[stabby::export]
pub extern "C" fn stable_fn(v: u8) {
    println!("{v}")
}

#[stabby::export(canaries)]
pub extern "C" fn unstable_fn(v: &[u8]) {
    println!("{v:?}")
}
