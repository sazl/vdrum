//! lxr-emu CLI.
//!
//!   lxr-emu render (--img <LXRV2_update_v1.71.img> | --elf <fw.elf>) <scenario.json> --out <file.raw> [--fz]
//!   lxr-emu hash   (--img <img> | --elf <elf>) <scenario.json | dir>... [--json <out.json>] [--jobs N] [--fz]
//!   lxr-emu check  (--img <img> | --elf <elf>) <reference.json> <scenario dir> [--jobs N]
//!
//! Output format: per block DAC1 (128 x int16 LE, L/R interleaved) then DAC2 (128 x int16).
//! `hash` prints the SHA-256 of that stream per scenario. `--fz` renders audio with
//! FPSCR.FZ = 1 (TIM2 interrupt context) instead of the main-loop FPSCR.
//! A directory argument means every *.json in it except params.json and reference_*.json.
use anyhow::{bail, Context, Result};
use lxr_emu::scenario::{self, Scenario};
use lxr_emu::{AudioContext, EmuEngine};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Clone)]
struct Args {
    cmd: String,
    img: Option<String>,
    elf: Option<String>,
    out: Option<String>,
    json: Option<String>,
    fz: bool,
    jobs: usize,
    files: Vec<String>,
}

fn parse() -> Result<Args> {
    let mut it = std::env::args().skip(1);
    let cmd = it.next().context("usage: lxr-emu render|hash|check ...")?;
    let mut a = Args {
        cmd,
        img: None,
        elf: None,
        out: None,
        json: None,
        fz: false,
        jobs: 1,
        files: vec![],
    };
    while let Some(x) = it.next() {
        match x.as_str() {
            "--img" => a.img = Some(it.next().context("--img <path>")?),
            "--elf" => a.elf = Some(it.next().context("--elf <path>")?),
            "--out" => a.out = Some(it.next().context("--out <path>")?),
            "--json" => a.json = Some(it.next().context("--json <path>")?),
            "--jobs" => a.jobs = it.next().context("--jobs N")?.parse()?,
            "--fz" => a.fz = true,
            _ => a.files.push(x),
        }
    }
    Ok(a)
}

fn engine(a: &Args) -> Result<EmuEngine> {
    let mut e = match (&a.img, &a.elf) {
        (Some(i), None) => EmuEngine::new_shipped(i)?,
        (None, Some(e)) => EmuEngine::new_elf(e)?,
        _ => bail!("give exactly one of --img / --elf"),
    };
    if a.fz {
        e.set_audio_context(AudioContext::Tim2Interrupt);
    }
    Ok(e)
}

fn scenario_files(args: &[String]) -> Result<Vec<PathBuf>> {
    let mut out = vec![];
    for f in args {
        let p = PathBuf::from(f);
        if p.is_dir() {
            let mut v: Vec<PathBuf> = std::fs::read_dir(&p)?
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    let n = p.file_name().unwrap().to_string_lossy().to_string();
                    n.ends_with(".json") && n != "params.json" && !n.starts_with("reference")
                })
                .collect();
            v.sort();
            out.extend(v);
        } else {
            out.push(p);
        }
    }
    Ok(out)
}

/// Render all scenarios on `jobs` threads; returns name -> (hash, ticks, seconds).
fn hash_all(a: &Args, files: &[PathBuf]) -> Result<BTreeMap<String, (String, u64, f64)>> {
    let jobs = a.jobs.max(1);
    let chunks: Vec<Vec<PathBuf>> = (0..jobs)
        .map(|j| files.iter().skip(j).step_by(jobs).cloned().collect())
        .collect();
    let results = std::thread::scope(|s| {
        let hs: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                let a = a.clone();
                s.spawn(move || -> Result<Vec<(String, String, u64, f64)>> {
                    let mut r = vec![];
                    for f in chunk {
                        let sc = Scenario::load(&f)?;
                        let mut eng = engine(&a)?;
                        let t = Instant::now();
                        let (_, h) =
                            scenario::render(&mut eng, &sc).with_context(|| format!("{f:?}"))?;
                        r.push((sc.name.clone(), h, sc.ticks, t.elapsed().as_secs_f64()));
                    }
                    Ok(r)
                })
            })
            .collect();
        hs.into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    let mut out = BTreeMap::new();
    for r in results {
        for (n, h, t, s) in r? {
            if out.insert(n.clone(), (h, t, s)).is_some() {
                bail!("duplicate scenario name {n}");
            }
        }
    }
    Ok(out)
}

fn main() -> Result<()> {
    let a = parse()?;
    match a.cmd.as_str() {
        "render" => {
            let [file] = a.files.as_slice() else {
                bail!("render takes one scenario")
            };
            let sc = Scenario::load(file)?;
            let mut eng = engine(&a)?;
            let t = Instant::now();
            let (bytes, h) = scenario::render(&mut eng, &sc)?;
            let dt = t.elapsed().as_secs_f64();
            let out = a.out.clone().context("--out <file.raw>")?;
            std::fs::write(&out, &bytes)?;
            eprintln!(
                "{} ticks in {:.2} s ({:.0} ticks/s)",
                sc.ticks,
                dt,
                sc.ticks as f64 / dt
            );
            println!("{h}  {}", sc.name);
        }
        "hash" | "hashes" => {
            let files = scenario_files(&a.files)?;
            let t0 = Instant::now();
            let res = hash_all(&a, &files)?;
            let dt = t0.elapsed().as_secs_f64();
            let ticks: u64 = res.values().map(|r| r.1).sum();
            let busy: f64 = res.values().map(|r| r.2).sum();
            for (n, (h, t, s)) in &res {
                println!("{h}  {n}  ({t} ticks, {s:.2} s)");
            }
            eprintln!(
                "{} scenarios, {ticks} ticks, wall {dt:.1} s with {} jobs; {:.0} ticks/s per job (render only)",
                res.len(),
                a.jobs,
                ticks as f64 / busy
            );
            if let Some(j) = &a.json {
                let m: BTreeMap<&String, &String> = res.iter().map(|(n, r)| (n, &r.0)).collect();
                std::fs::write(j, serde_json::to_string_pretty(&m)? + "\n")?;
            }
        }
        "check" => {
            let [reference, rest @ ..] = a.files.as_slice() else {
                bail!("check <reference.json> <scenarios>")
            };
            let want: BTreeMap<String, String> =
                serde_json::from_str(&std::fs::read_to_string(reference)?)?;
            let files = scenario_files(rest)?;
            let res = hash_all(&a, &files)?;
            let mut bad = 0;
            for (n, (h, _, _)) in &res {
                match want.get(n) {
                    Some(w) if w == h => {}
                    Some(w) => {
                        bad += 1;
                        println!("DIFF {n}: {h} (reference {w})");
                    }
                    None => println!("NEW  {n}: {h}"),
                }
            }
            println!("{} scenarios, {bad} differ", res.len());
            if bad > 0 {
                std::process::exit(1);
            }
        }
        c => bail!("unknown command {c}"),
    }
    Ok(())
}
