// What:     Tests for `search_service.rs` and `search_feed.rs`, pulled in by
//           `#[cfg(test)] #[path = "search_service_tests.rs"] mod tests;` at the bottom of `search_service.rs`. They run
//           the real worker and feeder threads: sequence numbers, coalescing, the refresh signal, generation guards,
//           the side-built index swap, and D111: typing while a deliberately slow scan adds tracks in batches.
// Why:      D111 requires results for the tracks already known and, later, for the tracks that arrive, with the
//           per-keystroke latency holding during the scan.
use super::*;
use crate::search_feed::feed_library;
use crate::search_results::SearchResultKind;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// What:     How long a test waits for an answer before it fails.
// Why:      A test must fail with a message instead of hanging when the worker is stuck.
const PATIENCE: Duration = Duration::from_secs(20);

// What:     `Heard`, an answer with the time it was delivered.
// Why:      Latency is the time from `query` to delivery.
struct Heard {
    answer: SearchAnswer,
    at: Instant,
}

// What:     `service_with_channel`, a service whose answers arrive on a channel.
// Why:      A test thread reads answers the way the window thread would receive them.
fn service_with_channel() -> (Arc<SearchService>, Receiver<Heard>) {
    let (sender, receiver): (Sender<Heard>, Receiver<Heard>) = mpsc::channel();
    let service: Arc<SearchService> = Arc::new(SearchService::start(move |answer| {
        let _sent = sender.send(Heard { answer, at: Instant::now() });
    }));
    return (service, receiver);
}

// What:     `names`, display paths `Artist {n % artists}/{prefix}{n}.flac` for n in a range.
// Why:      Predictable names let a test know which queries match which tracks.
fn names(range: std::ops::Range<usize>, prefix: &str) -> Vec<String> {
    return range.map(|number| return format!("Artist {}/{prefix} {number}.flac", number % 5)).collect();
}

// What:     `wait_for`, reads answers until one satisfies the predicate.
// Why:      Refresh answers arrive after a throttle, so a test waits for the one that shows the expected state.
fn wait_for(receiver: &Receiver<Heard>, predicate: impl Fn(&SearchAnswer) -> bool) -> SearchAnswer {
    let deadline: Instant = Instant::now() + PATIENCE;
    loop {
        let remaining: Duration = deadline.saturating_duration_since(Instant::now());
        match receiver.recv_timeout(remaining) {
            Ok(heard) if predicate(&heard.answer) => return heard.answer,
            Ok(_) => {}
            Err(error) => panic!("no matching answer within {PATIENCE:?}: {error}"),
        }
    }
}

// What:     `result_count`, the number of rows in a results state.
// Why:      Most assertions count rows.
fn result_count(state: &SearchState) -> usize {
    return match state {
        SearchState::Results { items, .. } => items.len(),
        _ => 0,
    };
}

// What:     A query is answered on the worker with its sequence number, and a later query supersedes an earlier one.
// Why:      The window drops answers whose sequence is older than its newest query.
#[test]
fn a_query_is_answered_with_its_sequence() {
    let (service, receiver) = service_with_channel();
    let generation: u64 = service.reset(10, 200);
    service.append_for(generation, &names(0..10, "song"));
    let first: u64 = service.query("song");
    let second: u64 = service.query("song 7");
    assert!(second > first);
    let answer: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == second);
    assert_eq!(answer.query, "song 7");
    assert_eq!(result_count(&answer.state), 1);
    assert!(!answer.refresh);
}

// What:     Many queries sent at once leave the newest answered; older ones may be skipped.
// Why:      A burst of keystrokes costs one ranking, not one per key.
#[test]
fn a_burst_of_queries_answers_the_newest() {
    let (service, receiver) = service_with_channel();
    let generation: u64 = service.reset(100, 3000);
    service.append_for(generation, &names(0..100, "song"));
    let mut newest: u64 = 0;
    for length in 1..=40 {
        newest = service.query(&"s".repeat(length));
    }
    let answer: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == newest);
    assert_eq!(answer.query, "s".repeat(40));
}

