macro_rules! run_day {
    ($day:path) => {{
        use $day::*;
        println!(
            "{}: part1 = {}, part2 = {}",
            stringify!($day),
            part1(&input()),
            part2(&input())
        );
    }};
}

fn main() {
    use aoc2021::*;
    run_day!(day01);
    // run_day!(day02);
    // run_day!(day03);
    // run_day!(day04);
    // run_day!(day05);
    // run_day!(day06);
    // run_day!(day07);
    // run_day!(day08);
    // run_day!(day09);
    // run_day!(day10);
    // run_day!(day11);
    // run_day!(day12);
    // run_day!(day13);
    // run_day!(day14);
    // run_day!(day15);
    // run_day!(day16);
    // run_day!(day17);
    // run_day!(day18);
    // run_day!(day19);
    // run_day!(day20);
    // run_day!(day21);
    // run_day!(day22);
    // run_day!(day23);
    // run_day!(day24);
    // run_day!(day25);
}
