# Control tables

> Superseded (2026-11-06): the current tables, generated from the surface spec, are in
> [generated/control-tables.md](generated/control-tables.md), with one image per mode. This file was
> generated from the former source/layout.py and is kept for its notes.

The same mappings as the images, as text. \[cycle\] steps through a list, \[on/off\] latches, \[hold\] acts while held, \[press\] acts once. Identifiers are lxr-params IDs (host parameter IDs) or plugin IDs; (new) marks an ID that was new in the design (open hi-hat voice 7, voice fader 7; added in phase 1). DAW-mode CCs: knob rows 13-20, 21-28, 29-36; faders 5-12; top buttons 37-44; bottom buttons 45-52. Left-hand buttons: [design.md](design.md), "Controls that are the same in every mode" (mode 8 differs, see below). Generated from source/layout.py.

## Mode 1: Drum 1

| Row | 1 OSC | 2 AMP | 3 PITCH EG | 4 FM | 5 TRANSIENT | 6 FILTER | 7 VELOCITY | 8 MIX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Fine `fine1` | Attack `veloa1` | Pitch EG decay `mod_eg1` | FM freq `fm_freq1` | Trans freq `trans1_freq` | Resonance `reso_1` | Vel dest `vel_dest_1` | Pan `pan1` |
| Knob row 2 | Pulse width `pwm_1` | Amp slope `vol_slope1` | Pitch EG slope `pitch_slope1` | FM wave `mod_wave_drum1` | (spare) | Filter drive `filter_drive_1` | Vel amount `velo_mod_amt_1` | Overdrive `drive1` |
| Knob row 3 | LFO Rate `freq_lfo1` | LFO Amount `amount_lfo1` | LFO Offset `offset_lfo1` | LFO Wave `wave_lfo1` | LFO Sync `sync_lfo1` | LFO Retrigger `retrigger_lfo1` | LFO Dest voice `voice_lfo1` | LFO Dest param `target_lfo1` |
| Fader | Coarse `coarse1` | Decay `velod1` | Pitch EG amt `modamnt1` | FM amount `fmamnt1` | Trans volume `trans1_vol` | Cutoff `filter_freq_1` | Sample rate `voice_decimation1` | Volume `vol1` |
| Top button | Osc wave \[cycle\] `osc_wave_drum1` | Copy voice \[press\] | Paste voice \[hold\]: 1 s | FM / Mix \[on/off\] `mix_mod_1` | Trans wave \[cycle\] `trans1_wave` | Filter type \[cycle\] `filter_type_1` | Vel > Vol \[on/off\] `volume_mod_on_off1` | Audio out \[cycle\] `audio_out1` |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |

## Mode 2: Drum 2

| Row | 1 OSC | 2 AMP | 3 PITCH EG | 4 FM | 5 TRANSIENT | 6 FILTER | 7 VELOCITY | 8 MIX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Fine `fine2` | Attack `veloa2` | Pitch EG decay `mod_eg2` | FM freq `fm_freq2` | Trans freq `trans2_freq` | Resonance `reso_2` | Vel dest `vel_dest_2` | Pan `pan2` |
| Knob row 2 | Pulse width `pwm_2` | Amp slope `vol_slope2` | Pitch EG slope `pitch_slope2` | FM wave `mod_wave_drum2` | (spare) | Filter drive `filter_drive_2` | Vel amount `velo_mod_amt_2` | Overdrive `drive2` |
| Knob row 3 | LFO Rate `freq_lfo2` | LFO Amount `amount_lfo2` | LFO Offset `offset_lfo2` | LFO Wave `wave_lfo2` | LFO Sync `sync_lfo2` | LFO Retrigger `retrigger_lfo2` | LFO Dest voice `voice_lfo2` | LFO Dest param `target_lfo2` |
| Fader | Coarse `coarse2` | Decay `velod2` | Pitch EG amt `modamnt2` | FM amount `fmamnt2` | Trans volume `trans2_vol` | Cutoff `filter_freq_2` | Sample rate `voice_decimation2` | Volume `vol2` |
| Top button | Osc wave \[cycle\] `osc_wave_drum2` | Copy voice \[press\] | Paste voice \[hold\]: 1 s | FM / Mix \[on/off\] `mix_mod_2` | Trans wave \[cycle\] `trans2_wave` | Filter type \[cycle\] `filter_type_2` | Vel > Vol \[on/off\] `volume_mod_on_off2` | Audio out \[cycle\] `audio_out2` |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |

