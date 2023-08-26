import sys, os
crates = ["stabby-macros", "stabby-abi", "stabby"]
if __name__ == "__main__":
    ws_root = os.path.dirname(__file__)
    if sys.argv[1] == "publish":
        for crate in crates:
            os.system(f"cd {ws_root}/{crate} && cargo publish {' '.join(sys.argv[2:])}")
    else:
        version = sys.argv[1]
        for crate in crates:
            os.system(f"""sed -i -E 's/^version\s*=.*/version = "{version}"/g' {ws_root}/{crate}/Cargo.toml""")
            os.system(f"""sed -i -E 's/(stabby-.*version\s*=\s*)\"[^\"]*\"(.*)/\\1"{version}"\\2/g' {ws_root}/{crate}/Cargo.toml""")

