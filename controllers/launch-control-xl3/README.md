# Launch Control XL 3 custom mode for VDrum

`VDrum.syx` is a custom mode for the Novation Launch Control XL 3, for use with Novation Components.
It is a performance layout: one column per drum voice, and an eighth column with the global sample
rate and extra controls.

Every control sends a CC that VDrum's engine already responds to, through the LXR-02 firmware's own
CC map (CC = parameter number + 1, see `plugin/README.md`, *MIDI*). No MIDI learn is needed and the
plugin is not changed. The same CCs also control a hardware LXR-02.

## Loading

1. Open Novation Components, go to the Launch Control XL 3 custom modes and import `VDrum.syx`.
2. Send it to a free custom mode slot.
3. Route the Launch Control XL 3's MIDI port to VDrum in the host.

Requirements:

* The Launch Control XL 3's global MIDI channel is 1 (its default). The mode uses the global
  channel for every control.
* VDrum's tracks are on MIDI channel 1 (the plugin default). The firmware only accepts a voice's
  CCs on that voice's track channel or on the global channel (`midiParser_cc2voiceNr`; drum 1's
  CCs follow the drum 2 channel).
* None of the CCs below is MIDI-learned in VDrum. A learned CC is taken out of the stream before it
  reaches the engine.

## Layout

| Column | Knob row 1 | Knob row 2 | Knob row 3 | Fader | Button row 1 | Button row 2 |
|---|---|---|---|---|---|---|
| 1 Drum 1 | Pitch | Decay | Filter | Volume | Swell (attack) | Resonance |
| 2 Drum 2 | Pitch | Decay | Filter | Volume | Swell | Resonance |
| 3 Drum 3 | Pitch | Decay | Filter | Volume | Swell | Resonance |
| 4 Snare | Pitch | Decay | Filter | Volume | Swell | Resonance |
| 5 Cymbal | Pitch | Decay | Filter | Volume | Swell | Resonance |
| 6 Closed hi-hat | Pitch | Closed decay | Filter | Volume | Swell | Resonance |
| 7 Open hi-hat | Mod osc frequency 1 | Open decay | Mod osc frequency 2 | Overdrive | Snare repeat | Cymbal repeat |
| 8 Global / extras | Cymbal mod osc frequency 1 | Snare noise/osc mix | Snare noise frequency | Global sample rate | Cymbal mod osc gain 1 | Cymbal mod osc gain 2 |

Choices and their reasons:

* **Knobs.** Rows are the same for the six voices: pitch, amp decay, filter cutoff.
* **Faders.** Voice Volume (`PAR_VOLx`). This is the kit's mixer volume, not the plugin's
  `vdrum_fader_x` hardware-fader parameters, which have no CC.
* **Column 7.** The LXR-02 has seven tracks but six voices: the closed and open hi-hat share one
  voice. Column 7 is the open hi-hat's decay and the hi-hat's metallic tone. The hi-hat has a single
  volume, which is already on fader 6, so fader 7 is the hi-hat overdrive.
* **Column 8.** The only global sound parameter with a CC is the global sample rate (fader 8; top =
  full rate). The FX bus parameters (type, P1–P4, dry/wet) are NRPN-only in the firmware
  (NRPN 106–117), so this CC-only mode cannot reach them. The knobs and buttons of column 8 are
  snare and cymbal controls instead.
* **Buttons.** A button sends 127 when pressed and 0 when released. A parameter learned to it goes
  to its maximum while held and to 0 on release, so 0 must be a sensible resting value. Every
  button parameter is 0 in the init kit:
  * *Swell*: amp attack 127, a slow fade-in of the next hits.
  * *Resonance*: filter resonance 127. Hold it and sweep the filter knob above.
  * *Snare / Cymbal repeat*: amp EG repeat count 127, which retriggers the envelope (clap/roll).
  * *Cymbal mod osc gain 1/2*: brings in the cymbal's FM modulators. Their frequency is knob 1 of
    column 8.

  If a kit stores a non-zero value for one of these parameters, releasing the button sets it to 0.

## Control table

Control IDs are those of the custom mode format (encoders `0x10`–`0x27`, faders `0x28`–`0x2F`,
buttons `0x30`–`0x3F`). Labels are what the controller's screen shows (at most 15 characters).
"Init kit value" is the parameter's value in VDrum's init kit.

