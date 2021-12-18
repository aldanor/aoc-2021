use std::fmt::{self, Debug, Display};
use std::mem;
use std::ptr;

use arrayvec::ArrayVec;

use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

const N: usize = 32;

#[inline]
fn arrayvec_concat<T: Copy, const N: usize>(left: &mut ArrayVec<T, N>, right: &ArrayVec<T, N>) {
    unsafe {
        ptr::copy_nonoverlapping(right.as_ptr(), left.as_mut_ptr().add(left.len()), N - left.len());
        left.set_len(left.len() + right.len());
    }
}

// This is only used for debugging purposes (e.g. displaying)
#[derive(Debug, Clone)]
enum Node {
    Literal(u8),
    Pair(Box<Node>, Box<Node>),
}

impl Node {
    pub fn fold<T>(&self, literal: impl Copy + Fn(u8) -> T, pair: impl Copy + Fn(T, T) -> T) -> T {
        match self {
            Self::Literal(v) => literal(*v),
            Self::Pair(left, right) => pair(left.fold(literal, pair), right.fold(literal, pair)),
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(v) => write!(f, "{}", v),
            Self::Pair(left, right) => write!(f, "[{},{}]", left, right),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
struct Cell {
    pub value: u8,
    pub depth: u8,
}

impl Cell {
    #[inline]
    pub const fn new(value: u8, depth: u8) -> Self {
        Self { value, depth }
    }
}

#[derive(Clone, Default)]
struct Number {
    cells: ArrayVec<Cell, N>,
}

impl Number {
    pub fn new(values: &[u8], depths: &[u8]) -> Self {
        let length = values.len();
        assert!(length <= N);
        assert_eq!(values.len(), depths.len());
        let mut cells = ArrayVec::new();
        for i in 0..length {
            cells.push(Cell::new(values[i], depths[i]));
        }
        Self { cells }
    }

    pub fn parse(s: impl AsRef<[u8]>) -> Self {
        let s = s.as_ref();
        let mut depth = 0;
        let mut cells = ArrayVec::new();
        for &c in s {
            match c {
                b'[' => depth += 1,
                b']' => depth -= 1,
                b',' => (),
                digit => unsafe {
                    cells.push_unchecked(Cell::new(digit - b'0', depth));
                },
            }
        }
        let out = Self { cells };
        debug_assert_eq!(Ok(out.to_string()), std::str::from_utf8(s).map(String::from));
        out
    }

    pub fn len(&self) -> usize {
        self.cells.len()
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn values(&self) -> Vec<u8> {
        self.cells.iter().map(|c| c.value).collect()
    }

    pub fn depths(&self) -> Vec<u8> {
        self.cells.iter().map(|c| c.depth).collect()
    }

    pub fn add(&mut self, other: &Self) {
        arrayvec_concat(&mut self.cells, &other.cells);
        for cell in &mut self.cells {
            cell.depth += 1;
        }
    }

    pub fn add_and_reduce(&mut self, other: &Self) {
        self.add(&other);
        self.reduce();
    }

    pub fn reduce(&mut self) {
        self.fast_reduce::<true>();
        self.fast_reduce::<false>();
    }

    pub fn to_nodes(&self) -> Node {
        let mut nodes = (0..self.len())
            .map(|i| (Node::Literal(self.cells[i].value), self.cells[i].depth))
            .collect::<Vec<_>>();
        'outer: while nodes.len() > 1 {
            for i in 0..nodes.len() - 1 {
                if nodes[i].1 == nodes[i + 1].1 {
                    let (left, depth) = nodes.remove(i);
                    let (right, _) = nodes.remove(i);
                    let node = Node::Pair(Box::new(left), Box::new(right));
                    nodes.insert(i, (node, depth - 1));
                    continue 'outer;
                }
            }
        }
        nodes.pop().unwrap().0
    }

    fn fast_reduce<const EXPLODE_ONLY: bool>(&mut self) {
        /*
        Assuming both sides of input are always valid numbers, there can be no split
        operations until there's at least one explode. Since explodes precede splits,
        the first iteration must be processing all explodes only. After that, explodes
        can only happen if splits occur - and they should be processed immediately,
        which we can try doing in-place.
         */

        let mut out = ArrayVec::<Cell, N>::new();
        let cells = &mut self.cells;

        // handle first case separately to avoid extra branching in explode-only case
        let mut i = 0;
        if EXPLODE_ONLY {
            if cells[0].depth == 5 {
                if cells.len() != 2 {
                    cells[2].value += cells[1].value;
                }
                out.push(Cell::new(0, 4));
                i = 2;
            }
        }

        // main loop
        while i < cells.len() {
            let cell = cells[i].clone();
            if cell.depth == 5 {
                // note that this cannot happen on the last iteration as we explode stuff
                // (because if the last pair explodes we'll catch the left element of it)
                let (left, right) = (cell.value, cells[i + 1].value);
                cells[i + 1] = Cell::new(0, 4);
                if i + 2 < cells.len() {
                    cells[i + 2].value += right;
                }
                if EXPLODE_ONLY {
                    // we know that i != 0 here since there's no backtracking
                    out.get_last_mut().value += left;
                    i += 1;
                } else if let Some(last) = out.pop() {
                    cells[i] = Cell::new(last.value + left, last.depth);
                } else {
                    i += 1;
                }
            } else if !EXPLODE_ONLY && cell.value >= 10 {
                // this is only applicable on the second iteration onwards
                // if we're not at the first position, rewind backwards so we check for explode
                let left = Cell::new(cell.value >> 1, cell.depth + 1);
                let right = Cell::new(cell.value - left.value, left.depth);
                if i != 0 {
                    cells[i - 1] = left;
                    cells[i] = right;
                    i -= 1;
                } else {
                    unsafe { cells.try_insert(0, left).unwrap_unchecked() };
                    cells[1] = right;
                }
            } else {
                unsafe { out.push_unchecked(cell) };
                i += 1;
            }
        }

        mem::swap(&mut self.cells, &mut out);
    }

    pub fn magnitude(&self) -> usize {
        let mut stack = ArrayVec::<(usize, u8), N>::new();
        for cell in &self.cells {
            unsafe { stack.push_unchecked((cell.value as _, cell.depth)) };
            while stack.len() > 1 {
                let n = stack.len();
                let left = stack.get_at(n - 2);
                let right = stack.get_at(n - 1);
                if left.1 == right.1 {
                    stack.set_at(n - 2, (left.0 * 3 + right.0 * 2, left.1 - 1));
                    unsafe { stack.set_len(n - 1) };
                } else {
                    break;
                }
            }
        }
        debug_assert_eq!(stack.len(), 1);
        stack.get_first().0
    }
}

impl PartialEq for Number {
    fn eq(&self, other: &Self) -> bool {
        self.cells() == other.cells()
    }
}

impl Eq for Number {}

impl Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Number")
            .field("length", &&self.len())
            .field("values", &format!("{:?}", &self.values()))
            .field("depths", &format!("{:?}", &self.depths()))
            .finish()
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_nodes())
    }
}

