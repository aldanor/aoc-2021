use std::hash::Hasher;
use std::mem::MaybeUninit;

use crate::utils::*;

use ahash::AHasher;
use arrayvec::ArrayVec;
use core_simd::Simd;

const K: usize = 32768; // max # of grid cells

const EMPTY: u8 = 0;
const LR: u8 = 1; // east (left-right)
const TB: u8 = 2; // south (top-bottom)

const MIN_WIDTH: usize = 129;
const MAX_WIDTH: usize = 191;
const MAX_HEIGHT: usize = 137;

#[derive(Clone, Copy, PartialEq, Eq)]
struct PackedGrid {
    blocks: [[u64; 4]; MAX_HEIGHT],
    width: usize,
    height: usize,
}

impl PackedGrid {
    pub fn new(width: usize, height: usize) -> Self {
        assert!(width >= MIN_WIDTH && width <= MAX_WIDTH && height <= MAX_HEIGHT);
        let blocks = [[0; 4]; MAX_HEIGHT];
        Self { blocks, width, height }
    }

    fn uninit(&self) -> Self {
        let blocks = unsafe { MaybeUninit::uninit().assume_init() };
        Self { blocks, ..*self }
    }

    fn last_block_size(&self) -> usize {
        self.width - 64 * 2
    }

    fn last_block_mask(&self) -> u64 {
        (1_u64 << self.last_block_size()) - 1
    }

    pub fn move_right(&self) -> Self {
        let mut out = self.uninit();
        let (s, m) = (self.last_block_size() - 1, self.last_block_mask());
        assert!(s < 64);
        for i in 0..self.height {
            let (src, dst) = (&self.blocks[i], &mut out.blocks[i]);
            dst[0] = src[0] << 1 | src[2] >> s;
            dst[1] = src[1] << 1 | src[0] >> 63;
            dst[2] = (src[2] << 1 | src[1] >> 63) & m;
        }
        out
    }

    pub fn move_left(&self) -> Self {
        let mut out = self.uninit();
        let (s, m) = (self.last_block_size() - 1, self.last_block_mask());
        assert!(s < 64);
        for i in 0..self.height {
            let (src, dst) = (&self.blocks[i], &mut out.blocks[i]);
            dst[0] = src[0] >> 1 | src[1] << 63;
            dst[1] = src[1] >> 1 | src[2] << 63;
            dst[2] = (src[2] >> 1 | src[0] << s) & m;
        }
        out
    }

    pub fn move_up(&self) -> Self {
        let mut out = self.uninit();
        for i in 0..self.height - 1 {
            out.blocks[i] = self.blocks[i + 1];
        }
        out.blocks[self.height - 1] = self.blocks[0];
        out
    }

    pub fn move_down(&self) -> Self {
        let mut out = self.uninit();
        for i in 1..self.height {
            out.blocks[i] = self.blocks[i - 1];
        }
        out.blocks[0] = self.blocks[self.height - 1];
        out
    }

    fn apply_binary_op(&self, other: &Self, func: impl Fn(u64, u64) -> u64) -> Self {
        let mut out = self.uninit();
        for i in 0..self.height {
            for j in 0..4 {
                out.blocks[i][j] = func(self.blocks[i][j], other.blocks[i][j]);
            }
        }
        out
    }

    pub fn bit_and(&self, other: &Self) -> Self {
        self.apply_binary_op(other, |x, y| x & y)
    }

    pub fn bit_or(&self, other: &Self) -> Self {
        self.apply_binary_op(other, |x, y| x | y)
    }

    pub fn bit_xor(&self, other: &Self) -> Self {
        self.apply_binary_op(other, |x, y| x ^ y)
    }

    pub fn blocks(&self) -> &[[u64; 4]] {
        &self.blocks[..self.height]
    }
}

#[derive(Clone, Copy)]
pub struct Map {
    lr: PackedGrid,
    tb: PackedGrid,
}

impl Map {
    pub fn parse(mut s: &[u8]) -> Self {
        let w = s.memchr(b'\n');
        let mut lr = PackedGrid::new(w, MAX_HEIGHT);
        let mut tb = PackedGrid::new(w, MAX_HEIGHT);
        let mut h = 0;
        while s.len() > 1 {
            for i in 0..w {
                let c = s.get_at(i);
                lr.blocks[h][i >> 6] |= ((c == b'>') as u64) << (i & 0x3f);
                tb.blocks[h][i >> 6] |= ((c == b'v') as u64) << (i & 0x3f);
            }
            s = s.advance(w + 1);
            h += 1;
        }
        assert!(h <= MAX_HEIGHT);
        lr.height = h;
        tb.height = h;
        Self { lr, tb }
    }

