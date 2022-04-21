pub use bitmath_macros::bitslice;

use std::fmt::{Display, Formatter, LowerHex};
use std::ops::{Index, IndexMut, Range, RangeInclusive};


fn bit(b: bool) -> usize { if b { 1 } else { 0 } }


#[derive(Default, Debug, Clone, Copy)]
struct SignedHex(i32);

impl LowerHex for SignedHex {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let prefix = if f.alternate() { "0x" } else { "" };
        let bare_hex = format!("{:x}", self.0.abs());
        f.pad_integral(self.0 >= 0, prefix, &bare_hex)
    }
}


#[derive(Debug, Copy, Clone)]
pub enum BitsError {
    ByteOutOfRange,
    InvalidInputString,
    /// (expected, found)
    BitWidthMismatch(usize, usize),
    BitIndexOutOfRange,
}


#[derive(Debug, Copy, Clone)]
pub struct Bits<const SIZE: usize>(pub [bool; SIZE]);

impl<const SIZE: usize> Bits<SIZE> {
    pub fn new() -> Self {
        Bits([false; SIZE])
    }

    pub fn from_signed(x: i32) -> Self {
        let mut bits = Vec::new();
        if SIZE <= 32 {
            for i in 0..SIZE {
                bits.push(((x >> (SIZE-1 - i)) & 1) != 0);
            }
        }
        else {
            let extend_bits = SIZE - 32;
            for _ in 0..extend_bits {
                bits.push(if x < 0 { true } else { false });
            }
            for i in 0..32 {
                bits.push(((x >> (31 - i)) & 1) != 0);
            }
        }
        Bits(bits.try_into().unwrap())
    }

    pub fn from_unsigned(x: u32) -> Self {
        let mut bits = Vec::new();
        if SIZE <= 32 {
            for i in 0..SIZE {
                bits.push(((x >> (SIZE-1 - i)) & 1) != 0);
            }
        }
        else {
            let extend_bits = SIZE - 32;
            for _ in 0..extend_bits {
                bits.push(false);
            }
            for i in 0..32 {
                bits.push(((x >> (31 - i)) & 1) != 0);
            }
        }
        Bits(bits.try_into().unwrap())
    }

    pub fn from_slice(slice: &[bool]) -> Result<Self, BitsError> {
        if slice.len() != SIZE {
            return Err(BitsError::BitWidthMismatch(SIZE, slice.len()));
        }
        let mut copied = [false; SIZE];
        for i in 0..SIZE {
            copied[i] = slice[i];
        }
        Ok(Bits(copied))
    }

    #[doc(hidden)]
    /// used internally for bitslice!() since #:# bit indexing works backwards
    pub fn from_reverse_index(slice: &[bool], hi: usize, lo: usize) -> Result<Self, BitsError> {
        let high = lo.max(hi);
        let low = lo.min(hi);
        let width = high - low + 1;
        if slice.len() - high < 1 { // we already know low is >=0 because usize
            return Err(BitsError::BitIndexOutOfRange);
        }
        if width != SIZE {
            return Err(BitsError::BitWidthMismatch(SIZE, width));
        }
        let mut copied = [false; SIZE];
        for i in 0..SIZE {
            copied[i] = slice[slice.len() - high - 1 + i];
        }
        Ok(Bits(copied))
    }

    pub const fn size(&self) -> usize { SIZE }

    pub fn get_bit(&self, n: usize) -> Option<&bool> { self.0.get(n) }

    pub fn get_bit_mut(&mut self, n: usize) -> Option<&mut bool> { self.0.get_mut(n) }

    pub fn unsigned_value(&self) -> u32 {
        let mut result = 0u32;
        let start_idx = (SIZE as i32 - 32).max(0) as usize;
        for i in 0..self.size().min(32) {
            result <<= 1;
            result |= bit(self.0[start_idx+i]) as u32;
        }
        result
    }

    pub fn signed_value(&self) -> i32 {
        let mut result = 0u32;
        let start_idx = (SIZE as i32 - 32).max(0) as usize;
        let extend_bits = (32 - SIZE as i32).max(0) as usize;
        let is_negative = self.0[0] == true;
        for _ in 0..extend_bits {
            result <<= 1;
            result |= if is_negative { 1 } else { 0 };
        }
        for i in 0..SIZE.min(32) {
            result <<= 1;
            result |= *self.get_bit(start_idx+i).unwrap() as u32;
        }
        unsafe { std::mem::transmute(result) }
    }