#[test]
fn test_day18_number() {
    let input = "[[[[0,7],4],[[7,8],[6,0]]],[8,1]]";
    let num = Number::parse(input);
    assert_eq!(num.to_string(), input);
    assert_eq!(num.len(), 9);
    assert_eq!(num.values(), &[0, 7, 4, 7, 8, 6, 0, 8, 1]);
    assert_eq!(num.depths(), &[4, 4, 3, 4, 4, 4, 4, 2, 2]);

    let input1 = "[[[[4,3],4],4],[7,[[8,4],9]]]";
    let mut num1 = Number::parse(input1);
    assert_eq!(num1.to_string(), input1);

    let input2 = "[1,1]";
    let num2 = Number::parse(input2);
    assert_eq!(num2.to_string(), input2);

    let mut num12 = num1.clone();
    num12.add(&num2);
    let input12 = "[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]";
    assert_eq!(num12.to_string(), input12);

    num1.add_and_reduce(&num2);
    assert_eq!(num1, num);

    num12.reduce();
    assert_eq!(num12, num);

    fn parse_nums(s: &str) -> Vec<Number> {
        s.trim().lines().map(|line| Number::parse(line.trim())).collect()
    }

    let nums = parse_nums(
        r#"
        [1,1]
        [2,2]
        [3,3]
        [4,4]
        [5,5]
        [6,6]
        "#,
    );
    let res = nums.iter().skip(1).fold(nums[0].clone(), |mut acc, num| {
        acc.add_and_reduce(num);
        acc
    });
    let expected = "[[[[5,0],[7,4]],[5,5]],[6,6]]";
    assert_eq!(res, Number::parse(expected));
    assert_eq!(res.to_string(), expected);

    let nums = parse_nums(
        r#"
        [[[0,[4,5]],[0,0]],[[[4,5],[2,6]],[9,5]]]
        [7,[[[3,7],[4,3]],[[6,3],[8,8]]]]
        [[2,[[0,8],[3,4]]],[[[6,7],1],[7,[1,6]]]]
        [[[[2,4],7],[6,[0,5]]],[[[6,8],[2,8]],[[2,1],[4,5]]]]
        [7,[5,[[3,8],[1,4]]]]
        [[2,[2,2]],[8,[8,1]]]
        [2,9]
        [1,[[[9,3],9],[[9,0],[0,7]]]]
        [[[5,[7,4]],7],1]
        [[[[4,2],2],6],[8,7]]
        "#,
    );
    let expected = parse_nums(
        r#"
        [[[[4,0],[5,4]],[[7,7],[6,0]]],[[8,[7,7]],[[7,9],[5,0]]]]
        [[[[6,7],[6,7]],[[7,7],[0,7]]],[[[8,7],[7,7]],[[8,8],[8,0]]]]
        [[[[7,0],[7,7]],[[7,7],[7,8]]],[[[7,7],[8,8]],[[7,7],[8,7]]]]
        [[[[7,7],[7,8]],[[9,5],[8,7]]],[[[6,8],[0,8]],[[9,9],[9,0]]]]
        [[[[6,6],[6,6]],[[6,0],[6,7]]],[[[7,7],[8,9]],[8,[8,1]]]]
        [[[[6,6],[7,7]],[[0,7],[7,7]]],[[[5,5],[5,6]],9]]
        [[[[7,8],[6,7]],[[6,8],[0,8]]],[[[7,7],[5,0]],[[5,5],[5,6]]]]
        [[[[7,7],[7,7]],[[8,7],[8,7]]],[[[7,0],[7,7]],9]]
        [[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]
        "#,
    );
    let mut res = nums[0].clone();
    for i in 1..nums.len() {
        res.add_and_reduce(&nums[i]);
        assert_eq!(res, expected[i - 1]);
    }

    let nums = parse_nums(
        r#"
        [[1,2],[[3,4],5]]
        [[[[0,7],4],[[7,8],[6,0]]],[8,1]]
        [[[[1,1],[2,2]],[3,3]],[4,4]]
        [[[[3,0],[5,3]],[4,4]],[5,5]]
        [[[[5,0],[7,4]],[5,5]],[6,6]]
        [[[[8,7],[7,7]],[[8,6],[7,7]]],[[[0,7],[6,6]],[8,7]]]
    "#,
    );
    assert_eq!(
        nums.iter().map(Number::magnitude).collect::<Vec<_>>(),
        vec![143, 1384, 445, 791, 1137, 3488]
    );
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    let mut num = Number::default();
    while s.len() > 1 {
        let k = s.memchr(b'\n');
        let mut other = Number::parse(&s[..k]);
        if num.len() == 0 {
            num = other;
        } else {
            num.add_and_reduce(&other);
        }
        s = s.advance(k + 1);
    }
    num.magnitude()
}

#[inline]
pub fn part2(mut s: &[u8]) -> usize {
    use rayon::prelude::*;

    let mut nums = Vec::with_capacity(100);
    while s.len() > 1 {
        let k = s.memchr(b'\n');
        nums.push(Number::parse(&s[..k]));
        s = s.advance(k + 1);
    }
    let n = nums.len();
    (0..n)
        .into_par_iter()
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i != j {
                        let mut num = nums[i].clone();
                        num.add_and_reduce(&nums[j]);
                        num.magnitude()
                    } else {
                        usize::MIN
                    }
                })
                .max()
                .unwrap_or(0)
        })
        .max()
        .unwrap_or(0)
}

#[test]
fn test_day18_part1() {
    assert_eq!(part1(input()), 4120);
}

#[test]
fn test_day18_part2() {
    assert_eq!(part2(input()), 4725);
}
