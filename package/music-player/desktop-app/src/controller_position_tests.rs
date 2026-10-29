// What:     Controller-level tests for the audible position and the track-end handoff, pulled in by
//           `#[cfg(test)] #[path = "controller_position_tests.rs"] mod position_tests;` at the bottom of
//           `controller_audio.rs`.
// Why:      The pure arithmetic is covered in `playback_clock_tests.rs`; these tests drive the real controller
//           with a real decoder and a hand-made ring buffer whose consumer plays the part of the output
//           callback, so a regression to "position follows the decoder" or "next track starts when the decoder
//           finishes" fails here.

// What:     `use super::*;`. Bring the parent module's items (the `Controller` methods) into scope.
// Why:      Tests call the private helpers `audible_secs` and `ring_queued_frames`.
use super::*;

// What:     `use ringbuf::{HeapCons, HeapRb};` and the ring traits.
// Why:      The test builds the ring itself and pops from it the way the output callback does.
use ringbuf::traits::{Consumer, Observer, Split};
use ringbuf::{HeapCons, HeapRb};

// What:     `use std::sync::{Arc, Mutex};`.
// Why:      The update callback appends what the controller emits to a list the test reads.
use std::sync::{Arc, Mutex};

// What:     `use crate::peakcache::CacheHandle;`.
// Why:      `Controller::new` takes a cache handle; the degraded one touches no disk.
use crate::peakcache::CacheHandle;

// What:     `const RATE: u32 = 8_000;`. The first test track's sample rate (mono).
// Why:      One second is 8000 samples, which keeps the ring and the expected counts small.
const RATE: u32 = 8_000;

// What:     `const TRACK_SECONDS: u32 = 3;`. Length of each test track.
// Why:      Three seconds is more than the one-second ring, so the decoder finishes while a ring is queued.
const TRACK_SECONDS: u32 = 3;

// What:     `const TRACK_SAMPLES: usize = 24_000;`. Samples in one test track.
// Why:      The tail-integrity assertions count them.
const TRACK_SAMPLES: usize = (RATE * TRACK_SECONDS) as usize;

// What:     `enum Seen { ... }`. The updates the tests care about.
// Why:      The closure keeps only these, so assertions stay short.
#[derive(Debug, Clone, PartialEq)]
enum Seen {
    Position(f64),
    NowPlaying(String),
}

