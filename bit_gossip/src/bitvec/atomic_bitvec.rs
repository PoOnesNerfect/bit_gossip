use super::{
    digit::{AtomicDigit, Digit, BITS},
    BitVec,
};
use std::fmt;
use std::sync::atomic::Ordering::Relaxed;

/// An array of atomic digits to work with underlying bits.
///
/// Uses `AtomicU64` for 64-bit architecture and `AtomicU32` for 32-bit architecture.
///
/// `AtomicBitVec` must be initialized with **maximum number of the bits** that will be used.
/// Although the internal data structure is a vec, the data structure itself
/// should be considered a fixed-size array.
/// This is because we only want to pass around the `AtomicBitVec` by reference,
/// which means we cannot resize the vec itself, but only mutate the internal data.
///
/// This means that AtomicBitVec is inherently less efficient than [BitVec].
/// However, to process and mutate values in parallel, it is necessary to use atomic values.
pub struct AtomicBitVec(pub Vec<AtomicDigit>);

impl AtomicBitVec {
    /// Initialize with zeros that is at least n bits long.
    /// n signifies the number of bits that will be used.
    /// Actually allocated bits will be rounded up to the
    /// nearest multiple of 32 or 64 depending on the target architecture.
    #[inline]
    pub fn zeros(n: usize) -> Self {
        let (i, j) = (n / BITS, n % BITS);
        let mut res = Self(Vec::with_capacity(i + (j > 0) as usize));
        for _ in 0..i {
            res.0.push(AtomicDigit::new(0));
        }
        if j > 0 {
            res.0.push(AtomicDigit::new(0));
        }

        res
    }

    /// Initialize with zeros that is at least n bits long,
    /// and set the bit at the given index to 1.
    #[inline]
    pub fn one(bit_index: usize, n: usize) -> Self {
        let res = Self::zeros(n);
        res.set_bit(bit_index, true);
        res
    }

    /// Set the bit at the given index to the given value.
    /// It assumes that the index is within the bit length of the array
    /// because we cannot resize the vec by reference.
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn set_bit(&self, index: usize, value: bool) {
        let (i, j) = (index / BITS, index % BITS);
        if value {
            self.0[i].fetch_or(1 << j, Relaxed);
        } else {
            self.0[i].fetch_and(!(1 << j), Relaxed);
        }
    }

    /// Get the bit at the given index.
    /// It assumes that the index is within the bit length of the array.
    /// Panics if the index is out of bounds.
    #[inline]
    pub fn get_bit(&self, index: usize) -> bool {
        let (i, j) = (index / BITS, index % BITS);
        (self.0[i].load(Relaxed) & (1 << j)) != 0
    }

    /// Check if all bits are zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|x| x.load(Relaxed) == 0)
    }

    /// Iterate over each bit that is 1, returning the index of the bit.
    #[inline]
    pub fn iter_ones(&self) -> IterOnes {
        IterOnes {
            data: self,
            array_index: 0,
            current: self.0[0].load(Relaxed),
        }
    }

    /// Iterate over each bit that is 0, returning the index of the bit.
    #[inline]
    pub fn iter_zeros(&self) -> IterZeros {
        IterZeros {
            data: self,
            array_index: 0,
            current: self.0[0].load(Relaxed),
        }
    }

    /// Set all bits to 0.
    #[inline]
    pub fn clear(&self) {
        self.0.iter().for_each(|x| x.store(0, Relaxed));
    }

    /// Convert from BitVec to AtomicBitVec.
    /// Set the length to at least n bits.
    #[inline]
    pub fn from_bitvec(bits: &BitVec, n: usize) -> Self {
        let res = Self::zeros(n);

        for (i, b) in bits.0.iter().enumerate() {
            if *b == 0 {
                continue;
            }

            res.0[i].store(*b, Relaxed);
        }

        res
    }

    /// Convert from AtomicBitVec to BitVec.
    #[inline]
    pub fn into_bitvec(&self) -> BitVec {
        let mut bits = BitVec(Vec::with_capacity(self.0.len()));
        for a in &self.0 {
            bits.0.push(a.load(Relaxed));
        }
        bits.normalize();
        bits
    }

    /// Set values of this AtomicBitVec from the given BitVec.
    #[inline]
    pub fn assign_from(&self, rhs: &BitVec) {
        for (a, b) in self.0.iter().zip(rhs.0.iter()) {
            a.store(*b, Relaxed);
        }

        if self.0.len() > rhs.0.len() {
            for a in self.0.iter().skip(rhs.0.len()) {
                a.store(0, Relaxed);
            }
        }
    }

    /// Truncate the size of the bitvec to the given length of bits.
    pub fn truncate(&mut self, bit_len: usize) {
        let (i, j) = (bit_len / BITS, bit_len % BITS);
        self.0.truncate(i + (j > 0) as usize);
        if j > 0 {
            self.0[i].fetch_and(Digit::MAX >> (BITS - j), Relaxed);
        }
    }
}

