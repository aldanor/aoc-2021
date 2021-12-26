use arrayvec::ArrayVec;

use crate::utils::*;

type T = i16; // element type
type C = i32; // cost function

const N: usize = 1 << 10;

#[inline]
fn parse_input(mut s: &[u8]) -> ArrayVec<T, N> {
    let mut x = ArrayVec::new_const();
    while s.len() > 1 {
        unsafe {
            x.push_unchecked(parse_int_fast_skip_custom::<_, 1, 4, 1>(&mut s));
        }
    }
    debug_assert_eq!(x.len() % 2, 0);
    x
}

#[inline]
fn median(x: &mut [T]) -> T {
    let n = x.len();
    let (_, mid, _) = x.select_nth_unstable(n >> 1);
    *mid
}

#[inline]
fn cost1(x: &[T], m: T) -> C {
    x.iter().map(|&x| (x - m).abs() as C).sum()
}

#[inline]
fn mean_floor(x: &[T]) -> T {
    (x.iter().map(|&x| x as C).sum::<C>() / x.len() as C) as T
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(mut s: &[u8]) -> C {
    /*
    Cost function:

    c(x_i) = abs(x_i - m)

    => L1 problem, solution is m := median(X)
     */
    let mut x = parse_input(s);
    let m = median(&mut x);
    cost1(&x, m)
}

pub fn part2(mut s: &[u8]) -> C {
    /*
    Cost function:

    c(x_i) =
        x_i == m => 0
        x_i > m => 1 + 2 + 3 + ... + (x_i - m)
        x_i < m => 1 + 2 + 3 + ... + (m - x_i)

        = k * (k + 1) / 2, where k = abs(x_i - m)
        = ((x_i - m) ^ 2 + abs(x_i - m)) / 2

    => mix of L1 and L2

    L1 minimum = median, L2 = minimum = mean

    sum(c'(X)) = 0:

        2 * sum(X - m) + sum(sgn(X - m)) = 0
        2 * sum(X) - 2 * m * n + sum(sgn(X - m)) = 0
        mean(X) - m + mean(sgn(X - m)) / 2 = 0
        => m \in [mean(X) - 1/2; mean(X) + 1/2]
     */
    let mut x = parse_input(s);
    let m = mean_floor(&x);

    // a bit more ugly, but lets us iterate over the array once only
    let (mut c1, mut c2) = (0, 0);
    for &x in &x {
        let d1 = (x - m) as C;
        c1 += d1 * d1 + d1.abs();
        let d2 = d1 - 1;
        c2 += d2 * d2 + d2.abs();
    }

    c1.min(c2) >> 1
}

#[test]
fn test_day07_part1() {
    assert_eq!(part1(input()), 335271);
}

#[test]
fn test_day07_part2() {
    assert_eq!(part2(input()), 95851339);
}