## Mode 3: Drum 3

| Row | 1 OSC | 2 AMP | 3 PITCH EG | 4 FM | 5 TRANSIENT | 6 FILTER | 7 VELOCITY | 8 MIX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Fine `fine3` | Attack `veloa3` | Pitch EG decay `mod_eg3` | FM freq `fm_freq3` | Trans freq `trans3_freq` | Resonance `reso_3` | Vel dest `vel_dest_3` | Pan `pan3` |
| Knob row 2 | Pulse width `pwm_3` | Amp slope `vol_slope3` | Pitch EG slope `pitch_slope3` | FM wave `mod_wave_drum3` | (spare) | Filter drive `filter_drive_3` | Vel amount `velo_mod_amt_3` | Overdrive `drive3` |
| Knob row 3 | LFO Rate `freq_lfo3` | LFO Amount `amount_lfo3` | LFO Offset `offset_lfo3` | LFO Wave `wave_lfo3` | LFO Sync `sync_lfo3` | LFO Retrigger `retrigger_lfo3` | LFO Dest voice `voice_lfo3` | LFO Dest param `target_lfo3` |
| Fader | Coarse `coarse3` | Decay `velod3` | Pitch EG amt `modamnt3` | FM amount `fmamnt3` | Trans volume `trans3_vol` | Cutoff `filter_freq_3` | Sample rate `voice_decimation3` | Volume `vol3` |
| Top button | Osc wave \[cycle\] `osc_wave_drum3` | Copy voice \[press\] | Paste voice \[hold\]: 1 s | FM / Mix \[on/off\] `mix_mod_3` | Trans wave \[cycle\] `trans3_wave` | Filter type \[cycle\] `filter_type_3` | Vel > Vol \[on/off\] `volume_mod_on_off3` | Audio out \[cycle\] `audio_out3` |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |

## Mode 4: Snare

| Row | 1 OSC | 2 AMP | 3 PITCH EG | 4 NOISE | 5 TRANSIENT | 6 FILTER | 7 VELOCITY | 8 MIX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Fine `fine4` | Attack `veloa4` | Pitch EG decay `mod_eg4` | Noise freq `noise_freq1` | Trans freq `trans4_freq` | Resonance `reso_4` | Vel dest `vel_dest_4` | Pan `pan4` |
| Knob row 2 | Pulse width `pwm_4` | Amp slope `vol_slope4` | Pitch EG slope `pitch_slope4` | (spare) | Amp repeat `repeat4` | Filter drive `filter_drive_4` | Vel amount `velo_mod_amt_4` | Overdrive `snare_distortion` |
| Knob row 3 | LFO Rate `freq_lfo4` | LFO Amount `amount_lfo4` | LFO Offset `offset_lfo4` | LFO Wave `wave_lfo4` | LFO Sync `sync_lfo4` | LFO Retrigger `retrigger_lfo4` | LFO Dest voice `voice_lfo4` | LFO Dest param `target_lfo4` |
| Fader | Coarse `coarse4` | Decay `velod4` | Pitch EG amt `modamnt4` | Noise/Osc mix `mix1` | Trans volume `trans4_vol` | Cutoff `filter_freq_4` | Sample rate `voice_decimation4` | Volume `vol4` |
| Top button | Osc wave \[cycle\] `osc_wave_snare` | Copy voice \[press\] | Paste voice \[hold\]: 1 s | (spare) | Trans wave \[cycle\] `trans4_wave` | Filter type \[cycle\] `filter_type_4` | Vel > Vol \[on/off\] `volume_mod_on_off4` | Audio out \[cycle\] `audio_out4` |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |

## Mode 5: Cymbal

