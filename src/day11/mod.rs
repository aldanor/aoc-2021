use std::iter;
use std::mem;

use crate::utils::*;

const N: usize = 10;
const W: usize = N + 2;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

type Cell = i16;
type Grid = [[Cell; W]; W];

fn parse_grid(mut s: &[u8]) -> Grid {
    let mut grid = [[Cell::MIN; W]; W];
    for i in 1..=N {
        for j in 1..=N {
            grid[i][j] = s.get_digit_at(j - 1) as Cell;
        }
        s = s.advance(N + 1);
    }
    grid
}

fn evolve_dfs(mut grid: Grid) -> impl Iterator<Item = usize> {
    type FlatGrid = [Cell; W * W];
    let mut grid: FlatGrid = unsafe { mem::transmute(grid) };

    type Stack = Vec<usize>;
    let mut stack = Stack::with_capacity(W * W);
    let mut next = stack.clone();

    #[inline]
    fn bump_and_check(stack: &mut Stack, grid: &mut FlatGrid, i: usize, reset: bool) {
        let cell = unsafe { grid.get_unchecked_mut(i) };
        if *cell < 9 {
            if reset {
                *cell = (*cell).max(0) + 1;
            } else {
                *cell += 1;
            }
        } else {
            *cell = Cell::MIN;
            stack.push(i);
        }
    }

    iter::from_fn(move || {
        let mut n_flashes = 0;
        for i in 1..=N {
            for j in 1..=N {
                bump_and_check(&mut stack, &mut grid, i * W + j, true);
            }
        }
        while !stack.is_empty() {
            n_flashes += stack.len();
            mem::swap(&mut stack, &mut next);
            unsafe {
                stack.set_len(0);
            }
            for &i in &next {
                for j in [i - W - 1, i - W, i - W + 1, i - 1, i + 1, i + W - 1, i + W, i + W + 1] {
                    bump_and_check(&mut stack, &mut grid, j, false);
                }
            }
        }
        Some(n_flashes)
    })
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    evolve_dfs(parse_grid(s)).take(100).sum()
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    1 + evolve_dfs(parse_grid(s)).take_while(|n| *n != N * N).count()
}

#[test]
fn test_day11_part1() {
    assert_eq!(part1(input()), 1632);
}

#[test]
fn test_day11_part2() {
    assert_eq!(part2(input()), 303);
}
