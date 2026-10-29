//! Integration tests for the decision cache,
//!  against a throwaway in-memory database.

use super::*;
use crate::decision::{Decision, DecisionKind};
use crate::policy::{CacheIdentity, SCHEMA_VERSION};

// A fixed identity for the tests; every key column matters for a hit.
// It carries the current schema version, as every shipped identity does.
fn identity() -> CacheIdentity {
    return CacheIdentity { policy_id: 11, meter_id: 22, decoder_stack_id: 33, schema_version: SCHEMA_VERSION }
}

fn probe(gain: f32) -> Decision {
    return Decision { gain, kind: DecisionKind::ProbeEstimate, measured_peak: 0.9, duration_secs: 200.0 }
}

fn exact(gain: f32) -> Decision {
    return Decision { gain, kind: DecisionKind::FullScanExact, measured_peak: 1.5, duration_secs: 200.0 }
}

// A stored decision reads back unchanged for a matching key.
#[tokio::test]
async fn round_trips_a_decision() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(42, identity(), &probe(0.8))
        .await
        .expect("probe decision should be cached");

    let hit = cache
        .get(42, identity())
        .await
        .expect("cached decision should be read")
        .expect("matching decision should exist");
    assert_eq!(hit, probe(0.8));
}

// A missing fingerprint, or a mismatched identity, is a miss.
#[tokio::test]
async fn misses_on_absent_or_mismatched_key() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(42, identity(), &probe(0.8))
        .await
        .expect("probe decision should be cached");

    assert!(
        cache
            .get(99, identity())
            .await
            .expect("absent decision lookup should succeed")
            .is_none()
    );
    let other = CacheIdentity { policy_id: 999, ..identity() };
    assert!(
        cache
            .get(42, other)
            .await
            .expect("mismatched decision lookup should succeed")
            .is_none()
    );
}

// A probe estimate must not overwrite an exact decision for the same key.
#[tokio::test]
async fn exact_is_not_downgraded_by_a_probe() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(42, identity(), &exact(0.5))
        .await
        .expect("exact decision should be cached");
    cache
        .put(42, identity(), &probe(0.8))
        .await
        .expect("lower-precedence write should be handled");

    let hit = cache
        .get(42, identity())
        .await
        .expect("cached decision should be read")
        .expect("exact decision should remain cached");
    assert_eq!(hit.kind, DecisionKind::FullScanExact);
    assert_eq!(hit.gain, 0.5);
}

// An exact decision does upgrade a prior probe estimate.
#[tokio::test]
async fn probe_is_upgraded_by_an_exact() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(42, identity(), &probe(0.8))
        .await
        .expect("probe decision should be cached");
    cache
        .put(42, identity(), &exact(0.5))
        .await
        .expect("exact decision should replace probe");

    let hit = cache
        .get(42, identity())
        .await
        .expect("cached decision should be read")
        .expect("exact decision should remain cached");
    assert_eq!(hit.kind, DecisionKind::FullScanExact);
    assert_eq!(hit.gain, 0.5);
}

// The exact-fingerprint snapshot lists only exact rows for this identity, not probes.
#[tokio::test]
async fn exact_fingerprints_lists_only_exact_rows() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(1, identity(), &exact(0.5))
        .await
        .expect("exact decision should be cached");
    cache
        .put(2, identity(), &probe(0.8))
        .await
        .expect("probe decision should be cached");
    let short = Decision { kind: DecisionKind::ShortFullScan, ..exact(0.6) };
    cache
        .put(3, identity(), &short)
        .await
        .expect("short full-scan decision should be cached");
    let other = CacheIdentity { policy_id: 999, ..identity() };
    cache
        .put(4, other, &exact(0.4))
        .await
        .expect("other-identity decision should be cached");

    let listed = cache
        .exact_fingerprints(identity())
        .await
        .expect("exact fingerprints should be listed");
    assert_eq!(listed, HashSet::from([1_u64, 3]));
}

