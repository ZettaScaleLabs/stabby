// use std::{fs::File, io::BufWriter, path::PathBuf};

// fn u(mut i: u16) -> String {
//     let mut result = "UTerm".into();
//     while i > 0 {
//         let bit = i & 1;
//         result = format!("UInt<{result}, B{bit}>");
//         i >>= 1;
//     }
//     result
// }

fn main() {
    // use std::io::Write;
    // let padding_rs = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("padding.rs");
    // let mut padding_file = BufWriter::new(File::create(padding_rs).unwrap());
    // let mut um1 = u(0);
    // for i in 1..128 {
    //     let u = u(i);
    //     writeln!(padding_file, "impl IPadding for {u} {{ type Padding = Tuple2<PadByte, <{um1} as IPadding>::Padding>; }}").unwrap();
    //     um1 = u;
    // }
}
