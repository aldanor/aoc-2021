use std::ops::Range;

use crate::utils::*;

type T = i32;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

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

fn dio(x: T) -> f64 {
    // xv * (xv + 1) / 2 = x => xv = (-1 + sqrt(1 + 8x)) / 2
    (-1. + (1. + 8. * x as f64).sqrt()) / 2.
}

#[inline]
pub fn part1(mut s: &[u8]) -> T {
    let ((_xmin, _xmax), (ymin, _ymax)) = parse(s);
    (ymin * (ymin + 1)) / 2
}

struct VelocitySet {
    xmax: T,
    ymin: T,
    bits: [[u64; 4]; 256],
}

impl VelocitySet {
    pub fn new(xmax: T, ymin: T) -> Self {
        Self { xmax, ymin, bits: [[0; 4]; 256] }
    }

    #[inline]
    pub fn insert(&mut self, (vx, vy): (T, T)) {
        let ix = (self.xmax - vx) as usize;
        let iy = (vy - self.ymin) as usize;
        self.bits[ix as usize][iy >> 6] |= 1 << (iy & 0x3f);
    }

    pub fn len(&self) -> usize {
        self.bits.iter().map(|row| row.iter().map(|b| b.count_ones()).sum::<u32>()).sum::<u32>()
            as _
    }
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    let ((xmin, xmax), (ymin, ymax)) = parse(s);

    // range of vx where eventually vx = 0 and we hit the box on x coordinate
    let (vx_stop_min, vx_stop_max) = (dio(xmin).ceil() as T, dio(xmax).floor() as T);

    let mut set = VelocitySet::new(xmax, ymin);

    fn v_range(min: T, max: T, n: T, acc: T, lb: T) -> Range<T> {
        use num_integer::div_floor;
        let vmin = div_floor(min + acc + n - 1, n).max(lb);
        let vmax = div_floor(max + acc, n);
        vmin.min(vmax + 1)..(vmax + 1)
        // if vmin > vmax {
        //     0..0
        // } else {
        //     vmin..(vmax + 1)
        // }
    }

    let n_max = 200;

    // handle the stop range
    for vx_init in vx_stop_min..=vx_stop_max {
        let mut vx = vx_init;
        let mut acc = 0;
        let mut x = 0;
        for n in 1..=n_max {
            if vx != 0 {
                x += vx;
                vx -= 1;
            }
            if x < xmin {
                acc += n;
                continue;
            }
            let vy_range = v_range(ymin, ymax, n, acc, T::MIN);
            for vy in vy_range {
                set.insert((vx_init, vy));
            }
            acc += n;
        }
    }

    // handle the continuous range
    let mut acc = 0;
    for n in 1..=n_max {
        let vx_range = v_range(xmin, xmax, n, acc, (vx_stop_max + 1).max(n));
        let vy_range = v_range(ymin, ymax, n, acc, T::MIN);
        for vx in vx_range {
            for vy in vy_range.clone() {
                set.insert((vx, vy));
            }
        }
        acc += n;
    }

    set.len()
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 3655);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 1447);
}
