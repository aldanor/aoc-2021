use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Forward(i32),
    Down(i32),
    Up(i32),
}

impl Dir {
    #[inline]
    pub fn parse_next(s: &mut &[u8]) -> Option<Self> {
        if s.len() <= 1 {
            return None;
        }
        let dir = match s.get_first() {
            b'f' => {
                *s = s.advance(8);
                Self::Forward(s.get_digit() as _)
            }
            b'd' => {
                *s = s.advance(5);
                Self::Down(s.get_digit() as _)
            }
            _ /* up */ => {
                *s = s.advance(3);
                Self::Up(s.get_digit() as _)
            }
        };
        *s = s.advance(2);
        Some(dir)
    }
}

#[inline]
pub fn part1(mut s: &[u8]) -> i32 {
    let (mut horizontal, mut depth) = (0, 0);
    while let Some(dir) = Dir::parse_next(&mut s) {
        match dir {
            Dir::Forward(x) => horizontal += x,
            Dir::Down(x) => depth += x,
            Dir::Up(x) => depth -= x,
        }
    }
    horizontal * depth
}

#[inline]
pub fn part2(mut s: &[u8]) -> i32 {
    let (mut horizontal, mut depth, mut aim) = (0, 0, 0);
    while let Some(dir) = Dir::parse_next(&mut s) {
        match dir {
            Dir::Forward(x) => {
                horizontal += x;
                depth += aim * x;
            }
            Dir::Down(x) => aim += x,
            Dir::Up(x) => aim -= x,
        }
    }
    horizontal * depth
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 1427868);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 1568138742);
}
