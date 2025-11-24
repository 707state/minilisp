// Translated from common.hpp

pub type Word = i64;
pub type Uword = u64;

pub const K_BITS_PER_BYTE: Word = 8;
pub const K_WORD_SIZE: Word = core::mem::size_of::<Word>() as Word;
pub const K_BITS_PER_WORD: Word = K_WORD_SIZE * K_BITS_PER_BYTE;

// tags
pub const K_INTEGER_TAG: u32 = 0x0;
pub const K_PAIR_TAG: u32 = 0x1;
pub const K_CHAR_TAG: u32 = 0x0F;
pub const K_BOOL_TAG: u32 = 0x1F;

pub const K_INTEGER_TAG_MASK: u32 = 0x3; // 0b11
// Immediate mask — for quick immediate checks (bool, char, ...)
pub const K_IMMEDIATE_TAG_MASK: u32 = 0x1F;
pub const K_BOOL_MASK: u32 = 0x1F;
// Heap pointer tagging
pub const K_HEAP_TAG_MASK: Uword = 0xF;
// Symbol tagging
pub const K_SYMBOL_TAG: u32 = 0x7;

pub const K_INTEGER_SHIFT: u32 = 2;
pub const K_INTEGER_BITS: usize = (K_BITS_PER_WORD as usize) - (K_INTEGER_SHIFT as usize);
pub const K_INTEGER_MAX: Word = ((1i128 << (K_INTEGER_BITS as i128 - 1)) - 1) as Word;
pub const K_INTEGER_MIN: Word = (-(1i128 << (K_INTEGER_BITS as i128 - 1))) as Word;

pub const K_CHAR_MASK: u32 = 0xFF;
pub const K_CHAR_SHIFT: u32 = 8;

pub const K_BOOL_SHIFT: u32 = 5;

pub const K_HEAP_PTR_MASK: Uword = !K_HEAP_TAG_MASK;
/*
 * This part of code is copied from Ruby YJIT
 */

// BitmaskImmediate: represents AArch64 logical immediate encoding (n, imms, immr)
// Layout in instruction (13 bits): [n:1 imms:6 immr:6] -> to_u32 returns that placed in low bits
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BitmaskImmediate {
    pub n: u8,    // 1 bit
    pub imms: u8, // 6 bits
    pub immr: u8, // 6 bits
}

impl BitmaskImmediate {
    #[inline]
    pub fn to_u32(self) -> u32 {
        ((self.n as u32) << 12) | ((self.immr as u32) << 6) | (self.imms as u32)
    }

    /// Try to encode a 64-bit value as an AArch64 logical-immediate (n, imms, immr).
    /// Returns Some(BitmaskImmediate) if encodable, otherwise None.
    pub fn try_from(value: u64) -> Option<BitmaskImmediate> {
        if value == 0 || value == u64::MAX {
            return None;
        }

        let safe_ctzll = |x: u64| -> u32 { if x == 0 { 64 } else { x.trailing_zeros() } };
        let safe_clzll = |x: u64| -> u32 { if x == 0 { 64 } else { x.leading_zeros() } };
        let rotate_right = |v: u64, r: u32| -> u64 { v.rotate_right(r & 0x3F) };

        // rotations = trailing_zeros(value & (value + 1))
        let tmp = value & (value.wrapping_add(1));
        let rotations = safe_ctzll(tmp);

        let normalized = rotate_right(value, rotations & 0x3F);

        let zeroes = safe_clzll(normalized) as u32;
        let ones = safe_ctzll(!normalized) as u32;

        let size = zeroes + ones; // pattern width

        if size == 0 {
            return None;
        }

        // check that rotating by 'size' yields same pattern (i.e. size is a valid element width)
        if rotate_right(value, size & 0x3F) != value {
            return None;
        }

        // n = (size >> 6) & 1
        let n = ((size >> 6) & 1) as u8;

        // imms = (((size << 1).wrapping_neg() | (ones - 1)) & 0x3F)
        let size_shifted = (size as u64) << 1;
        let neg_size_shifted = 0u64.wrapping_sub(size_shifted);
        let imms64 = (neg_size_shifted | ((ones as u64).wrapping_sub(1))) & 0x3F;
        let imms = imms64 as u8;

        // immr = ((rotations.wrapping_neg() & (size - 1)) & 0x3F)
        let neg_rot = 0u64.wrapping_sub(rotations as u64);
        let immr64 = (neg_rot & ((size as u64).wrapping_sub(1))) & 0x3F;
        let immr = immr64 as u8;

        Some(BitmaskImmediate { n, imms, immr })
    }
}

pub fn calc_bcond_imm19(pc_current: usize, target: usize) -> (isize, u32) {
    let offset_bytes = target as isize - pc_current as isize;
    let imm19: isize = offset_bytes / 2;
    let imm19_bin: u32 = if imm19 < 0 {
        ((1 << 20) as isize + imm19) as u32
    } else {
        imm19 as u32
    };
    println!("PC: 0x{:X}, Target: 0x{:X}", pc_current, target);
    println!("Offset bytes: {} -> imm19: {}", offset_bytes, imm19);
    println!("imm19 19-bit binary (hex) = 0x{:X}", imm19_bin);
    (imm19, imm19_bin)
}

