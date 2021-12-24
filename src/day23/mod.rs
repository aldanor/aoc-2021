use ahash::{AHashMap, AHashSet};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::fmt::{self, Debug, Formatter};
use std::iter;
use std::ops::Deref;

use crate::utils::*;

type Pod = u8;
const NIL: Pod = 0xff;
const N_PODS: usize = 4;

type Cost = i32;
const COSTS: [Cost; 4] = [1, 10, 100, 1000];

const N_HALLWAYS: usize = 7;
const N_BURROWS: usize = 4;
const MAX_DEPTH: usize = 4;

const HALLWAY_COORDS: [u8; 8] = [NIL, 0, 1, 3, 5, 7, 9, 10];
const HALLWAY_COORD_TO_BIT: [u8; 11] = [7, 6, NIL, 5, NIL, 4, NIL, 3, NIL, 2, 1];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Location(u8); // high 4 bits - hallway 1..=7, 2 bits burrow id, 2 bits burrow depth

impl Location {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn nil() -> Self {
        Self(NIL)
    }

    pub const fn hallway(pos: u8) -> Self {
        Self::new(pos << 4)
    }

    pub const fn burrow(burrow: u8, depth: u8) -> Self {
        Self::new(burrow << 2 | depth)
    }

    pub const fn is_nil(&self) -> bool {
        self.0 == NIL
    }

    pub const fn is_hallway(self) -> bool {
        (self.0 & 0xf0) != 0 && !self.is_nil()
    }

    pub const fn is_burrow(self) -> bool {
        !self.is_hallway()
    }

    pub const fn get_hallway(self) -> u8 {
        (self.0 & 0xf0) >> 4
    }

    pub const fn get_burrow(self) -> (u8, u8) {
        ((self.0 & 0x0c) >> 2, self.0 & 0x03)
    }

    pub const fn value(self) -> u8 {
        self.0
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.is_nil() {
            write!(f, "nil")
        } else if self.is_hallway() {
            write!(f, "h[{}]", self.get_hallway())
        } else {
            let (b, d) = self.get_burrow();
            write!(f, "b[{},{}]", b, d)
        }
    }
}

// high 1..=7 bits indicate whether the cell is free or not
// (note: bit 7 represents 1, bit represents 2, ... bit 1 represents 7, bit 0 is unused)
#[derive(Clone, Copy, PartialEq, Eq, Default)]
struct Hallway(u8);

impl Hallway {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    pub const fn is_occupied(self, hallway: u8) -> bool {
        (self.0 & (1 << (8 - hallway))) != 0
    }

    // pub const fn get_free_hallways(self, pod: u8, burrow: u8) -> [u8; 8] {
    //     const fn table() -> [[[[u8; 8]; 128]; N_BURROWS]; N_PODS] {
    //         let mut out = [[[[NIL; 8]; 128]; N_BURROWS]; N_PODS];
    //         let mut i_loc = 0;
    //         while i_loc < 128 {
    //             let loc = Location::new(i_loc << 1);
    //
    //             i_loc += 1;
    //         }
    //         out
    //     }
    //     const TABLE: [[[[u8; 8]; 128]; N_BURROWS]; N_PODS] = table();
    //     TABLE[pod as usize][burrow as usize][(self.0 >> 1) as usize]
    // }

    pub const fn toggle(mut self, hallway: u8) -> Self {
        self.0 ^= (1 << (8 - hallway));
        self
    }

    pub const fn exits_blocked(self) -> bool {
        self.0 & 0x7c == 0x7c
    }

    pub const fn is_path_free(self, burrow: u8, hallway: u8) -> bool {
        // is path free between burrow entry (0..=3) and hallway cell (1..=7)
        // (note: this doesn't account for the target hallway cell itself)
        const fn build_path_masks() -> [[u8; 8]; N_BURROWS] {
            let mut out = [[0_u8; 8]; N_BURROWS];
            let mut burrow = 0;
            while burrow < N_BURROWS {
                let mut hallway = 1;
                while hallway <= 7 {
                    let x_burrow = (2 + 2 * burrow) as i8;
                    let x_hallway = (HALLWAY_COORDS[hallway as usize]) as i8;
                    let (start, end) = if x_hallway < x_burrow {
                        (x_hallway + 1, x_burrow)
                    } else {
                        (x_burrow, x_hallway)
                    };
                    let mut mask = 0;
                    let mut x = start;
                    while x < end {
                        let bit = HALLWAY_COORD_TO_BIT[x as usize];
                        if bit != NIL {
                            mask |= 1 << bit;
                        }
                        x += 1;
                    }
                    out[burrow][hallway] = mask;
                    hallway += 1;
                }
                burrow += 1;
            }
            out
        }
        const PATH_MASKS: [[u8; 8]; N_BURROWS] = build_path_masks();
        return (PATH_MASKS[burrow as usize][hallway as usize] & self.0) == 0;
    }
}

