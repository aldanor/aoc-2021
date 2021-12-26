use std::fmt::{self, Debug};

use crate::utils::*;

const N: usize = 5;

type Number = u8;
type Score = u32;

#[inline]
fn parse_numbers(s: &mut &[u8]) -> Vec<Number> {
    let mut numbers = Vec::with_capacity(1 << 7);
    while s.get_at(0) != b'\n' {
        numbers.push(parse_int_fast::<_, 1, 2>(s));
    }
    *s = s.advance(2);
    numbers
}

type Board = [[Number; N]; N];

fn parse_board(s: &mut &[u8]) -> Board {
    let mut board = Board::default();
    for i in 0..N {
        for j in 0..N {
            if s.get_at(0) == b' ' {
                *s = s.advance(1);
            }
            let num = parse_int_fast::<Number, 1, 2>(s);
            board[i][j] = num;
        }
    }
    *s = s.advance(1);
    board
}

fn time_to_win(board: &Board, draw_times: &[usize]) -> usize {
    let (mut max_cols, mut max_rows) = ([0; N], [0; N]);
    for i in 0..N {
        for j in 0..N {
            let draw_time = unsafe {
                *draw_times.get_unchecked(*board.get_unchecked(i).get_unchecked(j) as usize)
            };
            max_cols[j] = max_cols[j].max(draw_time);
            max_rows[i] = max_rows[i].max(draw_time);
        }
    }
    max_cols.into_iter().min().unwrap_or(0).min(max_rows.into_iter().min().unwrap_or(0))
}

fn solve(mut s: &[u8], ttw_is_better: impl Fn(usize, usize) -> bool, ttw_init: usize) -> Score {
    // Credits for the algorithm idea: @orlp

    let numbers = parse_numbers(&mut s);
    let mut draw_times = [0; 1 << 8];
    for (t, &num) in numbers.iter().enumerate() {
        draw_times[num as usize] = t;
    }
    let mut boards = Vec::with_capacity(100);
    let (mut ttw_best, mut idx_best) = (ttw_init, usize::MAX);
    while s.len() > 1 {
        let board = parse_board(&mut s);
        let ttw = time_to_win(&board, &draw_times);
        if ttw_is_better(ttw, ttw_best) {
            ttw_best = ttw;
            idx_best = boards.len();
        }
        boards.push(board);
    }

    let board = boards[idx_best];
    let mut sum_unmasked = 0;
    for i in 0..N {
        for j in 0..N {
            let num = board[i][j];
            if draw_times.get_at(num as usize) > ttw_best {
                sum_unmasked += num as Score;
            }
        }
    }
    sum_unmasked * numbers.get_at(ttw_best) as Score
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> Score {
    solve(s, |ttw, ttw_best| ttw < ttw_best, usize::MAX)
}

pub fn part2(s: &[u8]) -> Score {
    solve(s, |ttw, ttw_best| ttw > ttw_best, 0)
}

// Below is the original mask-based solution (12us / 24us).

type Mask = u128;

#[derive(Clone, Copy, Default)]
pub struct BoardMasks {
    masks: [Mask; N * 2], // N horizontal, N vertical
}

impl BoardMasks {
    pub fn parse(s: &mut &[u8]) -> Self {
        let mut board = Self::default();
        for i in 0..N {
            for j in 0..N {
                if s.get_at(0) == b' ' {
                    *s = s.advance(1);
                }
                let num = parse_int_fast::<u8, 1, 2>(s);
                let mask = 1_u128 << (1 + num); // we add 1 to all values so 0 is free
                board.masks[i] |= mask;
                board.masks[N + j] |= mask;
            }
        }
        *s = s.advance(1);
        board
    }

    pub fn score(&self, number: Number) -> Score {
        let mut sum = 0;
        for i in 0..N {
            let mut mask = self.masks[i];
            let mut pos = 0;
            for _ in 0..mask.count_ones() {
                let num = mask.trailing_zeros();
                pos += num + 1;
                sum += pos - 2; // because we added 1 to all numbers
                mask >>= num + 1;
            }
        }
        sum * (number as Score)
    }
}

impl Debug for BoardMasks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..N {
            let row_mask = self.masks[i];
            for j in 0..N {
                let col_mask = self.masks[N + j];
                let num = (row_mask & col_mask).trailing_zeros() - 1; // make sure to subtract 1
                write!(f, "{:>w$}", num, w = 2 + (j != 0) as usize)?;
            }
            if i != N - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

fn parse_numbers_and_masks(mut s: &[u8]) -> (Vec<Number>, Vec<BoardMasks>) {
    let numbers = parse_numbers(&mut s);
    let mut boards = Vec::with_capacity(1 << 7);
    while s.len() > 1 {
        boards.push(BoardMasks::parse(&mut s));
    }
    (numbers, boards)
}

#[inline]
pub fn part1_u128(s: &[u8]) -> Score {
    let (numbers, mut boards) = parse_numbers_and_masks(s);
    for (k, &num) in numbers.iter().enumerate() {
        let m = !(1_u128 << (num + 1));
        for board in &mut boards {
            for mask in &mut board.masks {
                *mask &= m;
                if k >= N && *mask == 0 {
                    return board.score(num);
                }
            }
        }
    }
    0
}

#[inline]
pub fn part2_u128(s: &[u8]) -> Score {
    let (numbers, mut boards) = parse_numbers_and_masks(s);
    let n_boards = boards.len();
    let (mut n_winners, mut is_win) = (0, vec![false; n_boards]);
    for (k, &num) in numbers.iter().enumerate() {
        let m = !(1_u128 << (num + 1));
        'board: for (j, board) in boards.iter_mut().enumerate() {
            if is_win[j] {
                continue;
            }
            for mask in &mut board.masks {
                *mask &= m;
                if k >= N && *mask == 0 {
                    n_winners += 1;
                    is_win[j] = true;
                    if n_winners == n_boards {
                        return board.score(num);
                    }
                    continue 'board;
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