// Removing an exact key drops only that row: a positive control proves the row was present first.
#[tokio::test]
async fn remove_drops_only_the_named_key() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(1, identity(), &exact(0.5))
        .await
        .expect("exact decision should be cached");
    cache
        .put(2, identity(), &exact(0.6))
        .await
        .expect("second exact decision should be cached");
    assert!(
        cache
            .get(1, identity())
            .await
            .expect("cached decision should be read")
            .is_some(),
        "positive control: the row should exist before removal"
    );

    let removed = cache.remove(1, identity()).await.expect("remove should run");
    assert!(removed, "removing a stored key should report a removed row");
    assert!(
        cache
            .get(1, identity())
            .await
            .expect("removed decision lookup should succeed")
            .is_none()
    );
    assert!(
        cache
            .get(2, identity())
            .await
            .expect("neighbour lookup should succeed")
            .is_some(),
        "a different fingerprint must keep its decision"
    );
}

// Removing a key that was never stored reports nothing removed.
#[tokio::test]
async fn remove_of_absent_key_reports_false() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");

    let removed = cache.remove(99, identity()).await.expect("remove should run");
    assert!(!removed, "an absent key should report nothing removed");
}

// Removal is scoped to the identity: the same fingerprint under another identity survives.
#[tokio::test]
async fn remove_is_scoped_to_identity() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(42, identity(), &exact(0.5))
        .await
        .expect("exact decision should be cached");
    let other = CacheIdentity { policy_id: 999, ..identity() };

    let removed = cache.remove(42, other).await.expect("remove should run");
    assert!(!removed, "a mismatched identity must remove nothing");
    assert!(
        cache
            .get(42, identity())
            .await
            .expect("original identity lookup should succeed")
            .is_some(),
        "the original identity keeps its row"
    );
}

// After removal the key is empty, so a fresh probe is accepted where an exact row would have blocked it.
#[tokio::test]
async fn probe_is_stored_after_an_exact_row_is_removed() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(42, identity(), &exact(0.5))
        .await
        .expect("exact decision should be cached");
    cache.remove(42, identity()).await.expect("remove should run");
    cache
        .put(42, identity(), &probe(0.8))
        .await
        .expect("probe should be stored into the emptied key");

    let hit = cache
        .get(42, identity())
        .await
        .expect("cached decision should be read")
        .expect("probe should now be the stored decision");
    assert_eq!(hit.kind, DecisionKind::ProbeEstimate);
}

// The path of a throwaway database file unique to this process and test name.
fn scratch_db_path(name: &str) -> std::path::PathBuf {
    return crate::test_scratch::scratch_root().join(format!("truepeak-core-cache-{}-{name}.db", std::process::id()))
}

// Delete the throwaway database and any sibling journal files Turso created.
fn remove_scratch_db(path: &std::path::Path) {
    for suffix in ["", "-wal", "-log", "-shm"] {
        let mut sibling = path.as_os_str().to_owned();
        sibling.push(suffix);
        let _ = std::fs::remove_file(std::path::PathBuf::from(sibling));
    }
}

// Insert a row with an explicit schema-version literal, bypassing `put`, as an older build wrote it.
async fn insert_row_at_version(cache: &DecisionCache, fingerprint: i64, schema_version: i64) {
    cache
        .conn
        .execute(
            "INSERT INTO decisions (fingerprint, policy_id, meter_id, decoder_stack_id, schema_version, kind, gain, \
             measured_peak, duration_secs) VALUES (?1, 11, 22, 33, ?2, 2, 0.5, 1.5, 200.0)",
            (fingerprint, schema_version),
        )
        .await
        .expect("row should be inserted");
}

// Count every row in the table, whatever its version.
async fn count_rows(cache: &DecisionCache) -> i64 {
    let mut rows = cache
        .conn
        .query("SELECT COUNT(*) FROM decisions", ())
        .await
        .expect("count should run");
    let row = rows
        .next()
        .await
        .expect("count row should be read")
        .expect("count always returns one row");
    return *row
        .get_value(0)
        .expect("count column should exist")
        .as_integer()
        .expect("count should be an integer")
}

// The shipped schema version is above the first one, so rows written under the retired gxhash
// fingerprint (version 1) can never match a current read.
#[test]
fn the_schema_version_moved_past_the_gxhash_rows() {
    const { assert!(SCHEMA_VERSION >= 2, "version 1 rows hold gxhash fingerprints and must not match") };
}

