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

#[derive(Clone)]
struct Node {
    mask: u8,
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
    pub fn new(name: [u8; 2], mask: u8) -> Self {
        Self { mask, edges: ArrayVec::new(), name }
    }
}

type VisitedCounts = ArrayVec<[usize; 256], MAX_NODES>;

#[derive(Debug, Clone)]
struct G {
    nodes: ArrayVec<Node, MAX_NODES>,
    n_small: usize,
}

impl G {
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

    fn count_paths(&self, i: usize, mut mask: u8) -> usize {
        if i == END {
            return 1;
        }
        let node = &self.nodes[i];
        if node.mask & mask != 0 {
            return 0;
        }
        mask |= node.mask;
        node.edges.iter().map(|&j| self.count_paths(j, mask)).sum()
    }

    fn count_paths_2(&self, i: usize, mut mask: u8, mut twice: u8) -> usize {
        if i == END {
            return 1;
        }
        let node = &self.nodes[i];
        if node.mask & mask != 0 {
            if twice == 0 {
                twice = node.mask;
            } else {
                return 0;
            }
        } else {
            mask |= node.mask;
        }
        node.edges.iter().map(|&j| self.count_paths_2(j, mask, twice)).sum()
    }
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    let mut g = G::parse(s);
    g.count_paths(START, 0)
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    let mut g = G::parse(s);
    g.count_paths_2(START, 0, 0)
}

#[test]
fn test_day12_part1() {
    assert_eq!(part1(input()), 5756);
}

#[test]
fn test_day12_part2() {
    assert_eq!(part2(input()), 144603);
}
