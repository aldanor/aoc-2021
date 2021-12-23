use std::fmt::Debug;
use std::ops::{Deref, DerefMut, Range, Sub};

use petgraph::graph::{IndexType, NodeIndex, UnGraph};

const fn range<const N: usize>() -> [usize; N] {
    let mut i = 0;
    let mut out = [0; N];
    while i < N {
        out[i] = i;
        i += 1;
    }
    out
}

pub trait Endpoint: Copy + Ord + Sized + Default + Debug {}

impl<T: Copy + Ord + Sized + Default + Debug> Endpoint for T {}

// Right-open interval [a; b).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Interval<T>([T; 2]);

impl<T> Deref for Interval<T> {
    type Target = [T; 2];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Interval<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Endpoint> Interval<T> {
    pub fn new(start: T, end: T) -> Self {
        Self([start, end])
    }

    pub fn start(&self) -> T {
        self.0[0]
    }

    pub fn end(&self) -> T {
        self.0[1]
    }

    pub fn distance(&self) -> T
    where
        T: Sub<Output = T>,
    {
        (self.end() - self.start()).max(T::default())
    }

    pub fn as_range(&self) -> Range<T> {
        self.start()..self.end()
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let start = self.start().max(other.start());
        let end = self.end().min(other.end());
        if start < end {
            Some(Self::new(start, end))
        } else {
            None
        }
    }

    pub fn intersect_unchecked(&self, other: &Self) -> Self {
        Self::new(self.start().max(other.start()), self.end().min(other.end()))
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        self.start().max(other.start()) < self.end().min(other.end())
    }
}

impl<T> From<[T; 2]> for Interval<T> {
    fn from(v: [T; 2]) -> Self {
        Self(v)
    }
}

impl<T> From<Interval<T>> for [T; 2] {
    fn from(v: Interval<T>) -> Self {
        v.0
    }
}

// Right-open D-dimensional cuboid.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region<T, const D: usize>([Interval<T>; D]);

impl<T, const D: usize> Deref for Region<T, D> {
    type Target = [Interval<T>; D];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T, const D: usize> DerefMut for Region<T, D> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Endpoint, const D: usize> Default for Region<T, D> {
    fn default() -> Self {
        Self(range::<D>().map(|_| Interval::default()))
    }
}

impl<T, const D: usize> From<[Interval<T>; D]> for Region<T, D> {
    fn from(v: [Interval<T>; D]) -> Self {
        Self(v)
    }
}

impl<T, const D: usize> From<Region<T, D>> for [Interval<T>; D] {
    fn from(v: Region<T, D>) -> Self {
        v.0
    }
}

impl<T, const D: usize> From<[[T; 2]; D]> for Region<T, D> {
    fn from(v: [[T; 2]; D]) -> Self {
        Self(v.map(Into::into))
    }
}

impl<T, const D: usize> From<Region<T, D>> for [[T; 2]; D] {
    fn from(v: Region<T, D>) -> Self {
        v.0.map(Into::into)
    }
}

impl<T: Endpoint, const D: usize> Region<T, D> {
    pub fn volume(&self) -> T
    where
        T: Sub<Output = T> + iter::Product,
    {
        (0..D).map(|i| self[i].distance()).product::<T>()
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let mut out = Self::default();
        for i in 0..D {
            out[i] = self[i].intersect(&other[i])?;
        }
        Some(out)
    }

    pub fn intersect_unchecked(&self, other: &Self) -> Self {
        let mut out = Self::default();
        for i in 0..D {
            out[i] = self[i].intersect_unchecked(&other[i]);
        }
        out
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        (0..D).all(|i| self[i].overlaps(&other[i]))
    }
}

// pub fn intersect_regions<T: Number, S: IntoIterator<Item = &Region<T, D>>, const D: usize>(
//     regions: S,
// ) -> T {
//     // TODO: intersect_unchecked(), intersect_regions_unchecked()
//     regions.into_iter().fold()
// }

#[derive(Debug, Clone)]
pub struct IntersectionGraph<T, const D: usize> {
    g: UnGraph<Region<T, D>, ()>,
}

impl<T, const D: usize> Deref for IntersectionGraph<T, D> {
    type Target = UnGraph<Region<T, D>, ()>;

    fn deref(&self) -> &Self::Target {
        &self.g
    }
}

impl<T: Endpoint, const D: usize> IntersectionGraph<T, D> {
    pub fn new() -> Self {
        let g = UnGraph::new_undirected();
        Self { g }
    }

    pub fn region(&self, i: usize) -> &Region<T, D> {
        &self.g.raw_nodes()[i].weight
    }

