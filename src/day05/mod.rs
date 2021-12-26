mod part1;
mod part2;
mod projection; // used in part2

use crate::utils::*;

pub type Coord = i16;

#[inline]
pub fn minmax<T: Copy + PartialOrd>(a: T, b: T) -> (T, T) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

#[inline]
pub fn parse_num<const SKIP: usize>(s: &mut &[u8]) -> Coord {
    parse_int_fast_skip_custom::<Coord, 1, 3, SKIP>(s)
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> usize {
    self::part1::solve(s)
}

pub fn part2(s: &[u8]) -> usize {
    self::part2::solve(s)
}

#[test]
fn test_day05_part1() {
    assert_eq!(part1(input()), 5280);
}

#[test]
fn test_day05_part2() {
    assert_eq!(part2(input()), 16716);
}
