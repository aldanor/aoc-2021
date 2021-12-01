use core::hint::unreachable_unchecked;
use core::ops::{Add, AddAssign, Mul};
use core::slice;

use memchr::{memchr, memchr2};

pub trait Integer:
    Copy
    + From<u8>
    + Add<Output = Self>
    + Mul<Output = Self>
    + Default
    + PartialEq
    + Eq
    + Ord
    + PartialOrd
{
}

impl Integer for u8 {}
impl Integer for u16 {}
impl Integer for u32 {}
impl Integer for u64 {}
impl Integer for i16 {}
impl Integer for i32 {}
impl Integer for i64 {}

#[inline(always)]
pub fn parse_int_fast_skip_custom<T: Integer>(
    s: &mut &[u8], min_digits: usize, max_digits: usize, skip: usize,
) -> T {
    let mut v = T::from(s.get_digit());
    *s = s.advance(1);
    for _ in 1..min_digits {
        let d = s.get_digit();
        *s = s.advance(1);
        v = v * T::from(10u8) + T::from(d);
    }
    for _ in min_digits..max_digits {
        let d = s.get_digit();
        if d < 10 {
            *s = s.advance(1);
            v = v * T::from(10u8) + T::from(d);
        } else {
            *s = s.advance(skip);
            return v;
        }
    }
    *s = s.advance(skip);
    v
}

#[inline(always)]
pub fn parse_int_fast<T: Integer>(s: &mut &[u8], min_digits: usize, max_digits: usize) -> T {
    parse_int_fast_skip_custom(s, min_digits, max_digits, 1)
}

pub trait SliceExt<T: Copy> {
    fn get_len(&self) -> usize;
    fn get_at(&self, i: usize) -> T;
    fn get_mut_at(&mut self, i: usize) -> &mut T;
    fn set_at(&mut self, i: usize, v: T);
    fn advance(&self, n: usize) -> &Self;
    fn add_at(&mut self, i: usize, v: T);

    #[inline]
    fn get_first(&self) -> T {
        self.get_at(0)
    }

    #[inline]
    fn get_last(&self) -> T {
        self.get_at(self.get_len() - 1)
    }
}

impl<T: Copy + Add<Output = T> + AddAssign> SliceExt<T> for [T] {
    #[inline]
    fn get_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn get_at(&self, i: usize) -> T {
        unsafe { *self.get_unchecked(i) }
    }

    #[inline]
    fn get_mut_at(&mut self, i: usize) -> &mut T {
        unsafe { self.get_unchecked_mut(i) }
    }

    #[inline]
    fn set_at(&mut self, i: usize, v: T) {
        unsafe { *self.get_unchecked_mut(i) = v };
    }

    #[inline]
    fn advance(&self, n: usize) -> &Self {
        unsafe { slice::from_raw_parts(self.as_ptr().add(n), self.len().saturating_sub(n)) }
    }

    #[inline]
    fn add_at(&mut self, i: usize, v: T) {
        unsafe { *self.get_unchecked_mut(i) += v };
    }
}

pub trait ByteSliceExt: SliceExt<u8> {
    fn memchr(&self, c: u8) -> usize;
    fn memchr2(&self, c1: u8, c2: u8) -> usize;
    fn get_u16_ne(&self) -> u16;

    #[inline]
    fn get_digit(&self) -> u8 {
        self.get_first().wrapping_sub(b'0')
    }

    #[inline]
    fn get_digit_at(&self, i: usize) -> u8 {
        self.get_at(i).wrapping_sub(b'0')
    }

    #[inline]
    fn skip_past(&self, c: u8, i: usize) -> &Self {
        self.advance(1 + i + self.memchr(c))
    }
}

impl ByteSliceExt for [u8] {
    #[inline]
    fn memchr(&self, c: u8) -> usize {
        memchr(c, self).unwrap_or_else(|| unsafe { unreachable_unchecked() })
    }

    #[inline]
    fn memchr2(&self, c1: u8, c2: u8) -> usize {
        memchr2(c1, c2, self).unwrap_or_else(|| unsafe { unreachable_unchecked() })
    }

    #[inline]
    fn get_u16_ne(&self) -> u16 {
        let mut a = [0; 2];
        a.copy_from_slice(&self[..2]);
        u16::from_ne_bytes(a)
    }
}
