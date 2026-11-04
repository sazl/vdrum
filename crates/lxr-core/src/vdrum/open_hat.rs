//! The open hi-hat as a separate voice ("hat split"). VDrum extension, not part of the firmware.
//!
//! The LXR-02 has one hi-hat voice (`hatVoice`) for tracks 6 and 7: both share every parameter except the
//! decay (`PAR_VELOD6_CLOSED`, `PAR_VELOD6_OPEN`), and an open hit cuts a closed one because it retriggers
//! the same voice. With [`VdrumState::hat_split`] on, track 7 plays a second `HiHatVoice`
//! ([`VdrumState::ohVoice`]) with its own parameters ([`OH_NUM_PARAMS`], indexed by the `OH_*` constants),
//! its own LFO, velocity modulator, mixer routing, sample-rate reduction and fader. Track 6 keeps the
//! firmware voice and its parameters; `PAR_VELOD6_OPEN` then has no effect. A closed hit silences the open
//! hat when [`OH_CHOKE`] is on.
//!
//! The voice runs the firmware's hi-hat DSP (`hihat_init_voice`, `hihat_trigger_voice`,
//! `hihat_calc_async`, `HiHat_calcSyncBlock`), and the parameter handling below mirrors the hi-hat cases of
//! `midiParser_ccHandler`, so with the same parameter values the open hat sounds as the firmware's open
//! hat does.
//!
//! Modulation: the LFO and velocity destinations of the open hat ([`OH_TARGET_LFO`], [`OH_VEL_DEST`]) are
//! indices into an extended target list: `0..229` are the firmware's `modTargets` (any parameter of the
//! other voices), `229..255` ([`OH_MOD_TARGETS`]) are parameters of the open hat. An open-hat parameter as
//! a `ModulationNode` destination is `OH_DEST_BASE + OH_*`; the firmware only produces destinations below
//! `NUM_PARAMS` (302), so these never occur on the firmware path. The six firmware LFOs and velocity
//! modulators keep their firmware ranges (targets `0..=228`) and therefore cannot reach the open hat.

use crate::dsp::distortion::setDistortionShape;
use crate::dsp::hi_hat::{hihat_calc_async, hihat_init_voice, hihat_trigger_voice};
use crate::dsp::modulation_node::modNode_calcNewValue;
use crate::dsp::oscillator::osc_recalcFreq;
use crate::dsp::resonant_filter::{SVF_directSetFilterValue, SVF_recalcFreq, SVF_setDrive, SVF_setReso};
use crate::dsp::slope_eg2::{slopeEg2_calcDecay, slopeEg2_setAttack, slopeEg2_setSlope};
use crate::dsp::transient_generator::transient_setWaveform;
use crate::menu::cc2_text::{getModTargetGapIndex, getModTargetIdxFromGapIdx, modTargets};
use crate::midi::midi_parser::{float_to_clamped_u8, FILTER_SHAPER, RECIP_127};
use crate::midi::value_shaper::valueShaperF2F;
use crate::prelude::*;

