# Completed Sudoku grid unranking

## Goal

Build a package that maps every integer in
`0n <= index < 6_670_903_752_021_072_936_960n`
to exactly one completed 9 by 9 Sudoku grid.
Every completed grid must have exactly one index.

The interface under consideration is:

```ts
unrankCompletedSudoku(index: bigint): CompletedSudoku
```

An inverse `rankCompletedSudoku` can share the same canonical decomposition,
but decode-only behavior remains the requested scope.

## Rejected positional encoding

A fixed radix over 81 cells indexes every filled grid,
not only completed Sudoku grids.
Valid completed grids form a sparse subset,
so base-9 decoding cannot make every in-range index valid.

Cell-by-cell lexicographic unranking is mathematically correct:
each candidate digit receives a range sized by its number of valid completions.
It is not the implementation direction because it repeats exact completion counting throughout one lookup.

## Structural unranking

Felgenhauer and Jarvis counted
`6_670_903_752_021_072_936_960` completed grids.
Their normalization separates each grid into:

- one of `3_546_146_300_288` reduced grids;
- one digit relabeling among `9!` choices;
- one top-band normalization inverse among 72 choices;
- one left-stack normalization inverse among 72 choices.

The factors satisfy:

```text
3,546,146,300,288 * 9! * 72 * 72
= 6,670,903,752,021,072,936,960
```

A full unranker can therefore split the input into a reduced-grid rank and a mixed-radix transformation rank.
The transformation rank uses permutation unranking and fixed row or column operations.
The reduced-grid rank remains the irregular part.

The decomposition must define a deterministic canonicalization and inverse ordering.
Applying arbitrary Sudoku symmetry elements is insufficient because general symmetry orbits can have stabilizers.

## Rejected flat T-Doku reduced-grid index

T-Doku commit `af426180dc53aef89b82868e7b3fdfcf42165654`
implements numbered access to the reduced grids.
It is retained as a reference enumerator and source of verified counts,
not as the recommended package index.
It combines 36,288 normalized top-band configurations with 36,288 normalized left-stack configurations.
A generated `uint16_t` table stores the number of completions for every pair.
A sparse checkpoint index locates the neighborhood of a requested reduced rank.
T-Doku then enumerates only within the selected structural configuration.

Measured release assets:

- `grid.counts`:
   `2_633_637_888` bytes;
- `grid.index`:
   `20_291_214` bytes;
- compressed `tables.tar.xz`:
   `471_499_152` bytes.

The release notes state that generating the tables requires about 500 core-hours.
The source's example reports about 8 hours with 64 processes on a Threadripper 2990WX.
These are upstream measurements,
not measurements from this repository.

## Lookup benchmark

The released tables and exact T-Doku commit were built and exercised in a Fedora 44 container
capped at 4 GiB RAM and 2 CPUs.
The AVX2 executable was built inside the same container image used for execution.

A warm-cache batch of 1,000 random reduced-grid lookups completed in:

```text
Time (mean +/- sigma): 455.5 ms +/- 9.2 ms
Range: 441.7 ms to 462.8 ms
Runs: 5
```

This is 2,195 lookups per second when calculated from the measured mean.
The batch includes one container startup and table mapping per run.
It does not measure the unimplemented full-grid inverse transformations.

A 100-lookup batch measured `171.6 ms +/- 10.0 ms` over 5 runs,
showing that process and mapping overhead matter for short-lived batches.

## Endpoint defect found during verification

Unpatched T-Doku `grid_tools list_grids` reads one count past `grid.counts`
when a requested range ends in the final reduced-grid pattern.
The resulting `SIGSEGV` is unrelated to lookup throughput.
A one-line early-return prototype passed first-index and final-range probes.

Diagnosis and upstream-ready issue draft:
[`doc/troubleshooting/tdoku-final-grid-enumeration.md`](../troubleshooting/tdoku-final-grid-enumeration.md).

## Recommended 44-class hierarchy

Ed Russell's `equiv.c` generates the 36,288 normalized top-band configurations,
partitions them into 44 equivalence classes,
and records for each class:

- class multiplicity `m`,
  the number of normalized top-band members;
- normalized completion count `C`,
  shared by every member.

The reduced total is:

```text
sum(m * C) = 3,546,146,300,288
```

A reduced index first selects a class interval of size `m * C`.
Within that interval:

```text
memberIndex = remainder / C
completionIndex = remainder % C
```

Each actual top-band member receives one deterministic stored witness mapping it to its class representative.
Multiple possible witnesses use one fixed tie rule.
The witness must be encoded as exact Sudoku-preserving coordinate and digit operations,
not as an informal equivalence claim.

