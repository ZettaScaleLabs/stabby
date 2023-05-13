#[stabby::import(name = "library")]
extern "C" {
    pub fn stable_fn(v: u8);
}

#[stabby::import(canaries = "", name = "library")]
#[allow(improper_ctypes)]
extern "C" {
    pub fn unstable_fn(v: &[u8]);
}

fn main() {
    stable_fn(5);
    unsafe { unstable_fn(&[1, 2, 3, 4]) };
}
