# aoc-2021

Advent of Code 2021 -- highly optimized Rust solutions.

There's a lot of unsafe code, all kinds of bit hackery, and SIMD (since it uses portable-simd crate,
it can currently only compile on nightly). The main goal was to try and squeeze out as much performance
as possible, both via using smart non-bruteforce algorithms when applicable, and by making full use
of the hardware (be it SIMD or parallelisation). Most problems except a few run a single thread though.

These are the timings taken on Apple M1 laptop:

```
day       part 1    part 2    
------------------------------
day 01    3.70 μs   3.69 μs   
day 02    0.83 μs   0.83 μs   
day 03    0.32 μs   3.32 μs   
day 04    6.75 μs   6.76 μs   
day 05    65.9 μs   187 μs    
day 06    0.49 μs   1.25 μs   
day 07    3.31 μs   2.04 μs   
day 08    4.67 μs   20.2 μs   
day 09    0.86 μs   25.9 μs   
day 10    5.37 μs   5.80 μs   
day 11    12.9 μs   38.3 μs   
day 12    3.32 μs   7.88 μs   
day 13    10.5 μs   20.4 μs   
day 14    1.46 μs   5.29 μs   
day 15    92.6 μs   2963 μs   
day 16    1.83 μs   1.96 μs   
day 17    0.00 μs   0.66 μs   
day 18    60.2 μs   556 μs    
day 19    1045 μs   1043 μs   
day 20    71.8 μs   3422 μs   
day 21    0.72 μs   249 μs    
day 22    103 μs    379 μs    
day 23    1732 μs   10871 μs  
day 24    0.42 μs   0.41 μs   
day 25    4025 μs   0.00 μs   
------------------------------
total time = 27067 μs
```

Note, however, that day 23 is highly input-dependent and it takes about 40% of the total time.
Here are day 23 timings for another user's input (which is also included in the repository):

```
day 23    1657 μs   1623 μs  
```
