use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
fn count_increasing<T: Integer, const N: usize>(mut s: &[u8]) -> u16 {
    let mut count = 0;
    let mut buf = [T::default(); N];
    for i in 0..N - 1 {
        buf[i] = parse_int_fast::<T, 1, 4>(&mut s);
    }
    while s.len() > 1 {
        let num = parse_int_fast::<T, 1, 4>(&mut s);
        count += u16::from(num > buf[0]);
        buf[N - 1] = num;
        for i in 1..N {
            buf[i - 1] = buf[i];
        }
    }
    count
}

#[inline]
pub fn part1(s: &[u8]) -> u16 {
    count_increasing::<u16, 2>(s) as _
}

#[inline]
pub fn part2(s: &[u8]) -> u16 {
    count_increasing::<u16, 4>(s) as _
}

#[test]
fn test_day01_part1() {
    assert_eq!(part1(input()), 1475);
}

#[test]
fn test_day01_part2() {
    assert_eq!(part2(input()), 1516);
}