    pub fn from_regions_bruteforce<S: AsRef<[R]>, R: Into<Region<T, D>> + Copy>(
        regions: S,
    ) -> Self {
        let regions = regions.as_ref();
        let n = regions.len();

        let mut g = UnGraph::new_undirected();
        for &r in regions {
            g.add_node(r.into());
        }
        for i in 0..n - 1 {
            for j in i + 1..n {
                if g.raw_nodes()[i].weight.overlaps(&g.raw_nodes()[j].weight) {
                    g.add_edge(NodeIndex::new(i), NodeIndex::new(j), ());
                }
            }
        }

        Self { g }
    }
}

/*
   index = {}
   nbrs = {}
   for u in G:
       index[u] = len(index)
       # Neighbors of u that appear after u in the iteration order of G.
       nbrs[u] = {v for v in G[u] if v not in index}

   queue = deque(([u], sorted(nbrs[u], key=index.__getitem__)) for u in G)
   # Loop invariants:
   # 1. len(base) is nondecreasing.
   # 2. (base + cnbrs) is sorted with respect to the iteration order of G.
   # 3. cnbrs is a set of common neighbors of nodes in base.
   while queue:
       base, cnbrs = map(list, queue.popleft())
       yield base
       for i, u in enumerate(cnbrs):
           # Use generators to reduce memory consumption.
           queue.append(
               (
                   chain(base, [u]),
                   filter(nbrs[u].__contains__, islice(cnbrs, i + 1, None)),
               )
           )
*/

use std::collections::{BTreeSet, HashSet, VecDeque};
use std::iter;

pub fn find_cliques<N, E, Ix: IndexType>(
    g: &UnGraph<N, E, Ix>,
) -> impl Iterator<Item = Vec<usize>> {
    type NodeSet = BTreeSet<usize>;

    let adj: Vec<NodeSet> = g
        .node_indices()
        .map(|u| g.neighbors(u).filter(|&v| v != u).map(NodeIndex::index).collect())
        .collect();
    let mut subg: NodeSet = g.node_indices().map(NodeIndex::index).collect();
    let mut cand = subg.clone();

    let u = subg.iter().copied().max_by_key(|&u| adj[u].intersection(&cand).count()).unwrap_or(0);
    let mut ext_u: NodeSet = cand.difference(&adj[u]).copied().collect();

    let mut stack: Vec<(NodeSet, NodeSet, NodeSet)> = vec![];
    let mut qq: Vec<usize> = vec![!0];

    iter::from_fn(move || {
        loop {
            if let Some(q) = ext_u.pop_first() {
                cand.remove(&q);
                let n = qq.len();
                qq[n - 1] = q;
                // if let Some(last) = qq.last_mut() {
                //     *last = q;
                // } else {
                //     return None;
                // }
                let adj_q = &adj[q];
                let mut subg_q: NodeSet = subg.intersection(adj_q).copied().collect();
                if subg_q.is_empty() {
                    return Some(qq.clone());
                }
                let mut cand_q: NodeSet = cand.intersection(adj_q).copied().collect();
                if !cand_q.is_empty() {
                    stack.push((subg.clone(), cand.clone(), ext_u.clone()));
                    qq.push(!0);
                    subg = subg_q;
                    cand = cand_q;
                    let u = subg
                        .iter()
                        .copied()
                        .max_by_key(|&u| adj[u].intersection(&cand).count())
                        .unwrap_or(0);
                    ext_u = cand.difference(&adj[u]).copied().collect();
                }
            } else {
                qq.pop();
                if let Some((subg_new, cand_new, ext_u_new)) = stack.pop() {
                    subg = subg_new;
                    cand = cand_new;
                    ext_u = ext_u_new;
                } else {
                    return None;
                }
            }
        }
        None
    })
}

pub fn enumerate_all_cliques<N, E, Ix: IndexType>(
    g: &UnGraph<N, E, Ix>,
) -> impl Iterator<Item = Vec<usize>> {
    // adapted from networkx.Graph.enumerate_all_cliques()
    let neighbors: Vec<BTreeSet<usize>> = g
        .node_indices()
        .map(|i| g.neighbors(i).filter(|&j| j > i).map(|i| i.index()).collect())
        .collect();
    let mut queue: VecDeque<(Vec<usize>, Vec<usize>)> = g
        .node_indices()
        .zip(&neighbors)
        .map(|(u, neighbors)| (vec![u.index()], neighbors.iter().copied().collect()))
        .collect();
    iter::from_fn(move || {
        queue.pop_front().map(|(base, common_neighbors)| {
            for (i, &u) in common_neighbors.iter().enumerate() {
                let mut new_base = base.clone();
                new_base.push(u);
                let new_common_neighbors = common_neighbors[i + 1..]
                    .iter()
                    .copied()
                    .filter(|&j| neighbors[u].contains(&j))
                    .collect();
                queue.push_back((new_base, new_common_neighbors));
            }
            base
        })
    })
}