// What:     Appending tracks makes the worker re-answer the current query with `refresh` set, and the new track is in it.
// Why:      This is the refresh signal that updates an open Search page in place.
#[test]
fn an_append_refreshes_the_current_query() {
    let (service, receiver) = service_with_channel();
    let generation: u64 = service.reset(10, 200);
    service.append_for(generation, &names(0..3, "song"));
    let sequence: u64 = service.query("song");
    let before: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == sequence && !answer.refresh);
    assert_eq!(result_count(&before.state), 3);
    service.append_for(generation, &names(3..6, "song"));
    let after: SearchAnswer = wait_for(&receiver, |answer| return answer.refresh && answer.known_tracks == 6);
    assert_eq!(after.sequence, sequence);
    assert_eq!(result_count(&after.state), 6);
}

// What:     An append to a library that was replaced is refused, and `finish` of a replaced library does nothing.
// Why:      A feeder that outlives its library must not write into the new one.
#[test]
fn a_late_batch_for_a_replaced_library_is_refused() {
    let (service, _receiver) = service_with_channel();
    let old: u64 = service.reset(2, 40);
    let new: u64 = service.reset(2, 40);
    assert_ne!(old, new);
    assert!(!service.append_for(old, &names(0..2, "song")));
    assert_eq!(service.track_count(), 0);
    assert!(service.append_for(new, &names(0..2, "song")));
    service.finish(old);
    assert_eq!(service.track_count(), 2);
}

// What:     The completeness flag is false while a feed runs and true after `finish`.
// Why:      A partial scan cannot establish that nothing matches (D69).
#[test]
fn completeness_follows_finish() {
    let (service, receiver) = service_with_channel();
    let generation: u64 = service.reset(2, 40);
    service.append_for(generation, &names(0..2, "song"));
    let sequence: u64 = service.query("zzz");
    let partial: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == sequence);
    assert!(!partial.complete);
    assert_eq!(partial.state, SearchState::NoMatch("zzz".to_string()));
    service.finish(generation);
    let done: SearchAnswer = wait_for(&receiver, |answer| return answer.refresh && answer.complete);
    assert_eq!(done.state, SearchState::NoMatch("zzz".to_string()));
}

// What:     `search_now` answers on the caller's thread with the same ranking as the worker.
// Why:      Benchmarks and tests read the index without a worker round trip.
#[test]
fn search_now_matches_the_worker_answer() {
    let (service, receiver) = service_with_channel();
    let generation: u64 = service.reset(20, 400);
    service.append_for(generation, &names(0..20, "song"));
    let sequence: u64 = service.query("song 1");
    let answer: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == sequence);
    assert_eq!(answer.state, service.search_now("song 1"));
}

// What:     Feeding a library into an empty service makes tracks searchable before the feed ends, then complete.
// Why:      D111: Search is available from the moment any track is known.
#[test]
fn feeding_an_empty_service_is_searchable_while_it_runs() {
    let (service, receiver) = service_with_channel();
    let library: Arc<Vec<String>> = Arc::new(names(0..400, "song"));
    feed_library(&service, library, 100, Duration::from_millis(80));
    let deadline: Instant = Instant::now() + PATIENCE;
    while service.track_count() == 0 {
        assert!(Instant::now() < deadline, "the first batch never arrived");
        thread::sleep(Duration::from_millis(5));
    }
    let sequence: u64 = service.query("song");
    let partial: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == sequence);
    assert!(partial.known_tracks > 0 && partial.known_tracks < 400, "searched {} of 400", partial.known_tracks);
    assert!(!partial.complete);
    let done: SearchAnswer = wait_for(&receiver, |answer| return answer.known_tracks == 400 && answer.complete);
    assert_eq!(result_count(&done.state), 400);
}

