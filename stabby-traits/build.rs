pub fn main() {
    use std::io::Write;
    let holes_rs = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("holes.rs");
    let holes = std::fs::File::create(holes_rs).unwrap();
    let mut holes = std::io::BufWriter::new(holes);
    let bits = (0..256).map(|i| format!("Bit{}", i)).collect::<Vec<_>>();
    let bitstr = bits.join(", ");
    writeln!(
        holes,
        "pub struct Holes<{bitstr}>(core::marker::PhantomData<({bitstr})>);"
    )
    .unwrap();
    fn bitfield(b: usize) -> String {
        let mut bits = (0..64).map(|i| format!("((Bit{b}::U8 as u64) << {i})", b = b * 64 + i));
        let mut res = bits.next().unwrap();
        for bit in bits {
            res += " | ";
            res += bit.as_str();
        }
        res
    }
    writeln!(
        holes,
        r"impl<{bounds}>
Holes<{bitstr}> {{
		pub const BITFIELD: [u64; 4] = [
			{a},
			{b},
			{c},
			{d}
		];
	}}",
        bounds = bits.join(": typenum::Bit, ") + ": typenum::Bit,",
        a = bitfield(0),
        b = bitfield(1),
        c = bitfield(2),
        d = bitfield(3),
    )
    .unwrap();
    let bits2 = bits.iter().map(|s| s.clone() + "R").collect::<Vec<_>>();
    let bitstr2 = bits2.join(", ");
    let mut impl_op = |op, opfn| {
        writeln!(
            holes,
            r"impl<{bounds}>
	{op}<Holes<{bitstr2}>>
	for Holes<{bitstr}> {{
		type Output = Holes<{output}>;
		fn {opfn}(self, _: Holes<{bitstr2}>) -> Self::Output {{
			Holes(core::marker::PhantomData)
		}}
	}}",
            bounds = bits
                .iter()
                .zip(bits2.iter())
                .fold(bits2.join(", ") + ",", |acc, (b, br)| acc
                    + format!("{b}: {op}<{br}>, ").as_str()),
            output = (0..256)
                .map(|i| format!("<Bit{i} as {op}<Bit{i}R>>::Output, "))
                .fold(String::new(), |acc, it| acc + it.as_str())
        )
        .unwrap();
    };
    impl_op("core::ops::BitOr", "bitor");
    impl_op("core::ops::BitAnd", "bitand");
    writeln!(
        holes,
        r"impl<{bounds}>
core::ops::Not
for Holes<{bitstr}> {{
	type Output = Holes<{output}>;
	fn not(self) -> Self::Output {{
		Holes(core::marker::PhantomData)
	}}
}}",
        bounds = bits.iter().fold(String::new(), |acc, b| acc
            + format!("{b}: core::ops::Not, ").as_str()),
        output = (0..256)
            .map(|i| format!("<Bit{i} as core::ops::Not>::Output, "))
            .fold(String::new(), |acc, it| acc + it.as_str())
    )
    .unwrap();
}
