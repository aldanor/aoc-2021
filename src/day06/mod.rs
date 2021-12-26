use crate::utils::*;

type T = usize;

#[inline(always)]
const fn row_dot<const N: usize>(x: &[T; N], y: &[T; N]) -> T {
    let (mut i, mut v) = (0, 0);
    while i < N {
        v += x[i] * y[i];
        i += 1;
    }
    v
}

#[inline(always)]
const fn dot<const N: usize>(m: &[[T; N]; N], x: &[T; N]) -> [T; N] {
    let (mut i, mut v) = (0, [0; N]);
    while i < N {
        v[i] = row_dot(&m[i], x);
        i += 1;
    }
    v
}

#[inline]
pub fn solve16<const N16: usize>(mut s: &[u8]) -> usize {
    let mut v = [0_usize; 9];
    while s.len() > 1 {
        v[(s.get_first() - b'0') as usize] += 1;
        s = s.advance(2);
    }

    const M: [[usize; 9]; 9] = [
        [2, 0, 1, 0, 0, 0, 0, 1, 0],
        [0, 2, 0, 1, 0, 0, 0, 0, 1],
        [1, 0, 2, 0, 1, 0, 0, 0, 0],
        [0, 1, 0, 2, 0, 1, 0, 0, 0],
        [0, 0, 1, 0, 2, 0, 1, 0, 0],
        [1, 0, 0, 1, 0, 2, 0, 1, 0],
        [0, 1, 0, 0, 1, 0, 2, 0, 1],
        [1, 0, 0, 0, 0, 1, 0, 1, 0],
        [0, 1, 0, 0, 0, 0, 1, 0, 1],
    ];

    for _ in 0..N16 {
        v = dot(&M, &v);
    }
    v.into_iter().sum()
}

#[inline]
pub fn solve<const N: usize>(mut s: &[u8]) -> usize {
    let mut counts = [0_usize; 9];
    while s.len() > 1 {
        counts[(s.get_first() - b'0') as usize] += 1;
        s = s.advance(2);
    }
    for _ in 0..N {
        let new = counts[0];
        counts.rotate_left(1);
        counts[6] += new;
    }
    counts.into_iter().sum()
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> usize {
    solve16::<{ 80 >> 4 }>(s)
}

pub fn part2(s: &[u8]) -> usize {
    solve::<256>(s)
}

#[test]
fn test_day06_part1() {
    assert_eq!(part1(input()), 345387);
}

#[test]
fn test_day06_part2() {
    assert_eq!(part2(input()), 1574445493136);
}
