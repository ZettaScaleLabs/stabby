#[cfg(feature = "libloading")]
fn main() {
    use stabby::libloading::StabbyLibrary;
    unsafe {
        let lib = libloading::Library::new("./libfunctions.so").unwrap();
        let stable_fn = lib.get_stabbied::<extern "C" fn(u8)>(b"stable_fn").unwrap();
        let unstable_fn = lib
            .get_canaried::<extern "C" fn(&[u8])>(b"unstable_fn")
            .unwrap();
        stable_fn(5);
        unstable_fn(&[1, 2, 3, 4]);
    }
}
