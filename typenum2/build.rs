use std::{fs::File, io::BufWriter, path::PathBuf};

fn u(mut i: u128) -> String {
    let mut result = "UTerm".into();
    let mut ids = Vec::new();
    while i > 0 {
        let bit = i & 1;
        ids.push(bit as u8);
        i >>= 1;
    }
    for bit in ids.into_iter().rev() {
        result = format!("UInt<{result}, B{bit}>");
    }
    result
}

fn main() {
    use std::io::Write;
    let padding_rs = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("unsigned.rs");
    let mut padding_file = BufWriter::new(File::create(padding_rs).unwrap());

    for i in 0..1024 {
        let u = u(i as u128);
        writeln!(padding_file, "pub type U{i} = {u};").unwrap();
    }
    for i in [
        u16::MAX as u128,
        u32::MAX as u128,
        u64::MAX as u128,
        u128::MAX,
    ] {
        let u = u(i);
        writeln!(padding_file, "pub type U{i} = {u};").unwrap();
    }
}