impl Debug for Hallway {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut out = [b'_'; 11];
        for i in 1..=7 {
            out[HALLWAY_COORDS[i] as usize] =
                if self.0 & (1 << (8 - i)) != 0 { b'x' } else { b'.' };
        }
        write!(f, "{}", std::str::from_utf8(&out).unwrap_or_default())
    }
}

const fn added_min_cost(pod: Pod, burrow: u8, hallway: u8) -> Cost {
    // how much is added to the min cost if a pod moves from burrow to hallway
    // if the pod moves towards its burrow, it's optimal and doesn't change extra cost
    // (so we only need to account for the cases when it moves away from its burrow)
    const fn build_table() -> [[[Cost; 8]; N_BURROWS]; N_PODS] {
        let mut out = [[[0; 8]; N_BURROWS]; N_PODS];
        let mut pod = 0;
        while pod < N_PODS {
            let mut burrow = 0;
            while burrow < N_BURROWS {
                let mut hallway = 1;
                while hallway <= 7 {
                    let x_hallway = HALLWAY_COORDS[hallway as usize] as Cost;
                    let x_burrow = 2 + 2 * burrow as Cost;
                    let x_pod = 2 + 2 * pod as Cost;
                    let mut min_cost = (x_pod - x_burrow).abs();
                    if min_cost < 2 {
                        min_cost = 2; // move out of the home burrow and then back in
                    }
                    let real_cost = (x_hallway - x_burrow).abs() + (x_pod - x_hallway).abs();
                    debug_assert!(real_cost >= min_cost);
                    out[pod][burrow][hallway] = (real_cost - min_cost) * COSTS[pod];
                    hallway += 1;
                }
                burrow += 1;
            }
            pod += 1;
        }
        out
    }
    const TABLE: [[[Cost; 8]; N_BURROWS]; N_PODS] = build_table();
    TABLE[pod as usize][burrow as usize][hallway as usize]
}

#[derive(Clone, Copy)]
struct GameState<const D: usize> {
    pods: [[Location; D]; N_PODS], // exact locations of all pods (or NIL)
    n_remaining: [u8; N_PODS],     // remaining number of pods to place, 0..=4
    burrows: [u8; N_BURROWS],      // how many pods are currently in each burrow
    min_cost: Cost,                // total energy cost incurred so far
    hallway: Hallway,              // free/occupied cells in the hallway
    hallway_pods: [Pod; 8],        // what type of pod is in each hallway cell (or NIL)
}

