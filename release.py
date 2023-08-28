import sys, os, re
ws_root = os.path.dirname(__file__)
crates = ["stabby-macros", "stabby-abi", "stabby"]
if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "publish":
        for crate in crates:
            failure = os.system(f"cd {ws_root}/{crate} && cargo publish {' '.join(sys.argv[2:])}")
            if failure:
                raise f"Failed to release {crate}, stopping publication"
    else:
        changelog = f"{ws_root}/CHANGELOG.md"
        os.system(f"code --wait {changelog}")
        version = None
        with open(changelog) as clog:
            while version is None:
                line = clog.readline()
                versions = re.findall("^#\s+([^:\n]+)", line)
                version = versions[0] if len(versions) else None
        print(f"Updating Cargo.tomls with version={version}")
        for crate in crates:
            vupdate = f"""sed -i -E 's/^version\s*=.*/version = \"{version}\"/g' {ws_root}/{crate}/Cargo.toml"""
            os.system(vupdate)
            print(vupdate)
            dupdate = f"""sed -i -E 's/(stabby-.*version\s*=\s*)\"[^\"]*\"(.*)/\\1\"{version}\"\\2/g' {ws_root}/{crate}/Cargo.toml"""
            print(dupdate)
            os.system(dupdate)