    fn non_empty(&self) -> PackedGrid {
        self.lr.bit_or(&self.tb)
    }

    fn move_lr(&mut self) -> bool {
        let non_empty = self.non_empty();
        let after_move = self.lr.move_right();
        let is_blocked = after_move.bit_and(&non_empty);
        let moved_ok = after_move.bit_xor(&is_blocked);
        let stay_in_place = is_blocked.move_left();
        let lr = moved_ok.bit_or(&stay_in_place);
        let changed = lr.blocks() != self.lr.blocks();
        self.lr = lr;
        changed
    }

    fn move_tb(&mut self) -> bool {
        let non_empty = self.non_empty();
        let after_move = self.tb.move_down();
        let is_blocked = after_move.bit_and(&non_empty);
        let moved_ok = after_move.bit_xor(&is_blocked);
        let stay_in_place = is_blocked.move_up();
        let tb = moved_ok.bit_or(&stay_in_place);
        let changed = tb.blocks() != self.tb.blocks();
        self.tb = tb;
        changed
    }

    pub fn step(&mut self) -> bool {
        let lr_changed = self.move_lr();
        let tb_changed = self.move_tb();
        lr_changed || tb_changed
    }

    #[allow(unused)]
    pub fn to_grid(&self) -> Vec<u8> {
        // used for debugging only
        let (w, h) = (self.lr.width, self.lr.height);
        let mut v = vec![0; w * h];
        for i in 0..h {
            for j in 0..w {
                let lr = self.lr.blocks[i][j >> 6] & (1 << (j & 0x3f)) != 0;
                let tb = self.tb.blocks[i][j >> 6] & (1 << (j & 0x3f)) != 0;
                assert!(!lr || !tb);
                v[i * w + j] = if lr {
                    LR
                } else if tb {
                    TB
                } else {
                    EMPTY
                };
            }
        }
        v
    }

    #[allow(unused)]
    pub fn hash(&self) -> u64 {
        // used for debugging, to check correctness
        let v = self.to_grid();
        let mut h = AHasher::new_with_keys(0, 0);
        h.write(&v);
        h.finish()
    }

    #[allow(unused)]
    pub fn print(&self) {
        let (w, h) = (self.lr.width, self.lr.height);
        let v = self.to_grid();
        for i in 0..h {
            for j in 0..w {
                let c = match v[i * w + j] {
                    LR => '>',
                    TB => 'v',
                    _ => '.',
                };
                print!("{}", c);
            }
            println!();
        }
    }
}

const fn build_lookup_tables() -> [[u8; 64]; 2] {
    // the 6 bits are: "AACCBB" - after/center/before
    // for LR, before=left/after=right; for TB, before=top/after=bottom
    // first table is horizontal (LR); second table is vertical (TB)
    // the return value is (new_center, changed)
    let mut out = [[0; 64]; 2];
    let mut before = 0_u8;
    while before < 3 {
        let mut center = 0_u8;
        while center < 3 {
            let mut after = 0_u8;
            while after < 3 {
                let index = after << 4 | center << 2 | before;
                out[0][index as usize] = match (before, center, after) {
                    (LR, EMPTY, _) => LR,
                    (_, LR, EMPTY) => EMPTY,
                    (_, center, _) => center,
                };
                out[1][index as usize] = match (before, center, after) {
                    (TB, EMPTY, _) => TB,
                    (_, TB, EMPTY) => EMPTY,
                    (_, center, _) => center,
                };
                after += 1;
            }
            center += 1;
        }
        before += 1;
    }
    out
}

type Cells = ArrayVec<u8, K>;

#[allow(unused)]
fn fill_ghost_vertical(c: &mut Cells, w: usize, h: usize) {
    let mut row_offset = w + 2;
    for _ in 0..h {
        c[row_offset + 0] = c[row_offset + w];
        c[row_offset + (w + 1)] = c[row_offset + 1];
        row_offset += w + 2;
    }
}

fn fill_ghost_horizontal(c: &mut Cells, w: usize, h: usize) {
    unsafe {
        std::ptr::copy_nonoverlapping(
            c.as_ptr().add(h * (w + 2) + 1),
            c.as_mut_ptr().add(0 * (w + 2) + 1),
            w,
        );
        std::ptr::copy_nonoverlapping(
            c.as_ptr().add(1 * (w + 2) + 1),
            c.as_mut_ptr().add((h + 1) * (w + 2) + 1),
            w,
        );
    }
}

