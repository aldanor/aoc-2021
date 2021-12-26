use std::iter;
use std::mem;
use std::str;

use crate::utils::*;

type T = i16;
type Point = (T, T);

const N_CHARS: usize = 8;
const CHAR_W: usize = 4;
const CHAR_H: usize = 6;

#[inline]
fn fold_1d(coord: &mut T, pivot: T) {
    if *coord > pivot {
        *coord = 2 * pivot - *coord
    }
}

#[derive(Debug, Copy, Clone)]
struct Fold {
    pivot: T,
    is_y: bool,
}

impl Fold {
    #[inline]
    pub fn parse(s: &mut &[u8]) -> Option<Self> {
        if s.len() <= 1 {
            None
        } else {
            *s = s.advance(11);
            let is_y = s.get_first() == b'y';
            *s = s.advance(2);
            let coord = parse_int_fast::<T, 1, 4>(s);
            Some(Self { pivot: coord, is_y })
        }
    }

    #[inline]
    pub fn apply<'a>(&self, points: impl IntoIterator<Item = &'a mut Point>) {
        if self.is_y {
            for point in points {
                fold_1d(&mut point.1, self.pivot);
            }
        } else {
            for point in points {
                fold_1d(&mut point.0, self.pivot);
            }
        }
    }
}

fn parse_points(s: &mut &[u8]) -> Vec<(T, T)> {
    let mut points = Vec::with_capacity(1 << 10);
    while s.get_first() != b'\n' {
        let x = parse_int_fast::<T, 1, 4>(s);
        let y = parse_int_fast::<T, 1, 4>(s);
        points.push((x, y));
    }
    *s = s.advance(1);
    points
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(mut s: &[u8]) -> usize {
    let mut points = parse_points(&mut s);
    Fold::parse(&mut s).unwrap().apply(points.iter_mut());

    assert_eq!(mem::size_of::<Point>(), mem::size_of::<u32>());
    let mut points: Vec<u32> = unsafe { mem::transmute(points) }; // so that sort is faster
    points.sort_unstable();

    points.iter().fold((u32::MAX, 0), |(prev, sum), &x| (x, sum + (x != prev) as usize)).1
}

#[allow(unused)]
fn display_points<'a>(points: impl IntoIterator<Item = &'a Point>) -> String {
    let points = points.into_iter().collect::<Vec<_>>();
    let xmin = points.iter().map(|&p| p.0).min().unwrap();
    let ymin = points.iter().map(|&p| p.1).min().unwrap();
    let xmax = points.iter().map(|&p| p.0).max().unwrap();
    let ymax = points.iter().map(|&p| p.1).max().unwrap();
    let w = (xmax - xmin + 1) as usize;
    let h = (ymax - ymin + 1) as usize;
    let init = iter::repeat(b'.').take(w).collect::<Vec<_>>();
    let mut byte_lines = iter::repeat_with(|| init.clone()).take(h).collect::<Vec<_>>();
    for &point in points {
        byte_lines[(point.1 - ymin) as usize][(point.0 - xmin) as usize] = b'#';
    }
    let str_lines = byte_lines.into_iter().map(|line| str::from_utf8(&line).unwrap().to_owned());
    str_lines.fold(String::new(), |a, b| format!("{}\n{}", a, b))
}

pub fn extract_letters<'a>(points: impl IntoIterator<Item = &'a Point> + Copy) -> [u8; N_CHARS] {
    let (xmin, xmax, ymin, ymax) =
        points.into_iter().fold((T::MAX, T::MIN, T::MAX, T::MIN), |acc, &(x, y)| {
            (acc.0.min(x), acc.1.max(x), acc.2.min(y), acc.3.max(y))
        });
    let w = (xmax - xmin + 1) as usize;
    let h = (ymax - ymin + 1) as usize;
    assert_eq!(w, 39);
    assert_eq!(h, 6);

    let mut pixels = [[[false; CHAR_W]; CHAR_H]; N_CHARS];
    for &(x, y) in points {
        let x = (x - xmin) as usize;
        let y = (y - ymin) as usize;
        let (i, x) = (x / 5, x % 5);
        pixels[i][y][x] = true;
    }

    let mut letters_encoded = [0_u32; N_CHARS];
    for (i, &pixel) in pixels.iter().enumerate() {
        let mut j = 0;
        for row in pixel {
            for c in row {
                letters_encoded[i] |= (c as u32) << j;
                j += 1;
            }
        }
    }

    const KNOWN_LETTERS: [(u32, u8); 7] = [
        (0x00f1171f, b'E'),
        (0x00117997, b'P'),
        (0x00699999, b'U'),
        (0x00f11111, b'L'),
        (0x00117997, b'P'),
        (0x00799797, b'B'),
        (0x00957997, b'R'),
    ];

    let mut out = [0_u8; N_CHARS];
    for (i, &e) in letters_encoded.iter().enumerate() {
        for (known_encoded, known_char) in KNOWN_LETTERS {
            if e == known_encoded {
                out[i] = known_char;
            }
        }
    }
    out
}

pub fn part2(mut s: &[u8]) -> String {
    let mut points = parse_points(&mut s);
    while let Some(fold) = Fold::parse(&mut s) {
        fold.apply(&mut points);
    }
    str::from_utf8(&extract_letters(&points)).unwrap().to_owned()
}

#[test]
fn test_day13_part1() {
    assert_eq!(part1(input()), 770);
}

#[test]
fn test_day13_part2() {
    assert_eq!(part2(input()), "EPUELPBR");
}
