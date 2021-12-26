use std::collections::HashSet;
use std::fmt::{self, Debug};
use std::mem;

use arrayvec::ArrayVec;

use crate::utils::*;

type T = i16; // main coordinate type
type I = u16; // index type
type Point = [T; 3]; // x/y/z cordinate tuple
type D = i32; // distance type

const B: usize = 32; // max num detected beacons per scanner
const S: usize = 64; // max num scanners

const POS: usize = 0;
const NEG: usize = 1;
const DIR: [usize; 2] = [0, 1];
const SRC: usize = 0;
const DST: usize = 1;
const AXES: [usize; 3] = [0, 1, 2];

#[inline]
fn manhattan(a: Point, b: Point) -> T {
    (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs()
}

#[inline]
fn square_dist(a: Point, b: Point) -> i32 {
    let dx = (a[0] - b[0]) as i32;
    let dy = (a[1] - b[1]) as i32;
    let dz = (a[2] - b[2]) as i32;
    dx * dx + dy * dy + dz * dz
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Distance {
    d: D,
    i: I,
    j: I,
}

type ArrayDist = ArrayVec<Distance, { B * B }>;

impl Debug for Distance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} @{},{}", self.d, self.i, self.j)
    }
}

impl From<(D, I, I)> for Distance {
    fn from((d, i, j): (D, I, I)) -> Self {
        Self { d, i, j }
    }
}

#[inline]
fn distances_sort_unique_intersect(d0: &ArrayDist, d1: &ArrayDist) -> ([ArrayDist; 2], usize) {
    let mut ix0 = ArrayVec::new();
    let mut ix1 = ArrayVec::new();
    let (n0, n1) = (d0.len(), d1.len());
    let (mut i0, mut i1) = (0, 0);
    let mut early_stop = n0 + n1 - 66 * 2; // stop early if we can't collect 66 intersections
    while i0 != n0 && i1 != n1 && i0 + i1 <= early_stop {
        let (v0, v1) = (d0.get_at(i0), d1.get_at(i1));
        if v0.d == v1.d {
            unsafe {
                ix0.push_unchecked(v0);
                ix1.push_unchecked(v1);
                early_stop += 2;
            }
        }
        i0 += (v0.d <= v1.d) as usize;
        i1 += (v1.d <= v0.d) as usize;
    }
    let n_unique = ix0.len();
    ([ix0, ix1], n_unique)
}

#[inline]
fn arrayvec_sorted_intersect_by_key<T: Copy, K: Ord + Copy, F: Fn(&T) -> K, const CAP: usize>(
    d0: &ArrayVec<T, CAP>, d1: &ArrayVec<T, CAP>, key: F,
) -> ([ArrayVec<T, CAP>; 2], usize) {
    let mut ix0 = ArrayVec::new();
    let mut ix1 = ArrayVec::new();
    let (n0, n1) = (d0.len(), d1.len());
    let (mut i0, mut i1) = (0, 0);
    let mut n_unique = 0;
    while i0 < n0 && i1 < n1 {
        let (k0, k1) = (key(&d0[i0]), key(&d1[i1]));
        if k0 < k1 {
            i0 += 1;
        } else if k0 > k1 {
            i1 += 1;
        } else {
            n_unique += 1;
            let d = k0;
            while i0 < n0 && key(&d0[i0]) == d {
                unsafe { ix0.push_unchecked(d0[i0]) };
                i0 += 1;
            }
            while i1 < n1 && key(&d1[i1]) == d {
                unsafe { ix1.push_unchecked(d1[i1]) };
                i1 += 1;
            }
        }
    }
    ([ix0, ix1], n_unique)
}

fn find_polyhedron(ix: &ArrayDist, n: usize) -> Option<ArrayVec<usize, B>> {
    // generate vertex matrix
    let mut v = ArrayVec::<u32, B>::new();
    for i in 0..n {
        v.push(1 << i);
    }
    for d in ix {
        v[d.i as usize] |= 1 << d.j;
        v[d.j as usize] |= 1 << d.i;
    }
    let mut vertices = 0_u32;
    while {
        let mut changed = false;
        // kill rows/columns with less than 12 edges (including self)
        for i in 0..n {
            if v[i] != 0 && v[i].count_ones() < 12 {
                changed = true;
                v[i] = 0;
                for j in 0..n {
                    v[j] &= !(1 << i);
                }
            }
        }
        // important: we assume only one fully connected polyhedron >= 12 exists
        // (otherwise the problem of matching scanners may become ambiguous)
        // which means we can simply take a simple intersection here
        vertices = v.iter().fold(!0_u32, |acc, &x| acc & if x != 0 { x } else { !0_u32 });
        for i in 0..n {
            if v[i] != 0 {
                changed |= v[i] != vertices;
                v[i] &= vertices;
            }
        }
        changed
    } {}
    if vertices.count_ones() < 12 {
        None
    } else {
        let mut out = ArrayVec::new();
        for i in 0..n {
            if vertices & (1 << i) != 0 {
                out.push(i);
            }
        }
        Some(out)
    }
}

