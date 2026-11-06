// SPDX-License-Identifier: GPL-3.0-only
//! Print every parameter id the surface spec can bind (JSON list), for the spec builder's checks:
//! `cargo run -q -p vdrum-surface --example param_ids > tools/surface_spec/surface_spec/param_ids.json`

fn main() {
    println!("{}", vdrum_surface::spec::param_ids_json());
}
