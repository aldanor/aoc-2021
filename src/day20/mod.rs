use std::mem;

use core_simd::*;

use crate::utils::*;

const W: usize = 256; // max board width
type T = u64;
const SHIFT: usize = 6;
const BITS: usize = mem::size_of::<T>() << 3;
const MASK: usize = BITS - 1;
const K: usize = 256 / BITS;

type Bitmap = [u8; 512];

fn parse_bitmap(s: &mut &[u8]) -> Bitmap {
    let mut bitmap = [0; 512];
    for i in 0..512 {
        bitmap[i] = s.get_at(i) & 1;
    }
    *s = s.advance(514);
    bitmap
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

#[inline]
const fn transpose8x8(mut x: u64) -> u64 {
    // see ch. 7-3 of Hacker's Delight for details
    x = x & 0xaa55_aa55_aa55_aa55
        | (x & 0x00aa_00aa_00aa_00aa) << 7
        | (x >> 7) & 0x00aa_00aa_00aa_00aa;
    x = x & 0xcccc_3333_cccc_3333
        | (x & 0x0000_cccc_0000_cccc) << 14
        | (x >> 14) & 0x0000_cccc_0000_cccc;
    x = x & 0xf0f0_f0f0_0f0f_0f0f
        | (x & 0x0000_0000_f0f0_f0f0) << 28
        | (x >> 28) & 0x0000_0000_f0f0_f0f0;
    x
}

#[inline]
fn transpose8x8x8(x: [u64; 8]) -> [u8; 64] {
    const fn swizzle_transpose() -> [usize; 64] {
        let mut out = [0; 64];
        let mut i = 0;
        while i < 64 {
            out[i] = (i & 7) << 3 | (i >> 3);
            i += 1;
        }
        out
    }

    const SWIZZLE_TRANSPOSE: [usize; 64] = swizzle_transpose();

    unsafe {
        let x = u8x64::from_array(mem::transmute(x));
        let mut x: [u64; 8] = mem::transmute(simd_swizzle!(x, SWIZZLE_TRANSPOSE));
        for i in 0..8 {
            x[i] = transpose8x8(x[i]);
        }
        mem::transmute(x)
    }
}

#[inline]
const fn get_row(row: &[u64; 4], j: usize, inf: u64) -> [u64; 3] {
    let x = match j {
        0 => [inf, row[0], row[1]],
        1 => [row[0], row[1], row[2]],
        2 => [row[1], row[2], row[3]],
        _ => [row[2], row[3], inf],
    };
    [x[1] << 1 | x[0] >> 63, x[1], x[1] >> 1 | x[2] << 63]
}

struct PackedBoard {
    bits: [[[T; K]; W]; 2],
    size: usize,
    offset: usize,
    inf: T,
}

impl PackedBoard {
    pub fn parse(mut s: &[u8], bitmap: &Bitmap) -> Self {
        let mut bits = [[0; K]; W];
        let width = s.memchr(b'\n');
        let offset = (W - width) >> 1;
        for i in 0..width {
            for j in 0..width {
                let v = (s.get_at(j) & 1) as T;
                bits[offset + i][(offset + j) >> SHIFT] |= v << ((offset + j) & MASK);
            }
            s = s.advance(width + 1);
        }
        let inf = if bitmap[0] == 0 { 0 } else { !0 };
        Self { bits: [bits, [[inf; K]; W]], size: width, offset, inf }
    }

    #[allow(unused)]
    pub fn print(&self) {
        let bits = &self.bits[0]; // even only
        println!();
        for i in 0..self.size {
            for j in 0..self.size {
                let c = bits[self.offset + i][(self.offset + j) >> SHIFT]
                    & (1 << ((self.offset + j) & MASK))
                    != 0;
                print!("{}", if c { '#' } else { '.' });
            }
            println!();
        }
    }

    pub fn count_ones(&self) -> usize {
        let bits = &self.bits[0]; // even only
        let mut out = 0;
        for i in 0..self.size {
            for j in 0..K {
                out += bits[self.offset + i][j].count_ones();
            }
        }
        out as _
    }

    pub fn double_step(&mut self, bitmap: &Bitmap) {
        self.step::<true>(bitmap);
        self.step::<false>(bitmap);
    }

    fn step<const EVEN: bool>(&mut self, bitmap: &Bitmap) {
        self.size += 2;
        self.offset -= 1;

        let (even, odd) = self.bits.split_at_mut(1);
        let (src, dst) =
            if EVEN { (&mut even[0], &mut odd[0]) } else { (&mut odd[0], &mut even[0]) };

        for j in 0..K {
            let mut mid = get_row(&src[self.offset - 1], j, self.inf);
            let mut bottom = get_row(&src[self.offset], j, self.inf);

            for i in 0..self.size {
                let top = mid;
                mid = bottom;
                bottom = get_row(&src[self.offset + i + 1], j, self.inf);

                let idx_hi = transpose8x8x8([
                    bottom[1], bottom[0], mid[2], mid[1], mid[0], top[2], top[1], top[0],
                ]);
                let mut idx_lo = bottom[2];

                let mut out = 0;
                for k in 0..BITS {
                    let hi = (idx_hi[k] as u16) << 1;
                    let lo = (idx_lo as u16) & 1;
                    out |= (bitmap[usize::from(hi | lo)] as T) << k;
                    idx_lo >>= 1;
                }
                dst[self.offset + i][j] = out;
            }
        }
    }
}

struct Board {
    bits: [[u8; W * W]; 2], // 0 = even, 1 = odd
    width: usize,
    offset: usize,
    steps: usize,
}

#[allow(unused)]
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

    #[allow(unused)]
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

    pub fn step_naive(&mut self, bitmap: &Bitmap) -> usize {
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

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(mut s: &[u8]) -> usize {
    let bitmap = parse_bitmap(&mut s);
    let mut board = PackedBoard::parse(s, &bitmap);
    board.double_step(&bitmap);
    board.count_ones()
}

pub fn part2(mut s: &[u8]) -> usize {
    let bitmap = parse_bitmap(&mut s);
    let mut board = PackedBoard::parse(s, &bitmap);
    for _ in 0..25 {
        board.double_step(&bitmap);
    }
    board.count_ones()
}

#[test]
fn test_day20_part1() {
    assert_eq!(part1(input()), 5203);
}

#[test]
fn test_day20_part2() {
    assert_eq!(part2(input()), 18806);
}
