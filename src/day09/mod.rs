use crate::utils::*;

const N: usize = 100;
const M: usize = 1 << 10;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
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
