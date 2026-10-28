//! Debug probe: boot the firmware and dump state relevant to the engine contract.
//! usage: probe (--img <img> | --elf <elf>)
use lxr_emu::EmuEngine;
fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut e = if args[1] == "--img" {
        EmuEngine::new_shipped(&args[2])?
    } else {
        EmuEngine::new_elf(&args[2])?
    };
    println!(
        "rng calls during init: {} state {:#x}",
        e.rng_calls(),
        e.rng_state()
    );
    let ap = e.addr("activeParameters")?;
    let v = e.fw.read(ap, 302)?;
    println!("activeParameters: {:?}", v);
    let mc = e.addr("midi_MidiChannels")?;
    println!("midi_MidiChannels: {:?}", e.fw.read(mc, 8)?);
    println!(
        "midiParser_txRxFilter: {:#x}",
        e.fw.read_u8(e.addr("midiParser_txRxFilter")?)?
    );
    println!(
        "midiParser_routing: {:#x}",
        e.fw.read_u8(e.addr("midiParser_routing")?)?
    );
    println!(
        "frontParser_activeTrack: {}",
        e.fw.read_u8(e.addr("frontParser_activeTrack")?)?
    );
    println!(
        "menu_activePage: {}",
        e.fw.read_u8(e.addr("menu_activePage")?)?
    );
    println!(
        "systick_ticks: {}",
        e.fw.read_u32(e.addr("systick_ticks")?)?
    );
    println!(
        "bCurrentSampleValid: {:#x} dmaPtr {}",
        e.fw.read_u8(e.addr("bCurrentSampleValid")?)?,
        e.dma_ptr()?
    );
    let fv = e.addr("adc_faderValues")?;
    println!(
        "adc_faderValues: {:?}",
        (0..6)
            .map(|i| e.fw.read_f32(fv + 4 * i).unwrap())
            .collect::<Vec<_>>()
    );
    println!(
        "morphParameters ptr: {:#x}",
        e.fw.read_u32(e.addr("morphParameters")?)?
    );
    let t = std::time::Instant::now();
    e.midi(&[0x90, 48, 100])?;
    for _ in 0..1000 {
        e.tick()?;
    }
    println!(
        "1000 ticks (7 voices) {:?}, rng calls {}",
        t.elapsed(),
        e.rng_calls()
    );
    Ok(())
}
