use std::iter;
use std::mem;
use std::slice;

use crate::utils::*;

type Weight = u8;
type Score = u16;

const CAP: usize = 1 << 8;
const K: usize = 16; // because cell weights are <= 9

use arrayvec::ArrayVec;

struct Queue {
    store: [ArrayVec<usize, CAP>; K],
    min_score: Score,
    start: usize,
}

impl Queue {
    pub fn new() -> Self {
        let store =
            iter::repeat_with(ArrayVec::new).take(K).collect::<Vec<_>>().try_into().unwrap();
        let (min_score, start) = (0, 0);
        Self { store, min_score, start }
    }

    #[inline]
    pub fn iter_min(&mut self, mut func: impl FnMut(&mut Queue, usize)) -> bool {
        for _ in 0..K {
            if !self.store[self.start].is_empty() {
                // ok, this is the dodgy part - BUT:
                // min node weight is >= 1, so there won't be any contention since
                // mutated nodes won't overlap with the top row being iterated over
                let queue_ref = unsafe { &mut *(self as *mut _) };
                for &i in &self.store[self.start] {
                    func(queue_ref, i);
                }
                self.store[self.start].clear();
                return true;
            }
            self.start = (self.start + 1) % K;
            self.min_score += 1;
        }
        false
    }

    #[inline]
    pub fn push(&mut self, dist: Score, index: usize) {
        let i = (self.start + (dist - self.min_score) as usize) % K;
        unsafe { self.store[i].push_unchecked(index) };
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(packed)]
struct Cell {
    pub dist: Score,
    pub weight: Weight,
    pub is_free: bool,
}

struct Grid<const N: usize, const W: usize, const R: usize> {
    cells: Vec<Cell>,
}

impl<const N: usize, const W: usize, const R: usize> Grid<N, W, R> {
    const S: usize = (W - N * R) >> 1;
    const START: usize = W * Self::S + Self::S;
    const END: usize = W * W - W * Self::S - Self::S - 1;

    pub fn parse(mut s: &[u8]) -> Self {
        let mut cells = Vec::<Cell>::with_capacity(W * W);
        unsafe {
            cells.set_len(W * W);
            slice::from_raw_parts_mut(
                cells.as_mut_ptr().cast::<u8>(),
                W * W * mem::size_of::<Cell>(),
            )
            .fill(0xff);
        }
        for i in 0..N {
            for j in 0..N {
                let weight = s.get_digit_at(j);
                let index = (Self::S + i) * W + (Self::S + j);
                for i_r in 0..R {
                    for j_r in 0..R {
                        let weight = if i_r != 0 || j_r != 0 {
                            (((weight + i_r as Weight + j_r as Weight) - 1) % 9) + 1
                        } else {
                            weight
                        };
                        let cell = &mut cells[index + N * (i_r * W + j_r)];
                        cell.weight = weight;
                        cell.is_free = true;
                    }
                }
            }
            s = s.advance(N + 1);
        }
        let start = &mut cells[Self::START];
        start.is_free = false;
        start.dist = 0;
        Self { cells }
    }

    pub fn dijkstra(self) -> Score {
        let mut grid = self.cells;

        let mut queue = Queue::new();
        queue.push(0, Self::START);

        while queue.iter_min(|queue, i| {
            let cell = grid[i].clone();
            for j in [i - W, i - 1, i + 1, i + W] {
                let neighbor = &mut grid[j];
                if neighbor.weight == 0xff {
                    continue;
                }
                let alt = cell.dist + neighbor.weight as Score;
                if alt < neighbor.dist {
                    neighbor.dist = alt;
                    if neighbor.is_free {
                        neighbor.is_free = false;
                        queue.push(alt, j);
                    }
                }
            }
        }) {}
        grid[Self::END].dist
    }
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> Score {
    Grid::<100, 128, 1>::parse(s).dijkstra()
}

pub fn part2(s: &[u8]) -> Score {
    Grid::<100, 512, 5>::parse(s).dijkstra()
}

#[test]
fn test_day15_part1() {
    assert_eq!(part1(input()), 714);
}

#[test]
fn test_day15_part2() {
    assert_eq!(part2(input()), 2948);
}
