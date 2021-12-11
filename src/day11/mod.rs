use std::iter;

use crate::utils::*;

const N: usize = 10;
type Grid = [[i8; N]; N];

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

fn parse_grid(mut s: &[u8]) -> Grid {
    let mut grid = [[0; N]; N];
    for i in 0..N {
        for j in 0..N {
            grid[i][j] = s.get_digit_at(j) as _;
        }
        s = s.advance(N + 1);
    }
    grid
}

fn evolve(mut grid: Grid) -> impl Iterator<Item = usize> {
    fn dfs(grid: &mut Grid, i: usize, j: usize) {
        grid[i][j] += 1;
        if grid[i][j] >= 10 {
            grid[i][j] = i8::MIN;
            for y in i.saturating_sub(1)..=(i + 1).min(N - 1) {
                for x in j.saturating_sub(1)..=(j + 1).min(N - 1) {
                    if (y, x) != (i, j) {
                        dfs(grid, y, x);
                    }
                }
            }
        }
    }

    iter::from_fn(move || {
        for i in 0..N {
            for j in 0..N {
                dfs(&mut grid, i, j);
            }
        }
        let mut n_flashes = 0;
        for i in 0..N {
            for j in 0..N {
                if grid[i][j] < 0 {
                    n_flashes += 1;
                    grid[i][j] = 0;
                }
            }
        }
        Some(n_flashes)
    })
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    evolve(parse_grid(s)).take(100).sum()
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    1 + evolve(parse_grid(s)).take_while(|n| *n != N * N).count()
}

#[test]
fn test_day11_part1() {
    assert_eq!(part1(input()), 1632);
}

#[test]
fn test_day11_part2() {
    assert_eq!(part2(input()), 303);
}
