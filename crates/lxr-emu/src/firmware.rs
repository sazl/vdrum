//! Low-level ARM Cortex-M7 emulation of LXR firmware code with Unicorn.
//!
//! `Firmware` loads either an ELF (with symbols, e.g. the firmware built from
//! source with Arm GNU Toolchain 11.3) or the raw application binary from an
//! SD-card update image plus a symbol map, and lets the caller run individual
//! firmware functions with the AAPCS-VFP calling convention.

use anyhow::{anyhow, bail, Context, Result};
use std::collections::HashMap;
use unicorn_engine::{ArmCpuModel, Arch, Mode, Prot, RegisterARM, Unicorn};

pub const FLASH_BASE: u64 = 0x0800_0000;
pub const FLASH_SIZE: u64 = 0x0010_0000;
pub const APP_BASE: u64 = 0x0800_8000;
pub const ITCM_BASE: u64 = 0x0000_0000;
pub const ITCM_SIZE: u64 = 0x0000_4000;
pub const DTCM_BASE: u64 = 0x2000_0000;
pub const SRAM_END: u64 = 0x2008_0000;
/// Scratch RAM used by the harness for buffers it passes to firmware functions.
pub const SCRATCH_BASE: u64 = 0x6000_0000;
pub const SCRATCH_SIZE: u64 = 0x0040_0000;
/// Stack used while calling functions (top of a dedicated region, not firmware RAM).
pub const STACK_BASE: u64 = 0x6100_0000;
pub const STACK_SIZE: u64 = 0x0010_0000;
/// Return address trap: functions return here, which stops emulation.
pub const RETURN_TRAP: u64 = 0x6200_0000;

/// Bits of the FPSCR.
pub const FPSCR_FZ: u32 = 1 << 24;
pub const FPSCR_DN: u32 = 1 << 25;
pub const FPSCR_AHP: u32 = 1 << 26;

#[derive(Debug, Clone, Copy)]
pub struct Symbol {
    pub addr: u32,
    pub size: u32,
}

/// Hook state stored inside the Unicorn instance.
#[derive(Default)]
pub struct HookState {
    /// Unmapped/peripheral accesses: (pc, addr, size, is_write, value).
    pub mmio_log: Vec<(u64, u64, usize, bool, u64)>,
    pub log_mmio: bool,
    /// Values returned for peripheral reads (address -> value). Unlisted reads return 0.
    pub mmio_values: HashMap<u64, u64>,
    /// Function intercepts: address (without thumb bit) -> index into `intercepts`.
    pub intercept_addrs: HashMap<u64, usize>,
}

pub struct Firmware {
    pub uc: Unicorn<'static, HookState>,
    pub symbols: HashMap<String, Symbol>,
    scratch_next: u64,
    /// FPSCR value used for calls (default: FZ set, like the audio interrupt context).
    pub fpscr: u32,
    /// Max instructions per call (0 = unlimited).
    pub insn_limit: usize,
}