    pub fn unsigned_add(&self, other: Self) -> (Self, bool) {
        let a = self.unsigned_value() as u64;
        let b = other.unsigned_value() as u64;
        let sum = a + b;
        let mut mask = 1u64;
        for _ in 0..SIZE-1 {
            mask <<= 1;
            mask |= 1;
        }
        let result = (sum & mask) as u32;
        (Bits::from_unsigned(result), (sum >> SIZE) > 0)
    }

    pub fn signed_add(&self, other: Self) -> (Self, bool) {
        let a = self.signed_value() as i64;
        let b = other.signed_value() as i64;
        let sum = a + b;
        let mut mask = 1i64;
        for _ in 0..SIZE-1 {
            mask <<= 1;
            mask |= 1;
        }
        let result = (sum & mask) as i32;
        let overflow = sum < -(2u64.pow(SIZE as u32 - 1) as i64) || sum > (2u64.pow(SIZE as u32 - 1) - 1) as i64;
        (Bits::from_signed(result), overflow)
    }

    pub fn rotate_right(&self, n: usize) -> Self {
        let n = n % SIZE;
        let mut result = Bits::new();
        for i in 0..SIZE {
            result.0[(i+n)%SIZE] = self.0[i];
        }
        result
    }

    pub fn rotate_left(&self, n: usize) -> Self {
        let n = n % SIZE;
        let mut result = Bits::new();
        for i in 0..SIZE {
            // conversion to signed to prevent underflow
            result.0[(i+SIZE-n) % SIZE] = self.0[i];
        }
        result
    }

    pub fn bits_string(&self, pretty: bool) -> String {
        let mut bitstr: String = self.0.map(|b| if b { "1".into() } else { "0".into() })
            .into_iter()
            .collect::<Vec<String>>()
            .join("");
        if pretty {
            for i in 1..SIZE {
                let idx = SIZE - i;
                if idx % 4 == 0 {
                    bitstr.insert(idx, ' ');
                }
            }
        }
        bitstr
    }

    pub fn pretty_uhex_string(&self) -> String {
        let digits = (SIZE as f32 / 4.0).ceil() as usize;
        let hex_padding = digits % 2;
        let mut uhex_chars = vec![' '; hex_padding];
        uhex_chars.extend(format!("{:01$x}", self.unsigned_value(), digits)
            .chars()
            .into_iter());
        uhex_chars
            .chunks(2)
            // remove padding after chunks separated
            .map(|chunk| chunk.iter().map(|c| String::from(*c)).collect::<Vec<_>>().join("").replace(" ",""))
            .collect::<Vec<_>>()
            .join(" ")
    }
}


impl<const N: usize> Default for Bits<N> {
    fn default() -> Self {
        Bits([false; N])
    }
}


impl<const N: usize> Display for Bits<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bits<{0}>{{ {1} | dec {2}/{3} | hex {4:#x}/{5:#x} }}",
               N,
               self.bits_string(true),
               self.unsigned_value(),
               self.signed_value(),
               self.unsigned_value(),
               SignedHex(self.signed_value()))
    }
}


impl<const N: usize> TryFrom<&str> for Bits<N> {
    type Error = BitsError;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let input = input.replace(" ","");
        if input.len() > N || input.chars().any(|c| c != '0' && c != '1') {
            return Err(BitsError::InvalidInputString);
        }
        let mut result = Bits([false; N]);
        for i in 0..N {
            let c = input.chars().nth(i).unwrap();
            result.0[i] = if c == '0' { false } else { true };
        }
        Ok(result)
    }
}


impl<const N: usize> Index<usize> for Bits<N> {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        self.get_bit(index).unwrap()
    }
}


impl<const N: usize> IndexMut<usize> for Bits<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_bit_mut(index).unwrap()
    }
}


impl <const N: usize> Index<Range<usize>>for Bits<N> {
    type Output = [bool];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.0[index]
    }
}


impl <const N: usize> Index<RangeInclusive<usize>>for Bits<N> {
    type Output = [bool];

    fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
        &self.0[index]
    }
}


impl <const N: usize> IndexMut<Range<usize>>for Bits<N> {
    fn index_mut(&mut self, index: Range<usize>) -> &mut Self::Output {
        &mut self.0[index]
    }
}


impl <const N: usize> IndexMut<RangeInclusive<usize>>for Bits<N> {
    fn index_mut(&mut self, index: RangeInclusive<usize>) -> &mut Self::Output {
        &mut self.0[index]
    }
}