pub fn calc_b_imm26(pc_current: usize, target: usize) -> (isize, u32) {
    let offset_bytes = target as isize - pc_current as isize;

    let imm26: isize = offset_bytes / 4;
    let mut imm26_bin: u32 = if imm26 < 0 {
        ((1 << 28) as isize + imm26) as u32
    } else {
        imm26 as u32
    };
    imm26_bin &= 0x3ffffff;
    println!("PC: 0x{:X}, Target: 0x{:X}", pc_current, target);
    println!("Offset bytes: {} -> imm26: {}", offset_bytes, imm26);
    println!("imm26 26-bit binary (hex) = 0x{:X}", imm26_bin);

    (imm26, imm26_bin)
}
// Register enum
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterX {
    X0 = 0,
    X1 = 1,
    X2 = 2,
    X3 = 3,
    X4 = 4,
    X5 = 5,
    X6 = 6,
    X7 = 7,
    X8 = 8,
    X9 = 9,
    X10 = 10,
    X11 = 11,
    X12 = 12,
    X13 = 13,
    X14 = 14,
    X15 = 15,
    X16 = 16,
    X17 = 17,
    X18 = 18,
    X19 = 19,
    X20 = 20,
    X21 = 21,
    X22 = 22,
    X23 = 23,
    X24 = 24,
    X25 = 25,
    X26 = 26,
    X27 = 27,
    X28 = 28,
    X29 = 29,
    X30 = 30,
    XZR = 31,
    MovRegister = 63,
}
// SP和XZR/WZR寄存器的值是一致的。
pub const SP: RegisterX = RegisterX::XZR;

// IVCond
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IVCond {
    NE = 0,
    EQ,
    CC,
    CS,
    PL,
    MI,
    VC,
    VS,
    LS,
    HI,
    LT,
    GE,
    LE,
    GT,
}

// Cond
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cond {
    EQ = 0,
    NE,
    CS,
    CC,
    MI,
    PL,
    VS,
    VC,
    HI,
    LS,
    GE,
    LT,
    GT,
    LE,
    AL,
    NV,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shift {
    LSL = 0,
    LSR = 1,
    ASR = 2,
    ROR = 3,
}
pub const RESERVED: Shift = Shift::ROR;

// Base instruction encoding
macro_rules! define_opcodes {
    ($($name:ident = $value:expr;)+) => {
        $(pub const $name: u32 = $value;)+
    };
}

define_opcodes! {
    MOVZ=0x52800000;
    ORR_SHIFTED_REG= 0x2a000000;
    RET=0xd65f0000;
    ADD_SHIFTED_REG=0x0b000000;
    ADD_IMM=0x11000000;
    SUB_SHIFTED_REG=0x4b000000;
    SUB_IMM=   0x51000000;
    SUBS_SHIFTED_REG=0x6b000000;
    SUBS_IMM=0x71000000;
    UBFM=0x53000000;
    ORR_IMM=0x32000000;
    AND_IMM=0x12000000;
    CSEL=0x1a800000;
    CSET=0x1a9f07e0;
    STP= 0x2800_0000;
    LDP=0x2840_0000;
    STR_IMM=0xb800_0000;
    LDR_IMM=0xb840_0000;
    B_COND=0x54000000;
    BL=0x94000000;
    SVC=0xd4000001;
}

pub const ENTRY_POINT: &str = "entry";

pub fn is_initial_symbol_char(c: char) -> bool {
    c.is_ascii_alphabetic() || "+-*/!?<>=_".contains(c)
}

pub fn is_subsequent_symbol_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "+-*/!?<>=_".contains(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bitmask_imm_encoding_test() {
        #[derive(Debug)]
        struct Test {
            v: u64,
        }

        let tests = [
            Test { v: 1 },
            Test { v: 2 },
            Test { v: 3 },
            Test { v: 4 },
            Test {
                v: 0x5555555555555555,
            },
            Test {
                v: 0xF0F0F0F0F0F0F0F0,
            },
            Test {
                v: 0x0FF00FF00FF00FF0,
            },
            Test { v: 0xFF },
            Test {
                v: 0xFFFFFFFFFFFFFFFE,
            },
        ];

        for t in &tests {
            let result = BitmaskImmediate::try_from(t.v);

            match result {
                Some(b) => {
                    println!(
                        "value=0x{:016x} -> n={} imms=0x{:x} immr=0x{:x} enc=0x{:08x}",
                        t.v,
                        b.n,
                        b.imms,
                        b.immr,
                        b.to_u32()
                    );
                }
                None => {
                    println!("value=0x{:016x} -> NOT encodable", t.v);
                }
            }

            // 检查必须可编码（和原来的 CHECK_EQ(ok, true) 对应）
            assert!(result.is_some(), "Value 0x{:016x} is not encodable", t.v);
        }
    }
    #[test]
    fn calc_bcond_imm19_test() {
        let (imm, imm19_bin) = calc_bcond_imm19(0x18, 0x88);
        assert_eq!(imm, 56);
        assert_eq!(imm19_bin, 0x38);
        let (imm, imm19_bin) = calc_bcond_imm19(0x2c, 0x08);
        assert_eq!(imm, -18);
        assert_eq!(imm19_bin, 0xfffee);
    }
    #[test]
    fn calc_b_imm26_test() {
        let (imm, imm26_bin) = calc_b_imm26(0x70, 0x08);
        assert_eq!(imm, -26);
        assert_eq!(imm26_bin, 0x3ffffe6);
        let (imm, imm26_bin) = calc_b_imm26(0x74, 0x78);
        assert_eq!(imm, 1);
        assert_eq!(imm26_bin, 0x1);
    }
    #[test]
    fn calc_bl_imm26_test() {
        let (_, imm26_bin) = calc_bcond_imm19(0x2c, 0);
        assert_eq!(imm26_bin, 0xfffea);
    }
}
