use std::ops::Neg;

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
        let xmax = parse_int_fast_signed::<T, 1, D>(s);
        *s = s.advance(2);
        let ymin = parse_int_fast_signed::<T, 1, D>(s);
        *s = s.advance(1);
        let ymax = parse_int_fast_signed::<T, 1, D>(s);
        *s = s.advance(2);
        let zmin = parse_int_fast_signed::<T, 1, D>(s);
        *s = s.advance(1);
        let zmax = parse_int_fast_signed::<T, 1, D>(s);
        [[xmin, xmax], [ymin, ymax], [zmin, zmax]]
    }

    let mut cubes = Array::new();
    let mut state = Array::new();
    while s.len() > 1 {
        let is_on = s.get_at(1) == b'n';
        s = s.advance(5 + (!is_on) as usize);
        let cube = if s.get_at(2).is_ascii_digit() && s.get_at(3).is_ascii_digit() {
            if !full {
                break;
            }
            parse_bounds::<T, 5>(&mut s)
        } else {
            parse_bounds::<T, 2>(&mut s)
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

// #[inline]
// pub fn _part1(mut s: &[u8]) -> usize {
//     let s = include_bytes!("input_small.txt").as_ref();
//     type T = i16;
//     let (cubes, state) = parse::<T>(s, false);
//
//     const NIL: usize = !0_usize;
//     let mut coord_to_idx = [[NIL; 100]; 3];
//     let mut idx_to_coord = AXES.map(|_| Array::<T>::new());
//
//     for cube in &cubes {
//         for axis in AXES {
//             for i in 0..2 {
//                 let coord = cube[axis][i] + 50;
//                 if coord_to_idx[axis][coord as usize] == NIL {
//                     coord_to_idx[axis][coord as usize] = idx_to_coord[axis].len();
//                     idx_to_coord[axis].push(coord);
//                 }
//             }
//         }
//     }
//     for axis in AXES {
//         idx_to_coord[axis].sort_unstable();
//         for (i, &coord) in idx_to_coord[axis].iter().enumerate() {
//             coord_to_idx[axis][coord as usize] = i;
//         }
//     }
//     let mut dense_cubes = Array::new();
//     for &cube in &cubes {
//         dense_cubes.push(
//             AXES.map(|axis| cube[axis].map(|coord| coord_to_idx[axis][(coord + 50) as usize])),
//         );
//     }
//     let dims = AXES.map(|axis| idx_to_coord[axis].len());
//     let size = dims.into_iter().product::<usize>();
//     let strides = [dims[Y] * dims[Z], dims[Z], 1];
//     let [dx, dy, dz] = strides;
//
//     println!();
//     for dense_cube in &dense_cubes {
//         println!("{:?}", dense_cube);
//     }
//
//     let mut grid = Vec::<bool>::with_capacity(size);
//     unsafe { grid.set_len(size) };
//     grid.fill(false);
//
//     for (i, &[[xmin, xmax], [ymin, ymax], [zmin, zmax]]) in dense_cubes.iter().enumerate() {
//         for x in xmin..=xmax {
//             let offset = x * dx;
//             for y in ymin..=ymax {
//                 let offset = offset + y * dy;
//                 grid[offset + zmin..=offset + zmax].fill(state[i]);
//             }
//         }
//     }
//
//     let offsets = [
//         0 * dx + 0 * dy + 0 * dz,
//         0 * dx + 0 * dy + 1 * dz,
//         0 * dx + 1 * dy + 0 * dz,
//         0 * dx + 1 * dy + 1 * dz,
//         1 * dx + 0 * dy + 0 * dz,
//         1 * dx + 0 * dy + 1 * dz,
//         1 * dx + 1 * dy + 0 * dz,
//         1 * dx + 1 * dy + 1 * dz,
//     ];
//     let dist = AXES.map(|axis| {
//         let mut dist = Array::new();
//         for [a, b] in idx_to_coord[axis].array_windows::<2>() {
//             dist.push((b - a) as usize);
//         }
//         dist
//     });
//     let mut non_empty = Vec::with_capacity(size);
//     unsafe { non_empty.set_len(size) };
//     non_empty.fill(false);
//     let mut total = 0;
//     let mut n = 0;
//     for i_x in 0..dims[X] - 1 {
//         let x_side = 1 + dist[X].get_at(i_x);
//         let offset = i_x * dx;
//         for i_y in 0..dims[Y] - 1 {
//             let offset = offset + i_y * dy;
//             let y_side = 1 + dist[Y].get_at(i_y);
//             'outer: for i_z in 0..dims[Z] - 1 {
//                 // TODO: prev/this x4
//                 let offset = offset + i_z * dz;
//                 let grid = &grid[offset..];
//                 for &d in &offsets {
//                     if !grid[d] {
//                         continue 'outer;
//                     }
//                 }
//                 let z_side = 1 + dist[Z].get_at(i_z);
//                 let j = offset + dx + dy + dz;
//                 let x_prev = non_empty[j - dx];
//                 let y_prev = non_empty[j - dy];
//                 let z_prev = non_empty[j - dz];
//                 let xy_prev = non_empty[j - dx - dy];
//                 let xz_prev = non_empty[j - dx - dz];
//                 let yz_prev = non_empty[j - dy - dz];
//                 let xyz_prev = non_empty[j - dx - dy - dz];
//                 let v = (x_side * y_side * z_side)
//                     - x_prev as usize * (y_side - 1) * (z_side - 1)
//                     - y_prev as usize * (x_side - 1) * (z_side - 1)
//                     - z_prev as usize * (x_side - 1) * (y_side - 1)
//                     - (y_prev || z_prev || yz_prev) as usize * (x_side - 1)
//                     - (x_prev || z_prev || xz_prev) as usize * (y_side - 1)
//                     - (x_prev || y_prev || xy_prev) as usize * (z_side - 1)
//                     - (x_prev || y_prev || z_prev || xy_prev || xz_prev || yz_prev || xyz_prev)
//                         as usize;
//                 total += v;
//                 non_empty[j] = true;
//                 n += 1;
//                 dbg!(n, v);
//             }
//         }
//     }
//     total
// }

#[inline]
pub fn part2(mut s: &[u8]) -> i32 {
    0
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 583641);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 0);
}
