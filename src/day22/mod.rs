mod graph;

use std::iter;
use std::mem;
use std::ops::{Add, Neg, Sub};

use arrayvec::ArrayVec;

use crate::utils::*;

const N: usize = 512;
const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;

type Array<T> = ArrayVec<T, N>;
type Range<R> = [R; 2];
type Cube<R> = [Range<R>; 3];

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

fn parse<T: Integer + Neg<Output = T>>(mut s: &[u8], full: bool) -> (Array<Cube<T>>, Array<bool>) {
    fn parse_bounds<T: Integer + Neg<Output = T>, const D: usize>(s: &mut &[u8]) -> Cube<T> {
        let xmin = parse_int_fast_signed::<T, 1, D>(s);
        *s = s.advance(1);
        let xmax = parse_int_fast_signed::<T, 1, D>(s) + T::from(1);
        *s = s.advance(2);
        let ymin = parse_int_fast_signed::<T, 1, D>(s);
        *s = s.advance(1);
        let ymax = parse_int_fast_signed::<T, 1, D>(s) + T::from(1);
        *s = s.advance(2);
        let zmin = parse_int_fast_signed::<T, 1, D>(s);
        *s = s.advance(1);
        let zmax = parse_int_fast_signed::<T, 1, D>(s) + T::from(1);
        [[xmin, xmax], [ymin, ymax], [zmin, zmax]]
    }

    let mut cubes = Array::new();
    let mut state = Array::new();
    while s.len() > 1 {
        let is_on = s.get_at(1) == b'n';
        s = s.advance(5 + (!is_on) as usize);
        let cube = parse_bounds::<T, 6>(&mut s);
        if s.get_at(2).is_ascii_digit() && s.get_at(3).is_ascii_digit() && !full {
            break;
        };
        cubes.push(cube);
        state.push(is_on);
    }
    (cubes, state)
}

const AXES: [usize; 3] = [0, 1, 2];

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    type T = i16;
    let (cubes, state) = parse::<T>(s, false);
    const W: usize = 100;
    const SIZE: usize = W * W * W;
    const STRIDES: [usize; 3] = [W * W, W, 1];
    let mut grid = Vec::with_capacity(SIZE);
    unsafe { grid.set_len(SIZE) };
    grid.fill(false);
    for (i, &cube) in cubes.iter().enumerate() {
        let [[xmin, xmax], [ymin, ymax], [zmin, zmax]] =
            cube.map(|range| range.map(|coord| (coord + 50) as usize));
        for x in xmin..=xmax {
            let offset = x * STRIDES[X];
            for y in ymin..=ymax {
                let offset = offset + y * STRIDES[Y];
                grid[offset + zmin..=offset + zmax].fill(state[i]);
            }
        }
    }
    grid.iter().map(|&v| v as usize).sum()
}

use self::graph::{Endpoint, Region};
use petgraph::graph::{NodeIndex, UnGraph};
use std::collections::{BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy)]
struct CliqueRegion<T, const D: usize> {
    region: Region<T, D>, // intersection of all regions in the clique
    first_is_on: bool,    // whether the first region in the clique is 'on'
    even: bool,
}

impl<T: Endpoint, const D: usize> CliqueRegion<T, D> {
    pub fn new(region: Region<T, D>, is_on: bool) -> Self {
        Self { region, first_is_on: is_on, even: true }
    }

    pub fn extend(&self, region: &Region<T, D>) -> Self {
        Self {
            region: self.region.intersect_unchecked(region),
            first_is_on: self.first_is_on,
            even: !self.even,
        }
    }
}

pub fn find_total_volume<T, E, const D: usize>(g: &UnGraph<Region<T, D>, E>, is_on: &[bool]) -> T
where
    T: Endpoint + Sub<Output = T> + Add<Output = T> + Neg<Output = T> + iter::Product,
{
    // clique enumeration itself is adapted from networkx.Graph.enumerate_all_cliques()

    // for each node `i`, a set of neighbors `j` with `j` > `i`
    let neighbors: Vec<BTreeSet<usize>> = g
        .node_indices()
        .map(|i| g.neighbors(i).filter(|&j| j > i).map(|i| i.index()).collect())
        .collect();

    let mut queue: VecDeque<(CliqueRegion<T, D>, Vec<usize>)> = g
        .node_indices()
        .map(NodeIndex::index)
        .zip(&neighbors)
        .map(|(u, neighbors)| {
            (
                CliqueRegion::new(g.raw_nodes()[u].weight, is_on[u]),
                neighbors.iter().copied().collect(),
            )
        })
        .collect();

    let mut volume = T::default();

    while let Some((base, common_neighbors)) = queue.pop_front() {
        for (i, &u) in common_neighbors.iter().enumerate() {
            let mut new_base = base.extend(&g.raw_nodes()[u].weight);
            // we alter the inclusion/exclusion principle a bit because we also have
            // non-symmetric set differences; only modify the total volume if the
            // first element in the clique has the 'on' state
            let new_common_neighbors = common_neighbors[i + 1..]
                .iter()
                .copied()
                .filter(|&j| neighbors[u].contains(&j))
                .collect();
            queue.push_back((new_base, new_common_neighbors));
        }
        if base.first_is_on {
            // if the first element is 'on', it's the normal inclusion/exclusion principle
            let v = base.region.volume().into();
            volume = volume + if base.even { v } else { -v };
        }
    }
    volume
}

#[inline]
pub fn part2(mut s: &[u8]) -> i64 {
    use graph::*;
    let (cubes, state) = parse::<i64>(s, true);
    let g = IntersectionGraph::from_regions_bruteforce(&cubes);
    find_total_volume(&g, &state)
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 583641);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 1182153534186233);
}
