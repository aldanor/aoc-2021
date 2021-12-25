use std::ptr;

use crate::utils::*;

use arrayvec::ArrayVec;

const K: usize = 32768; // max # of grid cells

const EMPTY: u8 = 0;
const LR: u8 = 1; // east (left-right)
const TB: u8 = 2; // south (top-bottom)

const WAS_E: u8 = 4; // was east
const WAS_S: u8 = 8; // was south

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

type Cells = ArrayVec<u8, K>;

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

    fn step_super_naive(&mut self) -> bool {
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
}

#[inline]
pub fn part1(s: &[u8]) -> usize {
    let mut g = Grid::parse(s);
    for i in 1.. {
        if !g.step_super_naive() {
            return i;
        }
    }
    0
}

#[inline]
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
