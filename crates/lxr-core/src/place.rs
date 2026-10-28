//! Struct pointer parameters in safe Rust.
//!
//! C functions that take a struct pointer are ported with a Rust reference (`&mut T` / `&T`). When the function
//! also uses globals it is a method on `Lxr`, and a caller whose object lives inside `Lxr` (for example
//! `&voiceArray[0].osc`) cannot borrow it mutably while also passing `&mut self`. Two cases:
//!
//! 1. **The callee cannot reach the pointee through globals** (it only uses other globals, the RNG or tables).
//!    The signature is `&mut T`; the caller copies the object out and back in (all C structs are `Copy`):
//!    ```ignore
//!    let mut osc = self.voiceArray[v].osc;
//!    self.calcNoiseBlock(&mut osc, &mut buf, size, gain);
//!    self.voiceArray[v].osc = osc;
//!    ```
//!    This is exactly equivalent to the C code because nothing else can observe the object during the call.
//!    (Or split the borrow when the callee's globals are known and disjoint.)
//!
//! 2. **The callee may reach the pointee through globals** (e.g. `lfo_dispatchNextValue(Lfo*)` ends in
//!    `midiParser_ccHandler`, which can write any voice parameter, including fields of that LFO; or
//!    `modNode_setDestination`, which resets every modulation node through globals). Copy-out/copy-in would
//!    lose those writes. These parameters are *place accessors*:
//!    `impl Fn(&mut Lxr) -> &mut T + Copy`, a closure that returns the object inside `Lxr`. The callee
//!    re-borrows through it at each access, so it always sees the current value, exactly like the C pointer:
//!    ```ignore
//!    pub fn lfo_dispatchNextValue(&mut self, lfo: impl Fn(&mut Lxr) -> &mut Lfo + Copy) {
//!        let val = self.lfo_calc(lfo(self));                  // lfo_calc takes &mut Lfo... see note below
//!        self.modNode_updateValue(move |s| &mut lfo(s).modTarget, val);
//!    }
//!    // caller:
//!    self.lfo_dispatchNextValue(|s| &mut s.voiceArray[0].lfo);
//!    ```
//!    (When such a callee calls a case-1 method with the same object it copies out/in around that call.)
//!
//! The generator chose case 2 for exactly the struct pointer parameters of methods whose transitive set of
//! globals contains a global of a type that contains the pointee type; see `docs/PORTING_STATUS.md`.
//! A free function (no globals) always takes `&mut T`.

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    fn outer(lxr: &mut Lxr, lfo: impl Fn(&mut Lxr) -> &mut Lfo + Copy) -> f32 {
        lfo(lxr).freq = 2.0;
        inner(lxr, move |s| &mut lfo(s).modTarget);
        lfo(lxr).modTarget.amount
    }

    fn inner(lxr: &mut Lxr, vm: impl Fn(&mut Lxr) -> &mut ModulationNode + Copy) {
        // writes through a global path and through the place: both are visible
        lxr.voiceArray[0].lfo.modTarget.lastVal = 3.0;
        vm(lxr).amount = 1.5;
    }

    #[test]
    fn place_accessors_compose_and_alias_like_c_pointers() {
        let mut lxr = Lxr::new_zeroed();
        let a = outer(&mut lxr, |s| &mut s.voiceArray[0].lfo);
        assert_eq!(a, 1.5);
        assert_eq!(lxr.voiceArray[0].lfo.freq, 2.0);
        assert_eq!(lxr.voiceArray[0].lfo.modTarget.lastVal, 3.0);
    }
}
