//! VDrum extension: sequencer writes and host transport (`lxr_core::vdrum::seq`).

use lxr_core::vdrum::seq::VdrumStep;
use lxr_core::Lxr;

fn on_big_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new().stack_size(64 << 20).spawn(f).unwrap().join().unwrap();
}

fn energy(l: &mut Lxr, ticks: usize) -> f64 {
    let mut e = 0.0;
    for _ in 0..ticks {
        let b = l.tick();
        e += b.dac1.iter().chain(&b.dac2).map(|&s| (s as f64) * (s as f64)).sum::<f64>();
    }
    e
}

const ON: VdrumStep = VdrumStep { on: true, velocity: 127, prob: 100, note: 60, flam: 0, shift: 7, locks: [(255, 0); 2] };

#[test]
fn steps_play_on_host_pulses_only() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.set_hat_split(true);
        l.vdrum_seq_set_step(0, 0, 0, ON, true);
        l.vdrum_seq_set_step(0, 0, 4, ON, true);
        assert_eq!(l.seq_patternSet.seq_mainSteps[0][0][0], 0b1_0001);
        l.vdrum_seq_host_start(0);
        assert_eq!(l.seq_stepIndex[0], 15);
        assert_eq!(energy(&mut l, 50), 0.0, "nothing plays before the first pulse");
        l.vdrum_seq_host_pulse();
        assert!(energy(&mut l, 20) > 0.0);
        assert_eq!(l.seq_stepIndex[0], 0);
        // three pulses: steps 1..3 are off
        for _ in 0..3 {
            l.vdrum_seq_host_pulse();
            energy(&mut l, 5);
        }
        assert_eq!(l.seq_stepIndex[0], 3);
        l.vdrum_seq_stop();
        assert_eq!(l.seq_running, 0);
    });
}

#[test]
fn align_places_every_track_by_its_length() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.vdrum_seq_set_length(0, 1, 4, true);
        l.vdrum_seq_host_start(5);
        assert_eq!(l.seq_stepIndex[0], 4); // next pulse: step 5 of 16
        assert_eq!(l.seq_stepIndex[1], 0); // next pulse: step 1 of 4
        l.vdrum_seq_host_pulse();
        // pulse 5 is an odd 16th: the external clock path delays it by the shuffle time, which
        // ends one tick later at shuffle 0
        l.tick();
        assert_eq!(l.seq_stepIndex[0], 4);
        l.tick();
        assert_eq!(l.seq_stepIndex[0], 5);
        assert_eq!(l.seq_stepIndex[1], 1);
    });
}

#[test]
fn edits_of_the_playing_pattern_survive_a_pattern_change() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.vdrum_seq_set_step(0, 2, 7, ON, true);
        l.vdrum_seq_set_shuffle(0, 40, true);
        l.vdrum_seq_set_bits(3, 1, 0xf0f0);
        l.seq_setNextPattern(3);
        assert_eq!(l.seq_activePattern, 3);
        assert_eq!(l.seq_patternSet.seq_mainSteps[0][3][1], 0xf0f0);
        l.seq_setNextPattern(0);
        assert_eq!(l.seq_activePattern, 0);
        assert_eq!(l.seq_patternSet.seq_mainSteps[0][0][2], 1 << 7);
        assert_eq!(l.seq_patternSet.shuffle[0], 40);
        let s = l.seq_step_read(0, 2, 7);
        assert_eq!(s.volume, 0xff);
        l.vdrum_seq_set_prob(0, 2, 7, 30, true);
        assert_eq!(l.seq_step_read(0, 2, 7).prob, 30);
        assert_eq!(l.patternBackupBuffer.seq_subStepPattern[2][7].prob, 30);
    });
}
