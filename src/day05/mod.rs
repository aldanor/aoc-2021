use std::cmp::Ordering;
use std::collections::BTreeSet;

use arrayvec::ArrayVec;

use crate::utils::*;

pub type Coord = u16;
pub type Interval = (Coord, Coord); // a 1-D interval, both ends are included
pub type Intervals = ArrayVec<Interval, K>; // a variable-size array of 1-D intervals
pub type IntervalSet = [Intervals; N]; // interval array for each coordinate (=index)

pub const N: usize = 1 << 10; // max coord
pub const K: usize = 8; // max number of intervals per coord

macro_rules! zeroed {
    ($ty:ty) => {
        unsafe { ::std::mem::transmute::<_, $ty>([0_u8; ::std::mem::size_of::<$ty>()]) }
    };
}

#[derive(Clone, Debug)]
pub struct Lines {
    horizontal: IntervalSet,
    vertical: IntervalSet,
}

#[inline]
pub fn minmax(a: Coord, b: Coord) -> (Coord, Coord) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

#[inline]
fn parse_num<const SKIP: usize>(s: &mut &[u8]) -> Coord {
    parse_int_fast_skip_custom::<Coord, 1, 3, SKIP>(s)
}

impl Lines {
    pub fn new() -> Self {
        zeroed!(Self)
    }

    pub fn parse(mut s: &[u8]) -> Self {
        // Returns two line arrays:
        // - the ones with known overlaps with the same type (e.g. horizontal with horizontal)
        // - the ones with no overlaps with the same type (so can only overlap with other types)
        let mut lines = Self::new();
        while s.len() > 1 {
            lines.parse_line(&mut s);
        }
        lines
    }

    #[inline]
    fn parse_line(&mut self, s: &mut &[u8]) {
        let (x0, y0) = (parse_num::<1>(s), parse_num::<4>(s));
        let (x1, y1) = (parse_num::<1>(s), parse_num::<1>(s));
        if y0 == y1 {
            let (y, (x0, x1)) = (y0 as usize, minmax(x0, x1));
            unsafe { self.horizontal.get_unchecked_mut(y).push_unchecked((x0, x1)) };
        } else if x0 == x1 {
            let (x, (y0, y1)) = (x0 as usize, minmax(y0, y1));
            unsafe { self.vertical.get_unchecked_mut(x).push_unchecked((y0, y1)) };
        }
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

fn process_parallel_overlaps_hv(intervals: &mut Intervals, overlaps: &mut Intervals) -> usize {
    // Fix intervals so there's no trivial horizontal/vertical overlap.
    // Pushes a non-overlapping set of overlaps into a separate output array.
    // We'll just use a naive algorithm here since there's not many of these
    // (otherwise we could have used a 1-D sweep line but it's an overkill).
    let mut n_overlaps = 0;
    'outer: loop {
        for i in 0..intervals.len() - 1 {
            for j in i + 1..intervals.len() {
                let (a, b) = (intervals[i], intervals[j]);
                if a.0 <= b.1 && b.0 <= a.1 {
                    let (left, c) = minmax(a.0, b.0);
                    let (d, right) = minmax(a.1, b.1);
                    let overlap = minmax(c, d);
                    n_overlaps += (overlap.1 - overlap.0 + 1) as usize;
                    intervals.remove(j);
                    intervals.remove(i);
                    overlaps.push(overlap);
                    if left < overlap.0 {
                        intervals.push((left, overlap.0 - 1));
                    }
                    if right > overlap.1 {
                        intervals.push((overlap.1 + 1, right));
                    }
                    continue 'outer;
                }
            }
        }
        break;
    }
    // we're almost done, BUT: overlaps themselves may overlap, we need to fix that too
    loop {
        let mut overlap_overlaps = Intervals::new_const();
        if overlap_overlaps.len() == 0 {
            break;
        }
        dbg!(n_overlaps, &overlap_overlaps);
        n_overlaps -= process_parallel_overlaps_hv(overlaps, &mut overlap_overlaps);
        overlaps.extend(overlap_overlaps.drain(..));
    }
    n_overlaps
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Event {
    Start(Coord),
    Vertical(Interval),
    Finish(Coord),
}

impl Event {
    #[inline]
    pub fn priority(self) -> u8 {
        match self {
            Self::Start(_) => 0,
            Self::Vertical(_) => 1,
            Self::Finish(_) => 2,
        }
    }
}

impl PartialOrd for Event {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.priority().cmp(&other.priority()))
    }
}

