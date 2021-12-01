use criterion::{black_box, criterion_group, criterion_main, Criterion};

use aoc2021::*;

macro_rules! bench {
    ($c:expr, $path:path) => {{
        use $path::*;
        let s = input();
        $c.bench_function(concat!(stringify!($path), "::part1"), |b| {
            b.iter(|| black_box(part1(black_box(&s))))
        });
        $c.bench_function(concat!(stringify!($path), "::part2"), |b| {
            b.iter(|| black_box(part2(black_box(&s))))
        });
    }};
}

pub fn criterion_benchmark(c: &mut Criterion) {
    bench!(c, day01);
    // bench!(c, day02);
    // bench!(c, day03);
    // bench!(c, day04);
    // bench!(c, day05);
    // bench!(c, day06);
    // bench!(c, day07);
    // bench!(c, day08);
    // bench!(c, day09);
    // bench!(c, day10);
    // bench!(c, day11);
    // bench!(c, day12);
    // bench!(c, day13);
    // bench!(c, day14);
    // bench!(c, day15);
    // bench!(c, day16);
    // bench!(c, day17);
    // bench!(c, day18);
    // bench!(c, day19);
    // bench!(c, day20);
    // bench!(c, day21);
    // bench!(c, day22);
    // bench!(c, day23);
    // bench!(c, day24);
    // bench!(c, day25);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
