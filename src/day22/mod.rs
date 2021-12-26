use std::ops::Neg;

use arrayvec::ArrayVec;

use crate::utils::*;

const N: usize = 512;

type Range<R> = [R; 2];
type Cube<T, const D: usize> = [Range<T>; D];

fn parse_cube<T: Integer + Neg<Output = T>>(s: &mut &[u8]) -> Cube<T, 3> {
    let xmin = parse_int_fast_signed::<T, 1, 6>(s);
    *s = s.advance(1);
    let xmax = parse_int_fast_signed::<T, 1, 6>(s) + T::from(1);
    *s = s.advance(2);
    let ymin = parse_int_fast_signed::<T, 1, 6>(s);
    *s = s.advance(1);
    let ymax = parse_int_fast_signed::<T, 1, 6>(s) + T::from(1);
    *s = s.advance(2);
    let zmin = parse_int_fast_signed::<T, 1, 6>(s);
    *s = s.advance(1);
    let zmax = parse_int_fast_signed::<T, 1, 6>(s) + T::from(1);
    [[xmin, xmax], [ymin, ymax], [zmin, zmax]]
}

fn parse_input<T>(mut s: &[u8], full: bool) -> (ArrayVec<Cube<T, 3>, N>, ArrayVec<bool, N>)
where
    T: Integer + Neg<Output = T>,
{
    let mut cubes = ArrayVec::new();
    let mut state = ArrayVec::new();
    while s.len() > 1 {
        let is_on = s.get_at(1) == b'n';
        s = s.advance(5 + (!is_on) as usize);
        if s.get_at(2).is_ascii_digit() && s.get_at(3).is_ascii_digit() && !full {
            break;
        };
        let cube = parse_cube::<T>(&mut s);
        cubes.push(cube);
        state.push(is_on);
    }
    (cubes, state)
}

#[inline]
fn volume<T: Integer, const D: usize>(x: &Cube<T, D>) -> T {
    (0..D).map(|i| x[i][1] - x[i][0]).product::<T>()
}

#[inline]
fn overlaps<T: Integer, const D: usize>(a: &Cube<T, D>, b: &Cube<T, D>) -> bool {
    (0..D).all(|i| a[i][0].max(b[i][0]) < a[i][1].min(b[i][1]))
}

#[inline]
fn intersect_unchecked<T: Integer, const D: usize>(a: &Cube<T, D>, b: &Cube<T, D>) -> Cube<T, D> {
    let mut out = [[T::default(); 2]; D];
    for i in 0..D {
        out[i] = [a[i][0].max(b[i][0]), a[i][1].min(b[i][1])];
    }
    out
}

fn find_total_volume<T, const D: usize, const N: usize, const L: usize>(
    regions: &[Cube<T, D>], is_on: &[bool],
) -> T
where
    T: Integer + Neg<Output = T>,
{
    // idea in using cliques taken from: 10.1109/ICDM.2019.00160

    let n = regions.len();

    let mut queue = Vec::<(Cube<T, D>, ArrayVec<usize, L>, bool)>::with_capacity(n * L);
    let mut neighbors = [[false; N]; N]; // for bigger problems, can use btreeset/hashset

    // build overlap matrix (the upper-right triangular part of it) and initialize the queue
    for i in 0..n - 1 {
        let a = &regions[i];
        let mut neighbors_list = ArrayVec::new();
        for j in i + 1..n {
            let b = &regions[j];
            if overlaps(a, b) {
                neighbors[i][j] = true;
                neighbors_list.push(j);
            }
        }
        if is_on[i] {
            // this ^ is where the algorithm differs from the classic clique
            // enumeration algorithm; because we don't push cliques that start
            // with an 'off' node onto the queue, they will never be processed,
            // which essentially extends inclusion-exclusion principle to
            // support set subtraction and not just the union when it comes
            // to computing the total set cardinality.
            queue.push((regions[i], neighbors_list, true));
        }
    }
    if is_on[n - 1] {
        queue.push((regions[n - 1], ArrayVec::new(), true));
    }

    let mut total_volume = T::default();
    for (region, common_neighbors, is_even) in queue.drain(..) {
        total_volume += enumerate_cliques_recursive::<T, D, N, L>(
            &region,
            &common_neighbors,
            &neighbors,
            regions,
            is_even,
        );
    }
    total_volume

    /*
    // this is a modified/sequential version of the networkx.clique.find_clique() algorithm
    while let Some((region, common_neighbors, is_even)) = queue.pop() {
        for (i, &u) in common_neighbors.iter().enumerate() {
            let region = intersect_unchecked(&region, &regions[u]);
            let is_even = !is_even;
            let common_neighbors =
                common_neighbors[i + 1..].iter().copied().filter(|&j| neighbors[u][j]).collect();
            queue.push((region, common_neighbors, is_even));
        }
        // we know that the first element is always 'on' because we have filtered it out
        // at the queue creation stage, so the normal inclusion/exclusion principle applies
        let v = volume(&region);
        total_volume += if is_even { v } else { -v };
    }
    total_volume
     */
}

fn enumerate_cliques_recursive<T, const D: usize, const N: usize, const L: usize>(
    region: &Cube<T, D>, common_neighbors: &[usize], neighbors: &[[bool; N]; N],
    regions: &[Cube<T, D>], is_even: bool,
) -> T
where
    T: Integer + Neg<Output = T>,
{
    // this is a modified/recursive version of the networkx.clique.find_clique() algorithm
    let mut total_volume = volume(region);
    // we know that the first element is always 'on' because we have filtered it out
    // at the queue creation stage, so the normal inclusion/exclusion principle applies
    if !is_even {
        total_volume = -total_volume;
    }
    if let Some(last) = common_neighbors.last().copied() {
        let mut cn = ArrayVec::<usize, L>::new();
        let n = common_neighbors.len();
        for i in 0..n - 1 {
            let u = common_neighbors[i];
            cn.clear();
            for j in i + 1..n {
                let v = common_neighbors[j];
                if neighbors[u][v] {
                    cn.push(v);
                }
            }
            total_volume += enumerate_cliques_recursive::<T, D, N, L>(
                &intersect_unchecked(region, &regions[u]),
                &cn,
                neighbors,
                regions,
                !is_even,
            );
        }
        let mut v_last = volume(&intersect_unchecked(region, &regions[last]));
        if is_even {
            v_last = -v_last;
        }
        total_volume += v_last;
    }
    total_volume
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(s: &[u8]) -> i64 {
    let (cubes, state) = parse_input(s, false);
    find_total_volume::<_, 3, 32, 16>(&cubes, &state)
}

pub fn part2(s: &[u8]) -> i64 {
    let (cubes, state) = parse_input(s, true);
    find_total_volume::<_, 3, 512, 32>(&cubes, &state)
}

#[test]
fn test_day22_part1() {
    assert_eq!(part1(input()), 583641);
}

#[test]
fn test_day22_part2() {
    assert_eq!(part2(input()), 1182153534186233);
}
