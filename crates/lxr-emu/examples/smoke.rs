use lxr_emu::Firmware;
fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap();
    let mut fw = Firmware::from_elf(&std::fs::read(path)?)?;
    // DecayEg: init + setDecay + calc
    let eg = fw.alloc(64)?;
    fw.call("DecayEg_init", &[eg], &[])?;
    fw.call("DecayEg_setDecay", &[eg, 40], &[])?;
    fw.call("DecayEg_trigger", &[eg], &[])?;
    let mut v = vec![];
    for _ in 0..5 {
        v.push(fw.call("DecayEg_calc", &[eg], &[])?.1);
    }
    println!("{v:?}");
    let t = std::time::Instant::now();
    for _ in 0..100000 {
        fw.call("DecayEg_calc", &[eg], &[])?;
    }
    println!("100k calls {:?}", t.elapsed());
    Ok(())
}