| Row | 1 OSC | 2 AMP | 3 METAL 1 | 4 METAL 2 | 5 TRANSIENT | 6 FILTER | 7 VELOCITY | 8 MIX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Fine `fine5` | Attack `veloa5` | Metal freq 1 `mod_osc_f1_cym` | Metal freq 2 `mod_osc_f2_cym` | Trans freq `trans5_freq` | Resonance `reso_5` | Vel dest `vel_dest_5` | Pan `pan5` |
| Knob row 2 | Pulse width `pwm_5` | Amp slope `vol_slope5` | Metal wave 1 `wave2_cym` | Metal wave 2 `wave3_cym` | Amp repeat `repeat5` | Filter drive `filter_drive_5` | Vel amount `velo_mod_amt_5` | Overdrive `cymbal_distortion` |
| Knob row 3 | LFO Rate `freq_lfo5` | LFO Amount `amount_lfo5` | LFO Offset `offset_lfo5` | LFO Wave `wave_lfo5` | LFO Sync `sync_lfo5` | LFO Retrigger `retrigger_lfo5` | LFO Dest voice `voice_lfo5` | LFO Dest param `target_lfo5` |
| Fader | Coarse `coarse5` | Decay `velod5` | Metal gain 1 `mod_osc_gain1_cym` | Metal gain 2 `mod_osc_gain2_cym` | Trans volume `trans5_vol` | Cutoff `filter_freq_5` | Sample rate `voice_decimation5` | Volume `vol5` |
| Top button | Osc wave \[cycle\] `wave1_cym` | Copy voice \[press\] | Paste voice \[hold\]: 1 s | (spare) | Trans wave \[cycle\] `trans5_wave` | Filter type \[cycle\] `filter_type_5` | Vel > Vol \[on/off\] `volume_mod_on_off5` | Audio out \[cycle\] `audio_out5` |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |

## Mode 6: Closed hi-hat

| Row | 1 OSC | 2 AMP | 3 METAL 1 | 4 METAL 2 | 5 TRANSIENT | 6 FILTER | 7 VELOCITY | 8 MIX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Fine `fine6` | Attack `veloa6` | Metal freq 1 `mod_osc_f1` | Metal freq 2 `mod_osc_f2` | Trans freq `trans6_freq` | Resonance `reso_6` | Vel dest `vel_dest_6` | Pan `pan6` |
| Knob row 2 | Pulse width `pwm_6` | Amp slope `vol_slope6` | Metal wave 1 `wave2_hh` | Metal wave 2 `wave3_hh` | (spare) | Filter drive `filter_drive_6` | Vel amount `velo_mod_amt_6` | Overdrive `hat_distortion` |
| Knob row 3 | LFO Rate `freq_lfo6` | LFO Amount `amount_lfo6` | LFO Offset `offset_lfo6` | LFO Wave `wave_lfo6` | LFO Sync `sync_lfo6` | LFO Retrigger `retrigger_lfo6` | LFO Dest voice `voice_lfo6` | LFO Dest param `target_lfo6` |
| Fader | Coarse `coarse6` | Decay `velod6_closed` | Metal gain 1 `mod_osc_gain1` | Metal gain 2 `mod_osc_gain2` | Trans volume `trans6_vol` | Cutoff `filter_freq_6` | Sample rate `voice_decimation6` | Volume `vol6` |
| Top button | Osc wave \[cycle\] `wave1_hh` | Copy voice \[press\] | Paste voice \[hold\]: 1 s | Choke OH \[on/off\]: chokes open hat | Trans wave \[cycle\] `trans6_wave` | Filter type \[cycle\] `filter_type_6` | Vel > Vol \[on/off\] `volume_mod_on_off6` | Audio out \[cycle\] `audio_out6` |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |

## Mode 7: Open hi-hat