### Representative completion lookup

Only one horizontal count row is needed for each class representative.
Each row contains completion counts for the 36,288 normalized left-stack configurations.
The count core therefore requires:

```text
44 * 36,288 * 2 bytes = 3,193,344 bytes
```

A prefix total every 64 entries adds `99_792` bytes.
Lookup selects a prefix block,
scans at most 64 `uint16_t` counts,
generates the selected 45-cell structural pattern,
and enumerates only the residual completion within that pattern.

The representative completion is mapped to the selected member with the inverse witness.
That mapping must preserve the normalized lower-row orbit used by the outer 72-way restoration.
A safe witness either normalizes that group under conjugation or carries an explicit invertible correction coordinate.
A many-to-one re-normalization with no recorded coordinate is forbidden.

### Asset compiler

The offline compiler can derive the compact asset from the released T-Doku counts:

1. Generate all normalized top-band members in deterministic order.
2. Reproduce Ed Russell's 44-class coloring.
3. Select one T-Doku horizontal configuration ID for each representative.
4. Extract only those 44 horizontal rows from `grid.counts`.
5. Store every member's class ID and canonical transformation witness.
6. Generate block prefix totals and integrity hashes.
7. Verify each representative row sum equals Ed Russell's published `C`.
8. Verify class sizes and `sum(m * C)` against the reduced total.

The runtime asset does not need the original 2.5 GiB count table.

## Alternatives

### Counted canonical-prefix DAG

Pros:

- Could share equivalent residual states across representative classes.
- Supports rank and unrank through the same counted edges.
- Could reduce residual solver work.

Cons:

- Serialized size is unmeasured.
- Blank Sudoku has resisted compact generic ZDD representations.
- Requires a new compiler and canonical residual-state proof.

### Full extracted T-Doku table

Pros:

- Existing source and released data.
- Measured random reduced-grid access.

Cons:

- `471_499_152`-byte download.
- `2_653_929_102` extracted bytes across both table files.
- Encodes all 36,288 horizontal rows when the class hierarchy needs 44.

### Completion counts on demand

Pros:

- Avoids a generated count asset.

Cons:

- Repeats exact counting during each lookup.
- No measured implementation meets the requested random-access behavior.

Ranking:
44-class representative rows > counted canonical-prefix DAG > full T-Doku table > on-demand counts.
The class-row design ranks first because it applies the proven 44-class reduction directly
and has a calculated `3_193_344`-byte count core.
The DAG ranks next because it may compress further but has no measured artifact.
The full table ranks next because it works but stores 824 times as many count entries as the class-row core.
On-demand counting ranks last because it restores work to every lookup.

## Proof gates before implementation

No package implementation starts until a feasibility harness establishes:

- the 44 classes partition all 36,288 normalized top-band members;
- every member has a deterministic invertible witness;
- each witness preserves or explicitly transports the lower 72-way normalization coordinate;
- every extracted representative row sums to its published class completion count;
- class blocks sum to `3_546_146_300_288` reduced grids;
- the outer `9! * 72 * 72` restoration is a true transversal;
- solver enumeration order is pinned as part of the index format;
- first,
  last,
  class-boundary,
  member-boundary,
  left-stack-boundary,
  and residual-boundary indices round-trip through an independent ranker.

The next action is design validation of the hierarchy,
not package implementation.

## Sources

- Bertram Felgenhauer and Frazer Jarvis,
  [Enumerating possible Sudoku grids](http://www.afjarvis.org.uk/sudoku/sudoku.pdf).
- Ed Russell,
  [`equiv.c`](http://www.afjarvis.org.uk/sudoku/equiv.c).
- Frazer Jarvis,
  [44-class result summary](http://www.afjarvis.org.uk/sudoku/ed44.html).
- T-Doku,
  [`src/grid_lib.cc`](https://github.com/t-dillon/tdoku/blob/af426180dc53aef89b82868e7b3fdfcf42165654/src/grid_lib.cc).
- T-Doku,
  [`src/grid_tools.cc`][grid-tools-src].
- T-Doku,
  [`tables` release](https://github.com/t-dillon/tdoku/releases/tag/tables).
- Ben Lynn,
  [Sudoku ZDD notes](https://crypto.stanford.edu/pbc/notes/zdd/sudoku.html).

[grid-tools-src]: https://github.com/t-dillon/tdoku/blob/af426180dc53aef89b82868e7b3fdfcf42165654/src/grid_tools.cc