// ---- open hi-hat parameters (index into `VdrumState::ohParams`)
pub const OH_WAVE1: usize = 0;
pub const OH_COARSE: usize = 1;
pub const OH_FINE: usize = 2;
pub const OH_WAVE2: usize = 3;
pub const OH_WAVE3: usize = 4;
pub const OH_MOD_OSC_F1: usize = 5;
pub const OH_MOD_OSC_F2: usize = 6;
pub const OH_MOD_OSC_GAIN1: usize = 7;
pub const OH_MOD_OSC_GAIN2: usize = 8;
pub const OH_FILTER_FREQ: usize = 9;
pub const OH_RESO: usize = 10;
pub const OH_VELOA: usize = 11;
pub const OH_VELOD: usize = 12;
pub const OH_VOL_SLOPE: usize = 13;
pub const OH_VOL: usize = 14;
pub const OH_PAN: usize = 15;
pub const OH_DISTORTION: usize = 16;
pub const OH_DECIMATION: usize = 17;
pub const OH_FREQ_LFO: usize = 18;
pub const OH_AMOUNT_LFO: usize = 19;
pub const OH_FILTER_DRIVE: usize = 20;
pub const OH_VOLUME_MOD_ON_OFF: usize = 21;
pub const OH_VELO_MOD_AMT: usize = 22;
pub const OH_VEL_DEST: usize = 23;
pub const OH_WAVE_LFO: usize = 24;
pub const OH_VOICE_LFO: usize = 25;
pub const OH_TARGET_LFO: usize = 26;
pub const OH_RETRIGGER_LFO: usize = 27;
pub const OH_SYNC_LFO: usize = 28;
pub const OH_OFFSET_LFO: usize = 29;
pub const OH_FILTER_TYPE: usize = 30;
pub const OH_TRANS_VOL: usize = 31;
pub const OH_TRANS_WAVE: usize = 32;
pub const OH_TRANS_FREQ: usize = 33;
pub const OH_AUDIO_OUT: usize = 34;
pub const OH_PWM: usize = 35;
/// Closed hi-hat hits silence the open hi-hat (0/1). No firmware counterpart.
pub const OH_CHOKE: usize = 36;
/// Number of open hi-hat parameters.
pub const OH_NUM_PARAMS: usize = 37;

/// No firmware counterpart (see [`OH_TWIN`]).
pub const OH_NO_TWIN: u16 = u16::MAX;

/// The firmware hi-hat parameter each open hi-hat parameter corresponds to: same range, same meaning, and
/// the source of the value when a firmware kit (one shared hi-hat) is imported. The open hat's decay
/// corresponds to `PAR_VELOD6_OPEN`.
pub const OH_TWIN: [u16; OH_NUM_PARAMS] = [
    PAR_WAVE1_HH,
    PAR_COARSE6,
    PAR_FINE6,
    PAR_WAVE2_HH,
    PAR_WAVE3_HH,
    PAR_MOD_OSC_F1,
    PAR_MOD_OSC_F2,
    PAR_MOD_OSC_GAIN1,
    PAR_MOD_OSC_GAIN2,
    PAR_FILTER_FREQ_6,
    PAR_RESO_6,
    PAR_VELOA6,
    PAR_VELOD6_OPEN,
    PAR_VOL_SLOPE6,
    PAR_VOL6,
    PAR_PAN6,
    PAR_HAT_DISTORTION,
    PAR_VOICE_DECIMATION6,
    PAR_FREQ_LFO6,
    PAR_AMOUNT_LFO6,
    PAR_FILTER_DRIVE_6,
    PAR_VOLUME_MOD_ON_OFF6,
    PAR_VELO_MOD_AMT_6,
    PAR_VEL_DEST_6,
    PAR_WAVE_LFO6,
    PAR_VOICE_LFO6,
    PAR_TARGET_LFO6,
    PAR_RETRIGGER_LFO6,
    PAR_SYNC_LFO6,
    PAR_OFFSET_LFO6,
    PAR_FILTER_TYPE_6,
    PAR_TRANS6_VOL,
    PAR_TRANS6_WAVE,
    PAR_TRANS6_FREQ,
    PAR_AUDIO_OUT6,
    PAR_PWM_6,
    OH_NO_TWIN,
];

/// `ModulationNode::destination` of open hi-hat parameter `k`: `OH_DEST_BASE + k`.
pub const OH_DEST_BASE: u16 = 0x200;

/// First modulation target index of the open hi-hat (= the number of firmware `modTargets`).
pub const OH_TARGET_BASE: u8 = 229;