// What:     Feeding a service that already holds a library builds the new index on the side and swaps it at the end.
// Why:      The open page must keep its results during a rescan instead of flashing empty.
#[test]
fn a_second_library_replaces_the_first_only_when_complete() {
    let (service, _receiver) = service_with_channel();
    feed_library(&service, Arc::new(names(0..50, "old")), 25, Duration::ZERO);
    let deadline: Instant = Instant::now() + PATIENCE;
    while service.track_count() < 50 {
        assert!(Instant::now() < deadline, "the first library never arrived");
        thread::sleep(Duration::from_millis(5));
    }
    feed_library(&service, Arc::new(names(0..120, "new")), 10, Duration::from_millis(30));
    thread::sleep(Duration::from_millis(60));
    assert_eq!(service.track_count(), 50, "the old library keeps answering while the new one is built");
    assert_eq!(result_count(&service.search_now("old")), 50);
    while service.track_count() != 120 {
        assert!(Instant::now() < deadline, "the second library never arrived");
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(result_count(&service.search_now("old")), 0);
    assert_eq!(result_count(&service.search_now("new")), 120);
}

// What:     A newer feed cancels an older one that is still running.
// Why:      Opening another folder while the first is being indexed must not mix the two.
#[test]
fn a_newer_feed_cancels_an_older_one() {
    let (service, _receiver) = service_with_channel();
    feed_library(&service, Arc::new(names(0..1000, "first")), 10, Duration::from_millis(20));
    thread::sleep(Duration::from_millis(60));
    feed_library(&service, Arc::new(names(0..30, "second")), 10, Duration::ZERO);
    let deadline: Instant = Instant::now() + PATIENCE;
    while result_count(&service.search_now("second")) < 30 {
        assert!(Instant::now() < deadline, "the newer feed never finished");
        thread::sleep(Duration::from_millis(5));
    }
    thread::sleep(Duration::from_millis(100));
    assert_eq!(service.track_count(), 30);
    assert_eq!(result_count(&service.search_now("first")), 0);
}

// What:     D111: queries typed while a deliberately slow scan appends batches find the tracks known at that moment,
//           later find the tracks that arrived, get refresh answers, and answer within the latency bar.
// Why:      This is the hard requirement. The scan adds a batch every 60 ms; a typist sends a growing query every 15 ms.
#[test]
fn typing_during_a_slow_scan_finds_known_and_new_tracks_within_the_bar() {
    let (service, receiver) = service_with_channel();
    let batches: usize = 12;
    let per_batch: usize = 200;
    let library: Vec<String> = (0..batches * per_batch)
        .map(|number| return format!("Artist {}/batch{}-{number:05}.flac", number % 5, number / per_batch))
        .collect();
    let generation: u64 = service.reset(library.len(), library.len() * 30);
    let scan: Arc<SearchService> = Arc::clone(&service);
    let scanner = thread::spawn(move || {
        for chunk in library.chunks(per_batch) {
            scan.append_for(generation, chunk);
            thread::sleep(Duration::from_millis(60));
        }
        scan.finish(generation);
    });
    let mut latencies: Vec<Duration> = Vec::new();
    let mut saw_new_batch_after_first: bool = false;
    let mut refreshed_sizes: Vec<usize> = Vec::new();
    let mut sent: Vec<(u64, Instant)> = Vec::new();
    let started: Instant = Instant::now();
    let typing_ends: Duration = Duration::from_millis(60 * batches as u64 / 2);
    let scan_ends: Duration = Duration::from_millis(60 * batches as u64 + 200);
    while started.elapsed() < scan_ends {
        if started.elapsed() < typing_ends && service.track_count() > 0 {
            let query: String = ["b", "ba", "bat", "batc", "batch", "batch1", "batch1-"][sent.len() % 7].to_string();
            sent.push((service.query(&query), Instant::now()));
        }
        thread::sleep(Duration::from_millis(15));
        while let Ok(heard) = receiver.try_recv() {
            if heard.answer.refresh {
                refreshed_sizes.push(heard.answer.known_tracks);
            } else if let Some((_, at)) = sent.iter().find(|(sequence, _)| return *sequence == heard.answer.sequence) {
                latencies.push(heard.at.duration_since(*at));
            }
            if heard.answer.known_tracks > per_batch {
                saw_new_batch_after_first = true;
            }
        }
    }
    scanner.join().expect("the scanner thread finishes");
    let last: u64 = service.query("batch");
    let done: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == last && answer.complete);
    assert_eq!(done.known_tracks, batches * per_batch);
    assert!(saw_new_batch_after_first, "no answer ever saw a batch after the first");
    assert!(refreshed_sizes.len() >= 2, "too few refresh answers while the scan appended: {refreshed_sizes:?}");
    assert!(refreshed_sizes.windows(2).any(|pair| return pair[1] > pair[0]), "refresh answers never saw the library grow: {refreshed_sizes:?}");
    latencies.sort();
    let slowest: Duration = latencies.last().copied().unwrap_or_default();
    let bar: Duration = if cfg!(debug_assertions) { Duration::from_millis(1500) } else { Duration::from_millis(250) };
    assert!(!latencies.is_empty(), "no keystroke was answered during the scan");
    assert!(slowest < bar, "slowest keystroke {slowest:?} is over the {bar:?} bar; {} samples", latencies.len());
    let SearchState::Results { items, .. } = &done.state else {
        panic!("expected results after the scan, got {:?}", done.state);
    };
    assert_eq!(items.len(), 500, "the cap applies to the complete library");
    assert!(items.iter().all(|item| return item.kind == SearchResultKind::Track));
}