struct Grid {
    cells: Cells,
    width: usize,
    height: usize,
}

impl Grid {
    #[allow(unused)]
    pub fn parse(mut s: &[u8]) -> Self {
        let w = s.memchr(b'\n');
        let mut h = 0;
        let mut g = ArrayVec::new();
        for _ in 0..w + 2 {
            unsafe { g.push_unchecked(0) }; // ghost row (first)
        }
        while s.len() > 1 {
            unsafe { g.push_unchecked(0) }; // ghost column (first)
            for i in 0..w {
                let c = s.get_at(i);
                let v = (c + 10) >> 6; // '.'=46, '>'=62, 'v'=118
                unsafe { g.push_unchecked(v) };
            }
            unsafe { g.push_unchecked(0) }; // ghost column (last)
            s = s.advance(w + 1);
            h += 1;
        }
        for _ in 0..w + 2 {
            unsafe { g.push_unchecked(0) }; // ghost row (first)
        }
        Self { cells: g, width: w, height: h }
    }

    #[allow(unused)]
    pub fn print(&self) {
        for i in 0..self.height {
            for j in 0..self.width {
                let c = match self.cells[(i + 1) * (2 + self.width) + (j + 1)] {
                    LR => '>',
                    TB => 'v',
                    _ => '.',
                };
                print!("{}", c);
            }
            println!();
        }
    }

    #[allow(unused)]
    pub fn step_naive(&mut self) -> bool {
        // obviously super slow; used to verify correctness
        let (w, h) = (self.width, self.height);
        let mut changed = false;

        let mut tmp = ArrayVec::<u8, K>::new();
        unsafe { tmp.set_len((w + 2) * (h + 2)) };

        fill_ghost_vertical(&mut self.cells, w, h);
        let (src, dst) = (&self.cells, &mut tmp);
        for i in 1..(h + 1) {
            for j in 1..(w + 1) {
                let left = src[i * (w + 2) + (j - 1)];
                let center = src[i * (w + 2) + j];
                let right = src[i * (w + 2) + (j + 1)];
                dst[i * (w + 2) + j] = match (left, center, right) {
                    (LR, EMPTY, _) => {
                        changed = true;
                        LR
                    }
                    (_, LR, EMPTY) => {
                        changed = true;
                        EMPTY
                    }
                    (_, center, _) => center,
                };
            }
        }

        fill_ghost_horizontal(&mut tmp, w, h);
        let (src, dst) = (&tmp, &mut self.cells);
        for i in 1..(h + 1) {
            for j in 1..(w + 1) {
                let top = src[(i - 1) * (w + 2) + j];
                let center = src[i * (w + 2) + j];
                let bottom = src[(i + 1) * (w + 2) + j];
                dst[i * (w + 2) + j] = match (top, center, bottom) {
                    (TB, EMPTY, _) => {
                        changed = true;
                        TB
                    }
                    (_, TB, EMPTY) => {
                        changed = true;
                        EMPTY
                    }
                    (_, center, _) => center,
                };
            }
        }

        changed
    }