/// The open hi-hat modulation targets, target index `OH_TARGET_BASE + i`: (open hi-hat parameter, the
/// firmware hi-hat target index it corresponds to). Limited to 26 so that every target index fits the
/// `uint8_t` parameter value (229 + 26 = 255); not included: the velocity amount, velocity on/off and the
/// LFO wave / sync / retrigger / offset of the open hat.
pub const OH_MOD_TARGETS: [(u8, u8); 26] = [
    (OH_COARSE as u8, 191),
    (OH_FINE as u8, 192),
    (OH_WAVE1 as u8, 193),
    (OH_PWM as u8, 194),
    (OH_VELOA as u8, 195),
    (OH_VELOD as u8, 197),
    (OH_VOL_SLOPE as u8, 198),
    (OH_MOD_OSC_F1 as u8, 202),
    (OH_MOD_OSC_F2 as u8, 203),
    (OH_MOD_OSC_GAIN1 as u8, 204),
    (OH_MOD_OSC_GAIN2 as u8, 205),
    (OH_WAVE2 as u8, 206),
    (OH_WAVE3 as u8, 207),
    (OH_TRANS_WAVE as u8, 208),
    (OH_TRANS_VOL as u8, 209),
    (OH_TRANS_FREQ as u8, 210),
    (OH_FILTER_FREQ as u8, 211),
    (OH_RESO as u8, 212),
    (OH_FILTER_TYPE as u8, 213),
    (OH_FILTER_DRIVE as u8, 214),
    (OH_FREQ_LFO as u8, 215),
    (OH_AMOUNT_LFO as u8, 217),
    (OH_VOL as u8, 221),
    (OH_PAN as u8, 222),
    (OH_DECIMATION as u8, 223),
    (OH_DISTORTION as u8, 224),
];

/// Highest modulation target index of the open hi-hat's LFO and velocity destinations.
pub const OH_MAX_TARGET: u8 = OH_TARGET_BASE + OH_MOD_TARGETS.len() as u8 - 1;

/// The open hi-hat target that corresponds to a firmware hi-hat target (`191..=224`), if any. The closed
/// decay (196) maps to the open hat's decay as well.
pub fn oh_target_for_hat_target(t: u8) -> Option<u8> {
    let t = if t == 196 { 197 } else { t };
    OH_MOD_TARGETS.iter().position(|(_, ch)| *ch == t).map(|i| OH_TARGET_BASE + i as u8)
}

/// The firmware hi-hat target an open hi-hat target corresponds to.
pub fn hat_target_for_oh_target(t: u8) -> Option<u8> {
    let i = t.checked_sub(OH_TARGET_BASE)? as usize;
    OH_MOD_TARGETS.get(i).map(|(_, ch)| *ch)
}

/// Value range of open hi-hat parameter `k` (`(min, max)`, the twin's menu range; the destinations reach
/// the open hat's own targets, the LFO voice includes voice 7, the retrigger includes track 7).
pub fn oh_param_range(k: usize) -> (u8, u8) {
    match k {
        OH_WAVE1 | OH_WAVE2 | OH_WAVE3 => (0, 5),
        OH_VOLUME_MOD_ON_OFF | OH_CHOKE => (0, 1),
        OH_VEL_DEST | OH_TARGET_LFO => (0, OH_MAX_TARGET),
        OH_WAVE_LFO => (0, 7),
        OH_VOICE_LFO => (1, 7),
        OH_RETRIGGER_LFO => (0, 7),
        OH_SYNC_LFO => (0, 11),
        OH_FILTER_TYPE => (0, 7),
        OH_TRANS_WAVE => (0, 13),
        OH_AUDIO_OUT => (0, 6),
        _ => (0, 127),
    }
}

impl Lxr {
    /// Initialises the open hi-hat (called by `Lxr::new`; touches only `self.vdrum`): the voice as
    /// `HiHat_init` leaves the firmware hat, the fader at the top, the parameters copied from the
    /// firmware hi-hat ([`Lxr::oh_import_from_hat`]). Hat split stays off.
    pub fn vdrum_init(&mut self) {
        let x = &mut self.vdrum;
        hihat_init_voice(&mut x.ohVoice);
        // lfo_init without modNode_init's modNode_setDestination(0) (which would re-apply every firmware
        // modulation target): the node ends up the same (destination 0, NULL parameter pointer).
        let l = &mut x.ohVoice.lfo;
        l.phase = 0;
        l.phaseInc = 1;
        l.phaseOffset = 0;
        l.retrigger = 0;
        l.waveform = SINE;
        l.sync = 0;
        l.freq = 1.0;
        l.modNodeValue = 1.0;
        l.modTarget = ModulationNode::ZERO;
        x.ohVelocityModulator = ModulationNode::ZERO;
        x.ohDecimationRate = 1.0;
        self.oh_set_fader_adc(crate::engine::FADER_FULL_ADC);
        self.oh_import_from_hat();
    }

