// What:     Scale tests for the Search index at 1,000, 10,000, 100,000 and 1,000,000 tracks (D112), pulled in by
//           `#[cfg(test)] #[path = "search_scale_tests.rs"] mod search_scale_tests;` in `lib.rs`.
//           1,000 artists and 1,000,000 tracks are normal operation, so the 1M test belongs to the normal suite.
//           Each test builds a synthetic library, appends it in feed-sized batches, and times queries of 1, 2, 3 and 8
//           characters (median of five runs, with min and max), the cold and warm one-character answer, the batch
//           preparation and the push, the memory of the index, and the old matcher on a small library for parity.
// Why:      The old matcher took seconds per keystroke at 1M; these tests keep the keystroke in milliseconds and keep
//           the index small, and print the numbers the report quotes.
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::scale_library_tests::{SyntheticLibrary, build_library};
use crate::search_feed::{FEED_BATCH, prepare_in_order};
use crate::search_index::SearchIndex;
use crate::search_results::oracle::oracle_search_library;
use crate::search_results::{SEARCH_RESULT_LIMIT, SearchState};
use crate::track::Track;

// What:     Runs per query when a median is taken.
// Why:      The host is shared and noisy; the spread of five runs shows how far a median can be trusted. A debug build
//           runs once, because its timings are only informational and the suite must stay short.
const REPEATS: usize = if cfg!(debug_assertions) { 1 } else { 5 };

// What:     The release bars: the slowest push under the write lock, and the median of any one query, at every size.
// Why:      Measured in release at 1,000,000 tracks on a shared host with a load average near 40: the slowest push was
//           3 to 10 ms (55 ms once under heavy load), and the median query was 2 ms for one letter and 12 to 16 ms for
//           two, three and eight characters before large buckets were tiered on four threads (4 to 8 ms after, on real
//           names), with a spread of about 2 ms. The query bar is about three times the slowest of those medians and the
//           push bar is about ten times the usual push, so they catch a return of the old per-keystroke cost (seconds)
//           without failing on host noise.
const RELEASE_PUSH_BAR: Duration = Duration::from_millis(100);
const RELEASE_QUERY_BAR: Duration = Duration::from_millis(50);

// What:     The debug bar for both: unoptimized code is about ten times slower, and the suite must stay short.
// Why:      The debug run still catches a stage that grows with the library instead of with the bucket.
const DEBUG_BAR: Duration = Duration::from_millis(2000);

// What:     `Spread`, the median, minimum and maximum of repeated runs.
// Why:      A bar is only meaningful against the run-to-run spread on unchanged input.
struct Spread {
    median: Duration,
    min: Duration,
    max: Duration,
}

// What:     `spread_of`, runs `stage` `REPEATS` times and summarizes the durations.
// Why:      Every timed stage uses it.
fn spread_of<T>(mut stage: impl FnMut() -> T) -> (Spread, T) {
    let mut samples: Vec<Duration> = Vec::new();
    let mut last: Option<T> = None;
    for _ in 0..REPEATS {
        let started: Instant = Instant::now();
        let value: T = stage();
        samples.push(started.elapsed());
        last = Some(black_box(value));
    }
    samples.sort();
    let spread: Spread = Spread {
        median: samples[samples.len() / 2],
        min: samples[0],
        max: samples[samples.len() - 1],
    };
    return (spread, last.expect("at least one repeat ran"));
}

// What:     `millis`, a duration in milliseconds.
// Why:      Reports print milliseconds.
fn millis(duration: Duration) -> f64 {
    return duration.as_secs_f64() * 1000.0;
}

// What:     `library_of`, the synthetic library of `size` tracks, one thousand per artist.
// Why:      All sizes keep the artist shape, so one artist's content is the same at every size.
fn library_of(size: usize) -> SyntheticLibrary {
    return build_library((size / 1000).max(1), 1000);
}

// What:     `built_index`, the index of all names, appended in feed-sized batches, with the time of each push.
// Why:      The push is what runs under the write lock, so its worst case is the stall a keystroke could see.
fn built_index(names: &[String]) -> (SearchIndex, Duration, Vec<Duration>) {
    let started: Instant = Instant::now();
    let mut index: SearchIndex = SearchIndex::new(1);
    index.reserve(names.len(), names.iter().map(String::len).sum(), names.iter().map(String::len).sum::<usize>() / 2);
    let mut pushes: Vec<Duration> = Vec::new();
    prepare_in_order(names, FEED_BATCH, |_number, prepared| {
        let pushed: Instant = Instant::now();
        index.append(prepared);
        pushes.push(pushed.elapsed());
        return true;
    });
    return (index, started.elapsed(), pushes);
}

// What:     `eight_char_query`, eight characters taken from the start of a long word of the library.
// Why:      The eight-character case needs a query that matches something.
fn eight_char_query(names: &[String]) -> String {
    for name in names.iter().skip(names.len() / 2) {
        let title: String = crate::track_entries::track_title_for(name);
        if let Some(word) = title.split(' ').find(|word| return word.chars().count() >= 8) {
            return word.chars().take(8).collect();
        }
    }
    return "qzxjvkqz".to_string();
}