impl<const D: usize> GameState<D> {
    pub fn parse(mut s: &[u8]) -> Self {
        let mut pods = [[Location::nil(); D]; N_PODS];
        let mut n_pods = [0_usize; N_PODS];
        s = s.skip_past(b'\n', 0);
        for depth in (0..D).rev() {
            s = s.skip_past(b'\n', 1);
            for burrow in 0..N_BURROWS {
                s = s.advance(2);
                let pod = (s.get_first() - b'A') as usize;
                let loc = Location::burrow(burrow as _, depth as _);
                pods[pod][n_pods[pod]] = loc;
                n_pods[pod] += 1;
            }
        }
        let mut n_remaining = [D as u8; N_PODS];
        'x: for pod in 0..N_PODS {
            for depth in 0..D {
                if let Some(i) =
                    pods[pod].iter().position(|&loc| loc == Location::burrow(pod as _, depth as _))
                {
                    n_remaining[pod] -= 1;
                    pods[pod][i] = Location::nil();
                } else {
                    continue 'x;
                }
            }
        }
        for pod in 0..N_PODS {
            pods[pod].sort_unstable();
        }
        let hallway = Hallway::empty();
        let min_cost = 0;
        let burrows = [D as u8; N_BURROWS];
        let hallway_pods = [NIL; 8];
        Self { pods, n_remaining, burrows, min_cost, hallway, hallway_pods }
    }

    pub fn initial_min_cost(&self) -> Cost {
        // this function is only valid in the initial state before any moves are made
        let mut cost = 0;
        for pod in 0..N_PODS {
            for i in 0..self.n_remaining[pod] {
                let loc = self.pods[pod][i as usize];
                // we know all pods are initially in the burrow state
                let (b, d) = loc.get_burrow();
                let move_out_cost = D as u8 - d;
                let move_in_cost = i + 1;
                let hallway_cost = 2 * (b as i8 - pod as i8).abs().max(1) as u8;
                cost += (move_out_cost + hallway_cost + move_in_cost) as Cost * COSTS[pod];
            }
        }
        cost
    }

    pub fn is_done(&self) -> bool {
        self.n_remaining == [0; N_PODS]
    }

    pub fn get_at(&self, loc: Location) -> Option<Pod> {
        // only used for debugging and initialization
        if loc.is_burrow() {
            let (b, d) = loc.get_burrow();
            let n_placed = D as u8 - self.n_remaining[b as usize];
            if n_placed >= d + 1 {
                return Some(b);
            }
        }
        for pod in 0..N_PODS {
            for i in 0..D {
                if self.pods[pod][i] == loc {
                    return Some(pod as _);
                }
            }
        }
        None
    }

    pub fn key(&self) -> &[u32; D] {
        // used for hashing
        unsafe { std::mem::transmute(&self.pods) }
    }

    pub fn is_dead_end(&self) -> bool {
        self.hallway.exits_blocked()
            && (0..N_BURROWS).all(|b| {
                self.burrows[b] != (D as u8 - self.n_remaining[b]) || {
                    let (left, right) = (2 + b, 3 + b);
                    self.hallway_pods[left] != b as u8 && self.hallway_pods[right] != b as u8
                }
            })
    }

    pub fn iter_moves(&self, mut callback: impl FnMut(Self)) {
        for pod in (0..N_PODS).rev() {
            let n_remaining = self.n_remaining[pod];
            let n_placed = D as u8 - n_remaining;
            for i in 0..n_remaining as usize {
                let loc = self.pods[pod][i];
                if loc.is_hallway() {
                    // case 1: the pod is in the hallway
                    let h = loc.get_hallway();
                    debug_assert!(self.hallway.is_occupied(h));
                    // first check if the path is free to move back to the home burrow
                    if self.hallway.is_path_free(pod as _, h) {
                        // then, check if all pods in the home burrow are of correct type
                        if n_placed == self.burrows[pod] {
                            // the pod can move back to its home
                            let mut state = self.clone();
                            state.hallway = state.hallway.toggle(h);
                            state.hallway_pods[h as usize] = NIL;
                            state.n_remaining[pod] -= 1;
                            state.burrows[pod] += 1;
                            state.pods[pod][i..].rotate_left(1);
                            state.pods[pod][D - 1] = Location::nil();
                            // note: moving into its own burrow doesn't change min cost
                            callback(state);
                        }
                    }
                } else {
                    // case 2: the pod is still in the burrow
                    let (b, d) = loc.get_burrow();
                    debug_assert!(self.burrows[b as usize] >= 1);
                    // first check that it's the top pod in the burrow so it's not blocked
                    if d + 1 == self.burrows[b as usize] {
                        for h in 1..=7 {
                            // check if the pod can move into the hallway cell
                            // TODO: optimize iteration over free hallways
                            if !self.hallway.is_occupied(h) && self.hallway.is_path_free(b, h) {
                                // ok, we can safely move out
                                // the pod can move back to its home
                                let mut state = self.clone();
                                state.hallway = state.hallway.toggle(h);
                                state.hallway_pods[h as usize] = pod as u8;
                                state.burrows[b as usize] -= 1;
                                state.min_cost += added_min_cost(pod as _, b, h);
                                state.pods[pod][i] = Location::hallway(h);
                                state.pods[pod].sort_unstable();
                                if !state.is_dead_end() {
                                    callback(state);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<const D: usize> PartialEq for GameState<D> {
    fn eq(&self, other: &Self) -> bool {
        self.pods.eq(&other.pods)
    }
}

impl<const D: usize> Eq for GameState<D> {}

impl<const D: usize> Debug for GameState<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        let mut hallway = [b'.'; 11];
        for h in 1..=7 {
            if let Some(pod) = self.get_at(Location::hallway(h)) {
                hallway[HALLWAY_COORDS[h as usize] as usize] = b'A' + pod;
            }
        }
        writeln!(f, "{}", std::str::from_utf8(&hallway).unwrap_or_default())?;
        for depth in (0..D).rev() {
            write!(f, " ")?;
            for burrow in 0..N_PODS {
                write!(f, " ")?;
                if let Some(pod) = self.get_at(Location::burrow(burrow as _, depth as _)) {
                    write!(f, "{}", char::from(b'A' + pod))?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f);
        writeln!(f, "n_remaining = {:?}", self.n_remaining)?;
        writeln!(f, "    burrows = {:?}", self.burrows)?;
        writeln!(f, "    hallway = {:?}", self.hallway)?;
        writeln!(f, "   min_cost = {}", self.min_cost)?;
        Ok(())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Candidate<const D: usize> {
    state: GameState<D>,
}

impl<const D: usize> Candidate<D> {
    pub fn new(state: GameState<D>) -> Self {
        Self { state }
    }

    pub fn score(&self) -> (usize, Cost) {
        // return (0, -self.min_cost);
        // since we place this on a max-heap, states with the *highest* score will be checked first
        let pod_scores = self.state.pods.map(|pods| {
            pods.iter()
                .map(|&loc| {
                    if loc.is_nil() {
                        2
                        // N_PODS * N_PODS
                    } else if loc.is_hallway() {
                        1
                        // N_PODS
                    } else {
                        0
                    }
                })
                .sum::<usize>()
        });
        let pod_score = (0..N_PODS).map(|i| pod_scores[i] << (6 * i)).sum::<usize>();
        let min_cost = self.state.min_cost;
        (pod_score, -min_cost)
    }
}

impl<const D: usize> Deref for Candidate<D> {
    type Target = GameState<D>;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<const D: usize> PartialOrd for Candidate<D> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.score().partial_cmp(&other.score())
    }
}

impl<const D: usize> Ord for Candidate<D> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score().cmp(&other.score())
    }
}

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

fn solve<const D: usize, const V: bool>(state: GameState<D>) -> Cost {
    let initial_cost = state.initial_min_cost();

    let mut queue = Vec::with_capacity(1 << 15);
    queue.push(Candidate::new(state));
    let mut visited = AHashMap::<_, Cost>::with_capacity((1 << 18) * V as usize);

    let mut min_extra_cost = Cost::MAX;
    let mut n = 0;
    let mut n_skipped = 0;
    while let Some(candidate) = queue.pop() {
        n += 1;
        if V {
            visited
                .entry(*candidate.key())
                .and_modify(|cost| *cost = (*cost).min(candidate.min_cost))
                .or_insert(candidate.min_cost);
        }
        if candidate.min_cost >= min_extra_cost {
            n_skipped += 1;
            continue;
        } else if candidate.is_done() {
            min_extra_cost = candidate.min_cost;
        } else {
            candidate.iter_moves(|next| {
                if !V || visited.get(next.key()).map(|cost| next.min_cost < *cost).unwrap_or(true) {
                    queue.push(Candidate::new(next));
                }
            });
        }
    }
    initial_cost + min_extra_cost
}

#[inline]
pub fn part1(mut s: &[u8]) -> Cost {
    solve::<2, false>(GameState::parse(s))
}

#[inline]
pub fn part2(mut s: &[u8]) -> Cost {
    let mut v = s;
    for _ in 0..3 {
        s = s.skip_past(b'\n', 1);
    }
    let i = v.len() - s.len() - 1;
    let mut s = Vec::from(&v[..i]);
    s.extend(b"  #D#C#B#A#\n");
    s.extend(b"  #D#B#A#C#\n");
    s.extend(&v[i..]);
    let state = GameState::parse(&s);
    solve::<4, true>(state)
}

#[test]
fn test_day23_part1() {
    assert_eq!(part1(input()), 14510);
}

#[test]
fn test_day23_part2() {
    assert_eq!(part2(input()), 49180);
}
