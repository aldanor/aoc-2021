use std::ops::Range;

use num_integer::div_ceil;

use crate::utils::*;

type T = i32;

fn parse(mut s: &[u8]) -> ((T, T), (T, T)) {
    s = s.advance(15);
    let x0 = parse_int_fast::<T, 1, 3>(&mut s);
    s = s.advance(1);
    let x1 = parse_int_fast::<T, 1, 3>(&mut s);
    s = s.advance(3);
    assert_eq!(s.get_at(0), b'-');
    s = s.advance(1);
    let y0 = parse_int_fast::<T, 1, 3>(&mut s);
    s = s.advance(1);
    assert_eq!(s.get_at(0), b'-');
    s = s.advance(1);
    let y1 = parse_int_fast::<T, 1, 3>(&mut s);
    ((x0, x1), (-y0, -y1))
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> T {
    let ((_xmin, _xmax), (ymin, _ymax)) = parse(s);
    (ymin * (ymin + 1)) / 2
}

#[inline]
fn dio(x: T) -> f64 {
    // xv * (xv + 1) / 2 = x => xv = (-1 + sqrt(1 + 8x)) / 2
    (-1. + (1. + 8. * x as f64).sqrt()) / 2.
}

#[inline]
fn v_range(min: T, max: T, n: T, lb: T) -> Range<T> {
    let vmin = div_ceil(min, n).max(lb);
    let mut vmax = vmin;
    let mut bound = vmax * n;
    while bound <= max {
        vmax += 1;
        bound += n;
    }
    vmin..vmax
}

pub fn part2(s: &[u8]) -> usize {
    let ((xmin, xmax), (ymin, ymax)) = parse(s);

    // range of vx where eventually vx = 0 and we hit the box on x coordinate
    let (vx_stop_min, vx_stop_max) = (dio(xmin).ceil() as T, dio(xmax).floor() as T);

    let mut n_paths: T = 0;

    // first, handle the 'stop' range of vx where the trajectory eventually hits vx = 0
    for vx in vx_stop_min..=vx_stop_max {
        // fast-forward to vx=0 point first
        let mut n = vx;
        // now backtrack within the box
        let mut x = (vx * (vx + 1)) / 2;
        // println!("vx={}, n={}, x={}", vx, n, x);
        let mut vx_t = 0;
        loop {
            if x - vx_t - 1 < xmin {
                break;
            }
            vx_t += 1;
            x -= vx_t;
            n -= 1;
        }
        let mut acc = (n * (n - 1)) / 2;
        let mut vymax_prev = T::MIN;
        // for each n forward, search for all valid vy
        'outer: for n in n.. {
            let vyr = v_range(ymin + acc, ymax + acc, n, T::MIN);
            if vyr.is_empty() {
                acc += n;
                continue;
            }
            let (vymin, vymax) = (vyr.start, vyr.end);
            let dvy = vymax - vymin;
            n_paths += dvy;
            if vymin < vymax_prev {
                n_paths -= vymax_prev - vymin;
            }
            if vymax > -ymin - 1 {
                break 'outer;
            }
            vymax_prev = vymax;
            acc += n;
        }
    }

    // handle the 'continuous' range
    let mut acc = 0;
    let (mut vxmin_prev, mut vymax_prev) = (T::MAX, T::MIN);
    for n in 1.. {
        // search for valid vx
        let vxr = v_range(xmin + acc, xmax + acc, n, (vx_stop_max + 1).max(n));
        if vxr.is_empty() {
            break;
        }
        let (vxmin, vxmax) = (vxr.start, vxr.end);
        let dvx = vxmax - vxmin;
        // search for valid vy
        let vyr = v_range(ymin + acc, ymax + acc, n, T::MIN);
        if vyr.is_empty() {
            continue;
        }
        let (vymin, vymax) = (vyr.start, vyr.end);
        let dvy = vymax - vymin;
        // count the number of trajectories and account for overlaps
        n_paths += dvx * dvy;
        if vxmax > vxmin_prev && vymin < vymax_prev {
            n_paths -= (vxmax - vxmin_prev) * (vymax_prev - vymin);
        }
        vxmin_prev = vxmin;
        vymax_prev = vymax;
        acc += n;
    }

    n_paths as _
}

#[test]
fn test_day17_part1() {
    assert_eq!(part1(input()), 3655);
}

#[test]
fn test_day17_part2() {
    assert_eq!(part2(input()), 1447);
}