// What:     `fn write_tone_wav(path: &Path, rate: u32, hz: f64, amplitude: f64)`. Write a mono 16-bit PCM WAV of
//           `TRACK_SECONDS` of a sine.
// Why:      A real file keeps the decoder, the chunking and the end-of-stream signal real; the two tracks differ in
//           pitch and level so the samples around a join prove which track they belong to.
fn write_tone_wav(path: &Path, rate: u32, hz: f64, amplitude: f64) {
    let frames = rate * TRACK_SECONDS;
    let data_bytes = frames * 2;
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&rate.to_le_bytes());
    bytes.extend_from_slice(&(rate * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for n in 0..frames {
        let phase = 2.0 * std::f64::consts::PI * hz * f64::from(n) / f64::from(rate);
        let sample = (phase.sin() * amplitude) as i16;
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    std::fs::write(path, bytes).expect("the test track should be writable");
}

// What:     `struct Rig { ... }`. A controller with two tracks queued, the first loaded, a one-second ring, and the
//           consumer end of that ring.
// Why:      Every test starts from the same playing controller on the first track.
struct Rig {
    controller: Controller,
    consumer: HeapCons<f32>,
    seen: Arc<Mutex<Vec<Seen>>>,
    directory: PathBuf,
}

// What:     `fn rig(tag: &str, second_rate: u32) -> Rig`. Build the rig in a fresh temporary directory; the second
//           track has `second_rate`, so a different value forces the "output must be rebuilt" path.
// Why:      Tests must never touch the user's music or session.
fn rig(tag: &str, second_rate: u32) -> Rig {
    let directory = crate::test_scratch::scratch_root().join(format!("mp-position-{}-{}", std::process::id(), tag));
    std::fs::create_dir_all(&directory).expect("the test directory should be creatable");
    let first = directory.join("t1.wav");
    let second = directory.join("t2.wav");
    write_tone_wav(&first, RATE, 440.7, 8_000.0);
    write_tone_wav(&second, second_rate, 880.3, 12_000.0);
    let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let mut controller = Controller::new(
        Box::new(move |update| {
            let mut list = sink.lock().expect("the update list should not be poisoned");
            match update {
                Update::Position(secs) => list.push(Seen::Position(secs)),
                Update::NowPlaying { name, .. } => list.push(Seen::NowPlaying(name)),
                _ => {}
            }
        }),
        None,
        CacheHandle::open_degraded(),
    );
    controller.queue.set_tracks(vec![first, second]);
    controller.queue.play_index(0);
    controller.playing = true;
    assert!(controller.load_current(), "the first test track should load");
    let (producer, consumer) = HeapRb::<f32>::new(RATE as usize).split();
    controller.producer = Some(producer);
    return Rig { controller, consumer, seen, directory }
}

// What:     `fn pump_all(controller: &mut Controller)`. Pump until the controller has nothing to push.
// Why:      `pump_audio` returns false when the ring is full or the track is draining.
fn pump_all(controller: &mut Controller) {
    let mut guard = 0;
    while controller.pump_audio() {
        guard += 1;
        assert!(guard < 10_000, "pumping should settle");
    }
}

// What:     `fn consume(consumer: &mut HeapCons<f32>, samples: usize) -> Vec<f32>`. Pop up to `samples` as the output
//           would.
// Why:      The consumer's read index is the only thing the audible position may depend on.
fn consume(consumer: &mut HeapCons<f32>, samples: usize) -> Vec<f32> {
    let mut taken = Vec::new();
    while taken.len() < samples {
        match consumer.try_pop() {
            Some(sample) => taken.push(sample),
            None => break,
        }
    }
    return taken
}

// What:     `fn decode_all(path: &Path) -> Vec<f32>`. Every sample of a file, decoded straight from the source.
// Why:      The reference the joined output is compared with.
fn decode_all(path: &Path) -> Vec<f32> {
    let mut source = crate::decode::open(path).expect("the test track should open");
    let mut all = Vec::new();
    loop {
        let chunk = source.next_chunk().expect("the test track should decode");
        if chunk.is_empty() {
            return all;
        }
        all.extend(chunk);
    }
}

// What:     `fn positions(seen: &Arc<Mutex<Vec<Seen>>>) -> Vec<f64>`. Every `Position` emitted so far.
// Why:      Assertions compare these with the consumed frames.
fn positions(seen: &Arc<Mutex<Vec<Seen>>>) -> Vec<f64> {
    let list = seen.lock().expect("the update list should not be poisoned");
    return list
        .iter()
        .filter_map(|item| return match item {
            Seen::Position(secs) => Some(*secs),
            Seen::NowPlaying(_) => None,
        })
        .collect()
}

// What:     `fn now_playing(seen: &Arc<Mutex<Vec<Seen>>>) -> Vec<String>`. Every `NowPlaying` name emitted so far.
// Why:      The handoff tests check when the second track is announced.
fn now_playing(seen: &Arc<Mutex<Vec<Seen>>>) -> Vec<String> {
    let list = seen.lock().expect("the update list should not be poisoned");
    return list
        .iter()
        .filter_map(|item| return match item {
            Seen::NowPlaying(name) => Some(name.clone()),
            Seen::Position(_) => None,
        })
        .collect()
}

// What:     `fn names(list: &[&str]) -> Vec<String>`. Owned strings for comparisons.
// Why:      Keeps assertions on `now_playing` short.
fn names(list: &[&str]) -> Vec<String> {
    return list.iter().map(|item| return (*item).to_string()).collect()
}

#[test]
fn guard_a_full_ring_moves_no_position_before_the_output_takes_a_sample() {
    // The old behavior counted every pushed frame as played: a full one-second ring meant Position(~1.0).
    let mut rig = rig("full-ring", RATE);
    pump_all(&mut rig.controller);
    assert_eq!(rig.consumer.occupied_len(), RATE as usize, "the ring should be full");
    assert!(!rig.controller.tick_playback());
    assert_eq!(positions(&rig.seen), vec![0.0]);
    assert!(rig.controller.audible_secs().abs() < 1e-9);
}

#[test]
fn guard_position_follows_the_frames_the_output_consumed() {
    let mut rig = rig("follows", RATE);
    pump_all(&mut rig.controller);
    assert_eq!(consume(&mut rig.consumer, 4_000).len(), 4_000);
    rig.controller.tick_playback();
    let last = positions(&rig.seen).last().copied().expect("a position should have been emitted");
    assert!((last - 0.5).abs() < 1e-9, "half a second consumed should read 0.5, got {last}");
    // The decoder refills the ring; the position still reads only what was consumed.
    pump_all(&mut rig.controller);
    assert!((rig.controller.audible_secs() - 0.5).abs() < 1e-9);
    assert_eq!(consume(&mut rig.consumer, 800).len(), 800);
    rig.controller.tick_playback();
    let after = positions(&rig.seen).last().copied().expect("a position should have been emitted");
    assert!((after - 0.6).abs() < 1e-9, "0.6 s consumed should read 0.6, got {after}");
}

#[test]
fn guard_the_saved_position_is_the_audible_one() {
    // The session stores `audible_secs`; with a full ring and a quarter second consumed it must read 0.25, not 1.25.
    let mut rig = rig("session", RATE);
    pump_all(&mut rig.controller);
    consume(&mut rig.consumer, 2_000);
    assert!((rig.controller.audible_secs() - 0.25).abs() < 1e-9);
}

#[test]
fn guard_a_same_format_next_track_follows_in_the_same_ring_and_is_announced_when_audible() {
    let mut rig = rig("gapless", RATE);
    let first_gain = rig.controller.track_gain;
    let mut played: Vec<f32> = Vec::new();
    let mut staged_while_old_audible = false;
    let mut announced_at_samples = None;
    for _ in 0..1_000 {
        pump_all(&mut rig.controller);
        if rig.controller.staged.is_some() && now_playing(&rig.seen) == names(&["t1.wav"]) {
            staged_while_old_audible = true;
            // The old behavior had already rebuilt the output here and dropped the ring: no drain, no flush.
            assert!(!rig.controller.clock.is_draining());
            assert_eq!(rig.controller.queue.current_index(), Some(0));
        }
        played.extend(consume(&mut rig.consumer, 1_000));
        rig.controller.tick_playback();
        if announced_at_samples.is_none() && now_playing(&rig.seen).len() == 2 {
            announced_at_samples = Some(played.len());
        }
        if played.len() >= TRACK_SAMPLES + 4_000 {
            break;
        }
    }
    assert!(staged_while_old_audible, "the next track should be staged while the old one is audible");
    // The second track is announced exactly when the output has consumed the first one, to the 1 000-sample step.
    let announced = announced_at_samples.expect("the second track should be announced");
    assert!((TRACK_SAMPLES..TRACK_SAMPLES + 1_000).contains(&announced), "announced after {announced} samples");
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav", "t2.wav"]));
    assert_eq!(rig.controller.queue.current_index(), Some(1));
    assert!(rig.controller.staged.is_none());
    // No sample of the first track is lost and the second begins on the very next sample.
    let second_gain = rig.controller.track_gain;
    let first = decode_all(&rig.directory.join("t1.wav"));
    let second = decode_all(&rig.directory.join("t2.wav"));
    assert_eq!(first.len(), TRACK_SAMPLES);
    for (index, sample) in first.iter().enumerate() {
        assert!((played[index] - sample * first_gain).abs() < 1e-5, "sample {index} of the first track differs");
    }
    for index in 0..1_000 {
        assert!(
            (played[TRACK_SAMPLES + index] - second[index] * second_gain).abs() < 1e-5,
            "sample {index} of the second track differs"
        );
    }
    // The position of the new track counts from its own first frame.
    let all = positions(&rig.seen);
    assert!(all.iter().any(|secs| return (secs - f64::from(TRACK_SECONDS)).abs() < 0.11), "the old track reaches its length");
    assert!(all.last().is_some_and(|secs| return *secs < 1.0), "the new track counts from zero");
}

#[test]
fn guard_a_different_format_next_track_waits_for_the_tail_and_no_sample_is_lost() {
    // The second track has another sample rate, so the output has to be rebuilt after the first one played out.
    let mut rig = rig("handoff", 16_000);
    let mut consumed = 0;
    let mut saw_tail_while_draining = false;
    let mut handed_off = false;
    for _ in 0..1_000 {
        pump_all(&mut rig.controller);
        if rig.controller.clock.is_draining() && rig.consumer.occupied_len() > 0 {
            saw_tail_while_draining = true;
            // The old behavior had already announced the second track here and dropped the ring.
            assert_eq!(now_playing(&rig.seen), names(&["t1.wav"]));
            assert_eq!(rig.controller.queue.current_index(), Some(0));
            assert!(rig.controller.staged.is_none());
        }
        consumed += consume(&mut rig.consumer, 2_000).len();
        if rig.controller.tick_playback() {
            handed_off = true;
            break;
        }
    }
    assert!(handed_off, "the handoff should happen once the ring played out");
    assert!(saw_tail_while_draining, "the decoder should finish while audio is still queued");
    assert_eq!(consumed, TRACK_SAMPLES, "every sample of the first track should reach the output");
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav", "t2.wav"]));
    assert_eq!(rig.controller.queue.current_index(), Some(1));
    // The last position of the finished track is its full length, and the new track starts from zero.
    let all = positions(&rig.seen);
    assert!(all.iter().any(|secs| return (secs - f64::from(TRACK_SECONDS)).abs() < 0.11));
    assert!(all.last().is_some_and(|secs| return secs.abs() < 1e-9));
    assert!(!rig.controller.clock.is_draining());
}

#[test]
fn a_paused_player_does_not_start_the_next_format_until_play_resumes() {
    let mut rig = rig("paused", 16_000);
    let mut guard = 0;
    while !rig.controller.clock.is_draining() {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        guard += 1;
        assert!(guard < 1_000, "the decoder should finish");
    }
    consume(&mut rig.consumer, RATE as usize);
    assert_eq!(rig.consumer.occupied_len(), 0);
    rig.controller.playing = false;
    assert!(!rig.controller.tick_playback());
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav"]));
    rig.controller.playing = true;
    assert!(rig.controller.tick_playback());
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav", "t2.wav"]));
}

#[test]
fn seek_resets_the_position_to_the_target_and_counts_from_there() {
    let mut rig = rig("seek", RATE);
    pump_all(&mut rig.controller);
    consume(&mut rig.consumer, 4_000);
    rig.controller.tick_playback();
    // With an output, `seek` rebuilds the ring; here the test swaps in a fresh one the same way.
    rig.controller.seek(2.0);
    let (producer, consumer) = HeapRb::<f32>::new(RATE as usize).split();
    rig.controller.producer = Some(producer);
    rig.consumer = consumer;
    assert_eq!(positions(&rig.seen).last().copied(), Some(2.0));
    assert!((rig.controller.audible_secs() - 2.0).abs() < 1e-9);
    pump_all(&mut rig.controller);
    assert!((rig.controller.audible_secs() - 2.0).abs() < 1e-9);
    consume(&mut rig.consumer, 4_000);
    rig.controller.tick_playback();
    let last = positions(&rig.seen).last().copied().expect("a position should have been emitted");
    assert!((last - 2.5).abs() < 1e-9, "half a second after the seek should read 2.5, got {last}");
}

#[test]
fn seeking_during_the_drain_cancels_the_handoff() {
    let mut rig = rig("seek-drain", 16_000);
    let mut guard = 0;
    while !rig.controller.clock.is_draining() {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        guard += 1;
        assert!(guard < 1_000, "the decoder should finish");
    }
    rig.controller.seek(1.0);
    assert!(!rig.controller.clock.is_draining());
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav"]));
}

#[test]
fn seeking_while_the_next_track_is_staged_seeks_the_audible_track() {
    let mut rig = rig("seek-staged", RATE);
    let mut guard = 0;
    while rig.controller.staged.is_none() {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        rig.controller.tick_playback();
        guard += 1;
        assert!(guard < 1_000, "the next track should be staged");
    }
    assert_eq!(rig.controller.queue.current_index(), Some(0));
    rig.controller.seek(1.0);
    assert!(rig.controller.staged.is_none(), "the seek cancels the staged track");
    assert!(!rig.controller.clock.has_boundary());
    assert!((rig.controller.audible_secs() - 1.0).abs() < 1e-9);
    assert_eq!(rig.controller.queue.current_index(), Some(0));
    // The decoder is back on the first track near 1.0 s: the next samples are a run of the first track's samples
    // (the decoder may restart at the start of the packet that holds 1.0 s, so the run is searched for, not assumed).
    let (producer, mut consumer) = HeapRb::<f32>::new(RATE as usize).split();
    rig.controller.producer = Some(producer);
    pump_all(&mut rig.controller);
    let gain = rig.controller.track_gain;
    let reference = decode_all(&rig.directory.join("t1.wav"));
    let played = consume(&mut consumer, 500);
    assert_eq!(played.len(), 500);
    let found = (0..reference.len() - played.len()).find(|&offset| {
        return played.iter().enumerate().all(|(index, sample)| return (sample - reference[offset + index] * gain).abs() < 1e-5)
    });
    let start = found.expect("the samples after the seek should be a run of the first track");
    assert!(start.abs_diff(RATE as usize) <= 2_048, "the run starts near 1.0 s, not at {start}");
    // Nothing was announced for the second track.
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav"]));
}

#[test]
fn a_user_skip_while_the_next_track_is_staged_drops_the_staging() {
    let mut rig = rig("skip-staged", RATE);
    let mut guard = 0;
    while rig.controller.staged.is_none() {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        rig.controller.tick_playback();
        guard += 1;
        assert!(guard < 1_000, "the next track should be staged");
    }
    // Next, pressed while the first track is still audible, goes to the second track, as the listener expects.
    let moved = rig.controller.queue.advance(false);
    assert_eq!(moved, Some(1));
    assert!(rig.controller.load_current());
    assert!(rig.controller.staged.is_none());
    assert!(!rig.controller.clock.has_boundary());
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav", "t2.wav"]));
}

#[test]
fn a_queue_change_while_staged_reloads_instead_of_announcing_the_stale_track() {
    let mut rig = rig("stale-staged", RATE);
    let mut guard = 0;
    while rig.controller.staged.is_none() {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        rig.controller.tick_playback();
        guard += 1;
        assert!(guard < 1_000, "the next track should be staged");
    }
    // Switching to repeat-one makes the natural next track the current one again, not the staged file.
    rig.controller.queue.set_playback_mode(crate::command::PlaybackMode::Repeat);
    let mut handed_off = false;
    for _ in 0..1_000 {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        if rig.controller.tick_playback() {
            handed_off = true;
            break;
        }
    }
    assert!(handed_off);
    assert!(rig.controller.staged.is_none());
    assert_eq!(rig.controller.queue.current_index(), Some(0));
    assert_eq!(now_playing(&rig.seen), names(&["t1.wav", "t1.wav"]));
}

#[test]
fn silent_mode_reports_no_position_and_never_hands_off() {
    let mut rig = rig("silent", RATE);
    rig.controller.producer = None;
    assert!(!rig.controller.tick_playback());
    assert_eq!(positions(&rig.seen), vec![0.0]);
}

#[test]
fn the_engine_sleeps_for_its_idle_timeout_unless_a_drain_hold_is_running() {
    let mut rig = rig("park", RATE);
    let idle = Duration::from_millis(100);
    assert_eq!(rig.controller.idle_park_timeout(idle), idle);
    // Without an output there is nothing in flight, so the hold is zero and the drain ends as soon as the ring is empty.
    assert_eq!(rig.controller.drain_hold(), Duration::ZERO);
    let mut guard = 0;
    while !rig.controller.clock.is_draining() {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        guard += 1;
        assert!(guard < 1_000, "the decoder should finish");
    }
    consume(&mut rig.consumer, RATE as usize);
    assert!(rig.controller.tick_playback());
}

#[test]
fn the_shutdown_snapshot_keeps_the_ui_fields_of_the_stored_session() {
    let rig = rig("snapshot_keeps_ui_fields", RATE);
    let stored = Session {
        track_rows_template: Some("$mi(title)$".to_string()),
        playing_template: Some("$mi(track)$".to_string()),
        auto_play_on_select: false,
        text_sizes: crate::text_size::TextSizes::default().with(crate::text_size::TextElement::FolderName, 21),
        ..Session::default()
    };
    let snapshot = rig.controller.session_over(stored);
    assert_eq!(
        snapshot.text_sizes.get(crate::text_size::TextElement::FolderName),
        21,
        "a shutdown save must not reset the text sizes",
    );
    assert!(!snapshot.auto_play_on_select, "a shutdown save must not turn Auto-play on select on");
    assert_eq!(snapshot.track_rows_template.as_deref(), Some("$mi(title)$"));
    assert_eq!(snapshot.playing_template.as_deref(), Some("$mi(track)$"));
    let off = rig.controller.session_over(Session::default());
    assert!(off.auto_play_on_select, "and it must not turn it off either");
}

#[cfg(target_os = "linux")]
#[test]
fn the_shutdown_snapshot_reads_the_stored_session_of_the_config_home() {
    let rig = rig("snapshot_reads_config_home", RATE);
    crate::session::use_disposable_config_home("snapshot_reads_config_home");
    let stored = Session {
        track_rows_template: Some("$mi(title)$".to_string()),
        auto_play_on_select: false,
        ..Session::default()
    };
    stored.save().expect("the stored session is written to the disposable home");
    let snapshot = rig.controller.current_session();
    assert!(!snapshot.auto_play_on_select, "the snapshot keeps the stored setting");
    assert_eq!(snapshot.track_rows_template.as_deref(), Some("$mi(title)$"));
}
