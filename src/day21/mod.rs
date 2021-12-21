use crate::utils::*;

#[inline]
pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

fn parse(mut s: &[u8]) -> [u16; 2] {
    s = s.advance(28);
    let p0 = parse_int_fast::<_, 1, 2>(&mut s);
    s = s.advance(28);
    let p1 = parse_int_fast::<_, 1, 2>(&mut s);
    [p0, p1]
}

#[inline]
pub fn part1(mut s: &[u8]) -> usize {
    const fn deterministic_rolls<const R: usize>() -> [[u16; 16]; R] {
        let mut table = [[0; 16]; R];
        let mut i = 0;
        while i < R {
            let mut j = 1;
            while j <= 10 {
                table[i][j] = j as _;
                j += 1;
            }
            i += 1;
        }
        let mut i = 0;
        while i < 3 * R {
            let mut j = 1;
            while j <= 10 {
                table[i / 3][j] = 1 + (table[i / 3][j] + (i + 1) as u16 - 1) % 10;
                j += 1;
            }
            i += 1;
        }
        table
    }

    const ROLLS: [[u16; 16]; 100] = deterministic_rolls::<100>();

    let mut pos = parse(s);
    let mut score = [0_u16; 2];

    let mut player = 0;
    for roll in &ROLLS {
        pos[player] = roll[pos[player] as usize];
        score[player] += pos[player];
        player ^= 1;
    }
    let mut n = ROLLS.len();
    loop {
        for (j, roll) in ROLLS.iter().enumerate() {
            pos[player] = roll[pos[player] as usize];
            score[player] += pos[player];
            if score[player] >= 1000 {
                return (n + j + 1) * 3 * score[player ^ 1] as usize;
            }
            player ^= 1;
        }
        n += ROLLS.len();
    }
}

fn zero_out<T: ?Sized>(x: &mut T) {
    use std::mem;
    use std::slice;
    unsafe { slice::from_raw_parts_mut(x as *mut T as *mut u8, mem::size_of_val(x)).fill(0) }
}

type Dp = [[[[u64; 21]; 21]; 10]; 10];

fn dp_step<const PLAYER: usize>(dp: &Dp, next: &mut Dp, step: usize) -> u64 {
    const ROLLS: [(usize, u64); 7] = [(3, 1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)];

    let mut wins = 0;
    zero_out(next);

    let min_score = if step <= 1 { step } else { 2 * step - 1 };

    for (roll, count) in ROLLS {
        let mut wins_roll = 0;
        for pos_this in 0..10 {
            let new_pos = (pos_this + roll) % 10;
            let score_delta = 1 + new_pos;
            let min_win = 21 - score_delta;
            for pos_other in 0..10 {
                for score_this in min_score..min_win {
                    let new_score = score_this + score_delta;
                    for score_other in min_score..21 {
                        if PLAYER == 0 {
                            next[new_pos][pos_other][new_score][score_other] +=
                                dp[pos_this][pos_other][score_this][score_other] * count;
                        } else {
                            next[pos_other][new_pos][score_other][new_score] +=
                                dp[pos_other][pos_this][score_other][score_this] * count;
                        }
                    }
                }
                for score_this in min_win..21 {
                    for score_other in min_score..21 {
                        wins_roll += if PLAYER == 0 {
                            dp[pos_this][pos_other][score_this][score_other]
                        } else {
                            dp[pos_other][pos_this][score_other][score_this]
                        };
                    }
                }
            }
        }
        wins += count * wins_roll;
    }
    wins
}

#[inline]
pub fn part2(mut s: &[u8]) -> u64 {
    const ROLLS: [(usize, u64); 7] = [(3, 1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)];

    let p = parse(s);

    let mut buf1 = [[[[0_u64; 21]; 21]; 10]; 10];
    let mut buf2 = [[[[0_u64; 21]; 21]; 10]; 10];
    let mut dp = &mut buf1;
    dp[p[0] as usize - 1][p[1] as usize - 1][0][0] = 1;
    let mut next = &mut buf2;

    let mut wins = [0; 2];
    for step in 0.. {
        let prev_wins = wins;
        wins[0] += dp_step::<0>(&dp, &mut next, step);
        std::mem::swap(&mut dp, &mut next);
        wins[1] += dp_step::<1>(&dp, &mut next, step);
        std::mem::swap(&mut dp, &mut next);
        if step > 3 && wins == prev_wins {
            break;
        }
    }
    wins[0].max(wins[1])
}

#[test]
fn test_day02_part1() {
    assert_eq!(part1(input()), 707784);
}

#[test]
fn test_day02_part2() {
    assert_eq!(part2(input()), 157595953724471);
}