impl Ord for Event {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

const DEBUG: bool = false;

macro_rules! dprintln {
    ($($arg:tt)*) => (if DEBUG { println!($($arg)*) });
}

fn line_sweep_hv(horizontal: &IntervalSet, vertical: &IntervalSet) -> usize {
    let mut event_queue = zeroed!([ArrayVec<Event, K>; N]);

    for (x, intervals) in vertical.iter().enumerate() {
        for &interval in intervals {
            event_queue[x].push(Event::Vertical(interval));
        }
    }
    for (y, intervals) in horizontal.iter().enumerate() {
        for &(x0, x1) in intervals {
            event_queue[x0 as usize].push(Event::Start(y as _));
            event_queue[x1 as usize].push(Event::Finish(y as _));
        }
    }

    let mut active = BTreeSet::<Coord>::new();
    let mut n_overlaps = 0;
    for events in &mut event_queue {
        if events.len() > 1 {
            events.sort_unstable();
        }
        for &event in events as &_ {
            match event {
                Event::Start(y) => drop(active.insert(y)),
                Event::Vertical((y0, y1)) => n_overlaps += active.range(y0..=y1).count(),
                Event::Finish(y) => drop(active.remove(&y)),
            }
        }
    }
    n_overlaps
}

#[inline]
pub fn part1(s: &[u8]) -> usize {
    let Lines { mut horizontal, mut vertical } = Lines::parse(s);
    let (mut horizontal_overlaps, mut vertical_overlaps) =
        (zeroed!(IntervalSet), zeroed!(IntervalSet));

    let mut n_parallel_overlaps = 0;
    for (interval_set, overlaps) in
        [(&mut horizontal, &mut horizontal_overlaps), (&mut vertical, &mut vertical_overlaps)]
    {
        for (coord, intervals) in interval_set.iter_mut().enumerate() {
            if intervals.len() > 1 {
                n_parallel_overlaps +=
                    process_parallel_overlaps_hv(intervals, &mut overlaps[coord]);
            }
        }
    }

    let n_non_overlap_overlaps = line_sweep_hv(&horizontal, &vertical);
    let n_overlap_overlaps = line_sweep_hv(&horizontal_overlaps, &vertical_overlaps);
    let n_overlaps = n_parallel_overlaps + n_non_overlap_overlaps - n_overlap_overlaps;

    n_overlaps
}

#[inline]
pub fn part2(_s: &[u8]) -> usize {
    0
}

#[test]
fn test_day01_part1() {
    assert_eq!(part1(input()), 5280);
}

#[test]
fn test_day01_part2() {
    assert_eq!(part2(input()), 0);
}

// #[inline]
// fn sort_intervals(intervals: &mut [Interval]) {
//     let swap_if_greater = |intervals: &mut [Interval], i, j| {
//         if intervals.get_at(i).0 > intervals.get_at(j).0 {
//             intervals.swap(i, j);
//         }
//     };
//     match intervals.len() {
//         0 | 1 => (),
//         2 => {
//             swap_if_greater(intervals, 0, 1);
//         }
//         3 => {
//             swap_if_greater(intervals, 0, 2);
//             swap_if_greater(intervals, 0, 1);
//             swap_if_greater(intervals, 1, 2);
//         }
//         _ => intervals.sort_unstable_by_key(|x| x.0),
//     }
// }