    /// Switches the hat split on or off. Off: track 7 plays the firmware hi-hat voice again.
    pub fn set_hat_split(&mut self, on: bool) {
        if self.vdrum.hat_split == on {
            return;
        }
        if !on {
            // drop the open hat's modulation of firmware parameters before it stops being reset each block
            self.vdrum.ohVoice.lfo.modTarget.destination = 0;
            self.vdrum.ohVelocityModulator.destination = 0;
            self.modNode_resetTargets();
        }
        self.vdrum.hat_split = on;
        if on {
            // The hi-hat oscillators, filter, fader smoothing and sample-rate reduction run continuously,
            // also while the voice is silent. Start the open hat from the firmware hi-hat's running state
            // (then apply the open hat's own parameters), so that an open hit right after switching sounds
            // as it would have on the shared voice. A hat that is still sounding is not duplicated.
            let mut v = self.hatVoice;
            v.lfo.modTarget = ModulationNode::ZERO;
            if v.oscVolEg.value > 0.0 {
                v.oscVolEg.value = 0.0;
                v.oscVolEg.state = EG_D as u8;
            }
            self.vdrum.ohVoice = v;
            self.vdrum.ohDecimationCnt = self.mixer_decimation_cnt[5];
            self.vdrum.ohVoiceSample = self.mixer_voice_samples[5];
            self.oh_apply_all();
        }
    }

    /// The current value of open hi-hat parameter `k`.
    pub fn oh_get_parameter(&self, k: usize) -> u8 {
        self.vdrum.ohParams.get(k).copied().unwrap_or(0)
    }

    /// Sets open hi-hat parameter `k` (clamped to [`oh_param_range`]); the menu path of the open hat.
    pub fn oh_set_parameter(&mut self, k: usize, value: u8) {
        if k >= OH_NUM_PARAMS {
            return;
        }
        let (lo, hi) = oh_param_range(k);
        let value = value.clamp(lo, hi);
        match k {
            OH_VOICE_LFO => {
                // as the menu: keep the destination's parameter, move it to the selected voice
                self.vdrum.ohParams[k] = value;
                let cur = self.vdrum.ohParams[OH_TARGET_LFO];
                let t = oh_target_for_voice(cur, value);
                self.vdrum.ohParams[OH_TARGET_LFO] = t;
                self.oh_apply_destinations();
            }
            OH_TARGET_LFO | OH_VEL_DEST => {
                self.vdrum.ohParams[k] = value;
                self.oh_apply_destinations();
            }
            _ => {
                self.vdrum.ohParams[k] = value;
                self.oh_apply(k, value, value as i32 as f32 * RECIP_127, false);
            }
        }
    }

    /// Sets all open hi-hat parameters (a kit's open-hat block), then applies them.
    pub fn oh_set_parameters(&mut self, values: &[u8; OH_NUM_PARAMS]) {
        for k in 0..OH_NUM_PARAMS {
            let (lo, hi) = oh_param_range(k);
            self.vdrum.ohParams[k] = values[k].clamp(lo, hi);
        }
        self.oh_apply_all();
    }

    /// Legacy import: the open hi-hat takes the firmware hi-hat's current parameters (its decay from
    /// `PAR_VELOD6_OPEN`; destinations on the hi-hat move to the same parameter of the open hat), choke on.
    /// With these values an open hit sounds as the firmware's open hit.
    pub fn oh_import_from_hat(&mut self) {
        let mut v = [0u8; OH_NUM_PARAMS];
        for k in 0..OH_NUM_PARAMS {
            v[k] = match OH_TWIN[k] {
                OH_NO_TWIN => 1,
                p => self.activeParameters[p as usize],
            };
        }
        for k in [OH_TARGET_LFO, OH_VEL_DEST] {
            if let Some(t) = oh_target_for_hat_target(v[k]) {
                v[k] = t;
            }
        }
        if v[OH_TARGET_LFO] >= OH_TARGET_BASE {
            v[OH_VOICE_LFO] = 7;
        }
        self.oh_set_parameters(&v);
    }

