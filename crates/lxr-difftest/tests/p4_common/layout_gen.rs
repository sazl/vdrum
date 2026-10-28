impl CMem for OscInfo {
    const SIZE: usize = S::OscInfo as usize;
    fn rd(b: &[u8], o: usize) -> Self { OscInfo {
        output: <i16 as CMem>::rd(b, o + O::OscInfo_output as usize),
        phaseInc: <u32 as CMem>::rd(b, o + O::OscInfo_phaseInc as usize),
        phase: <u32 as CMem>::rd(b, o + O::OscInfo_phase as usize),
        freq: <f32 as CMem>::rd(b, o + O::OscInfo_freq as usize),
        waveform: <u8 as CMem>::rd(b, o + O::OscInfo_waveform as usize),
        tableOffset: <u8 as CMem>::rd(b, o + O::OscInfo_tableOffset as usize),
        pitchMod: <f32 as CMem>::rd(b, o + O::OscInfo_pitchMod as usize),
        fmMod: <f32 as CMem>::rd(b, o + O::OscInfo_fmMod as usize),
        modNodeValue: <f32 as CMem>::rd(b, o + O::OscInfo_modNodeValue as usize),
        midiFreq: <u16 as CMem>::rd(b, o + O::OscInfo_midiFreq as usize),
        baseNote: <u8 as CMem>::rd(b, o + O::OscInfo_baseNote as usize),
        startPhase: <u32 as CMem>::rd(b, o + O::OscInfo_startPhase as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.output.wr(b, o + O::OscInfo_output as usize);
        self.phaseInc.wr(b, o + O::OscInfo_phaseInc as usize);
        self.phase.wr(b, o + O::OscInfo_phase as usize);
        self.freq.wr(b, o + O::OscInfo_freq as usize);
        self.waveform.wr(b, o + O::OscInfo_waveform as usize);
        self.tableOffset.wr(b, o + O::OscInfo_tableOffset as usize);
        self.pitchMod.wr(b, o + O::OscInfo_pitchMod as usize);
        self.fmMod.wr(b, o + O::OscInfo_fmMod as usize);
        self.modNodeValue.wr(b, o + O::OscInfo_modNodeValue as usize);
        self.midiFreq.wr(b, o + O::OscInfo_midiFreq as usize);
        self.baseNote.wr(b, o + O::OscInfo_baseNote as usize);
        self.startPhase.wr(b, o + O::OscInfo_startPhase as usize);
    }
}

impl CMem for DecayEg {
    const SIZE: usize = S::DecayEg as usize;
    fn rd(b: &[u8], o: usize) -> Self { DecayEg {
        decay: <f32 as CMem>::rd(b, o + O::DecayEg_decay as usize),
        slope: <f32 as CMem>::rd(b, o + O::DecayEg_slope as usize),
        value: <f32 as CMem>::rd(b, o + O::DecayEg_value as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.decay.wr(b, o + O::DecayEg_decay as usize);
        self.slope.wr(b, o + O::DecayEg_slope as usize);
        self.value.wr(b, o + O::DecayEg_value as usize);
    }
}

impl CMem for TransientGenerator {
    const SIZE: usize = S::TransientGenerator as usize;
    fn rd(b: &[u8], o: usize) -> Self { TransientGenerator {
        output: <i16 as CMem>::rd(b, o + O::TransientGenerator_output as usize),
        phase: <u32 as CMem>::rd(b, o + O::TransientGenerator_phase as usize),
        pitch: <f32 as CMem>::rd(b, o + O::TransientGenerator_pitch as usize),
        waveform: <u8 as CMem>::rd(b, o + O::TransientGenerator_waveform as usize),
        volume: <f32 as CMem>::rd(b, o + O::TransientGenerator_volume as usize),
        midiFreq: <u16 as CMem>::rd(b, o + O::TransientGenerator_midiFreq as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.output.wr(b, o + O::TransientGenerator_output as usize);
        self.phase.wr(b, o + O::TransientGenerator_phase as usize);
        self.pitch.wr(b, o + O::TransientGenerator_pitch as usize);
        self.waveform.wr(b, o + O::TransientGenerator_waveform as usize);
        self.volume.wr(b, o + O::TransientGenerator_volume as usize);
        self.midiFreq.wr(b, o + O::TransientGenerator_midiFreq as usize);
    }
}

impl CMem for Lfo {
    const SIZE: usize = S::Lfo as usize;
    fn rd(b: &[u8], o: usize) -> Self { Lfo {
        phase: <u32 as CMem>::rd(b, o + O::Lfo_phase as usize),
        phaseInc: <u32 as CMem>::rd(b, o + O::Lfo_phaseInc as usize),
        waveform: <u8 as CMem>::rd(b, o + O::Lfo_waveform as usize),
        retrigger: <u8 as CMem>::rd(b, o + O::Lfo_retrigger as usize),
        phaseOffset: <u32 as CMem>::rd(b, o + O::Lfo_phaseOffset as usize),
        rnd: <f32 as CMem>::rd(b, o + O::Lfo_rnd as usize),
        sync: <u8 as CMem>::rd(b, o + O::Lfo_sync as usize),
        freq: <f32 as CMem>::rd(b, o + O::Lfo_freq as usize),
        modTarget: <ModulationNode as CMem>::rd(b, o + O::Lfo_modTarget as usize),
        modNodeValue: <f32 as CMem>::rd(b, o + O::Lfo_modNodeValue as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.phase.wr(b, o + O::Lfo_phase as usize);
        self.phaseInc.wr(b, o + O::Lfo_phaseInc as usize);
        self.waveform.wr(b, o + O::Lfo_waveform as usize);
        self.retrigger.wr(b, o + O::Lfo_retrigger as usize);
        self.phaseOffset.wr(b, o + O::Lfo_phaseOffset as usize);
        self.rnd.wr(b, o + O::Lfo_rnd as usize);
        self.sync.wr(b, o + O::Lfo_sync as usize);
        self.freq.wr(b, o + O::Lfo_freq as usize);
        self.modTarget.wr(b, o + O::Lfo_modTarget as usize);
        self.modNodeValue.wr(b, o + O::Lfo_modNodeValue as usize);
    }
}

impl CMem for ModulationNode {
    const SIZE: usize = S::ModulationNode as usize;
    fn rd(b: &[u8], o: usize) -> Self { ModulationNode {
        destination: <u16 as CMem>::rd(b, o + O::ModulationNode_destination as usize),
        r#type: <u8 as CMem>::rd(b, o + O::ModulationNode_type as usize),
        originalValue: <ptrValue as CMem>::rd(b, o + O::ModulationNode_originalValue as usize),
        amount: <f32 as CMem>::rd(b, o + O::ModulationNode_amount as usize),
        lastVal: <f32 as CMem>::rd(b, o + O::ModulationNode_lastVal as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.destination.wr(b, o + O::ModulationNode_destination as usize);
        self.r#type.wr(b, o + O::ModulationNode_type as usize);
        self.originalValue.wr(b, o + O::ModulationNode_originalValue as usize);
        self.amount.wr(b, o + O::ModulationNode_amount as usize);
        self.lastVal.wr(b, o + O::ModulationNode_lastVal as usize);
    }
}

impl CMem for SlopeEg2 {
    const SIZE: usize = S::SlopeEg2 as usize;
    fn rd(b: &[u8], o: usize) -> Self { SlopeEg2 {
        attack: <f64 as CMem>::rd(b, o + O::SlopeEg2_attack as usize),
        decay: <f64 as CMem>::rd(b, o + O::SlopeEg2_decay as usize),
        slope: <f64 as CMem>::rd(b, o + O::SlopeEg2_slope as usize),
        invSlope: <f64 as CMem>::rd(b, o + O::SlopeEg2_invSlope as usize),
        repeat: <u8 as CMem>::rd(b, o + O::SlopeEg2_repeat as usize),
        repeatCnt: <u8 as CMem>::rd(b, o + O::SlopeEg2_repeatCnt as usize),
        value: <f64 as CMem>::rd(b, o + O::SlopeEg2_value as usize),
        state: <u8 as CMem>::rd(b, o + O::SlopeEg2_state as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.attack.wr(b, o + O::SlopeEg2_attack as usize);
        self.decay.wr(b, o + O::SlopeEg2_decay as usize);
        self.slope.wr(b, o + O::SlopeEg2_slope as usize);
        self.invSlope.wr(b, o + O::SlopeEg2_invSlope as usize);
        self.repeat.wr(b, o + O::SlopeEg2_repeat as usize);
        self.repeatCnt.wr(b, o + O::SlopeEg2_repeatCnt as usize);
        self.value.wr(b, o + O::SlopeEg2_value as usize);
        self.state.wr(b, o + O::SlopeEg2_state as usize);
    }
}

impl CMem for Distortion {
    const SIZE: usize = S::Distortion as usize;
    fn rd(b: &[u8], o: usize) -> Self { Distortion {
        shape: <f32 as CMem>::rd(b, o + O::Distortion_shape as usize),
        inv_shape: <f32 as CMem>::rd(b, o + O::Distortion_inv_shape as usize),
        DriveParamFilter: <OnePoleFilter as CMem>::rd(b, o + O::Distortion_DriveParamFilter as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.shape.wr(b, o + O::Distortion_shape as usize);
        self.inv_shape.wr(b, o + O::Distortion_inv_shape as usize);
        self.DriveParamFilter.wr(b, o + O::Distortion_DriveParamFilter as usize);
    }
}

impl CMem for ResonantFilter {
    const SIZE: usize = S::ResonantFilter as usize;
    fn rd(b: &[u8], o: usize) -> Self { ResonantFilter {
        f: <f32 as CMem>::rd(b, o + O::ResonantFilter_f as usize),
        g: <f32 as CMem>::rd(b, o + O::ResonantFilter_g as usize),
        q: <f32 as CMem>::rd(b, o + O::ResonantFilter_q as usize),
        s1: <f32 as CMem>::rd(b, o + O::ResonantFilter_s1 as usize),
        s2: <f32 as CMem>::rd(b, o + O::ResonantFilter_s2 as usize),
        a: <f32 as CMem>::rd(b, o + O::ResonantFilter_a as usize),
        b: <f32 as CMem>::rd(b, o + O::ResonantFilter_b as usize),
        drive: <f32 as CMem>::rd(b, o + O::ResonantFilter_drive as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.f.wr(b, o + O::ResonantFilter_f as usize);
        self.g.wr(b, o + O::ResonantFilter_g as usize);
        self.q.wr(b, o + O::ResonantFilter_q as usize);
        self.s1.wr(b, o + O::ResonantFilter_s1 as usize);
        self.s2.wr(b, o + O::ResonantFilter_s2 as usize);
        self.a.wr(b, o + O::ResonantFilter_a as usize);
        self.b.wr(b, o + O::ResonantFilter_b as usize);
        self.drive.wr(b, o + O::ResonantFilter_drive as usize);
    }
}

impl CMem for SnapEg {
    const SIZE: usize = S::SnapEg as usize;
    fn rd(b: &[u8], o: usize) -> Self { SnapEg {
        value: <f32 as CMem>::rd(b, o + O::SnapEg_value as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.value.wr(b, o + O::SnapEg_value as usize);
    }
}

impl CMem for Dither {
    const SIZE: usize = S::Dither as usize;
    fn rd(b: &[u8], o: usize) -> Self { Dither {
        r1: <i16 as CMem>::rd(b, o + O::Dither_r1 as usize),
        r2: <i16 as CMem>::rd(b, o + O::Dither_r2 as usize),
        s1: <f32 as CMem>::rd(b, o + O::Dither_s1 as usize),
        s2: <f32 as CMem>::rd(b, o + O::Dither_s2 as usize),
        r#in: <f32 as CMem>::rd(b, o + O::Dither_in as usize),
        tmp: <f32 as CMem>::rd(b, o + O::Dither_tmp as usize),
        out: <i16 as CMem>::rd(b, o + O::Dither_out as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.r1.wr(b, o + O::Dither_r1 as usize);
        self.r2.wr(b, o + O::Dither_r2 as usize);
        self.s1.wr(b, o + O::Dither_s1 as usize);
        self.s2.wr(b, o + O::Dither_s2 as usize);
        self.r#in.wr(b, o + O::Dither_in as usize);
        self.tmp.wr(b, o + O::Dither_tmp as usize);
        self.out.wr(b, o + O::Dither_out as usize);
    }
}

impl CMem for OnePoleFilter {
    const SIZE: usize = S::OnePoleFilter as usize;
    fn rd(b: &[u8], o: usize) -> Self { OnePoleFilter {
        r: <f64 as CMem>::rd(b, o + O::OnePoleFilter_r as usize),
        b: <f64 as CMem>::rd(b, o + O::OnePoleFilter_b as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.r.wr(b, o + O::OnePoleFilter_r as usize);
        self.b.wr(b, o + O::OnePoleFilter_b as usize);
    }
}

impl CMem for DrumVoice {
    const SIZE: usize = S::DrumVoice as usize;
    fn rd(b: &[u8], o: usize) -> Self { DrumVoice {
        osc: <OscInfo as CMem>::rd(b, o + O::DrumVoice_osc as usize),
        modOsc: <OscInfo as CMem>::rd(b, o + O::DrumVoice_modOsc as usize),
        fmModAmount: <f32 as CMem>::rd(b, o + O::DrumVoice_fmModAmount as usize),
        vol: <f32 as CMem>::rd(b, o + O::DrumVoice_vol as usize),
        velo: <f32 as CMem>::rd(b, o + O::DrumVoice_velo as usize),
        pan: <u8 as CMem>::rd(b, o + O::DrumVoice_pan as usize),
        triggeredPanL: <f32 as CMem>::rd(b, o + O::DrumVoice_triggeredPanL as usize),
        triggeredPanR: <f32 as CMem>::rd(b, o + O::DrumVoice_triggeredPanR as usize),
        oscSample: <i16 as CMem>::rd(b, o + O::DrumVoice_oscSample as usize),
        oscPitchEg: <DecayEg as CMem>::rd(b, o + O::DrumVoice_oscPitchEg as usize),
        egPitchModAmount: <f32 as CMem>::rd(b, o + O::DrumVoice_egPitchModAmount as usize),
        offset: <f32 as CMem>::rd(b, o + O::DrumVoice_offset as usize),
        transGen: <TransientGenerator as CMem>::rd(b, o + O::DrumVoice_transGen as usize),
        lfo: <Lfo as CMem>::rd(b, o + O::DrumVoice_lfo as usize),
        oscVolEg: <SlopeEg2 as CMem>::rd(b, o + O::DrumVoice_oscVolEg as usize),
        egValueOscVol: <f32 as CMem>::rd(b, o + O::DrumVoice_egValueOscVol as usize),
        volEgValueBlock: core::array::from_fn(|i| <f32 as CMem>::rd(b, o + O::DrumVoice_volEgValueBlock as usize + i * <f32 as CMem>::SIZE)),
        distortion: <Distortion as CMem>::rd(b, o + O::DrumVoice_distortion as usize),
        filter: <ResonantFilter as CMem>::rd(b, o + O::DrumVoice_filter as usize),
        filterType: <u8 as CMem>::rd(b, o + O::DrumVoice_filterType as usize),
        ampFilterInput: <f64 as CMem>::rd(b, o + O::DrumVoice_ampFilterInput as usize),
        lastGain: <f64 as CMem>::rd(b, o + O::DrumVoice_lastGain as usize),
        mixOscs: <u8 as CMem>::rd(b, o + O::DrumVoice_mixOscs as usize),
        decimationCnt: <f32 as CMem>::rd(b, o + O::DrumVoice_decimationCnt as usize),
        decimationRate: <f32 as CMem>::rd(b, o + O::DrumVoice_decimationRate as usize),
        snapEg: <SnapEg as CMem>::rd(b, o + O::DrumVoice_snapEg as usize),
        volumeMod: <u8 as CMem>::rd(b, o + O::DrumVoice_volumeMod as usize),
        dither: <Dither as CMem>::rd(b, o + O::DrumVoice_dither as usize),
        combinedVolume: <f32 as CMem>::rd(b, o + O::DrumVoice_combinedVolume as usize),
        FaderVolumeFilter: <OnePoleFilter as CMem>::rd(b, o + O::DrumVoice_FaderVolumeFilter as usize),
        MixerPageFilter: core::array::from_fn(|i| <OnePoleFilter as CMem>::rd(b, o + O::DrumVoice_MixerPageFilter as usize + i * <OnePoleFilter as CMem>::SIZE)),
        pwm: <f32 as CMem>::rd(b, o + O::DrumVoice_pwm as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.osc.wr(b, o + O::DrumVoice_osc as usize);
        self.modOsc.wr(b, o + O::DrumVoice_modOsc as usize);
        self.fmModAmount.wr(b, o + O::DrumVoice_fmModAmount as usize);
        self.vol.wr(b, o + O::DrumVoice_vol as usize);
        self.velo.wr(b, o + O::DrumVoice_velo as usize);
        self.pan.wr(b, o + O::DrumVoice_pan as usize);
        self.triggeredPanL.wr(b, o + O::DrumVoice_triggeredPanL as usize);
        self.triggeredPanR.wr(b, o + O::DrumVoice_triggeredPanR as usize);
        self.oscSample.wr(b, o + O::DrumVoice_oscSample as usize);
        self.oscPitchEg.wr(b, o + O::DrumVoice_oscPitchEg as usize);
        self.egPitchModAmount.wr(b, o + O::DrumVoice_egPitchModAmount as usize);
        self.offset.wr(b, o + O::DrumVoice_offset as usize);
        self.transGen.wr(b, o + O::DrumVoice_transGen as usize);
        self.lfo.wr(b, o + O::DrumVoice_lfo as usize);
        self.oscVolEg.wr(b, o + O::DrumVoice_oscVolEg as usize);
        self.egValueOscVol.wr(b, o + O::DrumVoice_egValueOscVol as usize);
        for i in 0..64 { self.volEgValueBlock[i].wr(b, o + O::DrumVoice_volEgValueBlock as usize + i * <f32 as CMem>::SIZE); }
        self.distortion.wr(b, o + O::DrumVoice_distortion as usize);
        self.filter.wr(b, o + O::DrumVoice_filter as usize);
        self.filterType.wr(b, o + O::DrumVoice_filterType as usize);
        self.ampFilterInput.wr(b, o + O::DrumVoice_ampFilterInput as usize);
        self.lastGain.wr(b, o + O::DrumVoice_lastGain as usize);
        self.mixOscs.wr(b, o + O::DrumVoice_mixOscs as usize);
        self.decimationCnt.wr(b, o + O::DrumVoice_decimationCnt as usize);
        self.decimationRate.wr(b, o + O::DrumVoice_decimationRate as usize);
        self.snapEg.wr(b, o + O::DrumVoice_snapEg as usize);
        self.volumeMod.wr(b, o + O::DrumVoice_volumeMod as usize);
        self.dither.wr(b, o + O::DrumVoice_dither as usize);
        self.combinedVolume.wr(b, o + O::DrumVoice_combinedVolume as usize);
        self.FaderVolumeFilter.wr(b, o + O::DrumVoice_FaderVolumeFilter as usize);
        for i in 0..2 { self.MixerPageFilter[i].wr(b, o + O::DrumVoice_MixerPageFilter as usize + i * <OnePoleFilter as CMem>::SIZE); }
        self.pwm.wr(b, o + O::DrumVoice_pwm as usize);
    }
}

impl CMem for SnareVoice {
    const SIZE: usize = S::SnareVoice as usize;
    fn rd(b: &[u8], o: usize) -> Self { SnareVoice {
        osc: <OscInfo as CMem>::rd(b, o + O::SnareVoice_osc as usize),
        noiseOsc: <OscInfo as CMem>::rd(b, o + O::SnareVoice_noiseOsc as usize),
        filterType: <u8 as CMem>::rd(b, o + O::SnareVoice_filterType as usize),
        vol: <f32 as CMem>::rd(b, o + O::SnareVoice_vol as usize),
        pan: <u8 as CMem>::rd(b, o + O::SnareVoice_pan as usize),
        triggeredPanL: <f32 as CMem>::rd(b, o + O::SnareVoice_triggeredPanL as usize),
        triggeredPanR: <f32 as CMem>::rd(b, o + O::SnareVoice_triggeredPanR as usize),
        mix: <f32 as CMem>::rd(b, o + O::SnareVoice_mix as usize),
        velo: <f32 as CMem>::rd(b, o + O::SnareVoice_velo as usize),
        oscSample: <i16 as CMem>::rd(b, o + O::SnareVoice_oscSample as usize),
        noiseSample: <i32 as CMem>::rd(b, o + O::SnareVoice_noiseSample as usize),
        filter: <ResonantFilter as CMem>::rd(b, o + O::SnareVoice_filter as usize),
        oscPitchEg: <DecayEg as CMem>::rd(b, o + O::SnareVoice_oscPitchEg as usize),
        egPitchModAmount: <f32 as CMem>::rd(b, o + O::SnareVoice_egPitchModAmount as usize),
        lfo: <Lfo as CMem>::rd(b, o + O::SnareVoice_lfo as usize),
        transGen: <TransientGenerator as CMem>::rd(b, o + O::SnareVoice_transGen as usize),
        oscVolEg: <SlopeEg2 as CMem>::rd(b, o + O::SnareVoice_oscVolEg as usize),
        egValueOscVol: <f32 as CMem>::rd(b, o + O::SnareVoice_egValueOscVol as usize),
        distortion: <Distortion as CMem>::rd(b, o + O::SnareVoice_distortion as usize),
        snapEg: <SnapEg as CMem>::rd(b, o + O::SnareVoice_snapEg as usize),
        volumeMod: <u8 as CMem>::rd(b, o + O::SnareVoice_volumeMod as usize),
        combinedVolume: <f32 as CMem>::rd(b, o + O::SnareVoice_combinedVolume as usize),
        FaderVolumeFilter: <OnePoleFilter as CMem>::rd(b, o + O::SnareVoice_FaderVolumeFilter as usize),
        pwm: <f32 as CMem>::rd(b, o + O::SnareVoice_pwm as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.osc.wr(b, o + O::SnareVoice_osc as usize);
        self.noiseOsc.wr(b, o + O::SnareVoice_noiseOsc as usize);
        self.filterType.wr(b, o + O::SnareVoice_filterType as usize);
        self.vol.wr(b, o + O::SnareVoice_vol as usize);
        self.pan.wr(b, o + O::SnareVoice_pan as usize);
        self.triggeredPanL.wr(b, o + O::SnareVoice_triggeredPanL as usize);
        self.triggeredPanR.wr(b, o + O::SnareVoice_triggeredPanR as usize);
        self.mix.wr(b, o + O::SnareVoice_mix as usize);
        self.velo.wr(b, o + O::SnareVoice_velo as usize);
        self.oscSample.wr(b, o + O::SnareVoice_oscSample as usize);
        self.noiseSample.wr(b, o + O::SnareVoice_noiseSample as usize);
        self.filter.wr(b, o + O::SnareVoice_filter as usize);
        self.oscPitchEg.wr(b, o + O::SnareVoice_oscPitchEg as usize);
        self.egPitchModAmount.wr(b, o + O::SnareVoice_egPitchModAmount as usize);
        self.lfo.wr(b, o + O::SnareVoice_lfo as usize);
        self.transGen.wr(b, o + O::SnareVoice_transGen as usize);
        self.oscVolEg.wr(b, o + O::SnareVoice_oscVolEg as usize);
        self.egValueOscVol.wr(b, o + O::SnareVoice_egValueOscVol as usize);
        self.distortion.wr(b, o + O::SnareVoice_distortion as usize);
        self.snapEg.wr(b, o + O::SnareVoice_snapEg as usize);
        self.volumeMod.wr(b, o + O::SnareVoice_volumeMod as usize);
        self.combinedVolume.wr(b, o + O::SnareVoice_combinedVolume as usize);
        self.FaderVolumeFilter.wr(b, o + O::SnareVoice_FaderVolumeFilter as usize);
        self.pwm.wr(b, o + O::SnareVoice_pwm as usize);
    }
}

impl CMem for HiHatVoice {
    const SIZE: usize = S::HiHatVoice as usize;
    fn rd(b: &[u8], o: usize) -> Self { HiHatVoice {
        osc: <OscInfo as CMem>::rd(b, o + O::HiHatVoice_osc as usize),
        modOsc: <OscInfo as CMem>::rd(b, o + O::HiHatVoice_modOsc as usize),
        modOsc2: <OscInfo as CMem>::rd(b, o + O::HiHatVoice_modOsc2 as usize),
        fmModAmount1: <f32 as CMem>::rd(b, o + O::HiHatVoice_fmModAmount1 as usize),
        fmModAmount2: <f32 as CMem>::rd(b, o + O::HiHatVoice_fmModAmount2 as usize),
        vol: <f32 as CMem>::rd(b, o + O::HiHatVoice_vol as usize),
        pan: <u8 as CMem>::rd(b, o + O::HiHatVoice_pan as usize),
        triggeredPanL: <f32 as CMem>::rd(b, o + O::HiHatVoice_triggeredPanL as usize),
        triggeredPanR: <f32 as CMem>::rd(b, o + O::HiHatVoice_triggeredPanR as usize),
        velo: <f32 as CMem>::rd(b, o + O::HiHatVoice_velo as usize),
        noiseSample: <i32 as CMem>::rd(b, o + O::HiHatVoice_noiseSample as usize),
        filter: <ResonantFilter as CMem>::rd(b, o + O::HiHatVoice_filter as usize),
        filterType: <u8 as CMem>::rd(b, o + O::HiHatVoice_filterType as usize),
        transGen: <TransientGenerator as CMem>::rd(b, o + O::HiHatVoice_transGen as usize),
        decayClosed: <f32 as CMem>::rd(b, o + O::HiHatVoice_decayClosed as usize),
        decayOpen: <f32 as CMem>::rd(b, o + O::HiHatVoice_decayOpen as usize),
        distortion: <Distortion as CMem>::rd(b, o + O::HiHatVoice_distortion as usize),
        oscVolEg: <SlopeEg2 as CMem>::rd(b, o + O::HiHatVoice_oscVolEg as usize),
        egValueOscVol: <f32 as CMem>::rd(b, o + O::HiHatVoice_egValueOscVol as usize),
        lfo: <Lfo as CMem>::rd(b, o + O::HiHatVoice_lfo as usize),
        snapEg: <SnapEg as CMem>::rd(b, o + O::HiHatVoice_snapEg as usize),
        volumeMod: <u8 as CMem>::rd(b, o + O::HiHatVoice_volumeMod as usize),
        isOpen: <u8 as CMem>::rd(b, o + O::HiHatVoice_isOpen as usize),
        combinedVolume: <f32 as CMem>::rd(b, o + O::HiHatVoice_combinedVolume as usize),
        FaderVolumeFilter: <OnePoleFilter as CMem>::rd(b, o + O::HiHatVoice_FaderVolumeFilter as usize),
        pwm: <f32 as CMem>::rd(b, o + O::HiHatVoice_pwm as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.osc.wr(b, o + O::HiHatVoice_osc as usize);
        self.modOsc.wr(b, o + O::HiHatVoice_modOsc as usize);
        self.modOsc2.wr(b, o + O::HiHatVoice_modOsc2 as usize);
        self.fmModAmount1.wr(b, o + O::HiHatVoice_fmModAmount1 as usize);
        self.fmModAmount2.wr(b, o + O::HiHatVoice_fmModAmount2 as usize);
        self.vol.wr(b, o + O::HiHatVoice_vol as usize);
        self.pan.wr(b, o + O::HiHatVoice_pan as usize);
        self.triggeredPanL.wr(b, o + O::HiHatVoice_triggeredPanL as usize);
        self.triggeredPanR.wr(b, o + O::HiHatVoice_triggeredPanR as usize);
        self.velo.wr(b, o + O::HiHatVoice_velo as usize);
        self.noiseSample.wr(b, o + O::HiHatVoice_noiseSample as usize);
        self.filter.wr(b, o + O::HiHatVoice_filter as usize);
        self.filterType.wr(b, o + O::HiHatVoice_filterType as usize);
        self.transGen.wr(b, o + O::HiHatVoice_transGen as usize);
        self.decayClosed.wr(b, o + O::HiHatVoice_decayClosed as usize);
        self.decayOpen.wr(b, o + O::HiHatVoice_decayOpen as usize);
        self.distortion.wr(b, o + O::HiHatVoice_distortion as usize);
        self.oscVolEg.wr(b, o + O::HiHatVoice_oscVolEg as usize);
        self.egValueOscVol.wr(b, o + O::HiHatVoice_egValueOscVol as usize);
        self.lfo.wr(b, o + O::HiHatVoice_lfo as usize);
        self.snapEg.wr(b, o + O::HiHatVoice_snapEg as usize);
        self.volumeMod.wr(b, o + O::HiHatVoice_volumeMod as usize);
        self.isOpen.wr(b, o + O::HiHatVoice_isOpen as usize);
        self.combinedVolume.wr(b, o + O::HiHatVoice_combinedVolume as usize);
        self.FaderVolumeFilter.wr(b, o + O::HiHatVoice_FaderVolumeFilter as usize);
        self.pwm.wr(b, o + O::HiHatVoice_pwm as usize);
    }
}

impl CMem for CymbalVoice {
    const SIZE: usize = S::CymbalVoice as usize;
    fn rd(b: &[u8], o: usize) -> Self { CymbalVoice {
        osc: <OscInfo as CMem>::rd(b, o + O::CymbalVoice_osc as usize),
        modOsc: <OscInfo as CMem>::rd(b, o + O::CymbalVoice_modOsc as usize),
        modOsc2: <OscInfo as CMem>::rd(b, o + O::CymbalVoice_modOsc2 as usize),
        fmModAmount1: <f32 as CMem>::rd(b, o + O::CymbalVoice_fmModAmount1 as usize),
        fmModAmount2: <f32 as CMem>::rd(b, o + O::CymbalVoice_fmModAmount2 as usize),
        vol: <f32 as CMem>::rd(b, o + O::CymbalVoice_vol as usize),
        pan: <u8 as CMem>::rd(b, o + O::CymbalVoice_pan as usize),
        triggeredPanL: <f32 as CMem>::rd(b, o + O::CymbalVoice_triggeredPanL as usize),
        triggeredPanR: <f32 as CMem>::rd(b, o + O::CymbalVoice_triggeredPanR as usize),
        velo: <f32 as CMem>::rd(b, o + O::CymbalVoice_velo as usize),
        noiseSample: <i32 as CMem>::rd(b, o + O::CymbalVoice_noiseSample as usize),
        filter: <ResonantFilter as CMem>::rd(b, o + O::CymbalVoice_filter as usize),
        filterType: <u8 as CMem>::rd(b, o + O::CymbalVoice_filterType as usize),
        lfo: <Lfo as CMem>::rd(b, o + O::CymbalVoice_lfo as usize),
        transGen: <TransientGenerator as CMem>::rd(b, o + O::CymbalVoice_transGen as usize),
        distortion: <Distortion as CMem>::rd(b, o + O::CymbalVoice_distortion as usize),
        oscVolEg: <SlopeEg2 as CMem>::rd(b, o + O::CymbalVoice_oscVolEg as usize),
        egValueOscVol: <f32 as CMem>::rd(b, o + O::CymbalVoice_egValueOscVol as usize),
        volumeMod: <u8 as CMem>::rd(b, o + O::CymbalVoice_volumeMod as usize),
        snapEg: <SnapEg as CMem>::rd(b, o + O::CymbalVoice_snapEg as usize),
        combinedVolume: <f32 as CMem>::rd(b, o + O::CymbalVoice_combinedVolume as usize),
        FaderVolumeFilter: <OnePoleFilter as CMem>::rd(b, o + O::CymbalVoice_FaderVolumeFilter as usize),
        pwm: <f32 as CMem>::rd(b, o + O::CymbalVoice_pwm as usize),
    } }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.osc.wr(b, o + O::CymbalVoice_osc as usize);
        self.modOsc.wr(b, o + O::CymbalVoice_modOsc as usize);
        self.modOsc2.wr(b, o + O::CymbalVoice_modOsc2 as usize);
        self.fmModAmount1.wr(b, o + O::CymbalVoice_fmModAmount1 as usize);
        self.fmModAmount2.wr(b, o + O::CymbalVoice_fmModAmount2 as usize);
        self.vol.wr(b, o + O::CymbalVoice_vol as usize);
        self.pan.wr(b, o + O::CymbalVoice_pan as usize);
        self.triggeredPanL.wr(b, o + O::CymbalVoice_triggeredPanL as usize);
        self.triggeredPanR.wr(b, o + O::CymbalVoice_triggeredPanR as usize);
        self.velo.wr(b, o + O::CymbalVoice_velo as usize);
        self.noiseSample.wr(b, o + O::CymbalVoice_noiseSample as usize);
        self.filter.wr(b, o + O::CymbalVoice_filter as usize);
        self.filterType.wr(b, o + O::CymbalVoice_filterType as usize);
        self.lfo.wr(b, o + O::CymbalVoice_lfo as usize);
        self.transGen.wr(b, o + O::CymbalVoice_transGen as usize);
        self.distortion.wr(b, o + O::CymbalVoice_distortion as usize);
        self.oscVolEg.wr(b, o + O::CymbalVoice_oscVolEg as usize);
        self.egValueOscVol.wr(b, o + O::CymbalVoice_egValueOscVol as usize);
        self.volumeMod.wr(b, o + O::CymbalVoice_volumeMod as usize);
        self.snapEg.wr(b, o + O::CymbalVoice_snapEg as usize);
        self.combinedVolume.wr(b, o + O::CymbalVoice_combinedVolume as usize);
        self.FaderVolumeFilter.wr(b, o + O::CymbalVoice_FaderVolumeFilter as usize);
        self.pwm.wr(b, o + O::CymbalVoice_pwm as usize);
    }
}