// What:     `result_count`, the rows of a results state.
// Why:      Reports and checks count rows.
fn result_count(state: &SearchState) -> usize {
    return match state {
        SearchState::Results { items, .. } => items.len(),
        _ => 0,
    };
}

// What:     `report_index`, builds the index of a library of `size` tracks, times the queries, prints the report and
//           asserts the budgets.
// Why:      One routine serves every size so the numbers are comparable.
fn report_index(label: &str, size: usize) {
    let (push_bar, query_bar): (Duration, Duration) = if cfg!(debug_assertions) {
        (DEBUG_BAR, DEBUG_BAR)
    } else {
        (RELEASE_PUSH_BAR, RELEASE_QUERY_BAR)
    };
    let library: SyntheticLibrary = library_of(size);
    assert_eq!(library.artists, (size / 1000).max(1), "one thousand tracks per artist");
    let (index, build_time, pushes) = built_index(&library.names);
    assert_eq!(index.track_count(), library.names.len());
    let slowest_push: Duration = pushes.iter().copied().max().unwrap_or_default();
    println!(
        "== search index {label} ({} names): build {:.1} ms, {} pushes, slowest push {:.2} ms, heap {:.1} MiB ==",
        library.names.len(),
        millis(build_time),
        pushes.len(),
        millis(slowest_push),
        index.heap_bytes() as f64 / 1_048_576.0
    );
    assert!(slowest_push < push_bar, "a push took {slowest_push:?}, over the {push_bar:?} bar");
    let eight: String = eight_char_query(&library.names);
    let cold_started: Instant = Instant::now();
    let cold: SearchState = index.search("k");
    let cold_time: Duration = cold_started.elapsed();
    println!("  search 'k' cold (first keystroke)  {:.3} ms, rows {}", millis(cold_time), result_count(&cold));
    for (name, query) in [("1 char", "k"), ("2 chars", "ka"), ("3 chars", "kal"), ("8 chars", eight.as_str()), ("no match", "qzxjvk")] {
        let (spread, state) = spread_of(|| return index.search(query));
        println!(
            "  search {name:<9} {query:<10} median {:.3} ms (min {:.3}, max {:.3}), rows {}",
            millis(spread.median),
            millis(spread.min),
            millis(spread.max),
            result_count(&state)
        );
        assert!(spread.median < query_bar, "search {query:?} median {:?} is over the {query_bar:?} bar", spread.median);
        assert!(result_count(&state) <= SEARCH_RESULT_LIMIT);
    }
    if size >= 10_000 {
        assert_eq!(result_count(&index.search("k")), SEARCH_RESULT_LIMIT, "one letter fills the list at {size}");
    }
    assert!(matches!(index.search("qzxjvk"), SearchState::NoMatch(_)));
}

// What:     The index answers exactly as the old matcher on generated libraries of 1,000 and 3,000 tracks, for queries
//           of one to eight characters, several terms, other scripts and misses.
// Why:      The generated names are realistic (scripts, subfolders, loose root files), unlike the random names of the
//           parity tests in `search_index_tests.rs`; both must agree with the oracle.
#[test]
fn generated_libraries_match_the_oracle() {
    let sizes: &[usize] = if cfg!(debug_assertions) { &[1000] } else { &[1000, 3000] };
    for size in sizes.iter().copied() {
        let library: SyntheticLibrary = library_of(size);
        let tracks: Vec<Track> = library
            .names
            .iter()
            .zip(library.paths())
            .map(|(name, path)| return Track { uri: path.to_string_lossy().into_owned(), display_path: name.clone() })
            .collect();
        let (index, _built, _pushes) = built_index(&library.names);
        let eight: String = eight_char_query(&library.names);
        let queries: Vec<&str> = vec![
            "k", "a", "l", "s", "m", "ka", "kal", "kalo", "lo ri", "ven sa", "ASH", "あ", "カ", "夜", "ма", "Loose", "album 02", "cd 1",
            "01", "1", "#", "-", "qzxjvk", eight.as_str(),
        ];
        for query in queries {
            let expected: SearchState = oracle_search_library(query, &tracks, true);
            assert_eq!(index.search(query), expected, "query {query:?} at {size}");
        }
    }
}

// What:     The 1,000-track index meets the budgets.
// Why:      The smallest size, where the cap is not reached.
#[test]
fn search_index_scale_1k() {
    report_index("1k", 1000);
}

// What:     The 10,000-track index meets the budgets.
// Why:      The first size at which one letter fills the list.
#[test]
fn search_index_scale_10k() {
    report_index("10k", 10_000);
}

// What:     The 100,000-track index meets the budgets.
// Why:      The size the 1M numbers are compared with.
#[test]
fn search_index_scale_100k() {
    report_index("100k", 100_000);
}

// What:     The 1,000,000-track index meets the budgets.
// Why:      D112: a million tracks is normal operation, and a keystroke must stay in milliseconds.
#[test]
fn search_index_scale_1m() {
    report_index("1M", 1_000_000);
}