// A row left by the gxhash build (schema version written as the literal 1) is a miss for the
// current identity, and reopening the database deletes it. A current-version row survives the
// reopen as the positive control, and a newer-version row survives too, so a downgrade is harmless.
#[tokio::test]
async fn reopen_removes_gxhash_era_rows_and_keeps_current_and_newer_rows() {
    let path = scratch_db_path("legacy-cleanup");
    remove_scratch_db(&path);
    let path_text = path.to_str().expect("temp path should be UTF-8");
    let cache = DecisionCache::open(path_text)
        .await
        .expect("fresh database should open");
    insert_row_at_version(&cache, 0x1234_5678, 1).await;
    cache
        .put(0x1234_5678, identity(), &exact(0.7))
        .await
        .expect("current-version decision should be cached");
    insert_row_at_version(&cache, 0x9999, i64::from(SCHEMA_VERSION) + 1).await;
    assert_eq!(count_rows(&cache).await, 3, "positive control: three rows before the reopen");
    let legacy_identity = CacheIdentity { schema_version: 1, ..identity() };
    assert!(
        cache
            .get(0x1234_5678, legacy_identity)
            .await
            .expect("legacy lookup should succeed")
            .is_some(),
        "positive control: the legacy row is readable only under its own version"
    );
    let current = cache
        .get(0x1234_5678, identity())
        .await
        .expect("current lookup should succeed")
        .expect("the current-version row should be found");
    assert_eq!(current.gain, 0.7, "the current identity must see the current row, never the legacy one");
    drop(cache);

    let reopened = DecisionCache::open(path_text)
        .await
        .expect("database should reopen");
    assert_eq!(count_rows(&reopened).await, 2, "the version 1 row should be deleted on open");
    assert!(
        reopened
            .get(0x1234_5678, legacy_identity)
            .await
            .expect("legacy lookup after reopen should succeed")
            .is_none(),
        "the legacy row must be gone"
    );
    assert!(
        reopened
            .get(0x1234_5678, identity())
            .await
            .expect("current lookup after reopen should succeed")
            .is_some(),
        "the current-version row must survive"
    );
    let newer = CacheIdentity { schema_version: SCHEMA_VERSION + 1, ..identity() };
    assert!(
        reopened
            .get(0x9999, newer)
            .await
            .expect("newer lookup after reopen should succeed")
            .is_some(),
        "a newer-version row must survive"
    );
    drop(reopened);
    remove_scratch_db(&path);
}

// A cache written and reopened under the current version hits: the cleanup does not eat live rows.
#[tokio::test]
async fn a_current_row_hits_after_reopen() {
    let path = scratch_db_path("reopen-hit");
    remove_scratch_db(&path);
    let path_text = path.to_str().expect("temp path should be UTF-8");
    let cache = DecisionCache::open(path_text)
        .await
        .expect("fresh database should open");
    cache
        .put(77, identity(), &exact(0.5))
        .await
        .expect("decision should be cached");
    drop(cache);

    let reopened = DecisionCache::open(path_text)
        .await
        .expect("database should reopen");
    let hit = reopened
        .get(77, identity())
        .await
        .expect("lookup should succeed")
        .expect("the decision written before the reopen should hit");
    assert_eq!(hit, exact(0.5));
    drop(reopened);
    remove_scratch_db(&path);
}

//region Cold-write benchmark: the cost of storing a first analysis, measured on a real disk.

// An invented fingerprint for row `index`: a splitmix64 mix, so keys land across the B-tree the way
// real content hashes do, never in insertion order.
fn invented_fingerprint(index: u64) -> u64 {
    let mut mixed = index.wrapping_add(0x9E37_79B9_7F4A_7C15);
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    return mixed ^ (mixed >> 31)
}

// Read an environment variable the benchmark needs, or fail with how to set it.
fn bench_setting(name: &str, fallback: Option<&str>) -> String {
    if let Ok(value) = std::env::var(name) {
        return value;
    }
    return fallback
        .unwrap_or_else(|| panic!("set {name}; see the `bench:cache` task in truepeak-core/mise.toml"))
        .to_string()
}