impl AtomicBitVec {
    /// Checks if two bitvecs are equal.
    pub fn eq(&self, other: &BitVec) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }

        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(a, b)| a.load(Relaxed) == *b)
    }

    /// a |= b
    pub fn bitor_assign(&self, rhs: &BitVec) {
        for (a, b) in self.0.iter().zip(rhs.0.iter()) {
            if *b != 0 {
                a.fetch_or(*b, Relaxed);
            }
        }
    }

    /// a |= b
    pub fn bitor_assign_atomic(&self, rhs: &AtomicBitVec) {
        for (a, b) in self.0.iter().zip(rhs.0.iter()) {
            let b = b.load(Relaxed);

            if b != 0 {
                a.fetch_or(b, Relaxed);
            }
        }
    }
}

impl fmt::Debug for AtomicBitVec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AtomicBitVec(")?;
        for i in (0..self.0.len()).rev() {
            write!(f, "{:0>BITS$b}", self.0[i].load(Relaxed))?;
        }
        write!(f, ")")
    }
}

/// Iterates over each Digit element in the array,
/// and then iterates over each bit in the Digit element.
pub struct IterOnes<'a> {
    data: &'a AtomicBitVec,
    array_index: usize,
    current: Digit,
}

impl<'a> IterOnes<'a> {
    pub fn chunks(self, chunk_size: usize) -> ChunkIter<Self> {
        ChunkIter {
            iter: self,
            chunk_size,
            done: false,
        }
    }
}

impl<'a> Iterator for IterOnes<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current == 0 {
            self.array_index += 1;
            if self.array_index == self.data.0.len() {
                return None;
            }
            self.current = self.data.0[self.array_index].load(Relaxed);
        }

        let trailing_zeros = self.current.trailing_zeros();
        self.current &= !(1 << trailing_zeros);
        Some(self.array_index * BITS + trailing_zeros as usize)
    }
}

/// Iterates over each Digit element in the array,
/// and then iterates over each bit in the Digit element.
pub struct IterZeros<'a> {
    data: &'a AtomicBitVec,
    array_index: usize,
    current: Digit,
}

impl<'a> IterZeros<'a> {
    pub fn chunks(self, chunk_size: usize) -> ChunkIter<Self> {
        ChunkIter {
            iter: self,
            chunk_size,
            done: false,
        }
    }
}

impl<'a> Iterator for IterZeros<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current == Digit::MAX {
            self.array_index += 1;
            if self.array_index == self.data.0.len() {
                return None;
            }
            self.current = self.data.0[self.array_index].load(Relaxed);
        }

        let trailing_ones = self.current.trailing_ones();
        self.current |= 1 << trailing_ones;
        Some(self.array_index * BITS + trailing_ones as usize)
    }
}

pub struct ChunkIter<I> {
    iter: I,
    chunk_size: usize,
    done: bool,
}

impl<I: Iterator> Iterator for ChunkIter<I> {
    type Item = Vec<I::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let mut chunk = Vec::with_capacity(self.chunk_size);
        for _ in 0..self.chunk_size {
            if let Some(x) = self.iter.next() {
                chunk.push(x);
            } else {
                self.done = true;
                break;
            }
        }

        if chunk.is_empty() {
            None
        } else {
            Some(chunk)
        }
    }
}
