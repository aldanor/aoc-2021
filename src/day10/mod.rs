use arrayvec::ArrayVec;

use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

type Stack = ArrayVec<u8, 256>;

#[inline]
fn build_stack(s: &[u8]) -> (Option<u8>, Stack) {
    let mut stack = ArrayVec::<u8, 256>::new();
    for &c in s {
        match c {
            b'(' | b'<' | b'[' | b'{' => {
                unsafe { stack.push_unchecked(c) };
            }
            _ => {
                let o =
                    stack.pop().unwrap_or_else(|| unsafe { core::hint::unreachable_unchecked() });
                if c.wrapping_sub(o) > 2 {
                    return (Some(c), stack);
                }
            }
        }
    }
    (None, stack)
}

const fn part1_scores() -> [usize; 256] {
    let mut scores = [0_usize; 256];
    scores[b')' as usize] = 3;
    scores[b']' as usize] = 57;
    scores[b'}' as usize] = 1197;
    scores[b'>' as usize] = 25137;
    scores
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    const SCORES: [usize; 256] = part1_scores();
    let mut answer = 0;
    while s.len() > 1 {
        let n = s.memchr(b'\n');
        answer += SCORES.get_at(build_stack(&s[..n]).0.unwrap_or(0) as _);
        s = s.advance(n + 1);
    }
    answer
}

const fn part2_scores() -> [usize; 256] {
    let mut scores = [0; 256];
    scores[b'(' as usize] = 1;
    scores[b'[' as usize] = 2;
    scores[b'{' as usize] = 3;
    scores[b'<' as usize] = 4;
    scores
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    const SCORES: [usize; 256] = part2_scores();
    let mut scores = ArrayVec::<usize, 256>::new();
    while s.len() > 1 {
        let n = s.memchr(b'\n');
        let (ch, stack) = build_stack(&s[..n]);
        if ch.is_none() {
            scores.push(stack.iter().rev().fold(0, |acc, &ch| acc * 5 + SCORES.get_at(ch as _)));
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