    /// Applies every open hi-hat parameter to the voice.
    pub fn oh_apply_all(&mut self) {
        for k in 0..OH_NUM_PARAMS {
            let v = self.vdrum.ohParams[k];
            self.oh_apply(k, v, v as i32 as f32 * RECIP_127, false);
        }
        self.oh_apply_destinations();
    }

    /// Sets the destinations of the open hat's LFO and velocity modulator from their parameters.
    fn oh_apply_destinations(&mut self) {
        if !self.vdrum.hat_split {
            // not active: only remember the values (no firmware parameter may be modulated)
            return;
        }
        let t = self.vdrum.ohParams[OH_TARGET_LFO];
        self.modNode_setDestination(|s| &mut s.vdrum.ohVoice.lfo.modTarget, oh_target_dest(t));
        let t = self.vdrum.ohParams[OH_VEL_DEST];
        self.modNode_setDestination(|s| &mut s.vdrum.ohVelocityModulator, oh_target_dest(t));
    }

    /// The hi-hat cases of `midiParser_ccHandler` applied to the open hi-hat. `itg`/`flt`/`isf` as there
    /// (`isf`: a modulation value; `itg = (uint8_t)(v*127)`).
    pub fn oh_apply(&mut self, k: usize, itg: u8, flt: f32, isf: bool) {
        let nosc = NUM_OSC_WAVEFORMS as f32;
        let mosc = NUM_OSC_WAVEFORMS - 1;
        let x = &mut self.vdrum;
        let v = &mut x.ohVoice;
        macro_rules! midi_freq {
            ($osc:expr, coarse) => {{
                $osc.midiFreq &= 0x00ff;
                $osc.midiFreq |= (itg as u16) << 8;
                osc_recalcFreq(&mut $osc);
            }};
            ($osc:expr, fine) => {{
                $osc.midiFreq &= 0xff00;
                $osc.midiFreq |= itg as u16;
                osc_recalcFreq(&mut $osc);
            }};
        }
        let wave = || if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
        match k {
            OH_WAVE1 => v.osc.waveform = wave(),
            OH_WAVE2 => v.modOsc.waveform = wave(),
            OH_WAVE3 => v.modOsc2.waveform = wave(),
            OH_COARSE => midi_freq!(v.osc, coarse),
            OH_FINE => midi_freq!(v.osc, fine),
            OH_MOD_OSC_F1 => midi_freq!(v.modOsc, coarse),
            OH_MOD_OSC_F2 => midi_freq!(v.modOsc2, coarse),
            OH_MOD_OSC_GAIN1 => v.fmModAmount1 = flt,
            OH_MOD_OSC_GAIN2 => v.fmModAmount2 = flt,
            OH_FILTER_FREQ => SVF_directSetFilterValue(&mut v.filter, valueShaperF2F(flt, FILTER_SHAPER)),
            OH_RESO => SVF_setReso(&mut v.filter, flt),
            OH_VELOA => slopeEg2_setAttack(&mut v.oscVolEg, itg, 0),
            OH_VELOD => {
                // the open hat is always triggered as "open"
                let d = slopeEg2_calcDecay(itg) as f32;
                v.decayOpen = d;
                v.decayClosed = d;
            }
            OH_VOL_SLOPE => slopeEg2_setSlope(&mut v.oscVolEg, itg),
            OH_VOL => v.vol = flt,
            OH_PAN => v.pan = itg,
            OH_DISTORTION => setDistortionShape(&mut v.distortion, flt),
            OH_DECIMATION => x.ohDecimationRate = valueShaperF2F(flt, -0.7),
            OH_FREQ_LFO => {
                let mut lfo = self.vdrum.ohVoice.lfo;
                self.lfo_setFreq(&mut lfo, flt);
                self.vdrum.ohVoice.lfo = lfo;
            }
            OH_AMOUNT_LFO => v.lfo.modTarget.amount = flt,
            OH_FILTER_DRIVE => SVF_setDrive(&mut v.filter, flt),
            OH_VOLUME_MOD_ON_OFF => v.volumeMod = (itg != 0) as u8,
            OH_VELO_MOD_AMT => x.ohVelocityModulator.amount = flt,
            OH_WAVE_LFO => {
                v.lfo.waveform =
                    if isf { float_to_clamped_u8(NUM_LFO_WAVEFORMS as f32, flt, NUM_LFO_WAVEFORMS - 1) } else { itg }
            }
            OH_RETRIGGER_LFO => v.lfo.retrigger = if isf { ((7.0 * flt) as u32) as u8 } else { itg },
            OH_SYNC_LFO => {
                let mut lfo = self.vdrum.ohVoice.lfo;
                self.lfo_setSync(&mut lfo, itg);
                self.vdrum.ohVoice.lfo = lfo;
            }
            OH_OFFSET_LFO => v.lfo.phaseOffset = (flt * 4294967296.0) as u32,
            OH_FILTER_TYPE => {
                v.filterType = if isf {
                    let t = (vfma(1.0, flt, FILTER_NAIVE_2_POLE as f32) as u32) as u8;
                    if t >= FILTER_OFF {
                        FILTER_OFF - 1
                    } else {
                        t
                    }
                } else {
                    itg.wrapping_add(1)
                }
            }
            OH_TRANS_VOL => v.transGen.volume = flt,
            OH_TRANS_WAVE => {
                let w = if isf { float_to_clamped_u8(NUM_TRANSIENTS as f32, flt, (NUM_TRANSIENTS - 1) as u8) } else { itg };
                transient_setWaveform(&mut v.transGen, w);
            }
            OH_TRANS_FREQ => v.transGen.pitch = vfma(0.25, itg as i32 as f32, f32::from_bits(0x3cf1a6e3)),
            OH_AUDIO_OUT => x.ohAudioRouting = itg,
            OH_PWM => v.pwm = flt,
            // OH_VEL_DEST, OH_VOICE_LFO, OH_TARGET_LFO: destinations (oh_apply_destinations); OH_CHOKE: read
            // when the closed hat is triggered
            _ => {}
        }
    }