// all arrays here are in terms of source scanner
// (e.g. if source scanner is normal, then 0 is x, 1 is y, 2 is z)
#[derive(Debug, Clone, Copy)]
struct Mapping {
    pub offset: Point,
    pub axes: [usize; 3],  // for each axis in the dst, what is the axis in src
    pub is_neg: [bool; 3], // is dst axis reversed compared to src
}

impl Default for Mapping {
    fn default() -> Self {
        Self { offset: [0; 3], axes: [0, 1, 2], is_neg: [false; 3] }
    }
}

impl Mapping {
    pub fn dst_to_src(&self, point: Point) -> Point {
        AXES.map(|axis| {
            self.offset[axis]
                + if self.is_neg[axis] { -point[self.axes[axis]] } else { point[self.axes[axis]] }
        })
    }

    pub fn combine(&self, dst: &Self) -> Self {
        Self {
            offset: AXES.map(|axis| {
                self.offset[axis]
                    + if self.is_neg[axis] {
                        -dst.offset[self.axes[axis]]
                    } else {
                        dst.offset[self.axes[axis]]
                    }
            }),
            axes: AXES.map(|axis| dst.axes[self.axes[axis]]),
            is_neg: AXES.map(|axis| self.is_neg[axis] != dst.is_neg[self.axes[axis]]),
        }
    }
}

#[derive(Clone, Debug)]
struct Scanner {
    id: u8,
    beacons: ArrayVec<Point, B>,
    pairwise_distances: ArrayVec<Distance, { B * B }>,
    distance_matrix: ArrayVec<ArrayVec<i32, B>, B>,
}

impl Scanner {
    fn parse(s: &mut &[u8]) -> Self {
        // note: fill() is not being called here
        *s = s.advance(12);
        let id = parse_int_fast::<_, 1, 2>(s);
        *s = s.advance(4);
        let mut beacons = ArrayVec::new();
        while !s.is_empty() && s.get_first() != b'\n' {
            let x = parse_int_fast_signed::<_, 1, 4>(s);
            let y = parse_int_fast_signed::<_, 1, 4>(s);
            let z = parse_int_fast_signed::<_, 1, 4>(s);
            unsafe { beacons.push_unchecked([x, y, z]) };
        }
        *s = s.advance(1);
        let pairwise_distances = ArrayVec::new();
        let distance_matrix = ArrayVec::new();
        Self { id, beacons, pairwise_distances, distance_matrix }
    }

    pub fn parse_multiple(mut s: &[u8]) -> ArrayVec<Self, S> {
        use rayon::prelude::*;
        let mut scanners = ArrayVec::new();
        while !s.is_empty() {
            scanners.push(Self::parse(&mut s));
        }
        // note: par_iter barely does anything here, maybe a 5% speedup or so
        scanners.par_iter_mut().for_each(Self::fill);
        scanners
    }

    pub fn len(&self) -> usize {
        self.beacons.len()
    }

    fn fill(&mut self) {
        let n = self.len();
        unsafe {
            self.distance_matrix = mem::zeroed();
            self.distance_matrix.set_len(n);
            for i in 0..n {
                self.distance_matrix[i].set_len(n);
            }
        }
        self.pairwise_distances.clear();
        for i in 0..(n - 1) {
            let a = self.beacons[i];
            for j in (i + 1)..n {
                let b = self.beacons[j];
                let d = square_dist(a, b);
                self.distance_matrix[i][j] = d;
                self.distance_matrix[j][i] = d;
                self.pairwise_distances.push(Distance::from((d, i as I, j as I)));
            }
        }
        self.pairwise_distances.sort_unstable_by_key(|x| x.d);
        // // is it the easy case?
        // assert!(self.sorted_pairwise_distances.array_windows::<2>().all(|x| x[0] != x[1]));
    }

