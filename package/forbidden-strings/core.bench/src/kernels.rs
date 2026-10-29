//! Batch-kernel sweep:
//!  bucket width and sorting strategy for the single-DFA kernels.
//!
//! What:
//!  races the scalar per-line loop against the interleaved batch kernel at bucket
//! widths 8/16/32/64 on length-sorted input,
//!  the branchless tight kernel on exact-length
//! buckets,
//!  the bucketed public API,
//!  and the one- and two-byte Sheng permute kernels,
//!  and
//! reports the sort cost.
//!  Why:
//!  the scalar single-line scan is latency-bound on the
//! dependent transition load;
//!  this finds which layout beats it,
//!  and by how much,
//!  in the
//! realistic low-match regime a secret scanner sees.

/// Imports the optimization barrier so verdicts are not elided.
use std::hint::black_box;

/// Imports the map used to build exact-length buckets.
use std::collections::HashMap;

/// Imports the wall-clock timer.
use std::time::Instant;

/// Wall-clock budget timed per kernel configuration.
const BUDGET_SECS: f64 = 4.0;

/// Patterns whose DFA runs on every line,
///  spanning the realistic low-match regime.
const PATTERNS: [&str; 3] = ["[A-Za-z0-9]{8}", "[A-Za-z0-9_+/=.-]{20}", "[0-9a-f]{32}"];

/// Times one whole-corpus batch closure for the budget,
///  returning lines/s.
fn rate(refs: &[&[u8]], run: impl Fn(&[&[u8]]) -> Vec<bool>) -> f64 {
    let start = Instant::now();
    let mut scanned = 0u64;
    while start.elapsed().as_secs_f64() < BUDGET_SECS {
        black_box(run(black_box(refs)));
        scanned += refs.len() as u64;
    }
    return scanned as f64 / start.elapsed().as_secs_f64()
}

/// Times a kernel run over pre-grouped exact-length buckets,
///  returning lines/s.
fn bucket_rate(buckets: &[Vec<&[u8]>], run: impl Fn(&[&[u8]]) -> Vec<bool>) -> f64 {
    let lines: u64 = buckets.iter().map(|bucket| return bucket.len() as u64).sum();
    let start = Instant::now();
    let mut scanned = 0u64;
    while start.elapsed().as_secs_f64() < BUDGET_SECS {
        for bucket in buckets {
            black_box(run(bucket));
        }
        scanned += lines;
    }
    return scanned as f64 / start.elapsed().as_secs_f64()
}

/// Groups line refs into one bucket per exact byte length.
fn exact_buckets<'a>(refs: &[&'a [u8]]) -> Vec<Vec<&'a [u8]>> {
    let mut by_len: HashMap<usize, Vec<&'a [u8]>> = HashMap::new();
    for &line in refs {
        by_len.entry(line.len()).or_default().push(line);
    }
    return by_len.into_values().collect()
}

