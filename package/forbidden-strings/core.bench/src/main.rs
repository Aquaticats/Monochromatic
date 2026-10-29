//! Throughput benchmark:
//!  forbidden-regex vs the regex crate on the real ruleset.
//!
//! What:
//!  loads the shipped forbidden-strings rules,
//!  ports each into this engine's
//! dialect,
//!  compile-filters to the subset both engines accept,
//!  then feeds our
//! scanner the `&`/`~` versions and `regex` the complement-stripped versions and
//! times `is_match` over a mostly-non-matching corpus.
//!  Why:
//!  `regex` cannot express
//! `&`/`~`,
//!  but on the overlapping dialect the restricted engine should still win on
//! raw scanning throughput;
//!  this measures whether it does on realistic rules.

/// The synthetic corpus generator.
mod corpus;

/// The single-DFA batch-kernel microbenchmark.
mod kernels;

/// The dialect normalizer.
mod normalize;

/// The per-rule porter.
mod port;

/// The real ruleset loader.
mod rules;

/// Imports the optimization barrier so match work is not elided.
use std::hint::black_box;

/// Imports the wall-clock timer for throughput measurement.
use std::time::Instant;

/// Imports the corpus builder.
use crate::corpus::build_corpus;

/// Imports the ruleset loader.
use crate::rules::load_rules;

/// Wall-clock budget each engine is timed for.
const BUDGET_SECS: f64 = 10.0;

/// Lines checked for engine parity before timing.
const PARITY_SAMPLE: usize = 5_000;

