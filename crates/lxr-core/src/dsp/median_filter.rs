//! Port of `Core/Inc/Soundengine/DSPAudio/medianFilter.h`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define NIL 0` (Core/Inc/Soundengine/DSPAudio/medianFilter.h:20)
pub const NIL: i32 = 0;
/// `#define MEDIAN_FILTER_SIZE (7)` (Core/Inc/Soundengine/DSPAudio/medianFilter.h:21)
pub const MEDIAN_FILTER_SIZE: i32 = 7;

// ---- types
/// C: `struct pair` (Core/Inc/Soundengine/DSPAudio/medianFilter.h:25), 8 bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct pair {
    /// `struct pair *point; / * Pointers forming list linked in sorted order * /`
    ///
    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.
    pub point: CPtr,
    /// `uint32_t value; / * Values to sort * /`
    pub value: u32,
}
impl pair {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { point: 0, value: 0 };
}
impl Default for pair {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `uint32_t medianFilter(uint32_t datum)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/medianFilter.h:23`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn medianFilter(datum: u32) -> u32 {
    // The C function keeps its sorted linked list in function-local statics. It is not linked in the
    // firmware (removed by --gc-sections), so `Lxr` has no storage for those statics; use
    // `medianFilter_state` with an explicit `MedianFilterState` instead.
    let _ = datum;
    unimplemented!("medianFilter: not linked in the firmware; use medianFilter_state with a MedianFilterState")
}

/// Index of `small` (the chain stopper) in `MedianFilterState::nodes`.
const MF_SMALL: usize = MEDIAN_FILTER_SIZE as usize;
/// Index of `big` (the head of the list) in `MedianFilterState::nodes`.
const MF_BIG: usize = MEDIAN_FILTER_SIZE as usize + 1;
/// "NULL" link.
const MF_NULL: usize = usize::MAX;

/// The function-local statics of `medianFilter`: `buffer[MEDIAN_FILTER_SIZE]`, `small`, `big` (as indices
/// `0..7`, `7`, `8`; links are indices, `usize::MAX` = NULL) and `datpoint`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MedianFilterState {
    /// `(point, value)` of `buffer[0..7]`, `small`, `big`.
    pub nodes: [(usize, u32); MEDIAN_FILTER_SIZE as usize + 2],
    /// `datpoint` as an index into `buffer`.
    pub datpoint: usize,
}

impl MedianFilterState {
    /// The C static initialisation: `buffer` zero, `datpoint = buffer`, `small = {NULL, NIL}`, `big = {&small, 0}`.
    pub const INIT: Self = {
        let mut nodes = [(MF_NULL, 0u32); MEDIAN_FILTER_SIZE as usize + 2];
        nodes[MF_SMALL] = (MF_NULL, NIL as u32);
        nodes[MF_BIG] = (MF_SMALL, 0);
        Self { nodes, datpoint: 0 }
    };
}

impl Default for MedianFilterState {
    fn default() -> Self {
        Self::INIT
    }
}

/// `medianFilter` with its statics passed explicitly (faithful port of the C, not in the firmware build).
pub fn medianFilter_state(st: &mut MedianFilterState, datum: u32) -> u32 {
    let mut datum = datum;
    if datum == NIL as u32 {
        datum = NIL as u32 + 1; /* No stoppers allowed. */
    }

    st.datpoint += 1;
    if st.datpoint >= MEDIAN_FILTER_SIZE as usize {
        st.datpoint = 0; /* Increment and wrap data in pointer.*/
    }
    let datpoint = st.datpoint;
    let n = &mut st.nodes;

    n[datpoint].1 = datum; /* Copy in new datum */
    let successor = n[datpoint].0; /* Save pointer to old value's successor */
    let mut median = MF_BIG; /* Median initially to first in chain */
    let mut scan = MF_BIG; /* Points to pointer to first (largest) datum in chain */

    /* Handle chain-out of first item in chain as special case */
    if n[scan].0 == datpoint {
        n[scan].0 = successor;
    }
    let mut scanold = scan; /* Save this pointer and   */
    scan = n[scan].0; /* step down chain */

    /* Loop through the chain, normal loop exit via break. */
    for _ in 0..MEDIAN_FILTER_SIZE {
        /* Handle odd-numbered item in chain  */
        if n[scan].0 == datpoint {
            n[scan].0 = successor; /* Chain out the old datum.*/
        }

        if n[scan].1 < datum {
            /* If datum is larger than scanned value,*/
            n[datpoint].0 = n[scanold].0; /* Chain it in here.  */
            n[scanold].0 = datpoint; /* Mark it chained in. */
            datum = NIL as u32;
        }

        /* Step median pointer down chain after doing odd-numbered element */
        median = n[median].0; /* Step median pointer.  */
        if scan == MF_SMALL {
            break; /* Break at end of chain  */
        }
        scanold = scan; /* Save this pointer and   */
        scan = n[scan].0; /* step down chain */

        /* Handle even-numbered item in chain.  */
        if n[scan].0 == datpoint {
            n[scan].0 = successor;
        }

        if n[scan].1 < datum {
            n[datpoint].0 = n[scanold].0;
            n[scanold].0 = datpoint;
            datum = NIL as u32;
        }

        if scan == MF_SMALL {
            break;
        }

        scanold = scan;
        scan = n[scan].0;
    }
    n[median].1
}
