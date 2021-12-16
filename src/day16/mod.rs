use arrayvec::ArrayVec;

use crate::utils::*;

type BitVec = ArrayVec<u8, { 1 << 13 }>;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

const fn hex2bin_map() -> [[u8; 4]; 256] {
    // for each u8 character, if it's in 0-9A-F range, convert it to binary representation
    let mut map = [[0xff; 4]; 256];
    let mut i = 0_u8;
    while i < 16 {
        let c = if i < 10 { i + b'0' } else { i + b'A' - 10 };
        let mut j = 0_u8;
        while j < 4 {
            map[c as usize][j as usize] = (i & (1 << (3 - j)) != 0) as u8;
            j += 1;
        }
        i += 1;
    }
    map
}

const HEX2BIN_MAP: [[u8; 4]; 256] = hex2bin_map();

fn decode_bin<const N: usize>(s: &mut &[u8]) -> usize {
    assert!(N <= 16);
    let mut i = 0;
    let mut v = 0_u16;
    while i < N {
        v = (v << 1) | (s.get_at(i) as u16);
        i += 1;
    }
    *s = s.advance(N);
    v as usize
}

fn parse_hex2bin(mut s: &[u8]) -> BitVec {
    let mut v = BitVec::new();
    unsafe { v.set_len(s.len() * 4) };
    let mut ptr = v.as_mut_ptr().cast::<[u8; 4]>();
    for &c in s {
        unsafe {
            *ptr = HEX2BIN_MAP[c as usize];
            ptr = ptr.add(1);
        }
    }
    v
}

fn sum_versions(s: &mut &[u8]) -> usize {
    let version = s.get_at(0) * 4 + s.get_at(1) * 2 + s.get_at(2);
    let mut version_sum = version as usize;
    let is_literal = s.get_at(3) != 0 && s.get_at(4) == 0 && s.get_at(5) == 0;
    *s = s.advance(6);
    if is_literal {
        while s.get_at(0) != 0 {
            *s = s.advance(5);
        }
        *s = s.advance(5);
    } else {
        let is_bit_length = s.get_at(0) == 0;
        *s = s.advance(1);
        if is_bit_length {
            let n = decode_bin::<15>(s);
            let mut t = &s[..n];
            while t.len() > 6 {
                version_sum += sum_versions(&mut t);
            }
            *s = s.advance(n);
        } else {
            let n = decode_bin::<11>(s);
            for _ in 0..n {
                version_sum += sum_versions(s);
            }
        }
    }
    version_sum
}

#[inline]
fn reduce(type_id: u8, acc: usize, v: usize) -> usize {
    match type_id {
        0 => acc + v,
        1 => acc * v,
        2 => acc.min(v),
        3 => acc.max(v),
        5 => (acc > v) as _,
        6 => (acc < v) as _,
        7 => (acc == v) as _,
        _ => unsafe { core::hint::unreachable_unchecked() },
    }
}

fn eval_packet(s: &mut &[u8]) -> usize {
    let type_id = s.get_at(3) * 4 | s.get_at(4) * 2 | s.get_at(5);
    *s = s.advance(6);
    let mut v = 0_usize;
    if type_id == 4 {
        loop {
            let is_end = s.get_at(0) == 0;
            *s = s.advance(1);
            v = (v << 4) | decode_bin::<4>(s);
            if is_end {
                break;
            }
        }
    } else {
        let is_bit_length = s.get_at(0) == 0;
        *s = s.advance(1);
        if is_bit_length {
            let n = decode_bin::<15>(s);
            let mut t = &s[..n];
            v = eval_packet(&mut t);
            while t.len() > 6 {
                v = reduce(type_id, v, eval_packet(&mut t));
            }
            *s = s.advance(n);
        } else {
            let n = decode_bin::<11>(s);
            v = eval_packet(s);
            for _ in 1..n {
                v = reduce(type_id, v, eval_packet(s));
            }
        }
    }
    v
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    sum_versions(&mut parse_hex2bin(s).as_slice())
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    eval_packet(&mut parse_hex2bin(s).as_slice())
}

#[test]
fn test_day16_part1() {
    assert_eq!(part1(input()), 889);
}

#[test]
fn test_day16_part2() {
    assert_eq!(part2(input()), 739303923668);
}
