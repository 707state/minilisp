// Translated from common.hpp

pub type word = i64;
pub type uword = u64;

pub const K_BITS_PER_BYTE: usize = 8;
pub const K_WORD_SIZE: usize = core::mem::size_of::<word>();
pub const K_BITS_PER_WORD: usize = K_WORD_SIZE * K_BITS_PER_BYTE;

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
pub const K_HEAP_TAG_MASK: uword = 0xF;
// Symbol tagging
pub const K_SYMBOL_TAG: u32 = 0x7;

pub const K_INTEGER_SHIFT: u32 = 2;
pub const K_INTEGER_BITS: usize = (K_BITS_PER_WORD) - (K_INTEGER_SHIFT as usize);
pub const K_INTEGER_MAX: word = ((1i128 << (K_INTEGER_BITS as i128 - 1)) - 1) as word;
pub const K_INTEGER_MIN: word = (-(1i128 << (K_INTEGER_BITS as i128 - 1))) as word;

pub const K_CHAR_MASK: u32 = 0xFF;
pub const K_CHAR_SHIFT: u32 = 8;

pub const K_BOOL_SHIFT: u32 = 5;

pub const K_HEAP_PTR_MASK: uword = !K_HEAP_TAG_MASK;

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
}

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
}