/// Runs the bucket-width and sorting-strategy sweep over the corpus.
///
/// What:
///  for each pattern,
///  prints scalar lines/s and the ratios of the interleaved kernel
/// (four bucket widths,
///  length-sorted),
///  the tight kernel (exact buckets),
///  the bucketed
/// public API,
///  and the one- and two-byte Sheng kernels,
///  plus the sort cost.
///  Why:
///  answers
/// whether a length-bucketed batch and the permute kernels beat the per-line loop,
///  and how.
pub fn bench_buckets(corpus: &[Vec<u8>], avg_len: f64) {
    let refs: Vec<&[u8]> = corpus.iter().map(Vec::as_slice).collect();
    let mut sorted = refs.clone();
    sorted.sort_by_key(|line| return line.len());
    let buckets = exact_buckets(&refs);
    println!(
        "\n[buckets] {} lines (avg {:.1} B), {:.0}s each, {} exact-length buckets. ratios vs scalar:",
        refs.len(),
        avg_len,
        BUDGET_SECS,
        buckets.len(),
    );
    for pattern in PATTERNS {
        let re = forbidden_regex::compile(pattern).expect("pattern compiles");
        let oracle = re.is_match_batch_scalar(&refs);
        assert_eq!(re.batch_inter_w::<16>(&refs), oracle, "inter disagrees for {pattern}");
        let scalar = rate(&refs, |lines| return re.is_match_batch_scalar(lines));
        let inter = [
            rate(&sorted, |l| return re.batch_inter_w::<8>(l)),
            rate(&sorted, |l| return re.batch_inter_w::<16>(l)),
            rate(&sorted, |l| return re.batch_inter_w::<32>(l)),
            rate(&sorted, |l| return re.batch_inter_w::<64>(l)),
        ];
        // Correctness: the branchless tight kernel must match scalar on an exact bucket.
        if let Some(bucket) = buckets.iter().find(|bucket| return bucket.len() >= 64) {
            let bucket_oracle: Vec<bool> =
                bucket.iter().map(|line| return re.is_match_batch_scalar(&[line])[0]).collect();
            assert_eq!(re.batch_tight_w::<32>(bucket), bucket_oracle, "tight disagrees for {pattern}");
        }
        let tight = [
            bucket_rate(&buckets, |l| return re.batch_tight_w::<16>(l)),
            bucket_rate(&buckets, |l| return re.batch_tight_w::<32>(l)),
            bucket_rate(&buckets, |l| return re.batch_tight_w::<64>(l)),
        ];
        // The shipped public API: it sorts internally, so measure it both ways. On
        // already length-sorted input the internal sort is near-linear (the scanner case);
        // on unsorted input it pays the full sort (the convenience case).
        assert_eq!(re.is_match_batch_bucketed(&refs), oracle, "bucketed disagrees for {pattern}");
        assert_eq!(re.batch_sheng(&refs), oracle, "sheng disagrees for {pattern}");
        assert_eq!(re.batch_sheng2(&refs), oracle, "sheng2 disagrees for {pattern}");
        let api_presorted = rate(&sorted, |l| return re.is_match_batch_bucketed(l));
        // Sheng needs no bucketing or sorting: a per-line scan with permute transitions.
        let sheng = rate(&refs, |l| return re.batch_sheng(l));
        let sheng2 = rate(&refs, |l| return re.batch_sheng2(l));
        println!(
            "  {pattern:20} table={} scalar {scalar:>11.0} | sorted inter[8/16/32/64] {:.2} {:.2} {:.2} {:.2} | exact-tight[16/32/64] {:.2} {:.2} {:.2} | api-presorted {:.2} | sheng {:.2} | sheng2 {:.2}",
            re.is_table(),
            inter[0] / scalar, inter[1] / scalar, inter[2] / scalar, inter[3] / scalar,
            tight[0] / scalar, tight[1] / scalar, tight[2] / scalar,
            api_presorted / scalar,
            sheng / scalar,
            sheng2 / scalar,
        );
    }
    let start = Instant::now();
    let mut probe = refs.clone();
    probe.sort_by_key(|line| return line.len());
    black_box(&probe);
    let sort_ms = start.elapsed().as_secs_f64() * 1_000.0;
    println!(
        "[sort-cost] sorting {} refs by length: {sort_ms:.2} ms ({:.0} lines/s if amortized into one batch)",
        refs.len(),
        refs.len() as f64 / start.elapsed().as_secs_f64(),
    );
}

/// Compares the set-level batch layouts on the real ruleset over the corpus.
///
/// What:
///  confirms the concatenated-buffer gate sweep agrees with the per-line loop,
///  then
/// times both single-threaded.
///  Why:
///  the set pipeline's real cost is the per-line
/// prefilter on short lines;
///  one long-buffer sweep is the approach that can beat it.
pub fn bench_set_batch(set: &forbidden_regex::RegexSet, corpus: &[Vec<u8>]) {
    let refs: Vec<&[u8]> = corpus.iter().map(Vec::as_slice).collect();
    let oracle = set.is_match_batch(&refs);
    assert_eq!(set.is_match_batch_concat(&refs), oracle, "concat batch disagrees with per-line");
    let hits = oracle.iter().filter(|&&hit| return hit).count();
    let scalar = rate(&refs, |lines| return set.is_match_batch(lines));
    let concat = rate(&refs, |lines| return set.is_match_batch_concat(lines));
    println!(
        "\n[set-batch] over {} lines, hits {hits}: scalar-loop {scalar:>12.0}  concat-gate {concat:>12.0}  ({:.2}x)",
        refs.len(),
        concat / scalar,
    );
}

/// Times a per-line closure returning `(line index, rule index)` pairs for the budget,
/// returning lines/s.
///
/// What:
///  the pair-returning twin of [`rate`].
///  Why:
///  [`RegexSet::matches`] and
/// [`RegexSet::line_matches`] carry attribution (which rule,
///  on which line),
///  so their
/// closures return `Vec<(usize, usize)>` rather than `Vec<bool>`.
fn pair_rate(refs: &[&[u8]], run: impl Fn(&[&[u8]]) -> Vec<(usize, usize)>) -> f64 {
    let start = Instant::now();
    let mut scanned = 0u64;
    while start.elapsed().as_secs_f64() < BUDGET_SECS {
        black_box(run(black_box(refs)));
        scanned += refs.len() as u64;
    }
    return scanned as f64 / start.elapsed().as_secs_f64()
}