// Benchmark, skipped in normal runs: store `CACHE_BENCH_ROWS` (default 10,000) invented exact decisions into a
// fresh database file under `CACHE_BENCH_DIR`, as a first analysis does, and print the wall time.
// `CACHE_BENCH_MODE` picks the write path: `row` stores each decision with its own `put` (one commit per row),
// `normal` does the same after `PRAGMA synchronous = NORMAL`, `batch` stores `CACHE_BENCH_BATCH` rows (default
// 256) per `put_batch` call, and `batch-normal` combines both.
// The directory must be on a real disk: on tmpfs an fsync costs nothing and the benchmark measures nothing.
#[tokio::test]
#[ignore = "benchmark; run through the bench:cache mise task with CACHE_BENCH_DIR on a real disk"]
async fn bench_cold_write() {
    let directory = std::path::PathBuf::from(bench_setting("CACHE_BENCH_DIR", None));
    let mode = bench_setting("CACHE_BENCH_MODE", Some("row"));
    let rows: u64 = bench_setting("CACHE_BENCH_ROWS", Some("10000"))
        .parse()
        .expect("CACHE_BENCH_ROWS should be a whole number");
    let batch_size: usize = bench_setting("CACHE_BENCH_BATCH", Some("256"))
        .parse()
        .expect("CACHE_BENCH_BATCH should be a whole number");
    assert!(
        ["row", "normal", "batch", "batch-normal"].contains(&mode.as_str()),
        "unknown CACHE_BENCH_MODE {mode}"
    );
    let path = directory.join(format!("bench-{}-{mode}.db", std::process::id()));
    remove_scratch_db(&path);
    let path_text = path.to_str().expect("bench path should be UTF-8");
    let cache = DecisionCache::open(path_text)
        .await
        .expect("bench database should open");
    if mode.ends_with("normal") {
        cache
            .conn
            .execute("PRAGMA synchronous = NORMAL", ())
            .await
            .expect("synchronous pragma should apply");
    }
    let entries: Vec<(u64, Decision)> = (0..rows).map(|index| return (invented_fingerprint(index), exact(0.5))).collect();
    let started = std::time::Instant::now();
    if mode.starts_with("batch") {
        for chunk in entries.chunks(batch_size) {
            cache
                .put_batch(identity(), chunk)
                .await
                .expect("bench batch should be cached");
        }
    } else {
        for (fingerprint, decision) in &entries {
            cache
                .put(*fingerprint, identity(), decision)
                .await
                .expect("bench decision should be cached");
        }
    }
    let elapsed = started.elapsed();
    assert_eq!(count_rows(&cache).await, i64::try_from(rows).expect("row count fits i64"));
    println!(
        "CACHE_BENCH mode={mode} rows={rows} batch={batch_size} wall_ms={:.1}",
        elapsed.as_secs_f64() * 1000.0
    );
    drop(cache);
    remove_scratch_db(&path);
}

//endregion

//region put_batch: one transaction for many rows, with the same precedence rule as `put`.

// Every row of a batch reads back unchanged.
#[tokio::test]
async fn put_batch_stores_every_row() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    let entries: Vec<(u64, Decision)> = (0..50_u64).map(|key| return (key, exact(0.25))).collect();
    cache
        .put_batch(identity(), &entries)
        .await
        .expect("batch should be cached");
    assert_eq!(count_rows(&cache).await, 50);
    for (key, decision) in &entries {
        let hit = cache
            .get(*key, identity())
            .await
            .expect("lookup should run")
            .expect("every batched row should exist");
        assert_eq!(&hit, decision);
    }
}

// An empty batch opens no transaction and stores nothing.
#[tokio::test]
async fn an_empty_put_batch_is_a_no_op() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put_batch(identity(), &[])
        .await
        .expect("an empty batch should succeed");
    assert_eq!(count_rows(&cache).await, 0);
}

