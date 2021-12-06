use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
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

#[inline]
pub fn part1(s: &[u8]) -> usize {
    solve::<80>(s)
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    solve::<256>(s)
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 345387);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 1574445493136);
}