| Control | Column | CC | Label | Parameter | Init kit value |
|---|---|---|---|---|---|
| Knob row 1 | 1 | 9 | D1 Pitch | Drum 1 Oscillator Coarse (`coarse1`) | 31 |
| Knob row 2 | 1 | 51 | D1 Decay | Drum 1 Amp EG Decay (`velod1`) | 21 |
| Knob row 3 | 1 | 38 | D1 Filter | Drum 1 Filter Frequency (`filter_freq_1`) | 127 |
| Fader | 1 | 89 | D1 Level | Drum 1 Voice Volume (`vol1`) | 127 |
| Button row 1 | 1 | 50 | D1 Swell | Drum 1 Amp EG Attack (`veloa1`) | 0 |
| Button row 2 | 1 | 44 | D1 Reso | Drum 1 Filter Resonance (`reso_1`) | 0 |
| Knob row 1 | 2 | 11 | D2 Pitch | Drum 2 Oscillator Coarse (`coarse2`) | 41 |
| Knob row 2 | 2 | 53 | D2 Decay | Drum 2 Amp EG Decay (`velod2`) | 13 |
| Knob row 3 | 2 | 39 | D2 Filter | Drum 2 Filter Frequency (`filter_freq_2`) | 127 |
| Fader | 2 | 90 | D2 Level | Drum 2 Voice Volume (`vol2`) | 127 |
| Button row 1 | 2 | 52 | D2 Swell | Drum 2 Amp EG Attack (`veloa2`) | 0 |
| Button row 2 | 2 | 45 | D2 Reso | Drum 2 Filter Resonance (`reso_2`) | 0 |
| Knob row 1 | 3 | 13 | D3 Pitch | Drum 3 Oscillator Coarse (`coarse3`) | 58 |
| Knob row 2 | 3 | 55 | D3 Decay | Drum 3 Amp EG Decay (`velod3`) | 3 |
| Knob row 3 | 3 | 40 | D3 Filter | Drum 3 Filter Frequency (`filter_freq_3`) | 127 |
| Fader | 3 | 91 | D3 Level | Drum 3 Voice Volume (`vol3`) | 127 |
| Button row 1 | 3 | 54 | D3 Swell | Drum 3 Amp EG Attack (`veloa3`) | 0 |
| Button row 2 | 3 | 46 | D3 Reso | Drum 3 Filter Resonance (`reso_3`) | 0 |
| Knob row 1 | 4 | 15 | SD Pitch | Snare Oscillator Coarse (`coarse4`) | 45 |
| Knob row 2 | 4 | 57 | SD Decay | Snare Amp EG Decay (`velod4`) | 10 |
| Knob row 3 | 4 | 41 | SD Filter | Snare Filter Frequency (`filter_freq_4`) | 127 |
| Fader | 4 | 92 | SD Level | Snare Voice Volume (`vol4`) | 127 |
| Button row 1 | 4 | 56 | SD Swell | Snare Amp EG Attack (`veloa4`) | 0 |
| Button row 2 | 4 | 47 | SD Reso | Snare Filter Resonance (`reso_4`) | 0 |
| Knob row 1 | 5 | 17 | CY Pitch | Cymbal Oscillator Coarse (`coarse5`) | 127 |
| Knob row 2 | 5 | 59 | CY Decay | Cymbal Amp EG Decay (`velod5`) | 107 |
| Knob row 3 | 5 | 42 | CY Filter | Cymbal Filter Frequency (`filter_freq_5`) | 68 |
| Fader | 5 | 93 | CY Level | Cymbal Voice Volume (`vol5`) | 127 |
| Button row 1 | 5 | 58 | CY Swell | Cymbal Amp EG Attack (`veloa5`) | 0 |
| Button row 2 | 5 | 48 | CY Reso | Cymbal Filter Resonance (`reso_5`) | 0 |
| Knob row 1 | 6 | 19 | HH Pitch | Hi-hat Oscillator Coarse (`coarse6`) | 50 |
| Knob row 2 | 6 | 61 | CH Decay | Hi-hat Amp EG Decay Closed (`velod6_closed`) | 5 |
| Knob row 3 | 6 | 43 | HH Filter | Hi-hat Filter Frequency (`filter_freq_6`) | 104 |
| Fader | 6 | 94 | HH Level | Hi-hat Voice Volume (`vol6`) | 127 |
| Button row 1 | 6 | 60 | HH Swell | Hi-hat Amp EG Attack (`veloa6`) | 0 |
| Button row 2 | 6 | 49 | HH Reso | Hi-hat Filter Resonance (`reso_6`) | 0 |
| Knob row 1 | 7 | 34 | HH Metal 1 | Hi-hat Mod Osc Frequency 1 (`mod_osc_f1`) | 56 |
| Knob row 2 | 7 | 62 | OH Decay | Hi-hat Amp EG Decay Open (`velod6_open`) | 17 |
| Knob row 3 | 7 | 35 | HH Metal 2 | Hi-hat Mod Osc Frequency 2 (`mod_osc_f2`) | 127 |
| Fader | 7 | 108 | HH Drive | Hi-hat Voice Overdrive (`hat_distortion`) | 63 |
| Button row 1 | 7 | 69 | SD Repeat | Snare Amp EG Repeat Count (`repeat4`) | 0 |
| Button row 2 | 7 | 70 | CY Repeat | Cymbal Amp EG Repeat Count (`repeat5`) | 0 |
| Knob row 1 | 8 | 30 | CY Metal Frq | Cymbal Mod Osc Frequency 1 (`mod_osc_f1_cym`) | 0 |
| Knob row 2 | 8 | 29 | SD Snap | Snare Noise/Osc Mix (`mix1`) | 61 |
| Knob row 3 | 8 | 28 | SD Noise | Snare Noise Frequency (`noise_freq1`) | 127 |
| Fader | 8 | 115 | Sample Rate | Global Voice Sample Rate (`voice_decimation_all`) | 127 |
| Button row 1 | 8 | 32 | CY Metal 1 | Cymbal Mod Osc Gain 1 (`mod_osc_gain1_cym`) | 0 |
| Button row 2 | 8 | 33 | CY Metal 2 | Cymbal Mod Osc Gain 2 (`mod_osc_gain2_cym`) | 0 |

