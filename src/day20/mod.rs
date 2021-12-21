use std::mem;
use std::ptr;
use std::slice;

use core_simd::*;

use crate::utils::*;

const W: usize = 256; // max board width

type Bitmap = [u8; 512];

fn parse_bitmap(s: &mut &[u8]) -> Bitmap {
    let mut bitmap = [0; 512];
    for i in 0..512 {
        bitmap[i] = s.get_at(i) & 1;
    }
    *s = s.advance(514);
    bitmap
}

struct Board {
    bits: [[u8; W * W]; 2], // 0 = even, 1 = odd
    width: usize,
    offset: usize,
    steps: usize,
}

impl Board {
    pub fn parse(mut s: &[u8], bitmap: &Bitmap) -> Self {
        let mut bits = [0; W * W];
        let width = s.memchr(b'\n');
        let offset = ((W - width) / 2) * (W + 1);
        let mut row = offset;
        for _ in 0..width {
            let mut pos = row;
            for i in 0..width {
                bits[pos] = s.get_at(i) & 1;
                pos += 1;
            }
            s = s.advance(width + 1);
            row += W;
        }
        Self { bits: [bits, [bitmap[0]; W * W]], width, offset, steps: 0 }
    }

    pub fn print(&self) {
        let bits = &self.bits[self.steps & 1];
        println!();
        for i in 0..self.width {
            for j in 0..self.width {
                let c = bits[self.offset + i * W + j];
                print!("{}", if c == 1 { '#' } else { '.' });
            }
            println!();
        }
    }

    pub fn step_dumb(&mut self, bitmap: &Bitmap) -> usize {
        // note: offsets are from the top left corner to keep them non-negative
        const OFFSETS: [usize; 9] = [0, 1, 2, W, W + 1, W + 2, 2 * W, 2 * W + 1, 2 * W + 2];

        self.width += 2;
        self.offset -= W + 1;

        let (even, odd) = self.bits.split_at_mut(1);
        let (src, dst) = if self.steps & 1 == 0 {
            (&mut even[0], &mut odd[0])
        } else {
            (&mut odd[0], &mut even[0])
        };
        self.steps += 1;

        let mut total = 0;
        for i in 0..self.width {
            let row = self.offset + i * W;
            for j in 0..self.width {
                let index = range::<9>()
                    .map(|k| (src[row + j - W - 1 + OFFSETS[k]] as usize) << (8 - k))
                    .into_iter()
                    .sum::<usize>();
                let bit = bitmap[index];
                dst[row + j] = bit;
                total += bit as usize;
            }
        }
        total
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

const fn range<const N: usize>() -> [usize; N] {
    let mut out = [0; N];
    let mut i = 0;
    while i < N {
        out[i] = i;
        i += 1;
    }
    out
}

const fn input_indices() -> Simd<usize, 64> {
    // note: all indices are relative to the top left corner of the leftmost cell.
    // the top-left cell ('high bit') is ignored as it will be handled separately
    const UP: usize = 0;
    const MID: usize = W;
    const DOWN: usize = 2 * W;
    let mut idx = [0; 64];
    let mut i = 0;
    while i < 8 {
        idx[i * 8 + 0] = UP + 1;
        idx[i * 8 + 1] = UP + 2;
        idx[i * 8 + 2] = MID + 0;
        idx[i * 8 + 3] = MID + 1;
        idx[i * 8 + 4] = MID + 2;
        idx[i * 8 + 5] = DOWN + 0;
        idx[i * 8 + 6] = DOWN + 1;
        idx[i * 8 + 7] = DOWN + 2;
        i += 1;
    }
    Simd::from_array(idx)
}

const fn input_shifts() -> Simd<u8, 64> {
    let mut shifts = [0; 64];
    let mut i = 0;
    while i < 64 {
        shifts[i] = 7 - (i as u8 % 8);
        i += 1;
    }
    Simd::from_array(shifts)
}

#[inline]
unsafe fn step_x8(src: *const u8, dst: *mut u8, bitmap: *const u8) -> usize {
    // src pointer must point to the top left corner here
    let mut v = u8x64::gather_select_unchecked(
        slice::from_raw_parts(src, 0),
        Mask::splat(true),
        input_indices(),
        Simd::splat(0),
    );
    v <<= input_shifts();
    let indices: usizex8 = mem::transmute(v);
    let v = u8x8::gather_select_unchecked(
        slice::from_raw_parts(bitmap, 0),
        Mask::splat(true),
        indices,
        Simd::splat(0),
    );
    // dst pointer must point to the center cell here
    ptr::copy_nonoverlapping(v.as_array().as_ptr(), dst, 8);
    v.horizontal_sum() as _
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    let bitmap = parse_bitmap(&mut s);
    let mut board = Board::parse(s, &bitmap);
    board.step_dumb(&bitmap);
    board.step_dumb(&bitmap)
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    let bitmap = parse_bitmap(&mut s);
    let mut board = Board::parse(s, &bitmap);
    for _ in 0..49 {
        board.step_dumb(&bitmap);
    }
    board.step_dumb(&bitmap)
}

#[test]
fn test_day20_part1() {
    assert_eq!(part1(input()), 5203);
}

#[test]
fn test_day20_part2() {
    assert_eq!(part2(input()), 18806);
}
