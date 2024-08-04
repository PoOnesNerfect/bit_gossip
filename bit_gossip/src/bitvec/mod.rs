//! bit vector implementations for internal use.

#[cfg(feature = "parallel")]
mod atomic_bitvec;
#[cfg(feature = "parallel")]
pub use atomic_bitvec::AtomicBitVec;

mod bitvec;
pub use bitvec::BitVec;

mod digit {
    macro_rules! cfg_32 {
        ($($any:tt)+) => {
            #[cfg(not(target_pointer_width = "64"))] $($any)+
        }
    }

    macro_rules! cfg_64 {
        ($($any:tt)+) => {
            #[cfg(target_pointer_width = "64")] $($any)+
        }
    }

    macro_rules! cfg_digit {
        ($item32:item $item64:item) => {
            cfg_32!($item32);
            cfg_64!($item64);
        };
    }

    cfg_digit! {
        pub type Digit = u32;
        pub type Digit = u64;
    }

    cfg_digit! {
        pub type AtomicDigit = std::sync::atomic::AtomicU32;
        pub type AtomicDigit = std::sync::atomic::AtomicU64;
    }

    pub const BITS: usize = Digit::BITS as usize;
}
