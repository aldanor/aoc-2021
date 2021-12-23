use core::hint::unreachable_unchecked;
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use core::slice;

use std::iter;

use memchr::{memchr, memchr2};

pub trait Integer:
    Copy
    + From<u8>
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Default
    + PartialEq
    + Eq
    + Ord
    + PartialOrd
    + iter::Sum
    + iter::Product
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
pub fn parse_int_fast_skip_custom<
    T: Integer,
    const MIN_DIGITS: usize,
    const MAX_DIGITS: usize,
    const SKIP: usize,
>(
    s: &mut &[u8],
) -> T {
    let mut v = T::from(s.get_digit());
    *s = s.advance(1);
    for _ in 1..MIN_DIGITS {
        let d = s.get_digit();
        *s = s.advance(1);
        v = v * T::from(10u8) + T::from(d);
    }
    for _ in MIN_DIGITS..MAX_DIGITS {
        let d = s.get_digit();
        if d < 10 {
            *s = s.advance(1);
            v = v * T::from(10u8) + T::from(d);
        } else {
            *s = s.advance(SKIP);
            return v;
        }
    }
    *s = s.advance(SKIP);
    v
}

#[inline(always)]
pub fn parse_int_fast<T: Integer, const MIN_DIGITS: usize, const MAX_DIGITS: usize>(
    s: &mut &[u8],
) -> T {
    parse_int_fast_skip_custom::<T, MIN_DIGITS, MAX_DIGITS, 1>(s)
}

#[inline(always)]
pub fn parse_int_fast_signed_skip_custom<
    T: Integer + Neg<Output = T>,
    const MIN_DIGITS: usize,
    const MAX_DIGITS: usize,
    const SKIP: usize,
>(
    s: &mut &[u8],
) -> T {
    let neg = s.get_first() == b'-';
    if neg {
        *s = s.advance(1);
    }
    let num = parse_int_fast_skip_custom::<T, MIN_DIGITS, MAX_DIGITS, SKIP>(s);
    if neg {
        -num
    } else {
        num
    }
}

#[inline(always)]
pub fn parse_int_fast_signed<
    T: Integer + Neg<Output = T>,
    const MIN_DIGITS: usize,
    const MAX_DIGITS: usize,
>(
    s: &mut &[u8],
) -> T {
    parse_int_fast_signed_skip_custom::<T, MIN_DIGITS, MAX_DIGITS, 1>(s)
}

pub trait SliceExt<T: Copy> {
    fn get_len(&self) -> usize;
    fn get_at(&self, i: usize) -> T;
    fn get_ref_at(&self, i: usize) -> &T;
    fn get_mut_at(&mut self, i: usize) -> &mut T;
    fn set_at(&mut self, i: usize, v: T);
    fn advance(&self, n: usize) -> &Self;

    #[inline]
    fn get_first(&self) -> T {
        self.get_at(0)
    }

    #[inline]
    fn get_last(&self) -> T {
        self.get_at(self.get_len() - 1)
    }

    #[inline]
    fn get_last_mut(&mut self) -> &mut T {
        self.get_mut_at(self.get_len() - 1)
    }

    #[inline]
    fn set_last(&mut self, value: T) {
        self.set_at(self.get_len(), value)
    }
}

impl<T: Copy> SliceExt<T> for [T] {
    #[inline]
    fn get_len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn get_at(&self, i: usize) -> T {
        unsafe { *self.get_unchecked(i) }
    }

    #[inline]
    fn get_ref_at(&self, i: usize) -> &T {
        unsafe { &(*self.get_unchecked(i)) }
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
}

pub trait SliceExtNumeric<T: Copy> {
    fn add_at(&mut self, i: usize, v: T);
}

impl<T: Copy + Add<Output = T> + AddAssign> SliceExtNumeric<T> for [T] {
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

#[derive(Debug)]
pub struct UnsafeStack<'a, T> {
    stack: &'a mut [T],
    ptr: *mut T,
}

impl<'a, T: Copy> UnsafeStack<'a, T> {
    pub fn new(stack: &'a mut [T]) -> Self {
        let ptr = stack.as_mut_ptr();
        Self { stack, ptr }
    }

    #[inline]
    pub fn len(&self) -> usize {
        unsafe { self.ptr.offset_from(self.stack.as_ptr()) as usize }
    }

    #[inline]
    pub fn into_slice(self) -> &'a [T] {
        &self.stack[..self.len()]
    }

    #[inline]
    pub fn push(&mut self, v: T) {
        unsafe {
            (*self.ptr) = v;
            self.ptr = self.ptr.add(1);
        }
    }

    #[inline]
    pub fn pop(&mut self) -> T {
        unsafe {
            self.ptr = self.ptr.sub(1);
            *self.ptr
        }
    }
}
