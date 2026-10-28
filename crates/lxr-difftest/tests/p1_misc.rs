//! P1 native checks for functions that are not in the firmware build (no reference to diff against):
//! medianFilter (checked against a sorted-window median) and the BufferTools C semantics.
use lxr_core::dsp::buffer_tools::*;
use lxr_core::dsp::median_filter::*;
use lxr_difftest::Sweep;

#[test]
fn median_filter_matches_window_median() {
    let mut sw = Sweep(0x5eed_0001_2345);
    let mut st = MedianFilterState::INIT;
    let mut window: Vec<u32> = Vec::new();
    for n in 0..20_000 {
        let d = match sw.next_u32() % 4 {
            0 => sw.next_u32() % 16,
            _ => sw.next_u32() | 1,
        };
        let r = medianFilter_state(&mut st, d);
        window.push(if d == 0 { 1 } else { d });
        if window.len() > 7 {
            window.remove(0);
        }
        if n >= 6 {
            let mut s = window.clone();
            s.sort_unstable();
            assert_eq!(r, s[3], "sample {n}");
        }
    }
}

#[test]
fn buffer_tools_c_semantics() {
    let mut a = [i16::MAX, -5, i16::MIN, 7, 3];
    let mut b = [1i16, 5, -1, 1, 100];
    bufferTool_addBuffersSaturating(&mut a, &mut b, 5);
    assert_eq!(a, [i16::MAX, 0, i16::MIN, 8, 3]);
    let mut c = [1i16; 5];
    bufferTool_clearBuffer(&mut c, 5);
    assert_eq!(c, [0, 0, 0, 0, 1]);
    let mut g = [1000i16, -1000, 32767];
    bufferTool_addGain(&mut g, 0.5, 3);
    assert_eq!(g, [500, -500, 16383]);
    let mut s = [100i16; 4];
    bufferTool_addGainInterpolated(&mut s, 1.0, 0.0, 4);
    assert_eq!(s, [0, 33, 66, 100]);
}
