use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::utils::*;

const N: usize = 10;

type Rules = [[u8; N]; N];

fn parse(mut s: &[u8]) -> (Vec<u8>, Rules) {
    let k = s.memchr(b'\n');
    let word = &s[..k];
    s = s.advance(k + 2);
    let (mut rev_map, mut n_chars, mut rules) = ([0xff; 256], 0, [[0; N]; N]);
    for _ in 0..N * N {
        assert!(!s.is_empty());
        let mut line = [0; 3];
        for (i, j) in [0, 1, 6].into_iter().enumerate() {
            let c = s.get_at(j);
            if rev_map[c as usize] == 0xff {
                rev_map[c as usize] = n_chars as u8;
                n_chars += 1;
            }
            line[i] = rev_map[c as usize];
        }
        let [x, y, z] = line;
        rules[x as usize][y as usize] = z;
        s = s.advance(8);
    }
    let word = word.iter().map(|c| rev_map[*c as usize]).collect();
    (word, rules)
}

fn solve<T>(word: &[u8], rules: &Rules, n_iter: usize) -> T
where
    T: Sum
        + Default
        + Copy
        + From<u8>
        + AddAssign
        + SubAssign
        + Sub<Output = T>
        + Ord
        + Debug
        + Add<Output = T>,
{
    let mut matrix = [[T::default(); N]; N];
    for i in 1..word.len() {
        matrix[word[i - 1] as usize][word[i] as usize] += T::from(1_u8);
    }

    let mut next = matrix;
    for _ in 0..n_iter {
        for left in 0..N {
            for right in 0..N {
                let c = rules[left][right] as usize;
                let n = matrix[left][right];
                next[left][right] -= n;
                next[left][c] += n;
                next[c][right] += n;
            }
        }
        matrix = next;
    }

    let mut counts = matrix.map(|row| row.into_iter().sum::<T>());
    counts.sort_unstable();
    counts[N - 1] - counts[0] + T::from(1_u8)
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> i16 {
    let (word, rules) = parse(s);
    solve(&word, &rules, 10)
}

pub fn part2(s: &[u8]) -> i64 {
    let (word, rules) = parse(s);
    solve(&word, &rules, 40)
}

#[test]
fn test_day14_part1() {
    assert_eq!(part1(input()), 5656);
}

#[test]
fn test_day14_part2() {
    assert_eq!(part2(input()), 12271437788530);
}
