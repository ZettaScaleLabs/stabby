#[stabby::export]
pub extern "C" fn stable_fn(v: u8) {
    println!("{v}")
}

#[stabby::export(canaries)]
pub extern "C" fn unstable_fn(v: &[u8]) {
    println!("{v:?}")
}

// #[stabby::import(canaries = "", name = "test")]
// extern "C" {
//     pub fn imported_fn2();
// }
// #[stabby::import(name = "test")]
// extern "C" {
//     pub fn imported_fn3(_: u8);
// }

// fn test() {
//     imported_fn3(4);
//     unsafe { imported_fn2() }
// }