/// Times [`RegexSet::line_matches`] itself for the budget,
///  returning lines/s.
///
/// What:
///  calls the closure with `buf`/`starts` directly (its own signature,
///  not the
/// per-line-slice shape [`rate`] and [`pair_rate`] use),
///  so no per-iteration buffer
/// rebuild inflates its measured cost.
///  Why:
///  `line_matches` takes one whole-buffer
/// argument pair,
///  and the benchmark must time exactly the call the sidecar's own
/// [`RegexSet::is_match_batch_concat`] hook already races.
fn buf_rate(buf: &[u8], starts: &[usize], line_count: usize, run: impl Fn(&[u8], &[usize]) -> Vec<(usize, usize)>) -> f64 {
    let start = Instant::now();
    let mut scanned = 0u64;
    while start.elapsed().as_secs_f64() < BUDGET_SECS {
        black_box(run(black_box(buf), black_box(starts)));
        scanned += line_count as u64;
    }
    return scanned as f64 / start.elapsed().as_secs_f64()
}

/// Flattens every line's [`RegexSet::matches`] ids into `(line index, rule index)`
/// pairs,
///  in ascending line order.
///
/// What:
///  the naive per-line loop itself,
///  shared by the oracle below and the timed
/// per-line-loop arm in [`bench_line_matches`].
///  Why:
///  the oracle and the timed
/// per-line arm must run the exact same call,
///  or a divergence there (not a real
/// batch-vs-loop difference) could masquerade as a false speedup.
fn per_line_matches(set: &forbidden_regex::RegexSet, refs: &[&[u8]]) -> Vec<(usize, usize)> {
    return refs
        .iter()
        .enumerate()
        .flat_map(|(index, &line)| {
            return set.matches(line).map(move |rule| return (index, rule));
        })
        .collect();
}

/// Compares the per-line `matches()` loop,
///  the boolean concat-sweep hook,
///  and the
/// line-indexed buffer-batch path on the real ruleset over the corpus.
///
/// What:
///  builds the same concatenated buffer and line-start offsets
/// [`RegexSet::is_match_batch_concat`] builds internally,
///  confirms
/// [`RegexSet::line_matches`] agrees with the flattened per-line [`RegexSet::matches`]
/// oracle,
///  then times all three:
///  the naive per-line loop,
///  the existing boolean-only
/// concat-sweep hook (attribution-free,
///  already benched by [`bench_set_batch`]),
///  and
/// the new buffer-batch path.
///  Why:
///  #381 decides whether to route the seedless and
/// line-start rule groups through a batch sweep too;
///  these numbers are the evidence
/// for that call,
///  so all three sit in one report.
pub fn bench_line_matches(set: &forbidden_regex::RegexSet, corpus: &[Vec<u8>]) {
    let refs: Vec<&[u8]> = corpus.iter().map(Vec::as_slice).collect();
    // What:    Joins the lines with `\n` separators into one buffer, recording each
    //          line's start offset, mirroring is_match_batch_concat's own layout.
    // Why:     line_matches expects exactly this shape: one buffer plus ascending
    //          line-start offsets.
    let mut buf = Vec::new();
    let mut starts = Vec::with_capacity(refs.len());
    for &line in &refs {
        starts.push(buf.len());
        buf.extend_from_slice(line);
        buf.push(b'\n');
    }

    let oracle = per_line_matches(set, &refs);
    let batch = set.line_matches(&buf, &starts);
    assert_eq!(batch, oracle, "line_matches disagrees with the flattened per-line matches() oracle");

    let hits = oracle.len();
    let per_line = pair_rate(&refs, |lines| return per_line_matches(set, lines));
    let concat = rate(&refs, |lines| return set.is_match_batch_concat(lines));
    let line_batch = buf_rate(&buf, &starts, refs.len(), |b, s| return set.line_matches(b, s));
    println!(
        "\n[line-matches] over {} lines, {hits} (line, rule) pairs: per-line matches() {per_line:>12.0}  concat bool-only {concat:>12.0}  line_matches {line_batch:>12.0}  ({:.2}x vs per-line, {:.2}x vs concat-bool-only)",
        refs.len(),
        line_batch / per_line,
        line_batch / concat,
    );
}