impl Firmware {
    fn new_machine() -> Result<Unicorn<'static, HookState>> {
        let mut uc = Unicorn::new_with_data(Arch::ARM, Mode::THUMB | Mode::MCLASS, HookState::default())
            .map_err(|e| anyhow!("unicorn init: {e:?}"))?;
        uc.ctl_set_cpu_model(ArmCpuModel::CORTEX_M7 as i32)
            .map_err(|e| anyhow!("cpu model: {e:?}"))?;
        let rwx = Prot::ALL;
        for (base, size) in [
            (ITCM_BASE, ITCM_SIZE),
            (FLASH_BASE, FLASH_SIZE),
            (DTCM_BASE, SRAM_END - DTCM_BASE),
            (SCRATCH_BASE, SCRATCH_SIZE),
            (STACK_BASE, STACK_SIZE),
            (RETURN_TRAP, 0x1000),
        ] {
            uc.mem_map(base, size, rwx).map_err(|e| anyhow!("map {base:#x}: {e:?}"))?;
        }
        // Peripherals (APB/AHB) and the system control space as MMIO.
        for (base, size) in [(0x4000_0000u64, 0x2000_0000u64), (0xE000_0000, 0x0010_0000)] {
            uc.mmio_map(
                base,
                size,
                Some(move |uc: &mut Unicorn<HookState>, off: u64, sz: usize| -> u64 {
                    let addr = base + off;
                    let v = *uc.get_data().mmio_values.get(&addr).unwrap_or(&0);
                    if uc.get_data().log_mmio {
                        let pc = uc.reg_read(RegisterARM::PC).unwrap_or(0);
                        uc.get_data_mut().mmio_log.push((pc, addr, sz, false, v));
                    }
                    v
                }),
                Some(move |uc: &mut Unicorn<HookState>, off: u64, sz: usize, val: u64| {
                    let addr = base + off;
                    if uc.get_data().log_mmio {
                        let pc = uc.reg_read(RegisterARM::PC).unwrap_or(0);
                        uc.get_data_mut().mmio_log.push((pc, addr, sz, true, val));
                    }
                }),
            )
            .map_err(|e| anyhow!("mmio map {base:#x}: {e:?}"))?;
        }
        Ok(uc)
    }

    /// Load an ELF file (VMA addresses, so ITCM code and initialised data are placed directly).
    pub fn from_elf(data: &[u8]) -> Result<Self> {
        use object::{Object, ObjectSegment, ObjectSymbol, SymbolKind};
        let obj = object::File::parse(data).context("parse ELF")?;
        let mut uc = Self::new_machine()?;
        for seg in obj.segments() {
            let bytes = seg.data().context("segment data")?;
            if bytes.is_empty() {
                continue;
            }
            uc.mem_write(seg.address(), bytes)
                .map_err(|e| anyhow!("write segment {:#x}: {e:?}", seg.address()))?;
        }
        // Also place the LMA copy (flash image) so code reading flash constants works.
        if let Ok(object::File::Elf32(elf)) = object::File::parse(data) {
            use object::read::elf::ProgramHeader;
            let endian = elf.endian();
            for ph in elf.elf_program_headers() {
                if ph.p_type(endian) != object::elf::PT_LOAD {
                    continue;
                }
                let paddr = ph.p_paddr(endian) as u64;
                let vaddr = ph.p_vaddr(endian) as u64;
                if paddr != vaddr && (FLASH_BASE..FLASH_BASE + FLASH_SIZE).contains(&paddr) {
                    if let Ok(bytes) = ph.data(endian, data) {
                        let _ = uc.mem_write(paddr, bytes);
                    }
                }
            }
        }
        let mut symbols = HashMap::new();
        for s in obj.symbols() {
            if let Ok(name) = s.name() {
                if name.is_empty() || matches!(s.kind(), SymbolKind::Section | SymbolKind::File) {
                    continue;
                }
                let addr = if s.kind() == SymbolKind::Text { s.address() & !1 } else { s.address() };
                symbols.entry(name.to_string()).or_insert(Symbol { addr: addr as u32, size: s.size() as u32 });
            }
        }
        Ok(Self::finish(uc, symbols))
    }

    /// Load a raw application binary (as stored in an SD-card update image after the
    /// 16 byte header) at 0x08008000 with an external symbol map. Data/ITCM
    /// initialisation is not performed here; see `crate::engine` for running the
    /// firmware's own startup code.
    pub fn from_bin(bin: &[u8], symbols: HashMap<String, Symbol>) -> Result<Self> {
        let mut uc = Self::new_machine()?;
        uc.mem_write(APP_BASE, bin).map_err(|e| anyhow!("write bin: {e:?}"))?;
        Ok(Self::finish(uc, symbols))
    }

    fn finish(uc: Unicorn<'static, HookState>, symbols: HashMap<String, Symbol>) -> Self {
        Firmware { uc, symbols, scratch_next: SCRATCH_BASE, fpscr: FPSCR_FZ, insn_limit: 0 }
    }

    pub fn sym(&self, name: &str) -> Result<Symbol> {
        self.symbols.get(name).copied().ok_or_else(|| anyhow!("unknown symbol {name}"))
    }

    pub fn addr(&self, name: &str) -> Result<u32> {
        Ok(self.sym(name)?.addr)
    }

    /// Allocate zeroed scratch memory (8-byte aligned).
    pub fn alloc(&mut self, size: usize) -> Result<u32> {
        let a = (self.scratch_next + 7) & !7;
        if a + size as u64 > SCRATCH_BASE + SCRATCH_SIZE {
            bail!("scratch memory exhausted");
        }
        self.scratch_next = a + size as u64;
        self.write(a as u32, &vec![0u8; size])?;
        Ok(a as u32)
    }

    pub fn reset_scratch(&mut self) {
        self.scratch_next = SCRATCH_BASE;
    }

    pub fn read(&self, addr: u32, len: usize) -> Result<Vec<u8>> {
        self.uc.mem_read_as_vec(addr as u64, len).map_err(|e| anyhow!("read {addr:#x}+{len}: {e:?}"))
    }

    pub fn write(&mut self, addr: u32, bytes: &[u8]) -> Result<()> {
        self.uc.mem_write(addr as u64, bytes).map_err(|e| anyhow!("write {addr:#x}: {e:?}"))
    }

    pub fn read_u8(&self, addr: u32) -> Result<u8> {
        Ok(self.read(addr, 1)?[0])
    }
    pub fn read_u16(&self, addr: u32) -> Result<u16> {
        let b = self.read(addr, 2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn read_u32(&self, addr: u32) -> Result<u32> {
        let b = self.read(addr, 4)?;
        Ok(u32::from_le_bytes(b.try_into().unwrap()))
    }
    pub fn read_f32(&self, addr: u32) -> Result<f32> {
        Ok(f32::from_bits(self.read_u32(addr)?))
    }
    pub fn read_f64(&self, addr: u32) -> Result<f64> {
        let b = self.read(addr, 8)?;
        Ok(f64::from_le_bytes(b.try_into().unwrap()))
    }
    pub fn read_i16s(&self, addr: u32, n: usize) -> Result<Vec<i16>> {
        let b = self.read(addr, n * 2)?;
        Ok(b.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect())
    }
    pub fn write_u8(&mut self, addr: u32, v: u8) -> Result<()> {
        self.write(addr, &[v])
    }
    pub fn write_u16(&mut self, addr: u32, v: u16) -> Result<()> {
        self.write(addr, &v.to_le_bytes())
    }
    pub fn write_u32(&mut self, addr: u32, v: u32) -> Result<()> {
        self.write(addr, &v.to_le_bytes())
    }
    pub fn write_f32(&mut self, addr: u32, v: f32) -> Result<()> {
        self.write(addr, &v.to_bits().to_le_bytes())
    }
    pub fn write_i16s(&mut self, addr: u32, v: &[i16]) -> Result<()> {
        let b: Vec<u8> = v.iter().flat_map(|x| x.to_le_bytes()).collect();
        self.write(addr, &b)
    }

    /// Replace a firmware function with a host closure. The closure runs when the
    /// function is entered and must set the return registers; the harness then
    /// returns to the caller (LR).
    pub fn intercept(&mut self, addr: u32, mut f: impl for<'a> FnMut(&mut Unicorn<'a, HookState>) + 'static) -> Result<()> {
        let a = (addr & !1) as u64;
        self.uc
            .add_code_hook(a, a, move |uc, _addr, _size| {
                f(uc);
                let lr = uc.reg_read(RegisterARM::LR).unwrap();
                uc.reg_write(RegisterARM::PC, lr | 1).unwrap();
            })
            .map_err(|e| anyhow!("hook: {e:?}"))?;
        Ok(())
    }

    /// Call a function at `addr` with integer/pointer args (r0-r3, then stack) and
    /// float args (s0-s15), AAPCS-VFP. Returns (r0, s0).
    pub fn call_addr(&mut self, addr: u32, ints: &[u32], floats: &[f32]) -> Result<(u32, f32)> {
        if floats.len() > 16 {
            bail!("too many float args");
        }
        let regs = [RegisterARM::R0, RegisterARM::R1, RegisterARM::R2, RegisterARM::R3];
        let mut sp = STACK_BASE + STACK_SIZE - 0x100;
        let stack_args = if ints.len() > 4 { &ints[4..] } else { &[][..] };
        sp -= (stack_args.len() as u64 * 4 + 7) & !7;
        for (i, v) in stack_args.iter().enumerate() {
            self.write_u32((sp + 4 * i as u64) as u32, *v)?;
        }
        for (i, v) in ints.iter().take(4).enumerate() {
            self.uc.reg_write(regs[i], *v as u64).map_err(|e| anyhow!("{e:?}"))?;
        }
        for (i, v) in floats.iter().enumerate() {
            self.uc
                .reg_write(RegisterARM::S0 as i32 + i as i32, v.to_bits() as u64)
                .map_err(|e| anyhow!("{e:?}"))?;
        }
        self.uc.reg_write(RegisterARM::SP, sp).map_err(|e| anyhow!("{e:?}"))?;
        self.uc.reg_write(RegisterARM::LR, RETURN_TRAP | 1).map_err(|e| anyhow!("{e:?}"))?;
        self.uc.reg_write(RegisterARM::FPSCR, self.fpscr as u64).map_err(|e| anyhow!("{e:?}"))?;
        self.uc
            .emu_start((addr | 1) as u64, RETURN_TRAP, 0, self.insn_limit)
            .map_err(|e| {
                let pc = self.uc.reg_read(RegisterARM::PC).unwrap_or(0);
                anyhow!("emulation of {addr:#x} failed at pc={pc:#x}: {e:?}")
            })?;
        let pc = self.uc.reg_read(RegisterARM::PC).unwrap_or(0);
        if pc & !1 != RETURN_TRAP {
            bail!("call {addr:#x} stopped at pc={pc:#x} (instruction limit?)");
        }
        let r0 = self.uc.reg_read(RegisterARM::R0).map_err(|e| anyhow!("{e:?}"))? as u32;
        let s0 = self.uc.reg_read(RegisterARM::S0).map_err(|e| anyhow!("{e:?}"))? as u32;
        Ok((r0, f32::from_bits(s0)))
    }

    /// Call a function by symbol name.
    pub fn call(&mut self, name: &str, ints: &[u32], floats: &[f32]) -> Result<(u32, f32)> {
        let a = self.addr(name)?;
        self.call_addr(a, ints, floats)
    }
}
