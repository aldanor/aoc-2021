use crate::utils::*;

fn parse(mut s: &[u8]) -> [u16; 2] {
    s = s.advance(28);
    let p0 = parse_int_fast::<_, 1, 2>(&mut s);
    s = s.advance(28);
    let p1 = parse_int_fast::<_, 1, 2>(&mut s);
    [p0, p1]
}

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

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

pub fn part2(mut s: &[u8]) -> u64 {
    const ROLLS: [(usize, u64); 7] = [(3, 1), (4, 3), (5, 6), (6, 7), (7, 6), (8, 3), (9, 1)];
    let p = parse(s);
    // positions/players are rotating, but scores only increase (at least by 1), so we can sweep
    // diagonally from (0, 0) all the way to (20, 20)
    let mut dp = [[[[[0_u64; 2]; 10]; 10]; 21]; 21]; // [score0][score1][pos0][pos1][player]
    dp[0][0][p[0] as usize - 1][p[1] as usize - 1][0] = 1;
    let mut wins = [0_u64; 2];
    for sum in 0..21 * 2 {
        for score0 in 0..sum + 1 {
            let score1 = sum - score0;
            if score0 < 21 && score1 < 21 {
                for (roll, count) in ROLLS {
                    for pos0 in 0..10 {
                        for pos1 in 0..10 {
                            let new_pos0 = (pos0 + roll) % 10;
                            let new_score0 = score0 + new_pos0 + 1;
                            if new_score0 >= 21 {
                                wins[0] += dp[score0][score1][pos0][pos1][0] * count;
                            } else {
                                dp[new_score0][score1][new_pos0][pos1][1] +=
                                    dp[score0][score1][pos0][pos1][0] * count;
                            }
                            let new_pos1 = (pos1 + roll) % 10;
                            let new_score1 = score1 + new_pos1 + 1;
                            if new_score1 >= 21 {
                                wins[1] += dp[score0][score1][pos0][pos1][1] * count;
                            } else {
                                dp[score0][new_score1][pos0][new_pos1][0] +=
                                    dp[score0][score1][pos0][pos1][1] * count;
                            }
                        }
                    }
                }
            }
        }
    }
    wins[0].max(wins[1])
}

#[test]
fn test_day21_part1() {
    assert_eq!(part1(input()), 707784);
}

#[test]
fn test_day21_part2() {
    assert_eq!(part2(input()), 157595953724471);
}
