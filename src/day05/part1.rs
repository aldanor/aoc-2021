use std::collections::BTreeSet;

use arrayvec::ArrayVec;

use super::{minmax, parse_num, Coord};

type Interval = (Coord, Coord); // a 1-D interval, both ends are included
type Intervals = ArrayVec<Interval, K>; // a variable-size array of 1-D intervals
type IntervalSet = [Intervals; N]; // interval array for each coordinate (=index)

const N: usize = 1 << 10; // max coord
const K: usize = 8; // max number of intervals per coord

fn parse_horizontal_vertical(mut s: &[u8]) -> (IntervalSet, IntervalSet) {
    let mut horizontal = [0; N].map(|_| ArrayVec::new());
    let mut vertical = [0; N].map(|_| ArrayVec::new());

    while s.len() > 1 {
        let (x0, y0) = (parse_num::<1>(&mut s), parse_num::<4>(&mut s));
        let (x1, y1) = (parse_num::<1>(&mut s), parse_num::<1>(&mut s));
        if y0 == y1 {
            let (y, (x0, x1)) = (y0 as usize, minmax(x0, x1));
            horizontal[y].push((x0, x1));
        } else if x0 == x1 {
            let (x, (y0, y1)) = (x0 as usize, minmax(y0, y1));
            vertical[x].push((y0, y1));
        }
    }

    (horizontal, vertical)
}

fn process_overlaps_1d(intervals: &mut Intervals, overlaps: &mut Intervals) -> usize {
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
        if overlap_overlaps.is_empty() {
            break;
        }
        n_overlaps -= process_overlaps_1d(overlaps, &mut overlap_overlaps);
        overlaps.extend(overlap_overlaps.drain(..));
    }
    n_overlaps
}

#[inline]
fn process_overlaps(interval_set: &mut [Intervals], n_overlaps: &mut usize) -> IntervalSet {
    let mut overlaps = [0; N].map(|_| ArrayVec::new());
    for (coord, intervals) in interval_set.iter_mut().enumerate() {
        if intervals.len() > 1 {
            *n_overlaps += process_overlaps_1d(intervals, &mut overlaps[coord]);
        }
    }
    overlaps
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Event {
    Start(Coord),
    Vertical(Interval),
    Finish(Coord),
}

fn line_sweep_hv(horizontal: &[Intervals], vertical: &[Intervals]) -> usize {
    let mut event_queue = [0; N].map(|_| ArrayVec::<Event, K>::new());

    for (x, intervals) in vertical.iter().take(N).enumerate() {
        for &interval in intervals {
            event_queue[x].push(Event::Vertical(interval));
        }
    }
    for (y, intervals) in horizontal.iter().take(N).enumerate() {
        for &(x0, x1) in intervals {
            event_queue[x0 as usize].push(Event::Start(y as _));
            event_queue[x1 as usize].push(Event::Finish(y as _));
        }
    }

    let mut active = BTreeSet::<Coord>::new();
    let mut count = 0;
    for events in &mut event_queue {
        if events.len() > 1 {
            events.sort_unstable();
        }
        for &event in events as &_ {
            match event {
                Event::Start(y) => {
                    active.insert(y);
                }
                Event::Vertical((y0, y1)) => {
                    count += active.range(y0..=y1).count();
                }
                Event::Finish(y) => {
                    active.remove(&y);
                }
            }
        }
    }
    count
}

pub fn solve(s: &[u8]) -> usize {
    let (mut horizontal, mut vertical) = parse_horizontal_vertical(s);

    let mut n_parallel_overlaps = 0;
    let horizontal_overlaps = process_overlaps(&mut horizontal[..N], &mut n_parallel_overlaps);
    let vertical_overlaps = process_overlaps(&mut vertical[..N], &mut n_parallel_overlaps);
    let n_non_overlap_overlaps = line_sweep_hv(&horizontal, &vertical);
    let n_overlap_overlaps = line_sweep_hv(&horizontal_overlaps, &vertical_overlaps);

    n_parallel_overlaps + n_non_overlap_overlaps - n_overlap_overlaps
}
