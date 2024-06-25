import sys, os, re
ws_root = os.path.dirname(__file__)
crates = ["stabby-macros", "stabby-abi", "stabby"]

def factor(x, base):
    n = 0
    while x > 1 and x % base == 0:
        x /= base
        n += 1
    return n

def factor_version(version, base):
    return ".".join([str(factor(int(x), base)) for x in version.split(".")])

if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "publish":
        for crate in crates:
            failure = os.system(f"cd {ws_root}/{crate} && cargo publish {' '.join(sys.argv[2:])}")
            if failure:
                raise f"Failed to release {crate}, stopping publication"
    else:
        changelog = f"{ws_root}/CHANGELOG.md"
        print("Close the CHANGELOG to continue, the topmost version will be picked")
        os.system(f"code --wait {changelog}")
        version = None
        changelog_text = None
        with open(changelog) as clog:
            changelog_text = clog.read()
            for line in changelog_text.splitlines():
                versions = re.findall(r"^#\s+([\d\.]+)", line)
                version = versions[0] if len(versions) else None
                if version is not None:
                    break
        header = f"# {version} (api={factor_version(version, 2)}, abi={factor_version(version, 3)})"
        print(header)
        changelog_text = re.sub(r"^#\s+([\d\.]+)\s*(\(api[^\)]+\))?", header, changelog_text)
        with open(changelog, "w") as clog:
            clog.write(changelog_text)
        
        print(f"Updating Cargo.tomls with version={version}")

        ws_toml = None
        with open(f"{ws_root}/Cargo.toml") as toml:
            ws_toml = toml.read()
        ws_toml = re.sub(r"version\s*=\s*\"[^\"]+(?P<rest>\".*Track)", f"version = \"{version}\\g<rest>", ws_toml)
        with open(f"{ws_root}/Cargo.toml", "w") as toml:
            toml.write(ws_toml)