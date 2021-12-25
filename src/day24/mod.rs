use crate::utils::*;

use regex::bytes::Regex;

type D = i16;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

/*
The following pattern repeats 14 times:

    inp w
    mul x 0
    add x z
    mod x 26
    div z (a: in [1, 26])
    add x (b: -16..=0 or 10..)
    eql x w
    eql x 0
    mul y 0
    add y 25
    mul y x
    add y 1
    mul z y
    mul y 0
    add y w
    add y (c: in 0..=16)
    mul y x
    add z y

If we decode it:

    w = input()
    x = (z % 26 + b) != w
    z = z / a * (25 * x + 1) + (w + c) * x

Or:

    w = input()
    if (z % 26 + b) == w:
        z = z / a
    else:
        z = (z / a) * 26 + (w + c)

Since all div/mod operations are in base 26, z can be treated as a base-26 integer:
- if a == 26, the last digit is removed; otherwise, no digits are removed
- if x == 1, another digit is appended the end: (w + u); otherwise, no digits are added
- removal of digits happens before appending
- in order for this to always work, we need u to be in the range 0..=16
- x checks the mod of the very last digit only before any removal or appending
- in some cases x can be always 1 - specifically, if u > 9
- in the beginning, z == 0 (no digits); at the end, it's also 0, so all digits are removed
- the number and location of all pop operations is known, so we need to figure all the pop ops
- note also that if a is 26, b is always negative - so if pop happens, push is always optional
- also note that if a is 21, b is always >= 10 - if there's no pop, there's a definite push
- in the input, # pushes equals # pops/pushes => need to make sure no pushes happen during pops
 */

#[derive(Debug, Copy, Clone)]
enum Block {
    Pop(D),  // b <= 0
    Push(D), // c
}

fn parse(s: &[u8]) -> [Block; 14] {
    let re = Regex::new(include_str!("pattern.txt")).unwrap();
    let mut out = [Block::Pop(D::MIN); 14];
    let (mut n_push, mut n_pop) = (0, 0);
    for cap in re.captures_iter(s) {
        let [a, b, c] = [1, 2, 3]
            .map(|i| parse_int_fast_signed::<_, 1, 2>(&mut cap.get(i).unwrap().as_bytes()));
        assert!([1, 26].contains(&a));
        assert!((-16..=0).contains(&b) || b >= 10);
        assert!((0..=16).contains(&c));
        let index = n_push + n_pop;
        out[index] = if a == 1 {
            assert!(b >= 10);
            n_push += 1;
            Block::Push(c)
        } else {
            assert!(b <= 0);
            n_pop += 1;
            Block::Pop(b) // ignore c since we need to block all pushes here
        };
        assert!(n_push >= n_pop);
    }
    assert_eq!(n_push, 7);
    assert_eq!(n_pop, 7);
    out
}

fn solve(blocks: &[Block; 14], smallest: bool) -> u64 {
    let mut stack = vec![];
    let mut w = [0; 14];
    for (i, &block) in blocks.iter().enumerate() {
        match block {
            Block::Push(c) => stack.push((i, c)),
            Block::Pop(b) => {
                // we need to ensure that (w[j] + c) + b = w[i] (here b <= 0)
                // note that j < i; to get the smallest answer, we want smallest w[j] possible
                // (and vice versa if we want the biggest number)
                // also note that (i, j) pairs will be unique and will not repeat,
                // so we can solve it precisely, right here without further ado
                let (j, c) = stack.pop().unwrap();
                let d = b + c;
                assert!((-8..=8).contains(&d)); // w[j] + d == w[i]
                w[j] = if smallest { (1 - d).max(1) } else { (9 - d).min(9) };
                w[i] = w[j] + d;
                assert!((1..=9).contains(&w[i]));
                assert!((1..=9).contains(&w[j]));
                assert_eq!(w[j] + d, w[i]);
                // println!("i={} j={} b={} c={} d={} w[i]={}, w[j]={}", i, j, b, c, d, w[i], w[j]);
            }
        }
    }
    let mut out = 0;
    for digit in w {
        out = out * 10 + digit as u64;
    }
    out
}

#[inline]
pub fn part1(mut s: &[u8]) -> u64 {
    let blocks = parse(s);
    solve(&blocks, false)
}

#[inline]
pub fn part2(mut s: &[u8]) -> u64 {
    let blocks = parse(s);
    solve(&blocks, true)
}

#[test]
fn test_day24_part1() {
    assert_eq!(part1(input()), 91398299697996);
}

#[test]
fn test_day24_part2() {
    assert_eq!(part2(input()), 41171183141291);
}