    pub fn check_distance_overlaps(&self, other: &Self) -> Option<[ArrayVec<usize, B>; 2]> {
        // ok, we're not assuming unique distances here (although they are unique in the data)
        // and hence we're complicating our life a bit...
        let d0 = &self.pairwise_distances;
        let d1 = &other.pairwise_distances;
        let n = [self.len(), other.len()];

        // find initial distance overlaps
        let (ix, n_unique) =
            if d0.len() == (n[0] * (n[0] - 1)) / 2 && d1.len() == (n[1] * (n[1] - 1)) / 2 {
                distances_sort_unique_intersect(d0, d1)
            } else {
                arrayvec_sorted_intersect_by_key(d0, d1, |x| x.d)
            };

        if ix[0].len().min(ix[1].len()) < 66 {
            return None; // C(12, 2) = 66, minimum number of edges required
        }

        // // fast track
        // if (ix[0].len(), ix[1].len(), n_unique) == (66, 66, 66) {
        //     let mut v = [0_u32; 2];
        //     for k in 0..2 {
        //         for d in &ix[k] {
        //             v[k] |= (1 << d.i) | (1 << d.j);
        //         }
        //     }
        //     if v.map(|v| v.count_ones()) == [12, 12] {
        //         return Some([0, 1].map(|k| {
        //             let mut out = ArrayVec::new();
        //             for i in 0..n[k] {
        //                 if v[k] & (1 << i) != 0 {
        //                     out.push(i);
        //                 }
        //             }
        //             out
        //         }));
        //     }
        // }

        // try to build a fully connected polyhedron with at least 12 vertices
        let mut v = [find_polyhedron(&ix[0], n[0])?, find_polyhedron(&ix[1], n[1])?];
        if [0, 1].into_iter().all(|i| (v[i].len() * (v[i].len() - 1)) / 2 == ix[i].len())
            && v[0].len() == v[1].len()
        {
            // we found all vertices in both sets AND they cover the entire initial set
            // AND nothing has been removed AND vertex set lengths match each other

            // now, check for the all-unique-distances case
            if (ix[0].len(), ix[1].len()) == (n_unique, n_unique) {
                // let's map all 12 vertices first by going through a chain 0-1-2-...-0
                let mut chain1 = ArrayVec::<Distance, B>::new();
                let vn = v[0].len();
                for i in 0..vn {
                    let j = if i != vn - 1 { i + 1 } else { 0 };
                    let d = self.distance_matrix[v[0][i] as usize][v[0][j] as usize];
                    let edge1 = ix[1][ix[1].binary_search_by_key(&d, |x| x.d).ok()?];
                    chain1.push(edge1);
                }
                chain1.insert(0, chain1[vn - 1]); // (n - 1, 0), (0, 1), (1, 2), ... (n - 1, 0)
                let mut v1 = ArrayVec::<usize, B>::new();
                for i in 0..vn {
                    let (this, next) = (&chain1[i], &chain1[i + 1]);
                    v1.push(if this.i == next.i || this.i == next.j {
                        this.i as _
                    } else {
                        this.j as _
                    });
                }
                v[1] = v1;
                Some(v)
                // easy times. we have two distance matrices in the overlap AND all
                // distances are unique, so we can use them to match the vertices
            } else {
                // again, doable but too lazy for now. since we know that a unique
                // non-ambiguous solution to each overlap must exist, it should be
                // reconstructible directly from the two distance matrices
                todo!("too lazy, another time");
            }
        } else {
            // basically, iterate again, filter pairwise distances by vertex sets, then
            // find the vertices again etc; might also need to match counts in both sets etc.
            // doable but too much boilerplate
            todo!("too lazy, another time");
        }
    }

