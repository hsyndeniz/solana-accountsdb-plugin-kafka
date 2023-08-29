use cargo_lock::Lockfile;

fn main() -> anyhow::Result<()> {
    let lockfile = Lockfile::load("./Cargo.lock")?;

    println!(
        "cargo:rustc-env=GIT_VERSION={}",
        git_version::git_version!()
    );

    println!(
        "cargo:rustc-env=SOLANA_SDK_VERSION={}",
        get_pkg_version(&lockfile, "solana-sdk")
    );

    Ok(())
}

fn get_pkg_version(lockfile: &Lockfile, pkg_name: &str) -> String {
    lockfile
        .packages
        .iter()
        .filter(|pkg| pkg.name.as_str() == pkg_name)
        .map(|pkg| pkg.version.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
