use std::fmt::{self, Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::utils::*;

const N: usize = 5;
type Number = u8;

fn parse_numbers(s: &mut &[u8]) -> Vec<Number> {
    let mut numbers = Vec::with_capacity(1 << 7);
    while s.get_at(0) != b'\n' {
        numbers.push(parse_int_fast::<_, 1, 2>(s));
    }
    *s = s.advance(2);
    numbers
}

#[derive(Copy, Clone, Default)]
struct Row([Number; N]);

impl Debug for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for j in 0..N {
            if self.0[j] >= 100 {
                write!(f, "  X")?;
            } else {
                write!(f, "{:>w$}", self.0[j], w = 2 + (j != 0) as usize)?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, Default)]
struct Board([Row; N]);

impl Board {
    pub fn parse(s: &mut &[u8]) -> Self {
        let mut board = Self::default();
        for i in 0..N {
            for j in 0..N {
                if s.get_at(0) == b' ' {
                    *s = s.advance(1);
                }
                board.0[i].0[j] = parse_int_fast::<_, 1, 2>(s);
            }
        }
        *s = s.advance(1);
        board
    }
}

impl Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..N {
            write!(f, "{:?}", self.0[i])?;
            if i != N - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

fn sum_unmarked(board: &Board) -> u32 {
    let mut sum = 0;
    for i in 0..N {
        for j in 0..N {
            let x = board.0[i].0[j];
            if x != 127 {
                sum += x as u32;
            }
        }
    }
    sum
}

#[inline]
pub fn part1(mut s: &[u8]) -> u32 {
    let numbers = parse_numbers(&mut s);
    let mut boards = Vec::with_capacity(1 << 7);
    while s.len() > 1 {
        boards.push(Board::parse(&mut s));
    }
    for &number in &numbers {
        for board in &mut boards {
            for i in 0..N {
                for j in 0..N {
                    let el = &mut board.0[i].0[j];
                    if *el == number {
                        *el = 127;
                    }
                    for i in 0..N {
                        let (mut n_row, mut n_col) = (0, 0);
                        for j in 0..N {
                            n_row += (board.0[i].0[j] == 127) as usize;
                            n_col += (board.0[j].0[i] == 127) as usize;
                        }
                        if n_row == N || n_col == N {
                            return sum_unmarked(board) * (number as u32);
                        }
                    }
                }
            }
        }
    }
    0
}

#[inline]
pub fn part2(mut s: &[u8]) -> u32 {
    let numbers = parse_numbers(&mut s);
    let mut boards = Vec::with_capacity(1 << 7);
    while s.len() > 1 {
        boards.push(Board::parse(&mut s));
    }
    let n_boards = boards.len();
    let (mut is_win, mut n_winners) = (vec![false; n_boards], 0);
    for &number in &numbers {
        for (k, board) in boards.iter_mut().enumerate() {
            if is_win[k] {
                continue;
            }
            for i in 0..N {
                for j in 0..N {
                    let el = &mut board.0[i].0[j];
                    if *el == number {
                        *el = 127;
                    }
                    for i in 0..N {
                        let (mut n_row, mut n_col) = (0, 0);
                        for j in 0..N {
                            n_row += (board.0[i].0[j] == 127) as usize;
                            n_col += (board.0[j].0[i] == 127) as usize;
                        }
                        if n_row == N || n_col == N {
                            n_winners += 1;
                            is_win[k] = true;
                            if n_winners == n_boards {
                                return sum_unmarked(board) * (number as u32);
                            }
                            break;
                        }
                    }
                    if is_win[k] {
                        break;
                    }
                }
                if is_win[k] {
                    break;
                }
            }
        }
    }
    0
}

#[test]
fn test_day04_part1() {
    assert_eq!(part1(input()), 28082);
}

#[test]
fn test_day04_part2() {
    assert_eq!(part2(input()), 8224);
}
