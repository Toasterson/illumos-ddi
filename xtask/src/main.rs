//! Builds and links illumos kernel modules written against `ddi`.
//!
//! Five subcommands are available. `build <pkg>` compiles the package for
//! the kernel target with the pinned nightly and `-Zbuild-std`, producing
//! `<pkg>.a`. `link <pkg>` runs on illumos only and links that archive into
//! a kernel module with `ld -ztype=kmod`. `deps <pkg>` prints the module
//! dependencies and mapfiles the crate graph implies. `lint-man` checks
//! every manual reference in the `-sys` crates against a gate checkout and,
//! with `--coverage`, lists the section 9F names that are not bound yet.
//! `kernel-check` compares the layouts and symbols of the `-sys` crates with
//! a running kernel through an `mdb(1)` script.
//!
//! ```text
//! cargo xtask build <pkg>
//! cargo xtask link  <pkg>
//! cargo xtask deps  <pkg>
//! cargo xtask lint-man [--gate <illumos-gate>] [--coverage | --markdown]
//! cargo xtask kernel-check --emit | ssh root@host mdb -k | cargo xtask kernel-check --verify -
//! ```
//!
//! The `-N` dependencies and the mapfile names are collected from the
//! `[package.metadata.kmod]` table of every crate in the package's
//! dependency graph. A driver therefore never spells them out itself; see
//! section 6 of `docs/DESIGN.md`.

mod kernel_check;
mod lint_man;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const KMOD_TARGET: &str = "targets/x86_64-illumos-kmod.json";
// Pinned because -Zbuild-std tracks nightly closely; bump deliberately.
const NIGHTLY: &str = "nightly-2026-04-16";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let r = match args.as_slice() {
        [cmd, pkg] if cmd == "build" => build(pkg, true),
        [cmd, pkg, dbg] if cmd == "build" && dbg == "--debug" => build(pkg, false),
        [cmd, pkg] if cmd == "link" => link(pkg),
        [cmd, pkg] if cmd == "deps" => deps(pkg).map(|d| {
            println!("-N: {:?}", d.needed);
            println!("mapfiles: {:?}", d.mapfiles);
            println!("class: {}", d.class);
        }),
        [cmd, rest @ ..] if cmd == "lint-man" => lint_man::run(rest),
        [cmd, rest @ ..] if cmd == "kernel-check" => kernel_check::run(rest),
        _ => Err("usage: cargo xtask (build [--debug]|link|deps) <package> | lint-man [--gate DIR] [--coverage] | kernel-check --emit|--verify FILE".into()),
    };
    match r {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("xtask: {e}");
            ExitCode::FAILURE
        }
    }
}

struct KmodDeps {
    needed: BTreeSet<String>,
    mapfiles: BTreeSet<String>,
    class: String,
    conf: Option<PathBuf>,
}

fn workspace_root() -> PathBuf {
    let out = Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format=plain"])
        .output()
        .expect("cargo");
    let s = String::from_utf8_lossy(&out.stdout);
    Path::new(s.trim()).parent().expect("workspace root").to_path_buf()
}

/// Reads `cargo metadata` and merges the `[package.metadata.kmod]` tables of
/// every workspace crate into one set of dependencies and mapfiles.
fn deps(pkg: &str) -> Result<KmodDeps, String> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output()
        .map_err(|e| e.to_string())?;
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    let packages = meta["packages"].as_array().ok_or("no packages")?;
    let mut d = KmodDeps { needed: BTreeSet::new(), mapfiles: BTreeSet::new(), class: "drv".into(), conf: None };
    let mut found = false;
    for p in packages {
        let kmod = &p["metadata"]["kmod"];
        if kmod.is_null() {
            continue;
        }
        let is_target = p["name"] == pkg;
        found |= is_target;
        for n in kmod["depends"].as_array().into_iter().flatten() {
            d.needed.insert(n.as_str().unwrap_or_default().to_string());
        }
        for m in kmod["mapfiles"].as_array().into_iter().flatten() {
            d.mapfiles.insert(m.as_str().unwrap_or_default().to_string());
        }
        if is_target {
            if let Some(c) = kmod["class"].as_str() {
                d.class = c.to_string();
            }
            if let Some(c) = kmod["conf"].as_str() {
                let dir = Path::new(p["manifest_path"].as_str().unwrap_or_default()).parent().unwrap();
                d.conf = Some(dir.join(c));
            }
        }
    }
    if !found {
        return Err(format!("package {pkg} not found in workspace"));
    }
    Ok(d)
}

fn build(pkg: &str, release: bool) -> Result<(), String> {
    let root = workspace_root();
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&root)
        .arg(format!("+{NIGHTLY}"))
        .args(["build", "-p", pkg, "--target"])
        .arg(root.join(KMOD_TARGET))
        .args(["-Zjson-target-spec", "-Zbuild-std=core,alloc", "-Zbuild-std-features=compiler-builtins-mem"])
        .env("RUSTFLAGS", "-C symbol-mangling-version=v0 -C force-frame-pointers=yes");
    if release {
        cmd.arg("--release");
    }
    println!("{cmd:?}");
    let st = cmd.status().map_err(|e| e.to_string())?;
    if !st.success() {
        return Err("cargo build failed".into());
    }
    println!("archive: target/x86_64-illumos-kmod/{}/{pkg}.a", if release { "release" } else { "debug" });
    Ok(())
}

fn link(pkg: &str) -> Result<(), String> {
    if std::env::consts::OS != "illumos" {
        return Err("link needs the illumos ld(1); run this step on an illumos host".into());
    }
    let root = workspace_root();
    let d = deps(pkg)?;
    let archive = root.join(format!("target/x86_64-illumos-kmod/release/{pkg}.a"));
    let out = root.join(format!("target/x86_64-illumos-kmod/release/{pkg}"));
    let mut cmd = Command::new("ld");
    cmd.arg("-ztype=kmod");
    for n in &d.needed {
        cmd.arg(format!("-N{n}"));
    }
    if let Ok(gate) = std::env::var("ILLUMOS_GATE") {
        cmd.args(["-z", "defs"]);
        for m in &d.mapfiles {
            cmd.arg("-M").arg(format!("{gate}/usr/src/uts/common/mapfiles/{m}.mapfile"));
        }
    }
    cmd.args(["-z", "allextract"]).arg(&archive).arg("-o").arg(&out);
    println!("{cmd:?}");
    let st = cmd.status().map_err(|e| e.to_string())?;
    if !st.success() {
        return Err("ld failed".into());
    }
    println!("kernel module: {}", out.display());
    if let Some(c) = d.conf {
        println!(
            "install the conf file {} as /kernel/drv/{pkg}.conf and the module as /kernel/{}/amd64/{pkg}",
            c.display(),
            d.class
        );
    }
    Ok(())
}