// The never-downgrade rule holds inside a batch, against a stored row and against an earlier row of the same batch.
#[tokio::test]
async fn put_batch_never_downgrades_an_exact_row() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    cache
        .put(1, identity(), &exact(0.5))
        .await
        .expect("exact decision should be cached");
    let entries = vec![(1, probe(0.9)), (2, exact(0.4)), (2, probe(0.9))];
    cache
        .put_batch(identity(), &entries)
        .await
        .expect("batch should be cached");
    let stored = cache.get(1, identity()).await.expect("lookup should run");
    assert_eq!(stored, Some(exact(0.5)));
    let in_batch = cache.get(2, identity()).await.expect("lookup should run");
    assert_eq!(in_batch, Some(exact(0.4)));
}

// A row the table refuses (a NaN gain is stored as NULL, which a NOT NULL column rejects) fails the batch's
// transaction; the rows around it are still stored one by one, and the call reports how many failed.
#[tokio::test]
async fn a_refused_row_does_not_discard_its_neighbours() {
    let cache = DecisionCache::open(":memory:")
        .await
        .expect("in-memory cache should open");
    let refused = exact(f32::NAN);
    cache
        .put(99, identity(), &refused)
        .await
        .expect_err("positive control: a NaN gain is refused on its own");
    let entries = vec![(1, exact(0.1)), (2, refused), (3, exact(0.3))];
    let error = cache
        .put_batch(identity(), &entries)
        .await
        .expect_err("the refused row should be reported");
    assert!(error.message.contains("1 of 3"), "unexpected message: {}", error.message);
    assert_eq!(count_rows(&cache).await, 2);
    assert_eq!(cache.get(1, identity()).await.expect("lookup should run"), Some(exact(0.1)));
    assert_eq!(cache.get(3, identity()).await.expect("lookup should run"), Some(exact(0.3)));
}

//endregion

//region Crash safety: kill -9 during batched writes leaves a database that opens and holds whole batches only.

// Rows per batch in the crash tests: small enough that a kill often lands inside one.
const CRASH_BATCH: u64 = 64;

// Child side of the crash tests, run only through `run_crash_cycles`: store invented rows from `CACHE_CRASH_START`
// onward forever, `CRASH_BATCH` rows per commit, and print the running total after every commit.
// `CACHE_CRASH_WRITE` is `batch` (one `put_batch` per commit) or `row` (one `put` per row, the control).
// `CACHE_CRASH_SYNC` is `full` (the default) or `normal` (`PRAGMA synchronous = NORMAL`).
#[tokio::test]
#[ignore = "child process of the kill -9 tests; needs the CACHE_CRASH_* variables"]
async fn crash_writer_child() {
    let path = bench_setting("CACHE_CRASH_DB", None);
    let mut next: u64 = bench_setting("CACHE_CRASH_START", None)
        .parse()
        .expect("CACHE_CRASH_START should be a whole number");
    let write = bench_setting("CACHE_CRASH_WRITE", Some("batch"));
    let cache = DecisionCache::open(&path)
        .await
        .expect("crash database should open");
    if bench_setting("CACHE_CRASH_SYNC", Some("full")) == "normal" {
        cache
            .conn
            .execute("PRAGMA synchronous = NORMAL", ())
            .await
            .expect("synchronous pragma should apply");
    }
    loop {
        let entries: Vec<(u64, Decision)> = (next..next + CRASH_BATCH)
            .map(|index| return (invented_fingerprint(index), exact(0.5)))
            .collect();
        if write == "row" {
            for (fingerprint, decision) in &entries {
                cache
                    .put(*fingerprint, identity(), decision)
                    .await
                    .expect("crash row should be cached");
            }
        } else {
            cache
                .put_batch(identity(), &entries)
                .await
                .expect("crash batch should be cached");
        }
        next += CRASH_BATCH;
        println!("CRASH_COMMITTED {next}");
    }
}

// What a reopened database held after one kill.
struct CrashOutcome {
    // The total the child last reported before it was killed.
    reported: u64,
    // The rows found after reopening.
    found: u64,
}

