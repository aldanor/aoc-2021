# aoc-2021

Advent of Code 2021 -- highly optimized Rust solutions.

There's a lot of unsafe code, all kinds of bit hackery, and SIMD (since it uses portable-simd crate,
it can currently only compile on nightly). The main goal was to try and squeeze out as much performance
as possible, both via using smart non-bruteforce algorithms when applicable, and by making full use
of the hardware (be it SIMD or parallelisation). Most problems except a few run a single thread though.

These are the benchmark results taken on Apple M1 laptop (these include the time to parse input for
each part of each problem - there are no input 'generators' and so each input is parsed twice):

```
day       part 1    part 2    
------------------------------
day 01    3.67 μs   3.66 μs   
day 02    0.83 μs   0.83 μs   
day 03    0.32 μs   3.32 μs   
day 04    6.78 μs   6.79 μs   
day 05    38.9 μs   171 μs    
day 06    0.47 μs   1.21 μs   
day 07    3.33 μs   2.01 μs   
day 08    5.02 μs   14.4 μs   
day 09    0.35 μs   26.4 μs   
day 10    5.81 μs   6.17 μs   
day 11    12.2 μs   35.0 μs   
day 12    3.38 μs   10.9 μs   
day 13    10.5 μs   13.8 μs   
day 14    1.48 μs   5.14 μs   
day 15    92.4 μs   2859 μs   
day 16    1.84 μs   1.98 μs   
day 17    0.00 μs   0.71 μs   
day 18    59.5 μs   600 μs    
day 19    1082 μs   1026 μs   
day 20    72.2 μs   3699 μs   
day 21    0.73 μs   284 μs    
day 22    102 μs    378 μs    
day 23    28.2 μs   2587 μs   
day 24    0.54 μs   0.55 μs   
day 25    1079 μs   0.00 μs   
------------------------------
total time = 14350 μs
```

Quick notes on solutions to some of the problems that were less trivial (the problems that
are not mentioned were straightforward):

- Day 5: part 1 is a simple orthogonal line sweep. Part 2 uses a modified version of line
  sweep algorithm where the scanline is vertical and there are separate active sets for
  all three types of lines (horizontal and both diagonal types).
- Day 9: uses SIMD to speed up part 1. Part 2 is a known problem (4-CCL for binary images),
  so a union-find algorithm can be employed.
- Day 12: uses a cache when traversing the graph; also, in part 1 simplifies the graph by 
  removing some irrelevant nodes that have a single edge leading to a small cave.
- Day 18: exploits the fact that there can be no split operations until there's at least
  one explode, so everything is unrolled in two passes; the first pass is explodes-only
  and in the second pass we process the splits inline.
- Day 19: uses the fact that an unambiguous solution for a pair of scanners is guaranteed
  if there's a 12-point overlap; only L2 pairwise distances between beacons are used to
  find matching sets - then we build a connectivity graph between scanners.
- Day 20: naive solution; planned to optimize it via SIMD but haven't had time to finish.
- Day 21: classic bottom-up 5-D dynamic programming.
- Day 22: one of the most non-trivial solutions - used the fact that overlaps of different
  cardinality can be found by enumerating cliques of the overlap graph; ported and modified
  networkx clique finding algorithm to make it work (surprisingly, it ended up being fast).
- Day 23: track minimum possible remaining cost for all moves, use compact data structures
  and run DFS with some heuristics (try to select worst moves for A) and cost tracking.
- Day 24: relies on the specific structure of the input (stack machine).
- Day 25: used SIMD, ghost cells and lookup tables to speed it up (this is the BML traffic model).