| Row | 1 OSC | 2 AMP | 3 METAL 1 | 4 METAL 2 | 5 TRANSIENT | 6 FILTER | 7 VELOCITY | 8 MIX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Fine `fine7` (new) | Attack `veloa7` (new) | Metal freq 1 `mod_osc_f1_oh` (new) | Metal freq 2 `mod_osc_f2_oh` (new) | Trans freq `trans7_freq` (new) | Resonance `reso_7` (new) | Vel dest `vel_dest_7` (new) | Pan `pan7` (new) |
| Knob row 2 | Pulse width `pwm_7` (new) | Amp slope `vol_slope7` (new) | Metal wave 1 `wave2_oh` (new) | Metal wave 2 `wave3_oh` (new) | (spare) | Filter drive `filter_drive_7` (new) | Vel amount `velo_mod_amt_7` (new) | Overdrive `oh_distortion` (new) |
| Knob row 3 | LFO Rate `freq_lfo7` (new) | LFO Amount `amount_lfo7` (new) | LFO Offset `offset_lfo7` (new) | LFO Wave `wave_lfo7` (new) | LFO Sync `sync_lfo7` (new) | LFO Retrigger `retrigger_lfo7` (new) | LFO Dest voice `voice_lfo7` (new) | LFO Dest param `target_lfo7` (new) |
| Fader | Coarse `coarse7` (new) | Decay `velod7` (new) | Metal gain 1 `mod_osc_gain1_oh` (new) | Metal gain 2 `mod_osc_gain2_oh` (new) | Trans volume `trans7_vol` (new) | Cutoff `filter_freq_7` (new) | Sample rate `voice_decimation7` (new) | Volume `vol7` (new) |
| Top button | Osc wave \[cycle\] `wave1_oh` (new) | Copy voice \[press\] | Paste voice \[hold\]: 1 s | Choke by CH \[on/off\]: same switch | Trans wave \[cycle\] `trans7_wave` (new) | Filter type \[cycle\] `filter_type_7` (new) | Vel > Vol \[on/off\] `volume_mod_on_off7` (new) | Audio out \[cycle\] `audio_out7` (new) |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |

## Mode 8: Sequencer

