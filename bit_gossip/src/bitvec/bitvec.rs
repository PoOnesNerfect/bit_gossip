use super::{
    digit::{Digit, BITS},
    AtomicBitVec,
};
use std::{fmt, iter::repeat};

/// An array of digits to work with underlying bits.
///
/// Uses `u64` for 64-bit architecture and `u32` for 32-bit architecture.
///
/// Implementation is inspired by the `num-bigint` and `bitvec` crates.
///
/// Reasons for not using exiting crates:
/// - too many features that I dont't need,
/// - dont't have the specific behaviors that I want, and
/// - lack the convenience methods I need.
///
/// This data structure is very bare with the absolute minimum functionalities implemented.
#[derive(Clone)]
pub struct BitVec(pub Vec<Digit>);

impl BitVec {
    /// Initialize with empty vector.
    pub const ZERO: Self = Self(Vec::new());

    /// Initialize with a `true` bit at the given bit index.
    #[inline]
    pub fn one(bit_index: usize) -> Self {
        let (i, j) = (bit_index / BITS, bit_index % BITS);
        let mut res = Self(Vec::with_capacity(i + 1));

        res.0.resize(i, 0);
        res.0.push(1 << j);

        res
    }

    /// Initialize and fill with 1's for the given number of bits.
    #[inline]
    pub fn ones(bits: usize) -> Self {
        let (i, j) = (bits / BITS, bits % BITS);

        let mut res = Self(Vec::with_capacity(i + (j > 0) as usize));

        res.0.resize(i, Digit::MAX);
        if j > 0 {
            res.0.push(Digit::MAX >> (BITS - j));
        }

        res
    }

    /// Set the bit at the given index to the given value.
    ///
    /// If the bit index is out of range, it will resize the array.
    #[inline]
    pub fn set_bit(&mut self, bit_index: usize, value: bool) {
        let (i, j) = (bit_index / BITS, bit_index % BITS);

        // if setting value to 1, we might need to resize the array
        if value && i >= self.0.len() {
            self.0.resize(i + 1, 0);
        }

        if value {
            self.0[i] |= 1 << j;
        } else if i < self.0.len() {
            self.0[i] &= !(1 << j);
        }

        self.normalize();
    }

    /// Get the bit at the given index.
    ///
    /// If the bit index is out of range, it will return `false`.
    #[inline]
    pub fn get_bit(&self, bit_index: usize) -> bool {
        let (i, j) = (bit_index / BITS, bit_index % BITS);
        if i >= self.0.len() {
            return false;
        }
        (self.0[i] & (1 << j)) != 0
    }

    /// Count the number of 1's in the bit vector.
    #[inline]
    pub fn count_ones(&self) -> usize {
        self.0.iter().map(|x| x.count_ones() as usize).sum()
    }

    /// Check if the bit vector is all zeros.
    ///
    /// Internally, it just checks that the vector is empty.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over bits and return the bit index of each `true` bits.
    #[inline]
    pub fn iter_ones(&self) -> IterOnes {
        IterOnes {
            data: self,
            array_index: 0,
            current: self.0.get(0).cloned().unwrap_or(0),
        }
    }

    /// Iterate over bits and return the bit index of each `false` bits.
    ///
    /// Note that this iterator will continue to return zeros even after the array is exhausted.
    #[inline]
    pub fn iter_zeros(&self) -> IterZeros {
        IterZeros {
            data: self,
            array_index: 0,
            next_bit: 0,
            current: self.0.get(0).cloned().unwrap_or(Digit::MAX),
        }
    }

    /// Clear all bits.
    /// Same as setting value to 0, but keeping the allocated memory.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub(super) fn normalize(&mut self) {
        while let Some(&0) = self.0.last() {
            self.0.pop();
        }
    }
}