// Run `cycles` rounds: start the child on the same database file, wait until it has reported a few commits, kill it
// with SIGKILL (`Child::kill`), then reopen the file and check it is a consistent prefix of the invented keys.
// Returns what each round found, for the caller's alignment assertion.
async fn run_crash_cycles(write: &str, sync: &str, cycles: u32) -> Vec<CrashOutcome> {
    use std::io::BufRead;
    let path = scratch_db_path(&format!("crash-{write}-{sync}"));
    remove_scratch_db(&path);
    let path_text = path.to_str().expect("crash path should be UTF-8").to_string();
    let executable = std::env::current_exe().expect("the test binary should know its own path");
    let mut outcomes = Vec::new();
    let mut start: u64 = 0;
    for cycle in 0..cycles {
        let mut child = std::process::Command::new(&executable)
            .args(["--exact", "cache::tests::crash_writer_child", "--ignored", "--nocapture"])
            .env("CACHE_CRASH_DB", &path_text)
            .env("CACHE_CRASH_START", start.to_string())
            .env("CACHE_CRASH_WRITE", write)
            .env("CACHE_CRASH_SYNC", sync)
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("the crash writer should start");
        let reader = std::io::BufReader::new(child.stdout.take().expect("the child's stdout is piped"));
        // The line iterator stays alive until after the kill: closing the pipe early would make the child's next
        // `println!` panic and end it before the kill, which the test would then mistake for a kill.
        let mut lines = reader.lines();
        let mut reported = start;
        for read in lines.by_ref() {
            let line = read.expect("the child's stdout should be readable");
            if let Some(total) = line.strip_prefix("CRASH_COMMITTED ") {
                reported = total.parse().expect("the child reports whole numbers");
                if reported >= start + 5 * CRASH_BATCH {
                    break;
                }
            }
        }
        assert!(reported >= start + 5 * CRASH_BATCH, "the child stopped before it wrote enough");
        // The child commits a batch in about a millisecond, so a kill sent the instant its line arrives always
        // lands between batches. A varying pause (0 to 3.999 ms, from a fixed mix of the cycle number) spreads the
        // kills across the batch.
        std::thread::sleep(std::time::Duration::from_micros(invented_fingerprint(u64::from(cycle)) % 4000));
        child.kill().expect("SIGKILL should reach the child");
        child.wait().expect("the killed child should be reaped");
        drop(lines);
        let reopened = DecisionCache::open(&path_text)
            .await
            .expect("the database should open after kill -9");
        let mut check = reopened
            .conn
            .query("PRAGMA integrity_check", ())
            .await
            .expect("integrity check should run");
        let verdict = check.next().await.expect("integrity row should be read").expect("integrity returns a row");
        assert_eq!(verdict.get_value(0).expect("integrity cell").as_text().map(String::as_str), Some("ok"));
        let found = u64::try_from(count_rows(&reopened).await).expect("row count is not negative");
        let present = reopened
            .exact_fingerprints(identity())
            .await
            .expect("fingerprints should be read");
        assert!(found >= reported, "rows the child reported as committed were lost: {found} < {reported}");
        assert_eq!(present.len() as u64, found, "every row should be exact under the test identity");
        for index in 0..found {
            assert!(present.contains(&invented_fingerprint(index)), "row {index} of the prefix is missing");
        }
        outcomes.push(CrashOutcome { reported, found });
        start = found;
        drop(reopened);
    }
    remove_scratch_db(&path);
    return outcomes
}

// After a kill -9 during batched writes the database opens, passes the integrity check, holds every committed
// batch, and holds only whole batches: no half-stored batch is visible.
#[tokio::test]
async fn batched_writes_survive_kill_nine_with_whole_batches_only() {
    for sync in ["full", "normal"] {
        for outcome in run_crash_cycles("batch", sync, 4).await {
            assert_eq!(
                outcome.found % CRASH_BATCH,
                0,
                "sync={sync}: a batch was torn (reported {}, found {})",
                outcome.reported,
                outcome.found
            );
        }
    }
}

// Control for the test above: with one commit per row the same kill leaves a partial batch, which shows the kill
// lands inside a batch's worth of work and that the alignment check can fail.
#[tokio::test]
async fn row_writes_after_kill_nine_leave_partial_batches() {
    let outcomes = run_crash_cycles("row", "full", 12).await;
    assert!(
        outcomes.iter().any(|outcome| return outcome.found % CRASH_BATCH != 0),
        "no kill landed inside a batch of row commits: {:?}",
        outcomes.iter().map(|outcome| return (outcome.reported, outcome.found)).collect::<Vec<_>>()
    );
}

//endregion
