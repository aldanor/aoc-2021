use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::marker::PhantomData;
use std::ops::{Range, RangeBounds, RangeInclusive};

use arrayvec::ArrayVec;

use super::{minmax, parse_num, Coord};

use super::projection::{DiagNeg, DiagPos, Horizontal, Vertical};
use super::projection::{
    IntersectWith, IntersectableWith, LineDirection, ProjectFrom, ProjectableOnto,
};

type X = Coord;
type Y = Coord;
type Interval = (X, Y);
type Point = (X, Y);
type Line = (Point, Point);

const N: usize = 1 << 10;
const K: usize = 8;

fn parse_lines(mut s: &[u8]) -> impl Iterator<Item = Line> + '_ {
    iter::from_fn(move || {
        if s.len() > 1 {
            let p0 = (parse_num::<1>(&mut s), parse_num::<4>(&mut s));
            let p1 = (parse_num::<1>(&mut s), parse_num::<1>(&mut s));
            Some((p0, p1))
        } else {
            None
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LineType {
    Horizontal,
    DiagPos,
    DiagNeg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Segment {
    x_start: X,
    x_end: X,
    y: Y,
}

impl Segment {
    pub fn new((x_start, x_end): Interval, y: Y) -> Self {
        Self { x_start, x_end, y }
    }

    pub fn x_range(&self) -> RangeInclusive<X> {
        self.x_start..=self.x_end
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Event {
    Start(LineType, Segment),
    Vertical(Interval),
    Finish(LineType, Segment),
}

#[derive(Clone, Copy, Debug, Default)]
struct IntervalTrackerElement {
    len: usize,
    overlap_start: X,
}

impl IntervalTrackerElement {
    pub fn start(&mut self, x: X) -> bool {
        // return true if a new element is inserted
        self.len += 1;
        if self.len == 2 {
            self.overlap_start = x;
        }
        self.len == 1
    }

    pub fn finish(&mut self, x: X) -> (bool, Range<X>) {
        // returns (is_removed, range_of_overlaps)
        self.len -= 1;
        let range = if self.len == 1 { self.overlap_start..(x + 1) } else { 0..0 };
        (self.len == 0, range)
    }
}

#[derive(Clone, Debug)]
struct IntervalTracker {
    active: [IntervalTrackerElement; N * 2], // can also use BTreeMap...
}

impl IntervalTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&mut self, (x, y): Point) -> bool {
        self.active[((N as Y) + y) as usize].start(x) // TODO: bounds checks
    }

    pub fn finish(&mut self, (x, y): Point) -> (bool, Range<X>) {
        self.active[((N as Y) + y) as usize].finish(x) // TODO: bounds checks
    }
}

impl Default for IntervalTracker {
    fn default() -> Self {
        unsafe { std::mem::transmute([0_u8; std::mem::size_of::<Self>()]) }
    }
}

#[derive(Clone, Copy, Default, Debug)]
struct ActiveSetElement {
    x_overlap_start: X,
    x_ends: [X; K],
    len: usize,
}

impl ActiveSetElement {
    pub fn new(x_end: X) -> Self {
        let mut out = Self::default();
        out.x_ends[0] = x_end;
        out.len = 1;
        out
    }

    pub fn insert(&mut self, x_start: X, x_end: X) {
        debug_assert!(x_start > self.x_overlap_start);
        self.x_ends[self.len] = x_end;
        self.len += 1;
        if self.len == 2 {
            self.x_overlap_start = x_start;
        }
        (&mut self.x_ends[..self.len]).sort_unstable_by(|a, b| b.cmp(&a));
    }

    pub fn remove(&mut self, x_end: X) -> Range<X> {
        debug_assert!(x_end >= self.x_overlap_start);
        debug_assert_ne!(self.len, 0);
        if self.len == 1 {
            debug_assert_eq!(self.x_ends[0], x_end);
            self.len = 0;
            0..0
        } else {
            debug_assert!(self.x_ends.contains(&x_end));
            for i in 0..self.len {
                if self.x_ends[i] == x_end {
                    (&mut self.x_ends[i..]).rotate_left(1);
                    break;
                }
            }
            self.len -= 1;
            if self.len == 1 {
                self.x_overlap_start..(x_end + 1)
            } else {
                0..0
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn x_end(&self) -> X {
        self.x_ends[0]
    }
}

#[derive(Debug, Clone, Default)]
struct ActiveSet {
    active: BTreeMap<Y, ActiveSetElement>,
}

impl ActiveSet {
    pub fn insert(&mut self, segment: &Segment) {
        self.active
            .entry(segment.y)
            .and_modify(|element| element.insert(segment.x_start, segment.x_end))
            .or_insert(ActiveSetElement::new(segment.x_end));
    }

    pub fn remove(&mut self, segment: &Segment) -> Range<X> {
        match self.active.get_mut(&segment.y) {
            Some(element) => {
                let range = element.remove(segment.x_end);
                if element.is_empty() {
                    self.active.remove(&segment.y);
                }
                range
            }
            _ => 0..0,
        }
    }

    pub fn range(&self, range: impl RangeBounds<Y>) -> impl Iterator<Item = Y> + '_ {
        self.active.range(range).map(|(&k, _)| k)
    }

    pub fn range_with_endpoints(
        &self, range: impl RangeBounds<Y>,
    ) -> impl Iterator<Item = (Y, X)> + '_ {
        self.active.range(range).map(|(&k, element)| (k, element.x_end()))
    }
}

#[derive(Clone, Debug)]
struct Intersections {
    intersections: [[u64; N >> 6]; N],
}

impl Default for Intersections {
    fn default() -> Self {
        const S: usize = std::mem::size_of::<Intersections>();
        unsafe { std::mem::transmute([0_u8; S]) }
    }
}

impl Intersections {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn record(&mut self, x: X, y: Y) {
        let (x, y) = (x as usize, y as usize);
        unsafe {
            *self.intersections.get_unchecked_mut(y).get_unchecked_mut(x >> 6) |= 1 << (x & 0x3f);
        }
    }

    pub fn len(&self) -> usize {
        self.intersections
            .iter()
            .map(|i| i.iter().map(|n| n.count_ones()).sum::<u32>() as usize)
            .sum()
    }
}

#[derive(Debug, Clone, Default)]
struct ProjectedActiveSet<D: LineDirection> {
    active: ActiveSet,
    _marker: PhantomData<D>,
}

impl<D: LineDirection> ProjectedActiveSet<D> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_intersections<T: LineDirection + ProjectFrom<D> + IntersectWith<D>>(
        &mut self, other: &ProjectedActiveSet<T>, segment: &Segment,
        intersections: &mut Intersections,
    ) {
        let (x_range_this, y_this) = (segment.x_range(), segment.y);
        let y_range_other = x_range_this.project_onto::<D, T>(y_this);

        for (y_other, x_right_endpoint) in other.active.range_with_endpoints(y_range_other) {
            if let Some(x) = y_other.intersect_with::<T, D>(y_this) {
                if x <= x_right_endpoint {
                    let y = x.project_onto::<T, Horizontal>(y_other);
                    intersections.record(x, y);
                }
            }
        }
    }

    pub fn on_start(
        &mut self, a: &ProjectedActiveSet<D::A>, b: &ProjectedActiveSet<D::B>, segment: &Segment,
        intersections: &mut Intersections,
    ) {
        self.active.insert(segment);
        self.find_intersections(a, segment, intersections);
        self.find_intersections(b, segment, intersections);
    }

    pub fn on_vertical(&self, x: X, (y0, y1): Interval, intersections: &mut Intersections) {
        for y_d in self.active.range((y0..=y1).project_onto::<Vertical, D>(x)) {
            let y = x.project_onto::<D, Horizontal>(y_d);
            intersections.record(x, y);
        }
    }

    pub fn on_finish(&mut self, segment: &Segment, intersections: &mut Intersections) {
        let y_this = segment.y;
        for x in self.active.remove(segment) {
            let y = x.project_onto::<D, Horizontal>(y_this);
            intersections.record(x, y);
        }
    }
}

fn triple_line_sweep(events: &EventQueue) -> usize {
    let mut horizontal = ProjectedActiveSet::<Horizontal>::new();
    let mut diag_pos = ProjectedActiveSet::<DiagPos>::new();
    let mut diag_neg = ProjectedActiveSet::<DiagNeg>::new();

    let mut vertical_intervals = BTreeSet::new();
    let mut ix = Intersections::new();

    events.iter_events(|x, event| match event {
        Event::Start(line_type, s) => match line_type {
            LineType::Horizontal => horizontal.on_start(&diag_pos, &diag_neg, &s, &mut ix),
            LineType::DiagPos => diag_pos.on_start(&horizontal, &diag_neg, &s, &mut ix),
            LineType::DiagNeg => diag_neg.on_start(&horizontal, &diag_pos, &s, &mut ix),
        },
        Event::Vertical(interval) => {
            horizontal.on_vertical(x, interval, &mut ix);
            diag_pos.on_vertical(x, interval, &mut ix);
            diag_neg.on_vertical(x, interval, &mut ix);
            vertical_intervals.insert((interval.0, false, x));
            vertical_intervals.insert((interval.1, true, x));
        }
        Event::Finish(line_type, s) => match line_type {
            LineType::Horizontal => horizontal.on_finish(&s, &mut ix),
            LineType::DiagPos => diag_pos.on_finish(&s, &mut ix),
            LineType::DiagNeg => diag_neg.on_finish(&s, &mut ix),
        },
    });

    let mut tracker = IntervalTracker::new();
    for &(y_sweep, is_finish, x) in &vertical_intervals {
        if is_finish {
            for y in tracker.finish((y_sweep, x)).1 {
                ix.record(x, y);
            }
        } else {
            tracker.start((y_sweep, x));
        }
    }

    ix.len()
}

#[derive(Debug, Clone)]
struct EventQueue {
    events: [[ArrayVec<Event, K>; 3]; N], // [start, vertical, finish]
}

impl EventQueue {
    pub fn parse(s: &[u8]) -> Self {
        Self::from_lines(parse_lines(s))
    }

    pub fn from_lines(lines: impl Iterator<Item = Line>) -> Self {
        const START: usize = 0;
        const VERTICAL: usize = 1;
        const FINISH: usize = 2;
        let mut events = [0; N].map(|_| [0; 3].map(|_| ArrayVec::new()));
        for ((x0, y0), (x1, y1)) in lines {
            let (dx, dy) = (x1 - x0, y1 - y0);
            if dx == 0 {
                let (y0, y1) = minmax(y0, y1);
                events[x0 as usize][VERTICAL].push(Event::Vertical((y0, y1)));
            } else {
                let (line_type, y) = match (dy == 0, dx == dy) {
                    (true, _) => (LineType::Horizontal, y0),
                    (_, true) => (LineType::DiagPos, x0.project_onto::<Horizontal, DiagPos>(y0)),
                    _ => (LineType::DiagNeg, x0.project_onto::<Horizontal, DiagNeg>(y0)),
                };
                let (x0, x1) = minmax(x0, x1);
                let segment = Segment::new((x0, x1), y);
                events[x0 as usize][START].push(Event::Start(line_type, segment));
                events[x1 as usize][FINISH].push(Event::Finish(line_type, segment));
            }
        }
        Self { events }
    }

    pub fn iter_events(&self, mut func: impl FnMut(X, Event)) {
        for x in 0..N {
            for i in 0..3 {
                for &event in &self.events[x][i] {
                    func(x as _, event);
                }
            }
        }
    }
}

pub fn solve(s: &[u8]) -> usize {
    let events = EventQueue::parse(s);
    triple_line_sweep(&events)
}
