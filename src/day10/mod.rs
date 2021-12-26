use arrayvec::ArrayVec;

use crate::utils::*;

const fn map() -> [u8; 256] {
    let mut map = [0; 256];
    map[b'(' as usize] = 2;
    map[b')' as usize] = 3;
    map[b'[' as usize] = 4;
    map[b']' as usize] = 5;
    map[b'{' as usize] = 6;
    map[b'}' as usize] = 7;
    map[b'<' as usize] = 8;
    map[b'>' as usize] = 9;
    map
}

#[inline]
fn build_stack<'a>(s: &[u8], stack: &'a mut [u8]) -> (u8, &'a [u8]) {
    const MAP: [u8; 256] = map();
    let mut stack = UnsafeStack::new(stack);
    for &c in s {
        let c = MAP.get_at(c as _);
        if c & 1 == 0 {
            stack.push(c);
        } else if c != stack.pop() | 1 {
            return (c, stack.into_slice());
        }
    }
    (0, stack.into_slice())
}

const fn part1_scores() -> [u32; 10] {
    let mut scores = [0; 10];
    scores[3] = 3;
    scores[5] = 57;
    scores[7] = 1197;
    scores[9] = 25137;
    scores
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(mut s: &[u8]) -> u32 {
    const SCORES: [u32; 10] = part1_scores();
    let mut stack = [0; 128];
    let mut answer = 0;
    while s.len() > 1 {
        let n = s.memchr(b'\n');
        let c = build_stack(&s[..n], &mut stack).0;
        answer += SCORES.get_at(c as _);
        s = s.advance(n + 1);
    }
    answer
}

pub fn part2(mut s: &[u8]) -> usize {
    let mut scores = ArrayVec::<usize, 256>::new();
    let mut stack = [0; 128];
    while s.len() > 1 {
        let n = s.memchr(b'\n');
        let (c, a) = build_stack(&s[..n], &mut stack);
        if c == 0 {
            scores.push(a.iter().rev().fold(0, |acc, &c| acc * 5 + (c >> 1) as usize));
        }
        s = s.advance(n + 1);
    }
    let n = scores.len();
    *scores.select_nth_unstable(n >> 1).1
}

#[test]
fn test_day10_part1() {
    assert_eq!(part1(input()), 166191);
}

#[test]
fn test_day10_part2() {
    assert_eq!(part2(input()), 1152088313);
}