    pub fn infer_mapping(&self, other: &Self, v: &[ArrayVec<usize, B>; 2]) -> Option<Mapping> {
        assert_eq!(v[SRC].len(), v[DST].len());
        let scanners = [self, other];
        let n = v[SRC].len();

        // x/y/z coordinates relative to the first vertex for the 'source' (0) scanner
        // (includes both positive and negative coordinates, pos=0, neg=1)
        let mut src = DIR.map(|_| AXES.map(|_| ArrayVec::<T, B>::new()));
        // x/y/z coordinates relative to the first vertex for the 'destination' (1) scanner
        let mut dst = AXES.map(|_| ArrayVec::<T, B>::new());

        // first vertex in local coordinates for src/dst
        let mut first = [SRC, DST].map(|k| scanners[k].beacons[v[k][0]]);

        'outer: for i in 1..n {
            for axis in AXES {
                let src_rel_coord = scanners[SRC].beacons[v[SRC][i]][axis] - first[SRC][axis];
                src[POS][axis].push(src_rel_coord);
                src[NEG][axis].push(-src_rel_coord);
                let dst_rel_coord = scanners[DST].beacons[v[DST][i]][axis] - first[DST][axis];
                dst[axis].push(dst_rel_coord);
            }
            // either we're lucky (happens always / almost always) and we can
            // decode it from the first try, or we need to process all vertices
            if i == 1 || i == (n - 1) {
                let eq = AXES.map(|dst_axis| {
                    AXES.map(|src_axis| DIR.map(|dir| dst[dst_axis] == src[dir][src_axis]))
                });
                let mut axes = [Option::<usize>::None; 3];
                let mut is_neg = [Option::<bool>::None; 3];
                for dst_axis in AXES {
                    for src_axis in AXES {
                        for dir in DIR {
                            if eq[dst_axis][src_axis][dir] {
                                if axes[src_axis].is_some() || is_neg[src_axis].is_some() {
                                    continue 'outer;
                                }
                                axes[src_axis] = Some(dst_axis);
                                is_neg[src_axis] = Some(dir == NEG);
                            }
                        }
                    }
                }
                if let [Some(x), Some(y), Some(z)] = axes {
                    if let [Some(xneg), Some(yneg), Some(zneg)] = is_neg {
                        if x != y && y != z && (x | y | z) == 3 {
                            let mut mapping = Mapping::default();
                            mapping.axes = [x, y, z];
                            mapping.is_neg = [xneg, yneg, zneg];
                            let (v0_src, v0_dst) = (first[SRC], first[DST]);
                            let mut v0_dst_no_offset = mapping.dst_to_src(v0_dst);
                            mapping.offset = AXES.map(|i| v0_src[i] - v0_dst_no_offset[i]);
                            debug_assert!((0..n)
                                .all(|i| mapping.dst_to_src(scanners[DST].beacons[v[DST][i]])
                                    == scanners[SRC].beacons[v[SRC][i]]));
                            return Some(mapping);
                        }
                    }
                }
            }
        }
        None
    }
}

fn reconstruct_scanner_map(scanners: &ArrayVec<Scanner, S>) -> ArrayVec<Mapping, S> {
    let n = scanners.len();

    let mut mappings = ArrayVec::<Mapping, S>::new();
    for _ in 0..n {
        mappings.push(Mapping::default());
    }

    let mut queue = ArrayVec::<(usize, Mapping), S>::new();
    queue.push((0, Mapping::default()));
    let mut done = 0_u64 | 1;

    while let Some((i, src_mapping)) = queue.pop() {
        mappings[i] = src_mapping;
        for j in 1..n {
            if j != i && done & (1 << j) == 0 {
                if let Some(vertices) = scanners[i].check_distance_overlaps(&scanners[j]) {
                    if let Some(dst_mapping) = scanners[i].infer_mapping(&scanners[j], &vertices) {
                        done |= (1 << j);
                        queue.push((j, src_mapping.combine(&dst_mapping)));
                    }
                }
            }
        }
    }
    assert_eq!(done.count_ones(), n as u32);
    mappings
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(mut s: &[u8]) -> usize {
    let scanners = Scanner::parse_multiple(s);
    let mappings = reconstruct_scanner_map(&scanners);
    let mut set = HashSet::with_capacity(B * S);
    for i in 0..scanners.len() {
        for &beacon in &scanners[i].beacons {
            set.insert(mappings[i].dst_to_src(beacon));
        }
    }
    set.len()
}

pub fn part2(mut s: &[u8]) -> usize {
    let scanners = Scanner::parse_multiple(s);
    let mappings = reconstruct_scanner_map(&scanners);
    let mut max_dist = 0;
    for i in 0..(scanners.len() - 1) {
        for j in (i + 1)..scanners.len() {
            max_dist = max_dist.max(manhattan(mappings[i].offset, mappings[j].offset));
        }
    }
    max_dist as _
}

#[test]
fn test_day19_part1() {
    assert_eq!(part1(input()), 467);
}

#[test]
fn test_day19_part2() {
    assert_eq!(part2(input()), 12226);
}