// What:     Streamed scan batches make tracks searchable at once, and the final list is adopted without a rebuild when equal.
// Why:      D111: Search works while the walk runs; when the walk ends the same list arrives as the queue, and rebuilding
//           a million-name index for nothing would waste seconds.
#[test]
fn streamed_batches_are_searchable_and_adopted_when_the_final_list_is_equal() {
    let (service, receiver) = service_with_channel();
    let library: Vec<String> = names(0..300, "song");
    service.stream_batch(0, library[..100].to_vec());
    service.stream_batch(100, library[100..200].to_vec());
    service.stream_flush();
    let sequence: u64 = service.query("song");
    let partial: SearchAnswer = wait_for(&receiver, |answer| return answer.sequence == sequence);
    assert!(!partial.complete);
    assert!(partial.known_tracks >= 100);
    service.stream_batch(200, library[200..].to_vec());
    service.stream_flush();
    assert_eq!(service.track_count(), 300);
    let mut different: Vec<String> = library.clone();
    // Position 139 is one of the sixteen evenly spaced positions that `adopt_streamed` compares at 300 tracks.
    different[139] = "Elsewhere/other.flac".to_string();
    assert!(!service.adopt_streamed(&different), "a list that differs at a sampled position is not adopted");
    assert!(service.adopt_streamed(&library), "an equal list is adopted");
    let done: SearchAnswer = wait_for(&receiver, |answer| return answer.complete && answer.known_tracks == 300);
    assert_eq!(result_count(&done.state), 300);
}

// What:     A batch that does not continue the index is ignored, and a batch at position zero starts a new library.
// Why:      A position must be a queue position; a gap or an overlap would shift every later result.
#[test]
fn a_scan_batch_that_does_not_continue_the_index_is_ignored() {
    let (service, _receiver) = service_with_channel();
    service.stream_batch(0, names(0..50, "song"));
    service.stream_batch(80, names(80..100, "song"));
    service.stream_flush();
    assert_eq!(service.track_count(), 50, "a gap is ignored");
    service.stream_batch(30, names(30..60, "song"));
    service.stream_flush();
    assert_eq!(service.track_count(), 50, "an overlap is ignored");
    service.stream_batch(0, names(0..10, "fresh"));
    service.stream_flush();
    assert_eq!(service.track_count(), 10, "position zero starts a new library");
    assert_eq!(result_count(&service.search_now("song")), 0);
    assert_eq!(result_count(&service.search_now("fresh")), 10);
}

// What:     `running_threads`, how many threads the service still has to join.
// Why:      `shutdown` empties the list, so the count tells whether a second call has anything left to do.
fn running_threads(service: &SearchService) -> usize {
    return service.threads.lock().expect("no thread of the service panics while holding the list").len();
}

// What:     `shutdown` joins the three threads once, and the later `drop` and a second `shutdown` find nothing to join.
// Why:      `main` calls it while the threads are alive; the thread-local handle is dropped later, during process exit,
//           where a join on a killed thread panics on Windows.
#[test]
fn shutdown_ends_all_three_threads_and_a_second_call_does_nothing() {
    let (service, receiver) = service_with_channel();
    let generation: u64 = service.reset(10, 200);
    service.append_for(generation, &names(0..10, "song"));
    let sequence: u64 = service.query("song");
    wait_for(&receiver, |answer| return answer.sequence == sequence);
    assert_eq!(running_threads(&service), 3);
    service.shutdown();
    assert_eq!(running_threads(&service), 0);
    service.shutdown();
    assert_eq!(running_threads(&service), 0);
}

// What:     After `shutdown`, a query is not answered and nothing panics.
// Why:      The engine thread may still hand a batch or a query to the service while `main` ends.
#[test]
fn a_query_after_shutdown_is_ignored_without_a_panic() {
    let (service, receiver) = service_with_channel();
    service.shutdown();
    let _sequence: u64 = service.query("song");
    service.stream_batch(0, names(0..3, "song"));
    assert!(receiver.recv_timeout(Duration::from_millis(300)).is_err());
}
