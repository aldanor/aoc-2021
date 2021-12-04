use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> u32 {
    use core_simd::{u16x16, u16x32};
    const N: usize = 12;
    let mut count = 0_u16;
    let mut total = u16x32::splat(0);
    while s.len() >= 32 {
        let input: [u8; 32] = unsafe { *s.as_ptr().cast() };
        let mut buf = [0; 32];
        for i in 0..32 {
            buf[i] = input[i] as u16;
        }
        total += u16x32::from(buf);
        s = s.advance(26);
        count += 2;
    }
    let mut bits = [0_u16; 16];
    let total = total.to_array();
    for i in 0..N {
        bits[i] += total[i] + total[N + 1 + i];
    }
    while s.len() > 1 {
        for i in 0..N {
            bits[i] += s.get_at(i) as u16;
        }
        count += 1;
        s = s.advance(13);
    }
    let bits = u16x16::from(bits);
    let threshold = u16x16::splat(((b'0' as u16) * count) + (count >> 1));
    let most_bits = bits.lanes_ge(threshold).to_array();
    let least_bits = bits.lanes_lt(threshold).to_array();
    let (mut most, mut least) = (0_u16, 0_u16);
    for i in 0..N {
        most = most << 1 | most_bits[i] as u16;
        least = least << 1 | least_bits[i] as u16;
    }
    most as u32 * least as u32
}

#[inline]
pub fn part2(mut s: &[u8]) -> u32 {
    const N: usize = 12;
    let n = s.len() / (N + 1);
    let mut counts = [0_u8; 1 << N];
    for _ in 0..n {
        let mut value = 0;
        for i in 0..N {
            let bit = s.get_at(i) == b'1';
            value = (value << 1) | (bit as usize);
        }
        counts[value] += 1;
        s = s.advance(N + 1);
    }

    let mut offset_most = 0;
    let mut offset_least = 0;
    let mut total_most = n as u16;
    let mut total_least = n as u16;
    let mut size = 1 << N;
    for _ in 0..N {
        size >>= 1;
        let (mut most0, mut least0) = (0, 0);
        for j in 0..size {
            most0 += counts[offset_most + j] as u16;
            least0 += counts[offset_least + j] as u16;
        }
        let most1 = total_most - most0;
        let least1 = total_least - least0;
        total_most = if most0 == 0 || (most1 != 0 && most1 >= most0) {
            offset_most += size;
            most1
        } else {
            most0
        };
        total_least = if least0 == 0 || (least1 != 0 && least1 < least0) {
            offset_least += size;
            least1
        } else {
            least0
        };
    }
    offset_most as u32 * offset_least as u32
}

#[test]
fn test_day03_part1() {
    assert_eq!(part1(input()), 3959450);
}

#[test]
fn test_day03_part2() {
    assert_eq!(part2(input()), 7440311);
}
