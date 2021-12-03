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
    /*
    We will keep track of bit counts in a 'binary pyramid'.

    Here is the representation of rows:
        [u16; 2]        - # of 0s/1s in highest bit
        [u16; 4]        - # of 0s/1s in the first two bits
        [u16; 8]        - # of 0s/1s in the first three bits
        ...
        [u16; 1 << N]   - # of 0s/1s in all bits

    Total number of values in the pyramid:
        1 + 2 + 4 + ... N = (1 << (N + 1)) - 1
     */

    const N: usize = 12;
    let mut bits = [0_u16; 1 << (N + 1)];
    while s.len() > 1 {
        let mut row = &mut bits[..];
        let mut value = 0_u16;
        for i in 0..N {
            let bit = (s.get_at(i) == b'1') as u16;
            value = (value << 1) | bit;
            row[value as usize] += 1;
            row = &mut row[(1 << (i + 1))..];
        }
        s = s.advance(13);
    }

    /*
    Truth table for 'most':
        a := num0 != 0      (there exist numbers with 0 in this bit)
        b := num1 != 0      (there exist numbers with 1 in this bit)
        c := num1 >= num0   (there are more or equal numbers with 1 than 0 in this bit)

    a b c
    0 0 0     ? (n/a)
    0 0 1     ? (n/a)
    0 1 0     1
    0 1 1     1
    1 0 0     0
    1 0 1     0
    1 1 0     0
    1 1 1     1

    We'll set the first two values to 1 to derive the simplest expression:
        ~a || (b && c)

    Similarly for 'least', we set
        c := num1 < num0 (???)
     */
    let truth_table =
        |nonzero0: bool, nonzero1: bool, condition: bool| !nonzero0 || (nonzero1 && condition);

    let mut row = &bits[..];
    let (mut index_most, mut index_least) = (0, 0);
    for i in 0..N {
        index_most <<= 1;
        let (most0, most1) = (row[index_most], row[index_most + 1]);
        index_most |= truth_table(most0 != 0, most1 != 0, most1 >= most0) as usize;
        index_least <<= 1;
        let (least0, least1) = (row[index_least], row[index_least + 1]);
        index_least |= truth_table(least0 != 0, least1 != 0, least1 < least0) as usize;
        row = &row[(1 << (i + 1))..];
    }
    (index_most as u32) * (index_least as u32)
}

#[test]
fn test_day03_part1() {
    assert_eq!(part1(input()), 3959450);
}

#[test]
fn test_day03_part2() {
    assert_eq!(part2(input()), 7440311);
}
