"""Per-voice parameter ids (track t = 0..6: D1, D2, D3, SD, CY, CH, OH), as `voice_param` in
crates/vdrum-surface/src/layout.rs."""

_DRIVE = ["drive1", "drive2", "drive3", "snare_distortion", "cymbal_distortion", "hat_distortion", "oh_distortion"]
_WAVE = ["osc_wave_drum1", "osc_wave_drum2", "osc_wave_drum3", "osc_wave_snare", "wave1_cym", "wave1_hh", "wave1_oh"]
_KINDS = {
    "coarse": "coarse{v}", "fine": "fine{v}", "pwm": "pwm_{v}", "attack": "veloa{v}", "slope": "vol_slope{v}",
    "trans_wave": "trans{v}_wave", "trans_freq": "trans{v}_freq", "trans_vol": "trans{v}_vol",
    "ftype": "filter_type_{v}", "reso": "reso_{v}", "fdrive": "filter_drive_{v}", "cutoff": "filter_freq_{v}",
    "velvol": "volume_mod_on_off{v}", "veldest": "vel_dest_{v}", "velamt": "velo_mod_amt_{v}",
    "srate": "voice_decimation{v}", "out": "audio_out{v}", "pan": "pan{v}", "vol": "vol{v}",
    "lfo_rate": "freq_lfo{v}", "lfo_amt": "amount_lfo{v}", "lfo_ofs": "offset_lfo{v}", "lfo_wave": "wave_lfo{v}",
    "lfo_sync": "sync_lfo{v}", "lfo_rtg": "retrigger_lfo{v}", "lfo_voice": "voice_lfo{v}", "lfo_dest": "target_lfo{v}",
}


def vid(t, kind):
    """Parameter id of `kind` for track t (0-based)."""
    v = t + 1
    if kind == "drive":
        return _DRIVE[t]
    if kind == "wave":
        return _WAVE[t]
    if kind == "decay":
        return "velod6_closed" if v == 6 else f"velod{v}"
    return _KINDS[kind].format(v=v)