/// Builds both engines from the ported rules,
///  checks parity,
///  and times them.
///
/// What:
///  the whole benchmark,
///  end to end.
///  Why:
///  a single command that proves the
/// throughput claim or refutes it on the realistic ruleset.
fn main() {
    // Send the [phase]/[diag]/[oracle]/WARNING diagnostics to stderr via tracing (RUST_LOG,
    // default info); the throughput report stays on stdout via println!.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| return tracing_subscriber::EnvFilter::new("info")),
        )
        .with_writer(std::io::stderr)
        .init();
    let pairs = load_rules();
    tracing::info!(rules = pairs.len(), "loaded rules");

    // Keep only rules `regex` accepts, so its RegexSet build cannot fail wholesale.
    let regex_filter = Instant::now();
    let usable: Vec<&(String, String)> = pairs
        .iter()
        .filter(|(_, bare)| return regex::bytes::Regex::new(bare).is_ok())
        .collect();
    tracing::info!(
        kept = usable.len(),
        secs = regex_filter.elapsed().as_secs_f64(),
        "regex-filter kept rules"
    );

    // Build ours once, leniently: rules this dialect cannot express are dropped.
    // The freshly compiled set only feeds the serializer; the reloaded set below
    // keeps the `fset` name so the round-tripped bytes are what everything races.
    let ours_all: Vec<&str> = usable.iter().map(|(our, _)| return our.as_str()).collect();
    let build_ours = Instant::now();
    let (compiled, kept) = forbidden_regex::RegexSet::compile_lenient(&ours_all);
    let serialized = compiled.to_bytes().expect("forbidden-regex serializes");
    println!(
        "build+serialize ours: {:.2}s for {} rules, {} bytes",
        build_ours.elapsed().as_secs_f64(),
        kept.len(),
        serialized.len()
    );
    let fset =
        forbidden_regex::RegexSet::from_bytes(&serialized).expect("forbidden-regex reloads");
    tracing::debug!(
        seedless = fset.seedless_count(),
        groups = fset.seedless_group_count(),
        "seedless rules collapsed into union DFAs"
    );

    // Diagnostic: probe the "all-rules combined automaton" idea -- can a single DFA over
    // every kept rule even build, or does it state-explode (which is why the architecture
    // is per-rule gate-plus-fold)?
    let ours_kept: Vec<&str> = kept.iter().map(|&i| return ours_all[i]).collect();
    let combined_start = Instant::now();
    match forbidden_regex::try_combined_dfa(&ours_kept) {
        Ok(states) => tracing::debug!(
            states,
            secs = combined_start.elapsed().as_secs_f64(),
            "all-rules combined DFA built"
        ),
        Err(error) => tracing::debug!(
            cause = %error,
            secs = combined_start.elapsed().as_secs_f64(),
            "all-rules combined DFA not buildable"
        ),
    }

    // Build regex over exactly the rules ours kept, so both race the same set. The
    // size limits are raised because the stripped generic-shape rules compile to a
    // large automaton; serialized/compiled size is not a constraint here.
    let bare_kept: Vec<&str> = kept.iter().map(|&i| return usable[i].1.as_str()).collect();
    let build_regex = Instant::now();
    let rset = regex::bytes::RegexSetBuilder::new(&bare_kept)
        .size_limit(1 << 30)
        .dfa_size_limit(1 << 30)
        .build()
        .expect("regex compiles");
    println!("build regex: {:.2}s for {} rules", build_regex.elapsed().as_secs_f64(), bare_kept.len());

    let corpus = build_corpus();
    let lines = corpus.len();
    let bytes: usize = corpus.iter().map(Vec::len).sum();

    // Parity on a bounded sample: a systematic porter mismatch shows up here without
    // an unbounded full-corpus pass.
    let sample = &corpus[..corpus.len().min(PARITY_SAMPLE)];
    let fhits = sample.iter().filter(|line| return fset.is_match(line.as_slice())).count();
    let rhits = sample.iter().filter(|line| return rset.is_match(line.as_slice())).count();
    println!("parity ({} lines): forbidden-regex={fhits}, regex={rhits}", sample.len());
    if fhits != rhits {
        tracing::warn!("engines disagree on the sample; comparison is not apples to apples");
    }

    // Oracle: the fold must not MISS any literal-free rule's match. The counting union
    // over the original seedless rules is the independent reference (proven equal to the
    // old DFA groups); every line it flags must still be flagged by the folded is_match.
    tracing::debug!(
        groups = fset.seedless_group_count(),
        line_start = fset.line_start_count(),
        oracle_positions = fset.seedless_union_size(),
        "fold structure"
    );
    let missed = corpus
        .iter()
        .filter(|l| return fset.csa_only_is_match(l) && !fset.is_match(l))
        .count();
    if missed == 0 {
        tracing::info!(lines, "fold misses no literal-free match");
    } else {
        tracing::warn!(missed, lines, "fold misses literal-free matches (false negatives)");
    }

    let prefilter_hits = corpus.iter().filter(|l| return fset.prefilter_only_is_match(l)).count();
    tracing::debug!(prefilter_hits, lines, "prefilter flags lines (each triggers the per-rule fallback)");
    let seeded = fset.len() - fset.seedless_count();
    tracing::debug!(anchored = fset.anchored_count(), seeded, "anchored seeded rules (rest fall back to counting)");

    let avg_len = bytes as f64 / lines as f64;
    let threads = std::thread::available_parallelism().map_or(1, |n| return n.get());
    println!("scanning {threads} threads per engine, {BUDGET_SECS:.0}s each");
    // forbidden-regex is immutable, so all threads share the one instance (no
    // per-thread state). regex needs a mutable lazy-DFA cache, so each thread clones
    // it (the program is Arc-shared, so this is the cheap idiomatic parallel form and
    // lets regex scale instead of contending on a shared cache).
    let frate = throughput(&corpus, || (), |_, line| return fset.is_match(line), threads);
    let rrate = throughput(&corpus, || return rset.clone(), |r, line| return r.is_match(line), threads);
    // Profiling split: prefilter-only vs gate-only vs seedless-only, to locate the
    // per-line bottleneck (prefilter cost vs per-rule fallback vs literal-free scans).
    let prefilter_rate = throughput(&corpus, || (), |_, line| return fset.prefilter_only_is_match(line), threads);
    let candidates_rate = throughput(&corpus, || (), |_, line| return fset.candidates_only_is_match(line), threads);
    let anchored_rate = throughput(&corpus, || (), |_, line| return fset.gate_anchored_only_is_match(line), threads);
    let gate_rate = throughput(&corpus, || (), |_, line| return fset.gate_only_is_match(line), threads);
    let seedless_rate = throughput(&corpus, || (), |_, line| return fset.seedless_only_is_match(line), threads);
    let csa_rate = throughput(&corpus, || (), |_, line| return fset.csa_only_is_match(line), threads);
    report("forbidden-regex", frate, avg_len);
    report("  prefilter-only", prefilter_rate, avg_len);
    report("  candidates    ", candidates_rate, avg_len);
    report("  anchored-only ", anchored_rate, avg_len);
    report("  gate-only     ", gate_rate, avg_len);
    report("  seedless-only ", seedless_rate, avg_len);
    report("  csa-union-only", csa_rate, avg_len);
    report("regex          ", rrate, avg_len);
    println!("\nforbidden-regex is {:.2}x the throughput of regex", frate / rrate);

    // Batch-kernel experiment: race the scalar, interleaved, tight, bucketed, and Sheng
    // layouts on a seedless single DFA over the same corpus (the one place a many-lines DFA
    // kernel can pay off, since the set pipeline gates most lines before any DFA runs).
    kernels::bench_buckets(&corpus, avg_len);

    // Set-level batch experiment: one concatenated-buffer prefilter sweep over the whole
    // corpus versus the per-line loop, on the real ruleset.
    kernels::bench_set_batch(&fset, &corpus);

    // Line-indexed batch experiment: the attribution-carrying buffer-batch path
    // (line_matches, #377/#378) versus the naive per-line matches() loop and the
    // boolean-only concat-sweep hook, on the real ruleset. Feeds the #381 decision on
    // batching the seedless and line-start rule groups.
    kernels::bench_line_matches(&fset, &corpus);
}

