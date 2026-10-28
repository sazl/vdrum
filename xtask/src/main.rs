// SPDX-License-Identifier: GPL-3.0-only
//! `cargo xtask bundle vdrum --release` builds `target/bundled/VDrum.{clap,vst3}` and the standalone.
//!
//! `nih_plug_xtask` finds the workspace root as the outermost ancestor directory with a
//! `Cargo.toml`, which is wrong when this workspace is checked out inside another Cargo project (a
//! git worktree inside the main checkout, a vendored copy). `bundle` therefore uses this crate's
//! parent directory; everything else goes to `nih_plug_xtask` unchanged.

use std::path::{Path, PathBuf};

fn main() -> nih_plug_xtask::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("bundle") {
        return nih_plug_xtask::main_with_args("cargo xtask", args);
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("xtask is inside the workspace");
    std::env::set_current_dir(root)?;
    let target_dir = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| root.join("target"));

    // `bundle <package>... [-p <package>]... [cargo build args]`
    let mut packages = Vec::new();
    let mut other = Vec::new();
    let mut it = args.into_iter().skip(1).peekable();
    while let Some(a) = it.next() {
        if a == "-p" || a == "--package" {
            if let Some(p) = it.next() {
                packages.push(p);
            }
        } else if !a.starts_with('-') && other.is_empty() {
            packages.push(a);
        } else {
            other.push(a);
        }
    }
    if packages.is_empty() {
        return Err(std::io::Error::other("usage: cargo xtask bundle <package> [--release] [cargo build args]").into());
    }
    nih_plug_xtask::build(&packages, &other)?;
    for p in &packages {
        nih_plug_xtask::bundle(&target_dir, p, &other, false)?;
    }
    Ok(())
}