    /// `modNode_updateValue` for an open hi-hat destination (`dest >= OH_DEST_BASE`).
    pub(crate) fn oh_mod_update(&mut self, dest: u16, amount: f32, val: f32) {
        let k = (dest - OH_DEST_BASE) as usize;
        if k >= OH_NUM_PARAMS {
            return;
        }
        let v = modNode_calcNewValue(amount, val, self.vdrum.ohParams[k]);
        self.oh_apply(k, ((v * 127.0) as u32) as u8, v, true);
    }

    /// `paramArray_setParameter` for an open hi-hat destination: restores the unmodulated value.
    pub(crate) fn oh_reset_param(&mut self, dest: u16) {
        let k = (dest - OH_DEST_BASE) as usize;
        if k >= OH_NUM_PARAMS {
            return;
        }
        let v = self.vdrum.ohParams[k];
        self.oh_apply(k, v, v as i32 as f32 * RECIP_127, false);
    }

    /// Track 7 hit with the hat split on (in place of `HiHat_trigger(vel, 1, note)`).
    pub fn oh_trigger(&mut self, vel: u8, note: u8) {
        self.lfo_retrigger(6);
        let velo = vel as i32 as f32 * RECIP_127;
        self.modNode_updateValue(|s| &mut s.vdrum.ohVelocityModulator, velo);
        hihat_trigger_voice(&mut self.vdrum.ohVoice, velo, 1, note);
    }

    /// A closed hi-hat hit with the hat split on: silences the open hat if [`OH_CHOKE`] is on.
    pub fn oh_choke_by_closed(&mut self) {
        if self.vdrum.ohParams[OH_CHOKE] != 0 {
            let eg = &mut self.vdrum.ohVoice.oscVolEg;
            eg.value = 0.0;
            eg.state = EG_D as u8;
            eg.repeatCnt = 0;
        }
    }

    /// The open hat's part of `mixer_calcNextSampleBlock` before the voices render (LFO, filter, async).
    pub(crate) fn oh_mixer_prepare(&mut self) {
        self.lfo_dispatchNextValue(|s| &mut s.vdrum.ohVoice.lfo);
        SVF_recalcFreq(&mut self.vdrum.ohVoice.filter);
        hihat_calc_async(&mut self.vdrum.ohVoice);
    }