impl BitVec {
    /// Check if all bits are the same.
    pub fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0.iter().zip(other.0.iter()).all(|(a, b)| a == b)
    }

    /// a = a & !b
    pub fn bitand_not_assign(&mut self, rhs: &Self) {
        if self.is_zero() || rhs.is_zero() {
            return;
        }

        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a &= !b;
        }

        self.normalize();
    }

    /// a = a & !b
    ///
    /// Takes an `AtomicBitVec` as the right-hand side.
    pub fn bitand_not_assign_atomic(&mut self, rhs: &AtomicBitVec) {
        if self.is_zero() {
            return;
        }

        if rhs.is_zero() {
            return;
        }

        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a &= !b.load(std::sync::atomic::Ordering::Relaxed);
        }

        self.normalize();
    }

    /// a = a | b
    pub fn bitor_assign(&mut self, rhs: &Self) {
        if rhs.is_zero() {
            return;
        }

        if self.is_zero() {
            self.0 = rhs.0.clone();
            return;
        }

        for (a, b) in self.0.iter_mut().zip(rhs.0.iter()) {
            *a |= b;
        }

        if self.0.len() < rhs.0.len() {
            self.0.extend_from_slice(&rhs.0[self.0.len()..]);
        }
    }

    /// a = a | (b & c)
    pub fn bitor_and_assign(&mut self, rhs1: &Self, rhs2: &Self) {
        if rhs1.is_zero() || rhs2.is_zero() {
            return;
        }

        if !self.is_zero() {
            for (a, (b, c)) in self.0.iter_mut().zip(rhs1.0.iter().zip(rhs2.0.iter())) {
                *a |= b & c;
            }
        }

        let rhs_len = rhs1.0.len().min(rhs2.0.len());
        if self.0.len() < rhs_len {
            self.0.reserve_exact(rhs_len - self.0.len());
            for (b, c) in rhs1.0.iter().zip(rhs2.0.iter()).skip(self.0.len()) {
                self.0.push(b & c);
            }

            self.normalize();
        }
    }

    /// a = a | (!b & c)
    pub fn bitor_not_and_assign(&mut self, rhs1: &Self, rhs2: &Self) {
        if rhs2.is_zero() {
            return;
        }

        let mut bc = rhs1.0.iter().chain(repeat(&0)).zip(rhs2.0.iter());

        for a in self.0.iter_mut() {
            let Some((b, c)) = bc.next() else {
                break;
            };

            *a |= !b & c;
        }

        if rhs2.0.len() > self.0.len() {
            self.0.reserve_exact(rhs2.0.len() - self.0.len());

            for (b, c) in bc {
                self.0.push(!b & c);
            }
        }

        self.normalize();
    }
}

impl fmt::Debug for BitVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BitVec(")?;
        if self.0.is_empty() {
            return write!(f, "{})", 0);
        }

        for i in (0..self.0.len()).rev() {
            write!(f, "{:0>BITS$b}", self.0[i])?;
        }
        write!(f, ")")
    }
}

/// Iterates over each Digit element in the array,
/// and then iterates over each bit in the Digit element.
pub struct IterOnes<'a> {
    data: &'a BitVec,
    array_index: usize,
    current: Digit,
}

impl<'a> Iterator for IterOnes<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current == 0 {
            self.array_index += 1;
            if self.array_index >= self.data.0.len() {
                return None;
            }
            self.current = self.data.0[self.array_index];
        }

        let trailing_zeros = self.current.trailing_zeros();
        self.current &= !(1 << trailing_zeros);
        Some(self.array_index * BITS + trailing_zeros as usize)
    }
}

/// Iterates over each Digit element in the array,
/// and then iterates over each bit in the Digit element.
/// Even when the array is done iterated, it will continue
/// to return zeros.
pub struct IterZeros<'a> {
    data: &'a BitVec,
    array_index: usize,
    current: Digit,
    next_bit: usize,
}

impl<'a> Iterator for IterZeros<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // If we have exhausted the array, just return incremented bit index
        if self.array_index >= self.data.0.len() {
            let ret = self.next_bit;
            self.next_bit += 1;
            return Some(ret);
        }

        // If current is all ones, move to the next array index
        while self.current == Digit::MAX {
            self.array_index += 1;

            // If we have exhausted the array, just return incremented bit index
            if self.array_index >= self.data.0.len() {
                let ret = self.array_index * BITS;
                self.next_bit = ret + 1;
                return Some(ret);
            }

            self.current = self.data.0[self.array_index];
        }

        // Find the next zero bit in the current Digit
        let trailing_ones = self.current.trailing_ones() as usize;
        self.current |= 1 << trailing_ones; // Set this bit to 1 to mark it as visited

        let ret = self.array_index * BITS + trailing_ones;
        self.next_bit = ret + 1;
        Some(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter_zeros() {
        let mut bv = BitVec::ZERO;
        bv.set_bit(0, true);
        bv.set_bit(2, true);
        bv.set_bit(3, true);
        bv.set_bit(5, true);
        bv.set_bit(7, true);
        bv.set_bit(8, true);
        bv.set_bit(10, true);
        bv.set_bit(12, true);

        let zeros: Vec<_> = bv.iter_zeros().take(8).collect();
        assert_eq!(zeros, vec![1, 4, 6, 9, 11, 13, 14, 15]);
        println!("{:?}", zeros);
        println!("{bv:?}");

        let bv = BitVec::ones(2);
        let zeros: Vec<_> = bv.iter_zeros().take(14).collect();
        assert_eq!(zeros, (2..16).collect::<Vec<_>>());

        println!("{:?}", zeros);
        println!("{bv:?}");

        let bv = BitVec::ones(17);
        let zeros: Vec<_> = bv.iter_zeros().take(2).collect();
        assert_eq!(zeros, vec![17, 18]);

        println!("{:?}", zeros);
        println!("{bv:?}");
    }
}
