//! Support for the [`subtle`](https://crates.io/crates/subtle) crate.

#![cfg(feature = "subtle")]
#![cfg_attr(docsrs, doc(cfg(feature = "subtle")))]

use crate::Uint;
use subtle::{
    Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess,
};

impl<const BITS: usize, const LIMBS: usize> ConditionallySelectable for Uint<BITS, LIMBS> {
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        let mut limbs = [0_u64; LIMBS];
        for (limb, (a, b)) in limbs
            .iter_mut()
            .zip(a.as_limbs().iter().zip(b.as_limbs().iter()))
        {
            *limb = u64::conditional_select(a, b, choice);
        }
        Self::from_limbs(limbs)
    }
}

impl<const BITS: usize, const LIMBS: usize> ConstantTimeEq for Uint<BITS, LIMBS> {
    #[inline]
    fn ct_eq(&self, rhs: &Self) -> Choice {
        // Leverage ConstantTimeEq for &[u64]
        self.as_limbs().ct_eq(rhs.as_limbs())
    }
}

impl<const BITS: usize, const LIMBS: usize> ConstantTimeGreater for Uint<BITS, LIMBS> {
    fn ct_gt(&self, rhs: &Self) -> Choice {
        let mut equal = Choice::from(1); // True
        let mut greater = Choice::from(0); // False

        // Iterate limbs in big-endian order.
        for (l, r) in self
            .as_limbs()
            .iter()
            .rev()
            .zip(rhs.as_limbs().iter().rev())
        {
            greater |= equal & l.ct_gt(r);
            equal &= l.ct_eq(r);
        }
        greater
    }
}

impl<const BITS: usize, const LIMBS: usize> ConstantTimeLess for Uint<BITS, LIMBS> {
    fn ct_lt(&self, rhs: &Self) -> Choice {
        let mut equal = Choice::from(1); // True
        let mut less = Choice::from(0); // False

        // Iterate limbs in big-endian order.
        for (l, r) in self
            .as_limbs()
            .iter()
            .rev()
            .zip(rhs.as_limbs().iter().rev())
        {
            less |= equal & l.ct_lt(r);
            equal &= l.ct_eq(r);
        }
        less
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{const_for, nlimbs};
    use proptest::proptest;

    #[test]
    fn test_select() {
        const_for!(BITS in SIZES {
            const LIMBS: usize = nlimbs(BITS);
            type U = Uint<BITS, LIMBS>;
            proptest!(|(a: U, b: U, c: bool)| {
                let choice = Choice::from(c as u8);
                let r = U::conditional_select(&a, &b, choice);
                let e = if c { b } else { a };
                assert_eq!(r, e);
            });
        });
    }
    #[test]
    fn test_eq() {
        const_for!(BITS in SIZES {
            const LIMBS: usize = nlimbs(BITS);
            type U = Uint<BITS, LIMBS>;
            proptest!(|(a: U, b: U)| {
                let r = a.ct_eq(&b);
                let e = a == b;
                assert_eq!(bool::from(r), e);
            });
        });
    }

    #[test]
    fn test_lt() {
        const_for!(BITS in SIZES {
            const LIMBS: usize = nlimbs(BITS);
            type U = Uint<BITS, LIMBS>;
            proptest!(|(a: U, b: U)| {
                let r = a.ct_lt(&b);
                let e = a < b;
                assert_eq!(bool::from(r), e);
            });
        });
    }

    #[test]
    fn test_gt() {
        const_for!(BITS in SIZES {
            const LIMBS: usize = nlimbs(BITS);
            type U = Uint<BITS, LIMBS>;
            proptest!(|(a: U, b: U)| {
                let r = a.ct_gt(&b);
                let e = a > b;
                assert_eq!(bool::from(r), e);
            });
        });
    }
}
