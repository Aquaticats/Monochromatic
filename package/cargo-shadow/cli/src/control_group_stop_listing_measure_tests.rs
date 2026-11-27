//! What:
//! The measurement behind `LISTING_BOUND` (`control_group_stop.rs`):
//! how long the listing of one scope takes when its one member has a given number of threads.
//! Why:
//! The figures beside the constant are the whole reason its value is 500 ms,
//! and the release read of `0.1.5-alpha.12` (Medium 3,
//! 2026-11-27) found that nothing in the tree could reproduce
//! them:
//! the fixture they came from was not kept,
//! so a reader could neither check the arithmetic nor measure the same shape again.
//! This control is that fixture and that measurement,
//! ignored by default because it starts a scope with thousands of threads.
//! Run it,
//! after building the release tests,
//! with:
//!
//! ```sh
//! # package/cargo-shadow/cli
//! cargo nextest run --release --run-ignored only \
//!   -E 'test(=control_group_stop::measure_tests::the_listing_of_a_scope_with_this_many_threads_takes)'
//! ```
//!
//! `CARGO_SHADOW_LISTING_THREADS` (comma separated,
//! default `0,1000,3000`) picks the thread counts and
//! `CARGO_SHADOW_LISTING_ROUNDS` (default `3`) the rounds each count is read in.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // describe('the measurement behind the listing bound', () => { it('times the listing of N threads', ...) })
//! ```

/// Writes the fixture's source and its ready file.
use std::fs;
/// Owns the folder of the fixture and of its ready file.
use std::path::{
    Path,
    PathBuf,
};
/// Starts the fixture and reads its arguments.
use std::process::Command;
/// The clock the listing is timed with.
use std::time::{
    Duration,
    Instant,
};

/// A scope this test starts the way a run starts one,
/// the proof that this host can give it,
/// and the handshake that waits
/// for the fixture's threads.
use crate::cgroup_test_support::{
    TestScope,
    start_in_scope,
    this_test_needs_a_delegated_scope,
    wait_until,
};
/// The listing whose time is measured.
/// The short-lived folder of one control.
use crate::test_scratch::scratch;

/// What:
/// `THREADS_SOURCE` is the C source of the fixture:
/// a program that starts this many threads that sleep,
/// writes its ready
/// file,
/// and then lives until it is killed.
/// Why:
/// The listing reads one `stat` file per thread of each member,
/// so the count of threads in one member is what its cost grows with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THREADS_SOURCE = `// start N sleeping threads, touch the ready file, then sleep`
/// ```
const THREADS_SOURCE: &str = r#"
/* What: includes the thread calls (pthread_create), the file calls the ready file needs, and sleep.
   Why: the fixture is one process whose threads are the work the listing does. */
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

/* What: one thread that sleeps until the process is killed.
   Why: a thread that runs would move the scheduler's work into the measurement. */
static void *live(void *ignored) {
  (void) ignored;
  for (;;) {
    sleep(60);
  }
  return NULL;
}

/* What: starts `arguments[1]` threads, writes `arguments[2]` once they all exist, and then sleeps.
   Why: the caller must know the count is real before it times the listing. */
int main(int count, char **arguments) {
  if (count < 3) {
    return 2;
  }
  long threads = strtol(arguments[1], NULL, 10);
  if (threads < 0) {
    return 3;
  }
  pthread_t *ids = calloc((size_t) threads + 1, sizeof(pthread_t));
  if (ids == NULL) {
    return 4;
  }
  for (long i = 0; i < threads; i++) {
    if (pthread_create(&ids[i], NULL, live, NULL) != 0) {
      return 5;
    }
  }
  FILE *out = fopen(arguments[2], "w");
  if (out == NULL) {
    return 6;
  }
  fprintf(out, "%ld\n", threads);
  fclose(out);
  for (;;) {
    sleep(60);
  }
  return 0;
}
"#;

/// What:
/// `fn the_fixture(folder: &Path) -> PathBuf` compiles the fixture into `folder` and answers its path.
/// Why:
/// The measurement must run the same program every time,
/// built from a source that lives beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFixture(folder: string): string  // cc -pthread -o folder/threads folder/threads.c
/// ```
fn the_fixture(folder: &Path) -> PathBuf {
    let source: PathBuf = folder.join("threads.c");
    let program: PathBuf = folder.join("threads");
    fs::write(&source, THREADS_SOURCE).expect("writes the fixture's source");
    let status = Command::new("cc")
        .arg("-pthread")
        .arg("--output")
        .arg(&program)
        .arg(&source)
        .status()
        .expect(
            "a C compiler named cc must be on PATH: the measurement compiles the fixture it times",
        );
    assert!(status.success(), "the fixture did not compile");
    return program;
}

