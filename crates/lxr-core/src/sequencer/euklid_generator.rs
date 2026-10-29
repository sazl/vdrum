//! Port of `Core/Inc/Soundengine/Sequencer/EuklidGenerator.h`, `Core/Src/Soundengine/Sequencer/EuklidGenerator.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

/// Recursion limit of `euklid_calcRecursive` (port only, see there).
pub const EUKLID_MAX_ITERATIONS: u8 = 64;

// ---- functions without access to globals
/// C: `static inline uint8_t getObjSize(uint16_t group)`
///
/// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:53`. Reference build: inlined into its callers only. Group P3.
pub fn getObjSize(group: u16) -> u8 {
    let ret = 32 - (group as u32).leading_zeros();
    if ret == 0 {
        return 1;
    }
    ret as u8
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void euklid_init()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:36`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_patternBuffer`, `euklid_rotation`, `euklid_steps`.
    pub fn euklid_init(&mut self) {
        for i in 0..NUM_TRACKS as usize {
            self.euklid_length[i] = 16;
            self.euklid_steps[i] = 4;
            self.euklid_rotation[i] = 0;
        }
        self.euklid_patternBuffer.set_quadword(0);
    }

    /// C: `void euklid_calcRecursive(uint8_t length, uint8_t steps, uint8_t iteration, uint16_t group1, uint16_t group2)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:60`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_nextCnt1`, `euklid_originalLength`, `euklid_patternBuffer`.
    /// Calls: `sequencer::euklid_generator::getObjSize`.
    pub fn euklid_calcRecursive(&mut self, length: u8, steps: u8, iteration: u8, group1: u16, group2: u16) {
        // Port guard (not in the C source): for lengths above 17 (the menu allows 1..16, `set_parameter`
        // does not range-limit) some step counts never reach `objCnt2 <= 1` because the 16 bit groups
        // overflow, and the firmware recurses until the stack overflows. Terminating inputs need at most
        // 18 iterations (lengths 1..255), so stop there with the pattern buffer as it is (empty).
        // docs/ENGINE_CONTRACT.md section 9.
        if iteration > EUKLID_MAX_ITERATIONS {
            return;
        }
        // omit division by zero
        let remainder: u16 = if steps != 0 { (length % steps) as u16 } else { 0 };

        // In the first iteration we have to set the next count value to the number of steps
        if self.euklid_nextCnt1 == 0 {
            self.euklid_nextCnt1 = steps as u16;
        }

        // calculate the objects per group and their size
        let objectCnt1: i32 = self.euklid_nextCnt1 as i32;
        let objSize1: i32 = getObjSize(group1) as i32;
        let objSize2: i32 = getObjSize(group2) as i32;
        let objCnt2: i32 = (self.euklid_originalLength as i32 - objectCnt1 * objSize1) / objSize2;

        // the number of group 1 objects in the next iteration equals the count of objects in the smaller group
        self.euklid_nextCnt1 = (if objectCnt1 > objCnt2 { objCnt2 } else { objectCnt1 }) as u16;

        if objCnt2 <= 1 {
            let mut bufferPos: u8 = 0;
            let mut q = self.euklid_patternBuffer.quadword();
            // group1 objects
            for _ in 0..objectCnt1 {
                let mut j = objSize1 - 1;
                while j >= 0 {
                    if (group1 as u32) & (1u32 << j) != 0 {
                        q |= shl64(1, bufferPos as u32);
                    }
                    bufferPos = bufferPos.wrapping_add(1);
                    j -= 1;
                }
            }
            // group 2 objects
            for _ in 0..objCnt2 {
                let mut j = objSize2 - 1;
                while j >= 0 {
                    if (group2 as u32) & (1u32 << j) != 0 {
                        q |= shl64(1, bufferPos as u32);
                    }
                    bufferPos = bufferPos.wrapping_add(1);
                    j -= 1;
                }
            }
            self.euklid_patternBuffer.set_quadword(q);
            return;
        }

        // shift the group 1 value to make space for the new group2 part and save the original group content
        let originalGroup1 = group1;
        let mut group1 = ((group1 as u32) << objSize2) as u16;

        // paste the group 2 values to the free space at the end of group 1
        for i in 0..objSize2 {
            if (group2 as u32) & (1u32 << i) != 0 {
                group1 |= (1u32 << i) as u16;
            }
        }

        // if there are group 2 objects left group 2 stays the same
        // if no group 2 objects are left the rest of the old group1 will be the new group 2
        let group2 = if objCnt2 - objectCnt1 > 0 { group2 } else { originalGroup1 };

        // start the next iteration
        self.euklid_calcRecursive(steps, remainder as u8, iteration.wrapping_add(1), group1, group2);
    }

    /// C: `void euklid_generate(uint8_t trackNr, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:142`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_nextCnt1`, `euklid_originalLength`, `euklid_patternBuffer`, `euklid_rotation`, `euklid_steps`.
    /// Calls: `Lxr::euklid_calcRecursive`, `Lxr::euklid_rotatePattern`, `Lxr::euklid_transferPattern`.
    pub fn euklid_generate(&mut self, trackNr: u8, patternNr: u8) {
        let t = trackNr as usize;
        let length = self.euklid_length[t];
        let mut steps = self.euklid_steps[t];
        let rotation = self.euklid_rotation[t];

        if steps == 0 {
            steps += 1;
        }

        // reset the global values
        self.euklid_nextCnt1 = 0;
        self.euklid_originalLength = length as u16;

        // clear the buffer
        self.euklid_patternBuffer.set_quadword(0);

        // let's calculate the new pattern
        self.euklid_calcRecursive(length, steps, 1, 1, 0);
        // rotate resulting pattern
        self.euklid_rotatePattern(length, rotation);
        // and store it in the active track
        self.euklid_transferPattern(trackNr, patternNr);
    }

    /// C: `uint8_t euklid_getLength(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:169`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`.
    pub fn euklid_getLength(&mut self, trackNr: u8) -> u8 {
        self.euklid_length[trackNr as usize]
    }

    /// C: `uint8_t euklid_getSteps(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:174`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_steps`.
    pub fn euklid_getSteps(&mut self, trackNr: u8) -> u8 {
        self.euklid_steps[trackNr as usize]
    }

    /// C: `void euklid_setLength(uint8_t trackNr, uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:180`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`.
    pub fn euklid_setLength(&mut self, trackNr: u8, value: u8) {
        let value = if value == 0 { 1 } else { value };
        self.euklid_length[trackNr as usize] = value;
    }

    /// C: `void euklid_setSteps(uint8_t trackNr, uint8_t value, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:187`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_steps`.
    /// Calls: `Lxr::euklid_generate`.
    pub fn euklid_setSteps(&mut self, trackNr: u8, value: u8, patternNr: u8) {
        let mut value = if value == 0 { 1 } else { value };
        if value > self.euklid_length[trackNr as usize] {
            value = self.euklid_length[trackNr as usize];
        }
        self.euklid_steps[trackNr as usize] = value;
        self.euklid_generate(trackNr, patternNr);
    }

    /// C: `uint8_t euklid_getRotation(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:197`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_rotation`.
    pub fn euklid_getRotation(&mut self, trackNr: u8) -> u8 {
        self.euklid_rotation[trackNr as usize]
    }

    /// C: `void euklid_setRotation(uint8_t trackNr, uint8_t value, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:202`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_rotation`.
    /// Calls: `Lxr::euklid_generate`.
    pub fn euklid_setRotation(&mut self, trackNr: u8, value: u8, patternNr: u8) {
        let mut value = value;
        if value > self.euklid_length[trackNr as usize] {
            value = self.euklid_rotation[trackNr as usize];
        }
        self.euklid_rotation[trackNr as usize] = value;
        self.euklid_generate(trackNr, patternNr);
    }

    /// C: `void euklid_rotatePattern(uint8_t length, uint8_t amount)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:210`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_patternBuffer`.
    pub fn euklid_rotatePattern(&mut self, length: u8, amount: u8) {
        // `%` is `udiv` + `mls`: a zero length gives `amount % 0 == amount` on the Cortex-M7
        let rem = |a: u8, b: u8| if b == 0 { a } else { a % b };
        if amount == 0 || length == 1 || rem(amount, length) == 0 {
            return; // Nothing to be done
        }
        let amount = rem(amount, length);

        let q = self.euklid_patternBuffer.quadword();
        let mut q = shl64(q, amount as u32) | shr64(q, (length as i32 - amount as i32) as u32);
        q &= !shl64(0xffffffffffffffff, length as u32); // Remove "leftovers"
        self.euklid_patternBuffer.set_quadword(q);
    }

    /// C: `void euklid_transferPattern(uint8_t trackNr, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:221`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_patternBuffer`, `seq_patternSet`.
    pub fn euklid_transferPattern(&mut self, trackNr: u8, patternNr: u8) {
        let t = trackNr as usize;
        let p = patternNr as usize;
        let mut len = self.euklid_length[t];
        for i in 0..NUM_BARS as usize {
            self.seq_patternSet.seq_mainSteps[i][p][t] = self.euklid_patternBuffer.word[i];
        }
        // **PATROT - pattern end is now stored differently
        if len == 16 {
            len = 0;
        }
        self.seq_patternSet.length[p][t] = len;
    }
}

/// 64 bit `<<` by a variable amount as compiled for the Cortex-M7 (register shifts use the low byte of
/// the amount and give 0 for 32..255): 0 for amounts 64..255.
#[inline(always)]
fn shl64(v: u64, n: u32) -> u64 {
    let n = n & 0xff;
    if n >= 64 { 0 } else { v << n }
}

/// 64 bit `>>` (unsigned) by a variable amount, see `shl64`.
#[inline(always)]
fn shr64(v: u64, n: u32) -> u64 {
    let n = n & 0xff;
    if n >= 64 { 0 } else { v >> n }
}