Side buttons: Track left/right select the sequencer track 1 to 7 (the step buttons show its steps and its column's knob LEDs pulse); hold Mute/Select for patterns, Page up/down for bars 1 to 4, hold Record + a step or Track left/right to clear. The same step buttons, track and bar selection are available in every mode through the Solo/Arm sequencer view (see design.md); there, holding a step and moving a knob or fader that controls a parameter locks it. Mode 11: the top-row LEDs follow each track's LFO.

| Row | 1 DRUM 1 | 2 DRUM 2 | 3 DRUM 3 | 4 SNARE | 5 CYMBAL | 6 CL. HAT | 7 OP. HAT | 8 PATTERN |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | D1 Fills: Euclid pulses | D2 Fills: Euclid pulses | D3 Fills: Euclid pulses | SD Fills: Euclid pulses | CY Fills: Euclid pulses | CH Fills: Euclid pulses | OH Fills: Euclid pulses | Shuffle: seq\_setShuffle |
| Knob row 2 | D1 Rotate: Euclid offset | D2 Rotate: Euclid offset | D3 Rotate: Euclid offset | SD Rotate: Euclid offset | CY Rotate: Euclid offset | CH Rotate: Euclid offset | OH Rotate: Euclid offset | Next pattern: queued to bar end |
| Knob row 3 | D1 Length: 1-64 steps | D2 Length: 1-64 steps | D3 Length: 1-64 steps | SD Length: 1-64 steps | CY Length: 1-64 steps | CH Length: 1-64 steps | OH Length: 1-64 steps | Entry velocity: new steps |
| Fader | D1 Density: play probability | D2 Density: play probability | D3 Density: play probability | SD Density: play probability | CY Density: play probability | CH Density: play probability | OH Density: play probability | Density all: scales all tracks |
| Top button | Step 1 \[on/off\]: selected track | Step 2 \[on/off\]: selected track | Step 3 \[on/off\]: selected track | Step 4 \[on/off\]: selected track | Step 5 \[on/off\]: selected track | Step 6 \[on/off\]: selected track | Step 7 \[on/off\]: selected track | Step 8 \[on/off\]: selected track |
| Bottom button | Step 9 \[on/off\]: selected track | Step 10 \[on/off\]: selected track | Step 11 \[on/off\]: selected track | Step 12 \[on/off\]: selected track | Step 13 \[on/off\]: selected track | Step 14 \[on/off\]: selected track | Step 15 \[on/off\]: selected track | Step 16 \[on/off\]: selected track |

## Mode 8: Sequencer, step held

| Row | 1 DRUM 1 | 2 DRUM 2 | 3 DRUM 3 | 4 SNARE | 5 CYMBAL | 6 CL. HAT | 7 OP. HAT | 8 PATTERN |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Velocity: step volume | Probability: step prob | Note: step note | Flam / ratchet: stepTiming flam | Micro-shift: stepTiming shift | Lock 1 value: param1Val | Lock 2 value: param2Val | Clear locks: turn left |
| Knob row 2 | Lock Pitch: coarse | Lock Decay: amp decay | Lock Cutoff: filter freq | Lock Reso: resonance | Lock Drive: overdrive | Lock Pan: pan | Lock S.rate: sample rate | Lock FX send: audio out |
| Knob row 3 | D1 Length: 1-64 steps | D2 Length: 1-64 steps | D3 Length: 1-64 steps | SD Length: 1-64 steps | CY Length: 1-64 steps | CH Length: 1-64 steps | OH Length: 1-64 steps | Entry velocity: new steps |
| Fader | D1 Density: play probability | D2 Density: play probability | D3 Density: play probability | SD Density: play probability | CY Density: play probability | CH Density: play probability | OH Density: play probability | Density all: scales all tracks |
| Top button | Step 1 \[hold\]: select | Step 2 \[hold\]: select | Step 3 \[hold\]: select | Step 4 \[hold\]: select | Step 5 \[hold\]: select | Step 6 \[hold\]: select | Step 7 \[hold\]: select | Step 8 \[hold\]: select |
| Bottom button | Step 9 \[hold\]: select | Step 10 \[hold\]: select | Step 11 \[hold\]: select | Step 12 \[hold\]: select | Step 13 \[hold\]: select | Step 14 \[hold\]: select | Step 15 \[hold\]: select | Step 16 \[hold\]: select |

## Mode 9: Mix + FX

| Row | 1 DRUM 1 | 2 DRUM 2 | 3 DRUM 3 | 4 SNARE | 5 CYMBAL | 6 CL. HAT | 7 OP. HAT | 8 FX / EFFECTS |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | FX P1 `fx_p1` | FX P2 `fx_p2` | FX P3 `fx_p3` | FX P4 `fx_p4` | FX type `fx_type` | FX sub-type: per FX type | FX output `fx_routing` | Global S.rate `voice_decimation_all` |
| Knob row 2 | D1 Pan `pan1` | D2 Pan `pan2` | D3 Pan `pan3` | SD Pan `pan4` | CY Pan `pan5` | CH Pan `pan6` | OH Pan `pan7` (new) | Effect depth: momentary row |
| Knob row 3 | D1 Drive `drive1` | D2 Drive `drive2` | D3 Drive `drive3` | SD Drive `snare_distortion` | CY Drive `cymbal_distortion` | CH Drive `hat_distortion` | OH Drive `oh_distortion` (new) | Effect rate: stutter / sweep |
| Fader | D1 Volume `vol1` | D2 Volume `vol2` | D3 Volume `vol3` | SD Volume `vol4` | CY Volume `vol5` | CH Volume `vol6` | OH Volume `vol7` (new) | FX dry/wet `fx_dry_wet` |
| Top button | D1 > FX \[on/off\]: audio\_out = FX | D2 > FX \[on/off\]: audio\_out = FX | D3 > FX \[on/off\]: audio\_out = FX | SD > FX \[on/off\]: audio\_out = FX | CY > FX \[on/off\]: audio\_out = FX | CH > FX \[on/off\]: audio\_out = FX | OH > FX \[on/off\]: audio\_out = FX | FX on/off \[on/off\]: fx\_type <-> off |
| Bottom button | Stutter 1/16 \[hold\]: repeat last hits | Stutter 1/32 \[hold\]: repeat last hits | Filter sweep \[hold\]: cutoff down | Reso up \[hold\]: resonance max | Swell \[hold\]: attack up | Long decay \[hold\]: decay up | Lo-fi \[hold\]: sample rate down | FX throw \[hold\]: all to FX, wet |

## Mode 10: Perform

| Row | 1 DRUM 1 | 2 DRUM 2 | 3 DRUM 3 | 4 SNARE | 5 CYMBAL | 6 CL. HAT | 7 OP. HAT | 8 FX |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | D1 Pitch `coarse1` | D2 Pitch `coarse2` | D3 Pitch `coarse3` | SD Pitch `coarse4` | CY Pitch `coarse5` | CH Pitch `coarse6` | OH Pitch `coarse7` (new) | FX P1 `fx_p1` |
| Knob row 2 | D1 Decay `velod1` | D2 Decay `velod2` | D3 Decay `velod3` | SD Decay `velod4` | CY Decay `velod5` | CH Decay `velod6_closed` | OH Decay `velod7` (new) | FX P2 `fx_p2` |
| Knob row 3 | D1 Cutoff `filter_freq_1` | D2 Cutoff `filter_freq_2` | D3 Cutoff `filter_freq_3` | SD Cutoff `filter_freq_4` | CY Cutoff `filter_freq_5` | CH Cutoff `filter_freq_6` | OH Cutoff `filter_freq_7` (new) | Global S.rate `voice_decimation_all` |
| Fader | D1 Level `vdrum_fader_1` | D2 Level `vdrum_fader_2` | D3 Level `vdrum_fader_3` | SD Level `vdrum_fader_4` | CY Level `vdrum_fader_5` | CH Level `vdrum_fader_6` | OH Level `vdrum_fader_7` (new) | FX dry/wet `fx_dry_wet` |
| Top button | D1 Trigger \[press\] | D2 Trigger \[press\] | D3 Trigger \[press\] | SD Trigger \[press\] | CY Trigger \[press\] | CH Trigger \[press\] | OH Trigger \[press\] | FX on/off \[on/off\]: fx\_type <-> off |
| Bottom button | D1 Mute \[on/off\] | D2 Mute \[on/off\] | D3 Mute \[on/off\] | SD Mute \[on/off\] | CY Mute \[on/off\] | CH Mute \[on/off\] | OH Mute \[on/off\] | Mute all \[on/off\] |

## Mode 11: LFO + Mod

| Row | 1 DRUM 1 | 2 DRUM 2 | 3 DRUM 3 | 4 SNARE | 5 CYMBAL | 6 CL. HAT | 7 OP. HAT | 8 ALL |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | D1 LFO rate `freq_lfo1` | D2 LFO rate `freq_lfo2` | D3 LFO rate `freq_lfo3` | SD LFO rate `freq_lfo4` | CY LFO rate `freq_lfo5` | CH LFO rate `freq_lfo6` | OH LFO rate `freq_lfo7` (new) | Rate scale: all LFOs |
| Knob row 2 | D1 LFO amt `amount_lfo1` | D2 LFO amt `amount_lfo2` | D3 LFO amt `amount_lfo3` | SD LFO amt `amount_lfo4` | CY LFO amt `amount_lfo5` | CH LFO amt `amount_lfo6` | OH LFO amt `amount_lfo7` (new) | Vel scale: all voices |
| Knob row 3 | D1 LFO dest `target_lfo1` | D2 LFO dest `target_lfo2` | D3 LFO dest `target_lfo3` | SD LFO dest `target_lfo4` | CY LFO dest `target_lfo5` | CH LFO dest `target_lfo6` | OH LFO dest `target_lfo7` (new) | LFO offset all: phase |
| Fader | D1 Vel amt `velo_mod_amt_1` | D2 Vel amt `velo_mod_amt_2` | D3 Vel amt `velo_mod_amt_3` | SD Vel amt `velo_mod_amt_4` | CY Vel amt `velo_mod_amt_5` | CH Vel amt `velo_mod_amt_6` | OH Vel amt `velo_mod_amt_7` (new) | Depth scale: all LFOs |
| Top button | D1 LFO wave \[cycle\] `wave_lfo1` | D2 LFO wave \[cycle\] `wave_lfo2` | D3 LFO wave \[cycle\] `wave_lfo3` | SD LFO wave \[cycle\] `wave_lfo4` | CY LFO wave \[cycle\] `wave_lfo5` | CH LFO wave \[cycle\] `wave_lfo6` | OH LFO wave \[cycle\] `wave_lfo7` (new) | Sync to host \[on/off\]: all LFOs |
| Bottom button | D1 LFO sync \[cycle\] `sync_lfo1` | D2 LFO sync \[cycle\] `sync_lfo2` | D3 LFO sync \[cycle\] `sync_lfo3` | SD LFO sync \[cycle\] `sync_lfo4` | CY LFO sync \[cycle\] `sync_lfo5` | CH LFO sync \[cycle\] `sync_lfo6` | OH LFO sync \[cycle\] `sync_lfo7` (new) | Reset phases \[press\]: all LFOs |

## Mode 12: Morph + Macro

| Row | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 / ALL |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | Macro 1 depth: range scale | Macro 2 depth: range scale | Macro 3 depth: range scale | Macro 4 depth: range scale | Macro 5 depth: range scale | Macro 6 depth: range scale | Macro 7 depth: range scale | Macro 8 depth: range scale |
| Knob row 2 | D1 Morph: to its target | D2 Morph: to its target | D3 Morph: to its target | SD Morph: to its target | CY Morph: to its target | CH Morph: to its target | OH Morph: to its target | Morph all: PAR\_MORPH |
| Knob row 3 | D1 Target: snapshot 1-8 | D2 Target: snapshot 1-8 | D3 Target: snapshot 1-8 | SD Target: snapshot 1-8 | CY Target: snapshot 1-8 | CH Target: snapshot 1-8 | OH Target: snapshot 1-8 | Target all: snapshot 1-8 |
| Fader | Macro 1: user assigned | Macro 2: user assigned | Macro 3: user assigned | Macro 4: user assigned | Macro 5: user assigned | Macro 6: user assigned | Macro 7: user assigned | Macro 8: user assigned |
| Top button | Snapshot 1 \[press\]: hold 1 s: store | Snapshot 2 \[press\]: hold 1 s: store | Snapshot 3 \[press\]: hold 1 s: store | Snapshot 4 \[press\]: hold 1 s: store | Snapshot 5 \[press\]: hold 1 s: store | Snapshot 6 \[press\]: hold 1 s: store | Snapshot 7 \[press\]: hold 1 s: store | Snapshot 8 \[press\]: hold 1 s: store |
| Bottom button | Learn 1 \[on/off\]: assign macro | Learn 2 \[on/off\]: assign macro | Learn 3 \[on/off\]: assign macro | Learn 4 \[on/off\]: assign macro | Learn 5 \[on/off\]: assign macro | Learn 6 \[on/off\]: assign macro | Learn 7 \[on/off\]: assign macro | Learn 8 \[on/off\]: assign macro |

## Mode 16: Global

| Row | 1 DRUM 1 | 2 DRUM 2 | 3 DRUM 3 | 4 SNARE | 5 CYMBAL | 6 CL. HAT | 7 OP. HAT | 8 GLOBAL |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Knob row 1 | D1 note `midi_note1` | D2 note `midi_note2` | D3 note `midi_note3` | SD note `midi_note4` | CY note `midi_note5` | CH note `midi_note6` | OH note `midi_note7` | Tempo: bpm |
| Knob row 2 | D1 channel `midi_chan_1` | D2 channel `midi_chan_2` | D3 channel `midi_chan_3` | SD channel `midi_chan_4` | CY channel `midi_chan_5` | CH channel `midi_chan_6` | OH channel `midi_chan_7` | Global ch. `midi_chan_global` |
| Knob row 3 | LED bright: device CC 111 | Screen bright: device CC 112 | Popup time: device CC 113 | Enc. curve: device CC 121 | Pad velocity: plugin setting | List speed: plugin setting | Fader mode: pickup / jump | Clock: host / internal |
| Fader | D1 HW fader `vdrum_fader_1` | D2 HW fader `vdrum_fader_2` | D3 HW fader `vdrum_fader_3` | SD HW fader `vdrum_fader_4` | CY HW fader `vdrum_fader_5` | CH HW fader `vdrum_fader_6` | OH HW fader `vdrum_fader_7` (new) | Global S.rate `voice_decimation_all` |
| Top button | Sum aux \[on/off\] `vdrum_sum_aux` | HW clock \[on/off\] `vdrum_hardware_clock` | Init kit \[hold\]: 1 s | Save kit \[hold\]: 1 s | Compare \[hold\]: saved kit | Undo \[press\] | Redo \[press\] | Panic \[press\]: all voices off |
| Bottom button | D1 pad \[press\] | D2 pad \[press\] | D3 pad \[press\] | SD pad \[press\] | CY pad \[press\] | CH pad \[press\] | OH pad \[press\] | Accent \[hold\]: pads at vel 127 |