/// Scans the corpus on repeat for `BUDGET_SECS` across `threads`,
///  returning total
/// lines per second.
///
/// What:
///  each worker builds its OWN matcher via `make` (a per-thread clone),
///  then
/// loops full corpus passes until the budget elapses,
///  summing every line scanned.
/// Why:
///  a per-thread matcher gives each engine independent state,
///  so both scale up
/// with cores;
///  sharing one matcher would serialize an engine on its internal cache
/// and unfairly slow it down.
///  The fixed time budget bounds the run.
fn throughput<M, Make, Run>(corpus: &[Vec<u8>], make: Make, run: Run, threads: usize) -> f64
where
    Make: Fn() -> M + Sync,
    Run: Fn(&M, &[u8]) -> bool + Sync,
{
    let start = Instant::now();
    let total: u64 = std::thread::scope(|scope| {
        let workers: Vec<_> = (0..threads)
            .map(|_| {
                return scope.spawn(|| {
                    let matcher = make();
                    let mut scanned = 0u64;
                    let mut acc = 0u64;
                    while start.elapsed().as_secs_f64() < BUDGET_SECS {
                        for line in corpus {
                            if run(&matcher, black_box(line.as_slice())) {
                                acc += 1;
                            }
                        }
                        scanned += corpus.len() as u64;
                    }
                    black_box(acc);
                    return scanned
                })
            })
            .collect();
        // expect, not unwrap: the repo's clippy.toml bans Result::unwrap; a worker join only
        // fails if that benchmark thread panicked, which is a bug we want to surface.
        return workers.into_iter().map(|w| return w.join().expect("benchmark worker thread panicked")).sum()
    });
    return total as f64 / start.elapsed().as_secs_f64()
}

/// Prints one engine's throughput line.
///
/// What:
///  reports lines/s and the derived MB/s from the average line length.
///  Why:
///  the
/// two figures make the comparison legible at a glance.
fn report(name: &str, lines_per_sec: f64, avg_len: f64) {
    let mb_per_sec = lines_per_sec * avg_len / 1_000_000.0;
    println!("{name}: {lines_per_sec:>13.0} lines/s   {mb_per_sec:>8.1} MB/s");
}
