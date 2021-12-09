use crate::utils::*;

const N: usize = 100;
const M: usize = 1 << 10;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

macro_rules! lanes_min {
    ($lhs:expr, $rhs:expr) => {
        ($lhs).lanes_ge(($rhs)).select(($rhs), ($lhs))
    };
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    type T = i8;
    type S = core_simd::Simd<T, 64>;

    let mut prev = [S::splat(T::MAX); 2];
    let mut prev_min3 = [S::splat(T::MAX); 2];
    let mut out = 0;

    let zero = S::splat(0);
    let threshold = S::splat((b'0' - 1) as _);

    for i in 1..=(N + 2) {
        let mut arr = [T::MAX; 128];

        if i <= N {
            unsafe { std::ptr::copy_nonoverlapping(s.as_ptr(), arr.as_mut_ptr().add(1).cast(), N) };
            s = s.advance(N + 1);
        }

        for j in 0..2 {
            let left = S::from_array(unsafe { *arr.as_ptr().add(0 + 64 * j).cast() });
            let this = S::from_array(unsafe { *arr.as_ptr().add(1 + 64 * j).cast() });
            let right = S::from_array(unsafe { *arr.as_ptr().add(2 + 64 * j).cast() });

            let this_min3 = lanes_min!(left, right);
            let this_min3 = lanes_min!(this_min3, prev[j]);

            if i >= 2 {
                let min_cross = lanes_min!(this, prev_min3[j]);
                let value = prev[j] - threshold;
                out += prev[j].lanes_lt(min_cross).select(value, zero).horizontal_sum() as usize;
            }

            prev[j] = this;
            prev_min3[j] = this_min3;
        }
    }
    out as _
}

#[allow(unused)]
pub fn part1_naive(mut s: &[u8]) -> usize {
    let mut grid = [[0xff_u8; N + 2]; N + 2];
    for i in 1..=N {
        for j in 1..=N {
            grid[i][j] = s.get_digit_at(j - 1);
        }
        s = s.advance(N + 1);
    }
    let mut sum = 0;
    for i in 1..=N {
        for j in 1..=N {
            let x = grid[i][j];
            if x < grid[i - 1][j] && x < grid[i][j - 1] && x < grid[i + 1][j] && x < grid[i][j + 1]
            {
                sum += (1 + x) as usize;
            }
        }
    }
    sum
}

#[derive(Debug, Default, Copy, Clone)]
struct Element {
    parent: u16,
    rank: u16,
}

struct UnionFind {
    elements: [Element; M],
    len: u16,
}

impl UnionFind {
    pub fn new() -> Self {
        Self { elements: [Element::default(); M], len: 0 }
    }

    fn get(&self, x: u16) -> &Element {
        unsafe { &*self.elements.get_unchecked(x as usize) }
    }

    fn get_mut(&mut self, x: u16) -> &mut Element {
        unsafe { &mut *self.elements.get_unchecked_mut(x as usize) }
    }

    pub fn add_leaf(&mut self) -> u16 {
        let x = self.len;
        self.len += 1;
        *self.get_mut(x) = Element { parent: x, rank: 0 };
        x
    }

    pub fn find(&mut self, mut x: u16) -> u16 {
        loop {
            let e = self.get(x);
            let p = e.parent;
            if p == x {
                break;
            }
            let gp = self.get(p).parent;
            self.get_mut(x).parent = gp;
            x = gp;
        }
        x
    }

    pub fn union(&mut self, x: u16, y: u16) {
        let x = self.find(x);
        let y = self.find(y);
        if x == y {
            return;
        }
        let (rx, ry) = (self.get(x).rank, self.get(y).rank);
        let (x, y) = if rx < ry { (y, x) } else { (x, y) };
        self.get_mut(y).parent = x;
        if rx == ry {
            self.get_mut(x).rank += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.len as _
    }
}

#[allow(unused)]
pub fn part2_dfs(mut s: &[u8]) -> usize {
    // Simple DFS implementation - works but is 3x slower than union-find

    fn dfs(grid: &mut [u8], i: usize, j: usize, coord: usize) -> usize {
        if grid.get_at(coord) == b'9' {
            0
        } else {
            grid.set_at(coord, b'9');
            let mut size = 1;
            if j != 0 {
                size += dfs(grid, i, j - 1, coord - 1);
            }
            if j != N - 1 {
                size += dfs(grid, i, j + 1, coord + 1);
            }
            if i != 0 {
                size += dfs(grid, i - 1, j, coord - N - 1);
            }
            if i != N - 1 {
                size += dfs(grid, i + 1, j, coord + N + 1);
            }
            size
        }
    }

    let mut grid = s.to_vec();
    let mut sizes = vec![];
    for i in 0..N {
        let row = i * (N + 1);
        for j in 0..N {
            let size = dfs(&mut grid, i, j, row + j);
            if size != 0 {
                sizes.push(size);
            }
        }
    }
    sizes.sort_unstable();
    sizes[sizes.len() - 3..].iter().product::<usize>()
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    // This is basically a 4-connected CCL problem for binary images.

    let mut labels_this = [u16::MAX; M];
    let mut labels_prev = [u16::MAX; M];

    let mut counts = [0_usize; M];
    let mut uf = UnionFind::new();

    for _ in 1..=N {
        for j in 1..=N {
            labels_this[j] = if s.get_at(j - 1) != b'9' {
                let n = labels_prev[j];
                let w = labels_this[j - 1];
                let label = match (n, w) {
                    (u16::MAX, u16::MAX) => uf.add_leaf(),
                    (u16::MAX, _) => w,
                    (_, u16::MAX) => n,
                    (n, w) if n == w => n,
                    _ => {
                        uf.union(n, w);
                        n.min(w)
                    }
                };
                counts.add_at(label as _, 1);
                label
            } else {
                u16::MAX
            };
        }
        s = s.advance(N + 1);
        labels_prev = labels_this;
    }

    let mut sizes = vec![0_usize; uf.len()];
    for (i, &c) in counts.iter().take(uf.len() as usize).enumerate() {
        sizes[uf.find(i as _) as usize] += c;
    }
    sizes.retain(|&s| s != 0);
    sizes.sort_unstable();
    sizes[sizes.len() - 3..].iter().product::<usize>()
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 518);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 949905);
}
