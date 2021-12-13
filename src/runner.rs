use std::io::Write;
use std::time::{Duration, Instant};

use eyre::{ensure, Result};
use structopt::StructOpt;

const N_DAYS: usize = 13;

const W_DAY: usize = 10;
const W_PART: usize = 10;

fn black_box<T>(dummy: T) -> T {
    unsafe {
        let ret = std::ptr::read_volatile(&dummy);
        std::mem::forget(dummy);
        ret
    }
}

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(help = "Day: 1 to 25. If not selected, all days are used.")]
    pub day: Option<u8>,
    #[structopt(long, short, requires("day"), help = "Part of the day: 1 or 2.")]
    pub part: Option<u8>,
    #[structopt(long, short, help = "Print benchmark times instead of problem answers.")]
    pub bench: bool,
    #[structopt(
        long,
        short,
        requires("bench"),
        help = "Time in seconds allowed for timing each problem part. [default: 1.0]"
    )]
    pub seconds: Option<f64>,
    #[structopt(
        long,
        short,
        requires("bench"),
        help = "Fraction of time used for warmup before benching (0-0.5). [default: 0.2]"
    )]
    pub warmup: Option<f64>,
}

fn run_one_raw(day: u8, part: u8, times: usize) -> (Duration, String) {
    macro_rules! if_day {
        ($day:expr, $day_i:ident) => {
            if_day!($day, $day_i, 1, part1);
            if_day!($day, $day_i, 2, part2);
        };
        ($day:expr, $day_i:ident, $part:expr, $part_i:ident) => {
            if day == $day && part == $part {
                let t0 = Instant::now();
                let out = black_box(aoc2021::$day_i::$part_i(&aoc2021::$day_i::input()));
                for _ in 1..times {
                    let _ = black_box(aoc2021::$day_i::$part_i(&aoc2021::$day_i::input()));
                }
                let t1 = Instant::now();
                return (t1 - t0, out.to_string());
            }
        };
    }

    if_day!(1, day01);
    if_day!(2, day02);
    if_day!(3, day03);
    if_day!(4, day04);
    if_day!(5, day05);
    if_day!(6, day06);
    if_day!(7, day07);
    if_day!(8, day08);
    if_day!(9, day09);
    if_day!(10, day10);
    if_day!(11, day11);
    if_day!(12, day12);
    if_day!(13, day13);
    // if_day!(14, day14);
    // if_day!(15, day15);
    // if_day!(16, day16);
    // if_day!(17, day17);
    // if_day!(18, day18);
    // if_day!(19, day19);
    // if_day!(20, day20);
    // if_day!(21, day21);
    // if_day!(22, day22);
    // if_day!(23, day23);
    // if_day!(24, day24);
    // if_day!(25, day25);

    return Default::default();
}

fn print_header(part: Option<u8>) {
    print!("{:<w$}", "day", w = W_DAY);
    if part.unwrap_or(1) == 1 {
        print!("{:<w$}", "part 1", w = W_PART);
    }
    if part.unwrap_or(2) == 2 {
        print!("{:<w$}", "part 2", w = W_PART);
    }
    println!();
    println!("{:-<w$}", "", w = W_DAY + W_PART * (2 - part.is_some() as usize));
}

fn print_day(day: u8) {
    print!("{:<w$}", format!("day {:02}", day), w = W_DAY);
}

fn run_output(day_part: Option<(u8, Option<u8>)>) {
    let day = day_part.map(|x| x.0);
    let days = if let Some(day) = day { vec![day] } else { (1..=N_DAYS as u8).collect() };
    let part = day_part.map(|x| x.1).unwrap_or_default();
    let parts = if let Some(part) = part { vec![part] } else { vec![1, 2] };
    print_header(part);

    for &day in &days {
        print_day(day);
        for &part in &parts {
            let out = run_one_raw(day, part, 1).1;
            print!("{:<w$}", out, w = W_PART);
        }
        println!();
    }
}