## File format

Novation's *Launch Control XL 3 programmer's reference guide* (v1.0) does not document the custom
mode SysEx. The file was built from `template-default.syx`, the default custom mode exported from
Components (the mode name in it is "LXR02"), and keeps its structure:

* Two SysEx messages: page `00` holds the 24 encoders, page `03` the 8 faders and 16 buttons. Each
  starts `F0 00 20 29 02 15 05 00 45 <page> 7F 20 <name length> <name>`.
* A control definition is `49 <id> 02 <b3> <b4> <b5> <flags> 00 <cc> 7F 00`.
* A label is `<60 + length> <id> <ASCII text>`; the template only has empty labels (`60 <id>`).

Compared with the template, only the mode name ("VDrum"), the CC byte of each control and the
labels differ. All other bytes of each control definition (LED colours, channel and flag bytes)
are copied from the template. The meaning of the name, CC and label fields agrees with two
independent reverse-engineered encoders: audiocontrol-org/audiocontrol (`launch-control-xl3`) and
the midi.guide Launch Control XL 3 builder.

## Verified and not verified

Verified:

* The encoder used to write `VDrum.syx` rebuilds `template-default.syx` byte for byte, given the
  template's name, CCs and empty labels.
* Each of the 48 CCs, sent on channel 1 to VDrum's engine (`lxr-core`, plugin MIDI defaults), sets
  the parameter listed above. Values 0 and 127 map to the parameter's minimum and maximum.
* The 48 CCs are distinct, and none is CC 0 (kit change), 6, 98 or 99 (NRPN).

Not verified (no hardware or Components available here):

* That Components imports the file and the device accepts it.
* That labels with text display as intended. The length-prefixed label encoding comes from the
  reverse-engineered sources, not from a Novation file with labels.
* Button behaviour (momentary or toggle). The template's buttons were copied unchanged, and the
  layout assumes momentary. If they toggle, each press latches the effect on or off. The behaviour
  can be changed per button in Components.
* How each effect sounds. Only the parameter values were checked, not the audio.