    /// Renders the open hat and adds it to the outputs, as `mixer_calcNextSampleBlock` does for the hi-hat.
    pub(crate) fn oh_mixer_render(&mut self, sampleData: &mut [i16], output: &mut [i16], output2: &mut [i16], fxBusData: &mut [i16]) {
        // HiHat_calcSyncBlock works on `hatVoice` and fader 5: run it on the open hat's voice and fader
        core::mem::swap(&mut self.hatVoice, &mut self.vdrum.ohVoice);
        core::mem::swap(&mut self.adc_faderValues[5], &mut self.vdrum.ohFaderValue);
        self.HiHat_calcSyncBlock(sampleData, OUTPUT_DMA_SIZE as u8);
        core::mem::swap(&mut self.hatVoice, &mut self.vdrum.ohVoice);
        core::mem::swap(&mut self.adc_faderValues[5], &mut self.vdrum.ohFaderValue);

        // mixer_decimateBlock with the open hat's state
        let phaseInc = self.vdrum.ohDecimationRate * self.mixer_decimation_rate[6];
        for s in sampleData.iter_mut().take(OUTPUT_DMA_SIZE as usize) {
            self.vdrum.ohDecimationCnt += phaseInc;
            if self.vdrum.ohDecimationCnt >= 1.0 {
                self.vdrum.ohDecimationCnt -= 1.0;
                self.vdrum.ohVoiceSample = *s;
            }
            *s = self.vdrum.ohVoiceSample;
        }
        let (panL, panR) = (self.vdrum.ohVoice.triggeredPanL, self.vdrum.ohVoice.triggeredPanR);
        let route = self.vdrum.ohAudioRouting;
        self.mixer_addDataToOutput(route, panL, panR, sampleData, output, output2, fxBusData);
    }

    /// The open hat fader (`set_fader_adc(6, adc)`): same hysteresis and curve as the firmware faders.
    pub fn oh_set_fader_adc(&mut self, adc: u16) {
        const HYSTERSIS: i32 = 40;
        let (new, old) = ((adc >> 4) as i32, (self.vdrum.ohFaderAdcOld >> 4) as i32);
        if new > old + HYSTERSIS || old > new + HYSTERSIS || self.vdrum.ohFaderAdcOld == 0 {
            let mut v = new as f32 / 4096.0;
            self.vdrum.ohFaderAdcOld = adc;
            if v < 0.01 {
                v = 0.0;
            }
            self.vdrum.ohFaderValue = v * v;
        }
    }
}

/// `ModulationNode::destination` for an extended target index (`0..229` firmware, `229..` open hat).
pub fn oh_target_dest(t: u8) -> u16 {
    if t >= OH_TARGET_BASE {
        match OH_MOD_TARGETS.get((t - OH_TARGET_BASE) as usize) {
            Some((k, _)) => OH_DEST_BASE + *k as u16,
            None => 0,
        }
    } else {
        // what the front panel sends for a firmware target (`modTargets[t].param`, 8 bits)
        modTargets.get(t as usize).map_or(0, |m| m.param as u8 as u16)
    }
}

/// The destination after the LFO voice changed to `voice` (1..=7): the same parameter on that voice, as the
/// menu does for the firmware LFOs (`getModTargetGapIndex` / `getModTargetIdxFromGapIdx`), 0 if the voice
/// has no such parameter.
pub fn oh_target_for_voice(cur: u8, voice: u8) -> u8 {
    // express the current target as a firmware target
    let fw = if cur >= OH_TARGET_BASE { hat_target_for_oh_target(cur).unwrap_or(0) } else { cur };
    let gap = getModTargetGapIndex(fw);
    if voice >= 7 {
        let hat = getModTargetIdxFromGapIdx(5, gap);
        oh_target_for_hat_target(hat).unwrap_or(0)
    } else {
        getModTargetIdxFromGapIdx(voice.saturating_sub(1), gap)
    }
}
