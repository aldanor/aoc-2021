use std::fmt::{self, Debug};

use arrayvec::ArrayVec;

use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

const MAX_NODES: usize = 16;
const MAX_EDGES: usize = 8;
const START: usize = 0;
const END: usize = 1;
const START_NAME: [u8; 2] = [0, 0];
const END_NAME: [u8; 2] = [0xff, 0xff];

type Mask = u8;
type VisitedCache = ArrayVec<[[u32; 256]; 2], MAX_NODES>;

#[inline]
fn cache_get_mut(cache: &mut VisitedCache, i: usize, twice: bool, mask: Mask) -> &mut u32 {
    unsafe {
        cache
            .get_unchecked_mut(i)
            .get_unchecked_mut(twice as usize)
            .get_unchecked_mut(mask as usize)
    }
}

#[derive(Clone)]
struct Node {
    mask: Mask,
    edges: ArrayVec<usize, MAX_EDGES>,
    name: [u8; 2],
}

impl Debug for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Node(")?;
        if self.name == START_NAME {
            write!(f, "start")?;
        } else if self.name == END_NAME {
            write!(f, "end")?;
        } else {
            write!(f, "\"{}{}\"", char::from(self.name[0]), char::from(self.name[1]))?;
        }
        if self.mask != 0 {
            write!(f, ", mask={}", self.mask)?;
        }
        write!(f, ", edges={:?})", self.edges)
    }
}

impl Node {
    pub fn new(name: [u8; 2], mask: Mask) -> Self {
        Self { mask, edges: ArrayVec::new(), name }
    }
}

#[derive(Debug, Clone)]
struct Graph {
    nodes: ArrayVec<Node, MAX_NODES>,
    n_small: usize,
}

impl Graph {
    pub fn new() -> Self {
        let mut nodes = ArrayVec::new();
        nodes.push(Node::new(START_NAME, 0));
        nodes.push(Node::new(END_NAME, 0));
        Self { nodes, n_small: 0 }
    }

    fn find_or_insert_node(&mut self, name: &[u8]) -> usize {
        match name.len() {
            5 => START,
            3 => END,
            2 => {
                let name = [name.get_at(0), name.get_at(1)];
                self.nodes.iter().position(|n| n.name == name).unwrap_or_else(|| {
                    let index = self.nodes.len();
                    let mask = if name[0].is_ascii_lowercase() {
                        self.n_small += 1;
                        assert!(self.n_small <= 8);
                        1 << (self.n_small - 1)
                    } else {
                        0
                    };
                    self.nodes.push(Node::new(name, mask));
                    index
                })
            }
            _ => unsafe { core::hint::unreachable_unchecked() },
        }
    }

    fn add_edge(&mut self, i: usize, j: usize) {
        if i != END && j != START {
            self.nodes[i].edges.push(j);
        }
        if j != END && i != START {
            self.nodes[j].edges.push(i);
        }
    }

    pub fn trim(mut self) -> Self {
        // remove nodes that will be unused anyway (that have a single edge leading to small)
        let mut remove = ArrayVec::<usize, MAX_NODES>::new();
        for (i, node) in self.nodes.iter().enumerate().rev() {
            if node.edges.len() == 1 && self.nodes[node.edges[0]].mask != 0 {
                remove.push(i);
            }
        }
        for i in remove {
            if self.nodes[i].mask != 0 {
                self.n_small -= 1;
            }
            for node in &mut self.nodes {
                node.edges = node
                    .edges
                    .iter()
                    .filter_map(|&e| {
                        if e == i {
                            None
                        } else if e < i {
                            Some(e)
                        } else {
                            Some(e - 1)
                        }
                    })
                    .collect();
            }
            self.nodes.remove(i);
        }
        self
    }

    pub fn parse(mut s: &[u8]) -> Self {
        let mut g = Self::new();
        while s.len() > 1 {
            let k = s.memchr(b'-');
            let i = g.find_or_insert_node(&s[..k]);
            s = s.advance(k + 1);
            let k = s.memchr(b'\n');
            let j = g.find_or_insert_node(&s[..k]);
            s = s.advance(k + 1);
            g.add_edge(i, j);
        }
        g
    }

    pub fn count_paths(&self, allow_twice: bool) -> usize {
        let mut cache = VisitedCache::new();
        for _ in 0..self.nodes.len() {
            cache.push([[u32::MAX; 256]; 2]);
        }
        self.count_paths_impl(START, false, 0, allow_twice, &mut cache) as _
    }

    fn count_paths_impl(
        &self, i: usize, twice: bool, mask: Mask, allow_twice: bool, cache: &mut VisitedCache,
    ) -> u32 {
        let cached = *cache_get_mut(cache, i, twice, mask);
        if cached != u32::MAX {
            cached
        } else if i == END {
            1
        } else {
            let node = unsafe { self.nodes.get_unchecked(i) };
            let (new_mask, new_twice) = if node.mask & mask != 0 {
                if allow_twice && !twice {
                    (mask, true)
                } else {
                    return 0;
                }
            } else {
                (mask | node.mask, twice)
            };
            let count = node
                .edges
                .iter()
                .map(|&j| self.count_paths_impl(j, new_twice, new_mask, allow_twice, cache))
                .sum();
            *cache_get_mut(cache, i, twice, mask) = count;
            count
        }
    }

    #[allow(unused)]
    pub fn count_paths_naive(&self, allow_twice: bool) -> usize {
        self.count_paths_naive_impl(START, 0, false, allow_twice)
    }

    fn count_paths_naive_impl(
        &self, i: usize, mut mask: Mask, mut twice: bool, allow_twice: bool,
    ) -> usize {
        if i == END {
            return 1;
        }
        let node = &self.nodes[i];
        if node.mask & mask != 0 {
            if allow_twice && !twice {
                twice = true;
            } else {
                return 0;
            }
        } else {
            mask |= node.mask;
        }
        node.edges.iter().map(|&j| self.count_paths_naive_impl(j, mask, twice, allow_twice)).sum()
    }
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    Graph::parse(s).trim().count_paths(false)
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    Graph::parse(s).trim().count_paths(true)
}

#[test]
fn test_day12_part1() {
    assert_eq!(part1(input()), 5756);
}

#[test]
fn test_day12_part2() {
    assert_eq!(part2(input()), 144603);
}