    #[allow(unused)]
    pub fn step_simd(&mut self) -> bool {
        const LOOKUP: [[u8; 64]; 2] = build_lookup_tables();

        const LANES: usize = 8; // 8 lanes (or 16) is best on M1; on Intel, could be higher
        type S = Simd<u8, LANES>;
        const S_EMPTY: S = S::splat(EMPTY);
        const S_LR: S = S::splat(LR);
        const S_TB: S = S::splat(TB);

        let (w, h) = (self.width, self.height);
        let n_simd = w / LANES;
        let serial_start = n_simd * LANES + 1;

        let mut changed = false;

        let mut tmp = ArrayVec::<u8, K>::new();
        unsafe { tmp.set_len((w + 2) * (h + 2)) };

        // LR
        let (src, dst) = (&mut self.cells, &mut tmp);
        for i in 1..(h + 1) {
            let row = i * (w + 2);
            // ghost cells
            src[row + 0] = src[row + w];
            src[row + (w + 1)] = src[row + 1];
            // SIMD LR pass
            for k in 0..n_simd {
                let offset = row + 1 + k * LANES;
                let src_ptr = unsafe { src.as_ptr().add(offset) };
                let dst_ptr = unsafe { dst.as_mut_ptr().add(offset) };
                let left = S::from_array(unsafe { *src_ptr.sub(1).cast() });
                let center = S::from_array(unsafe { *src_ptr.cast() });
                let right = S::from_array(unsafe { *src_ptr.add(1).cast() });
                let mask_lr = left.lanes_eq(S_LR) & center.lanes_eq(S_EMPTY);
                let mask_empty = center.lanes_eq(S_LR) & right.lanes_eq(S_EMPTY);
                changed = changed || mask_lr.any() || mask_empty.any();
                let out = mask_empty.select(S_EMPTY, mask_lr.select(S_LR, center));
                unsafe { *dst_ptr.cast() = out.to_array() };
            }
            // serial LR pass
            let mut after = src.get_at(row + serial_start);
            let mut lookup = after << 4 | src.get_at(row + serial_start - 1) << 2;
            for j in serial_start..(w + 1) {
                let center = after;
                after = src.get_at(row + j + 1);
                lookup >>= 2;
                lookup |= after << 4;
                let new_center = LOOKUP[0].get_at(lookup as _);
                changed = changed || center != new_center;
                dst[row + j] = new_center;
            }
        }

        // TB
        fill_ghost_horizontal(&mut tmp, w, h); // ghost cells
        let (src, dst) = (&tmp, &mut self.cells);
        // SIMD TB pass (transpose the iteration order to prevent reloading)
        for k in 0..n_simd {
            let col_offset = 1 + k * LANES;
            let mut src_ptr = unsafe { src.as_ptr().add(col_offset) };
            let mut dst_ptr = unsafe { dst.as_mut_ptr().add(col_offset) };
            let mut center = S::from_array(unsafe { *src_ptr.cast() });
            let mut bottom = S::from_array(unsafe { *src_ptr.add(w + 2).cast() });
            for _ in 1..(h + 1) {
                unsafe { src_ptr = src_ptr.add(w + 2) };
                unsafe { dst_ptr = dst_ptr.add(w + 2) };
                let top = center;
                center = bottom;
                bottom = S::from_array(unsafe { *src_ptr.add(w + 2).cast() });
                let mask_tb = top.lanes_eq(S_TB) & center.lanes_eq(S_EMPTY);
                let mask_empty = center.lanes_eq(S_TB) & bottom.lanes_eq(S_EMPTY);
                changed = changed || mask_tb.any() || mask_empty.any();
                let out = mask_empty.select(S_EMPTY, mask_tb.select(S_TB, center));
                unsafe { *dst_ptr.cast() = out.to_array() };
            }
        }
        // serial TB pass
        for j in serial_start..(w + 1) {
            let mut after = src.get_at(1 * (w + 2) + j);
            let mut lookup = after << 4 | src.get_at(0 * (w + 2) + j) << 2;
            for i in 1..(h + 1) {
                let center = after;
                after = src.get_at((i + 1) * (w + 2) + j);
                lookup >>= 2;
                lookup |= after << 4;
                let new_center = LOOKUP[1].get_at(lookup as _);
                changed = changed || center != new_center;
                dst[i * (w + 2) + j] = new_center;
            }
        }

        changed
    }

    #[allow(unused)]
    pub fn hash(&self) -> u64 {
        // used for debugging, to check correctness
        let (w, h) = (self.width, self.height);
        let mut v = vec![0; w * h]; // exclude ghost cells
        for i in 0..h {
            v[i * w..][..w].copy_from_slice(&self.cells[(i + 1) * (w + 2) + 1..][..w]);
        }
        let mut h = AHasher::new_with_keys(0, 0);
        h.write(&v);
        h.finish()
    }
}

#[test]
fn test_correctness() {
    let s = include_bytes!("input.txt").as_ref();
    let mut g_naive = Grid::parse(s);
    let mut g_simd = Grid::parse(s);
    for i in 1..=100 {
        assert_eq!((i, g_naive.step_naive()), (i, g_simd.step_simd()));
        assert_eq!((i, g_naive.hash()), (i, g_simd.hash()));
    }
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> usize {
    let mut m = Map::parse(s);
    for i in 1.. {
        if !m.step() {
            return i;
        }
    }
    return usize::MAX;
}

pub fn part2(_: &[u8]) -> usize {
    0
}

#[test]
fn test_day25_part1() {
    assert_eq!(part1(input()), 601);
}

#[test]
fn test_day25_part2() {
    assert_eq!(part2(input()), 0);
}
