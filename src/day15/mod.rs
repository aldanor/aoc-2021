use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
use std::iter;
use std::mem;
use std::slice;

use ringbuffer::{
    ConstGenericRingBuffer as RingBuf, RingBuffer, RingBufferExt, RingBufferRead, RingBufferWrite,
};

use crate::utils::*;

type Weight = u8;
type Score = u16;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

const CAP: usize = 1 << 8;
const K: usize = 16; // because cell weights are <= 9

struct Queue {
    store: [RingBuf<usize, CAP>; K],
    min_score: Score,
    start: usize,
}

impl Queue {
    pub fn new() -> Self {
        let init = RingBuf::new();
        let store = iter::repeat(init).take(K).collect::<Vec<_>>().try_into().unwrap();
        let (min_score, start) = (0, 0);
        Self { store, min_score, start }
    }

    #[inline]
    pub fn push(&mut self, dist: Score, index: usize) {
        let i = (self.start + (dist - self.min_score) as usize) % K;
        self.store[i].push(index);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<usize> {
        for i in self.start..(self.start + K) {
            let i = i % K;
            if !self.store[i].is_empty() {
                return self.store[i].dequeue();
            }
            self.start = (self.start + 1) % K;
            self.min_score += 1;
        }
        None
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

        while let Some(i) = queue.pop() {
            if i == Self::END {
                break;
            }
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
        }
        grid[Self::END].dist
    }
}

#[inline]
pub fn part1(mut s: &[u8]) -> Score {
    Grid::<100, 128, 1>::parse(s).dijkstra()
}

#[inline]
pub fn part2(mut s: &[u8]) -> Score {
    Grid::<100, 600, 5>::parse(s).dijkstra()
}

#[test]
fn test_day15_part1() {
    assert_eq!(part1(input()), 714);
}

#[test]
fn test_day15_part2() {
    assert_eq!(part2(input()), 2948);
}
