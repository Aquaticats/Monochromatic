# forbidden-regex-bench

Throughput benchmark comparing `forbidden-regex` against the `regex` crate.

This is a sidecar of `package/forbidden-strings/core`,
 kept out of that
crate's build and test graph so the comparison dependency (`regex`) never enters
the published engine.
 `resharp` is deliberately excluded:
 its serialized matcher is
too slow to be a useful baseline,
 so that comparison is deferred.

## What it measures

Both engines compile the same secret-detection pattern set (chosen from the dialect
overlap,
 so no intersection or complement).
 `forbidden-regex` is serialized and
reloaded first,
 so the timed form is the one a caller would load from disk.
 The
benchmark then verifies both engines agree on how many corpus lines match,
 and times
`is_match` over a mostly-non-matching,
 code-like corpus (the secret scanner's common
case:
 scan a whole line,
 find nothing).

The thesis:
 even though `regex` is handicapped out of `&`/`~`,
 the restricted
dialect lets `forbidden-regex` win on raw scanning throughput.

## Run

```sh
# from the repo root
mise run //package/forbidden-strings/core.bench:run
```

It prints lines/s and MB/s for each engine,
 the speedup ratio,
 and the serialized
ruleset size.
 The corpus and pass counts are fixed in `src/`,
 sized to keep a run to
a few seconds on a modest machine.

## Buffer-batch (`line_matches`) results

`RegexSet::line_matches(buf, starts)` (added in the engine's `src/regex/batch.rs` by
#377/#378) takes one whole-buffer plus precomputed line-start offsets and returns
`(line index, rule index)` attribution pairs,
 batching only the seeded-rule gate's
prefilter sweep.
 `bench_line_matches` in `src/kernels.rs` compares it against two
other ways to get the same attribution,
 on the same real ruleset and code-like
corpus this bench already builds:

- the naive per-line loop (`matches()` called once per line)
- the existing boolean-only concat-sweep hook (`is_match_batch_concat`,
 no
  attribution,
 already benched by `bench_set_batch`)
- `line_matches` itself,
 given a buffer and line starts built once,
 the shape a
  scanner holding a whole file already has

Measured on an AMD Ryzen 7 8700F (8 cores,
 16 threads),
 62 GiB RAM,
 Linux,
single-threaded,
 over 1,158,950 corpus lines (average 35.5 bytes/line,
 14,265
matched (line,
 rule) pairs) against the shipped ruleset (261 rules):

- per-line `matches()` loop:
   8,245,404 lines/s (292.7 MB/s)
- concat bool-only hook:
   5,863,874 lines/s (208.2 MB/s)
- `line_matches`:
   7,587,844 lines/s (269.4 MB/s)

Speedup ratios:
 `line_matches` is 0.92x the per-line loop (8% slower) and 1.29x the
concat bool-only hook (29% faster).

Why `line_matches` beats the concat hook but not the per-line loop:
 the concat hook
rebuilds its buffer from scratch on every call in this benchmark,
 the cost a caller
without a pre-split buffer pays,
 while `line_matches` is timed with its buffer and
line starts built once outside the timed loop,
 the shape a scanner holding a whole
file already has.
 That gap is exactly the buffer copy #378 removed.
 The per-line
loop still edges out `line_matches` because both spend most of their time in the
line-start and seedless (literal-free) rule groups,
 which run per line either way;
the seeded-gate sweep `line_matches` batches is not this ruleset's dominant
per-line cost on this corpus.
 That is the evidence for #381:
 batching the seedless
and line-start rule groups too,
 not just the seeded gate,
 is where the next
throughput gain would come from.