/// What:
/// `fn the_counts(environment: &str) -> Vec<usize>` reads a comma-separated list of thread counts,
/// and answers the default when the variable is not set.
/// Why:
/// The counts are the measurement's input,
/// so a reader can repeat the same shapes,
/// or ask for another one,
/// without an edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theCounts(environment: string | undefined): number[]  // split(',').map(Number)
/// ```
fn the_counts(environment: &Option<String>) -> Vec<usize> {
    let text: &str = environment.as_deref().unwrap_or("0,1000,3000");
    let mut counts: Vec<usize> = Vec::new();
    for part in text.split(',') {
        counts.push(
            part.trim()
                .parse::<usize>()
                .expect("a thread count the environment names"),
        );
    }
    return counts;
}

/// What:
/// `fn the_median(times: &[Duration]) -> Duration` is the middle reading of an odd number of rounds.
/// Why:
/// One round on a loaded host says nothing on its own,
/// and the middle reading is what the constant's comment quotes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theMedian(times: number[]): number  // sort, then times[(n-1)/2]
/// ```
fn the_median(times: &[Duration]) -> Duration {
    let mut sorted: Vec<Duration> = times.to_vec();
    sorted.sort();
    return sorted[sorted.len() / 2];
}

/// What:
/// `fn the_listings_of(scope: &TestScope, rounds: usize) -> Vec<Duration>` times one listing of the scope per round,
/// and checks that each read the one member the fixture is.
/// Why:
/// A listing that failed or read no member would time a failure rather than the walk the bound covers.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theListingsOf(scope: TestScope, rounds: number): number[]  // time group.listings() each round
/// ```
fn the_listings_of(
    scope: &TestScope,
    rounds: usize,
) -> Vec<Duration> {
    let mut times: Vec<Duration> = Vec::new();
    for _round in 0..rounds {
        // What:     `Instant::now()` before and after the listing is the time the walk it makes takes.
        // Why:      The bound is spent inside that walk, so this is the reading the constant must cover.
        let started: Instant = Instant::now();
        let listing = scope.group.listings();
        times.push(started.elapsed());
        assert!(
            matches!(listing, crate::control_group_stop::report::Listing::Read(_)),
            "the listing did not read the whole scope, so the round times something else"
        );
    }
    return times;
}

/// Release read of `0.1.5-alpha.12`,
/// Medium 3:
/// the measurement beside `LISTING_BOUND`,
/// kept where it can be run again:
/// one process with N threads in a delegated scope,
/// timed once per round,
/// with the fixture's source in this file.
/// It is ignored by default:
/// a scope of thousands of threads is not part of the suite's work.
#[test]
#[ignore = "starts a scope of thousands of threads to measure the listing's bound; run it by hand (see the module comment)"]
fn the_listing_of_a_scope_with_this_many_threads_takes() {
    this_test_needs_a_delegated_scope();
    let folder = scratch();
    let program: PathBuf = the_fixture(folder.path());
    let counts: Vec<usize> = the_counts(&std::env::var("CARGO_SHADOW_LISTING_THREADS").ok());
    let rounds: usize = std::env::var("CARGO_SHADOW_LISTING_ROUNDS")
        .ok()
        .map(|text| {
            return text
                .parse::<usize>()
                .expect("a round count the environment names");
        })
        .unwrap_or(3);

    for threads in counts {
        let ready: PathBuf = folder.path().join(format!("ready-{threads}"));
        let mut command: Command = Command::new(&program);
        command.arg(threads.to_string()).arg(&ready);
        let scope: TestScope = start_in_scope(&mut command);
        wait_until("the threads of the fixture", &|| return ready.exists());
        let times: Vec<Duration> = the_listings_of(&scope, rounds);
        println!(
            "LISTING_BOUND measurement: {threads} threads, {rounds} rounds, median {:?}, every round {:?}",
            the_median(&times),
            times
        );
        let members: Vec<i32> = scope
            .group
            .members()
            .expect("the members of the scope the fixture is in");
        assert_eq!(
            members.len(),
            1,
            "the fixture is not the one member of the scope, so the measurement is of another shape: {members:?}"
        );
    }
}
