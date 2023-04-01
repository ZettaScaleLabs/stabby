use std::process::Command;

fn encode(value: String) -> String {
    value
}

fn main() -> Result<(), std::io::Error> {
    use std::{
        fs::File,
        io::{BufWriter, Write},
        path::PathBuf,
    };
    let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let output = String::from_utf8(
        Command::new(rustc)
            .arg("-v")
            .arg("-V")
            .output()
            .expect("Couldn't get rustc version")
            .stdout,
    )
    .unwrap();
    let env_vars = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("env_vars.rs");
    let mut env_vars = BufWriter::new(File::create(env_vars).unwrap());
    let mut rustc: [u16; 3] = [0; 3];
    let mut llvm: [u16; 3] = [0; 3];
    let mut commit = "";
    for line in output.lines() {
        let line = line.trim();
        if let Some(release) = line.strip_prefix("release: ") {
            for (i, s) in release.split('.').enumerate() {
                rustc[i] = s.parse().unwrap_or(0);
            }
        }
        if let Some(release) = line.strip_prefix("LLVM version: ") {
            for (i, s) in release.split('.').enumerate() {
                llvm[i] = s.parse().unwrap_or(0);
            }
        }
        if let Some(hash) = line.strip_prefix("commit-hash: ") {
            commit = hash;
        }
    }
    writeln!(
        env_vars,
        r#"pub (crate) const RUSTC_COMMIT: &str = "{commit}";"#
    )?;
    writeln!(
        env_vars,
        "pub (crate) const RUSTC_MAJOR: u16 = {};",
        rustc[0]
    )?;
    writeln!(
        env_vars,
        "pub (crate) const RUSTC_MINOR: u16 = {};",
        rustc[1]
    )?;
    writeln!(
        env_vars,
        "pub (crate) const RUSTC_PATCH: u16 = {};",
        rustc[2]
    )?;
    // writeln!(env_vars, "pub (crate) const LLVM_MAJOR: u16 = {};", llvm[0])?;
    // writeln!(env_vars, "pub (crate) const LLVM_MINOR: u16 = {};", llvm[1])?;
    // writeln!(env_vars, "pub (crate) const LLVM_PATCH: u16 = {};", llvm[2])?;
    for (key, value) in ["OPT_LEVEL", "DEBUG", "NUM_JOBS", "TARGET", "HOST"]
        .iter()
        .filter_map(|&name| std::env::var(name).map_or(None, |val| Some((name, val))))
    {
        writeln!(
            env_vars,
            r#"pub (crate) const {key}: &str = "{}";"#,
            encode(value)
        )?;
    }
    Ok(())
}
