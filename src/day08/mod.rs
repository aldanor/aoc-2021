use crate::utils::*;

pub fn input() -> &'static [u8] {
    include_bytes!("input.txt")
}

pub fn part1(mut s: &[u8]) -> usize {
    // 1, 4, 7, 8: number of digits = 2, 4, 3, 7
    let mut count = 0;
    'outer: loop {
        s = s.skip_past(b'|', 1);
        loop {
            let n = s.memchr2(b' ', b'\n');
            if let 2 | 3 | 4 | 7 = n {
                count += 1;
            }
            let c = s.get_at(n);
            s = s.advance(n + 1);
            if s.len() <= 1 {
                break 'outer;
            } else if c == b'\n' {
                break;
            }
        }
    }
    count
}

pub fn parse_digit(s: &mut &[u8]) -> u8 {
    let mut digit = 0;
    loop {
        let c = s.get_first().wrapping_sub(b'a');
        *s = s.advance(1);
        if c < 7 {
            digit |= 1_u8 << c;
        } else {
            break;
        }
    }
    digit
}

fn parse_digits<const N: usize>(s: &mut &[u8]) -> [u8; N] {
    let mut digits = [0; N];
    for i in 0..N {
        digits[i] = parse_digit(s);
    }
    digits
}

fn parse_line<const N: usize>(s: &mut &[u8]) -> ([u8; 10], [u8; N]) {
    let inputs = parse_digits::<10>(s);
    *s = s.advance(2);
    let outputs = parse_digits::<N>(s);
    (inputs, outputs)
}

fn decode_segments(inputs: &[u8; 10]) -> [u8; 7] {
    /*
      0
    1   2
      3
    4   5
      6
     */

    // First, split the inputs into buckets by number of segments
    let (mut one, mut four, mut seven) = (0, 0, 0);
    let mut zero_six_nine = [0; 3];
    let eight = 0b0111_1111;
    for &x in inputs {
        match x.count_ones() {
            2 => one = x,
            3 => seven = x,
            4 => four = x,
            6 => {
                zero_six_nine.rotate_right(1);
                zero_six_nine[0] = x;
            }
            _ => (), // it's eight or a 5-segment digit
        }
    }

    // top segment can be inferred directly
    let mut segments = [0; 7];
    segments[0] = seven & !one;

    // now let's process 6-segment digits, that should be sufficient
    for x in &zero_six_nine[..3] {
        // look which digit is missing from eight
        let c = eight & !x;
        if c & one != 0 {
            // it's 6
            segments[2] = c;
            segments[5] = one & !c;
        } else if c & four != 0 {
            // it's 0
            segments[3] = c;
            segments[1] = four & !one & !c;
        } else {
            // it's 9
            segments[4] = c;
            segments[6] = x & !four & !seven;
        }
    }
    segments
}

fn decode_digits(segments: &[u8; 7]) -> [u8; 10] {
    let eight = 0b0111_1111;
    let one = segments[2] | segments[5];
    let seven = one | segments[0];
    let four = one | segments[1] | segments[3];
    let zero = eight & !segments[3];
    let six = eight & !segments[2];
    let nine = eight & !segments[4];
    let two = eight & !segments[1] & !segments[5];
    let three = eight & !segments[1] & !segments[4];
    let five = eight & !segments[2] & !segments[4];
    [zero, one, two, three, four, five, six, seven, eight, nine]
}

fn decode_outputs<const N: usize>(outputs: &[u8; N], digits: &[u8; 10]) -> usize {
    let mut decoder = [0; 256];
    for i in 0..10 {
        decoder.set_at(digits[i] as _, i);
    }
    let mut number = 0;
    for i in 0..N {
        number *= 10;
        number += decoder.get_at(outputs[i] as _) as usize;
    }
    number
}

pub fn part2(mut s: &[u8]) -> usize {
    let mut sum = 0;
    while s.len() > 1 {
        let (inputs, outputs) = parse_line::<4>(&mut s);
        let segments = decode_segments(&inputs);
        let digits = decode_digits(&segments);
        sum += decode_outputs(&outputs, &digits);
    }
    sum
}

pub fn part2_faster(mut s: &[u8]) -> usize {
    // implementation of @orlp's solution (probably the fastest possible here)
    let mut decoder = [0_u8; 256];
    decoder[42] = 0;
    decoder[17] = 1;
    decoder[34] = 2;
    decoder[39] = 3;
    decoder[30] = 4;
    decoder[37] = 5;
    decoder[41] = 6;
    decoder[25] = 7;
    decoder[49] = 8;
    decoder[45] = 9;

    let mut sum = 0;
    while s.len() > 1 {
        let mut input_counts = [0_u8; 256];
        for i in 0..58 {
            input_counts.add_at(s.get_at(i) as _, 1);
        }
        s = s.advance(61);
        let mut num = 0_usize;
        let mut output_counts = 0_u8;
        loop {
            match s.get_first() {
                ch @ b'a'..=b'g' => {
                    output_counts += input_counts.get_at(ch as _);
                }
                ch => {
                    let digit = decoder.get_at(output_counts as _) as usize;
                    num = num * 10 + digit;
                    output_counts = 0;
                    if ch == b'\n' {
                        s = s.advance(1);
                        sum += num;
                        break;
                    }
                }
            }
            s = s.advance(1);
        }
    }
    sum
}

#[test]
fn test_day08_part1() {
    assert_eq!(part1(input()), 301);
}

#[test]
fn test_day08_part2() {
    assert_eq!(part2(input()), 908067);
}