fn bench_one(day: u8, part: u8, seconds: f64, warmup: f64, fmt_pre: impl Fn()) -> f64 {
    macro_rules! status {
        ($($arg:tt)*) => {
            print!("\r");
            fmt_pre();
            print!($($arg)*);
        }
    }

    const SPINNER: &'static str = &"↑↗→↘↓↙←↖";
    const N_CYCLES: usize = 2;
    let n_spinner = SPINNER.chars().count();
    let n_chunks = n_spinner * N_CYCLES;

    let (mut n_estimate, mut tm) = (1, 0.);
    while tm < 0.01 {
        tm = run_one_raw(day, part, n_estimate).0.as_secs_f64();
        n_estimate *= 2;
    }
    let n_total = (seconds / tm * (n_estimate as f64)).ceil() as usize;
    let n_bench = (((n_total as f64) * (1. - warmup) / (n_chunks as f64)).ceil() as usize).max(1);

    // warmup
    let n_warmup = ((n_total as f64) * warmup).ceil().min((n_total as f64) - 1.) as usize;
    status!(".");
    let _ = run_one_raw(day, part, n_warmup);

    // bench
    let mut tm_total = Duration::default();
    for i in 0..n_chunks {
        status!("{}", SPINNER.chars().skip(i % n_spinner).next().unwrap());
        std::io::stdout().flush().unwrap();
        tm_total += run_one_raw(day, part, n_bench).0;
    }

    // result
    print!("\r");
    tm_total.as_secs_f64() / ((n_chunks * n_bench) as f64)
}

fn format_time(seconds: f64) -> String {
    let mics = seconds * 1e6;
    let prec = match mics {
        m if m < 10. => 2,
        m if m < 100. => 1,
        _ => 0,
    };
    let mics_fmt = format!("{:.p$}", mics, p = prec);
    let units = "μs";
    format!("{} {}", mics_fmt, units)
}

fn run_bench(day_part: Option<(u8, Option<u8>)>, seconds: f64, warmup: f64) {
    let day = day_part.map(|x| x.0);
    let days = if let Some(day) = day { vec![day] } else { (1..=N_DAYS as u8).collect() };
    let part = day_part.map(|x| x.1).unwrap_or_default();
    let parts = if let Some(part) = part { vec![part] } else { vec![1, 2] };

    print_header(part);
    let mut tm_total = 0.;
    for &day in &days {
        let mut tms_day = vec![];
        for &part in &parts {
            let tm = bench_one(day, part, seconds, warmup, || {
                print_day(day);
                if part == 2 && parts.len() == 2 {
                    print!("{:<w$}", format_time(tms_day[0]), w = W_PART);
                }
            });
            tm_total += tm;
            tms_day.push(tm);
        }
        print_day(day);
        for &tm in &tms_day {
            print!("{:<w$}", format_time(tm), w = W_PART);
        }
        println!();
    }

    if day.is_none() && part.is_none() {
        println!("{:-<w$}", "", w = W_DAY + W_PART * 2);
        println!("total time = {}", format_time(tm_total));
    }
}

fn main() -> Result<()> {
    let Args { day, part, bench, seconds, warmup } = StructOpt::from_args_safe()?;

    if let Some(day) = day {
        ensure!((1..=N_DAYS as u8).contains(&day), "day must be 1..={}", N_DAYS);
    }
    if let Some(part) = part {
        ensure!((1..=2).contains(&part), "part must be 1..=2");
    }

    let day_part = day.map(|day| (day, part));
    if bench {
        let seconds = seconds.unwrap_or(1.0);
        ensure!(seconds > 0., "seconds must be a positive number");
        let warmup = warmup.unwrap_or(0.2);
        ensure!((0.0..=0.5).contains(&warmup), "warmup must be in [0.0; 0.5]");
        run_bench(day_part, seconds, warmup);
    } else {
        run_output(day_part);
    }

    Ok(())
}
