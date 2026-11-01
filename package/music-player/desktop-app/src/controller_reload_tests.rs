// What:     Controller tests for a selected file that is replaced on disk (D113), pulled in by `controller_reload.rs`.
//           They use a real WAV decoder and a hand-made ring buffer whose consumer plays the part of the output callback.
// Why:      A replaced playing file must keep its place, state, page and volume while its audio, peak and gain follow the new
//           file; a paused one keeps its place when the new file has it; a replacement that does not open yet changes
//           nothing and shows nothing.

use super::*;
use ringbuf::traits::{Consumer, Split};
use ringbuf::{HeapCons, HeapRb};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::peakcache::CacheHandle;

/// The sample rate of every test track (mono).
const RATE: u32 = 8_000;

/// What: `Seen` is what the tests keep of the updates the controller emits.
/// Why: assertions read short lists.
#[derive(Debug, Clone, PartialEq)]
enum Seen {
    Position(f64),
    NowPlaying(String),
    Duration(f64),
    Unplayable,
    Peak,
}

/// What: `write_wav` writes a mono 16-bit WAV of `seconds` of a sine at `amplitude`.
/// Why: a real file keeps the decoder, the chunking and the end-of-stream signal real; pitch and level tell files apart.
fn write_wav(path: &Path, seconds: u32, hz: f64, amplitude: f64) {
    let frames = RATE * seconds;
    let data_bytes = frames * 2;
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16_u32.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&RATE.to_le_bytes());
    bytes.extend_from_slice(&(RATE * 2).to_le_bytes());
    bytes.extend_from_slice(&2_u16.to_le_bytes());
    bytes.extend_from_slice(&16_u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_bytes.to_le_bytes());
    for n in 0..frames {
        let phase = 2.0 * std::f64::consts::PI * hz * f64::from(n) / f64::from(RATE);
        bytes.extend_from_slice(&((phase.sin() * amplitude) as i16).to_le_bytes());
    }
    std::fs::write(path, bytes).expect("the test track should be writable");
}

/// What: `Rig` is a controller with two tracks queued, the first loaded, and the consumer end of its ring.
/// Why: every test starts from the same controller and changes the first track's file.
struct Rig {
    controller: Controller,
    consumer: HeapCons<f32>,
    seen: Arc<Mutex<Vec<Seen>>>,
    directory: PathBuf,
}

/// Methods of `Rig`.
impl Rig {
    /// What: `new` builds the rig; `playing` decides whether the first track plays or is paused.
    /// Why: tests of both states share the setup.
    fn new(tag: &str, playing: bool) -> Rig {
        let directory = crate::test_scratch::scratch_root().join(format!("mp-reload-{}-{}", std::process::id(), tag));
        std::fs::create_dir_all(&directory).expect("the test directory should be creatable");
        write_wav(&directory.join("t1.wav"), 3, 440.7, 8_000.0);
        write_wav(&directory.join("t2.wav"), 3, 880.3, 12_000.0);
        let seen: Arc<Mutex<Vec<Seen>>> = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let mut controller = Controller::new(
            Box::new(move |update| {
                let mut list = sink.lock().expect("the update list should not be poisoned");
                match update {
                    Update::Position(secs) => list.push(Seen::Position(secs)),
                    Update::NowPlaying { name, .. } => list.push(Seen::NowPlaying(name)),
                    Update::Duration(secs) => list.push(Seen::Duration(secs)),
                    Update::Unplayable(_) => list.push(Seen::Unplayable),
                    Update::Measures(measures) if measures.iter().any(|(_, measure)| return measure.peak_dbtp.is_some()) => list.push(Seen::Peak),
                    _ => {}
                }
            }),
            None,
            CacheHandle::open_degraded(),
        );
        controller.queue.set_tracks(vec![directory.join("t1.wav"), directory.join("t2.wav")]);
        controller.queue.play_index(0);
        controller.playing = playing;
        assert!(controller.load_current(), "the first test track should load");
        let (producer, consumer) = HeapRb::<f32>::new(RATE as usize).split();
        controller.producer = Some(producer);
        return Rig { controller, consumer, seen, directory };
    }

    /// What: `play_to` pumps and consumes until the audible position is `secs` and the ring is empty.
    /// Why: the audible position is what the output has consumed, so the test consumes the way the output does.
    fn play_to(&mut self, secs: f64) {
        let wanted = (secs * f64::from(RATE)) as usize;
        let mut consumed = 0;
        while consumed < wanted {
            pump_all(&mut self.controller);
            let taken = consume(&mut self.consumer, (wanted - consumed).min(2_000));
            consumed += taken.len();
        }
        assert!((self.controller.audible_secs() - secs).abs() < 1e-6, "the rig is at {secs} s, not {}", self.controller.audible_secs());
    }

    /// What: `flush` empties the ring, as the output does when the stream is rebuilt for the replaced file.
    /// Why: the tests have no output, and `install_source_silently` relies on `reconfigure` for the flush.
    fn flush(&mut self) {
        consume(&mut self.consumer, usize::MAX);
    }

    /// What: `replace` writes a new file over the first track.
    /// Why: the change on disk.
    fn replace(&self, seconds: u32, hz: f64, amplitude: f64) {
        write_wav(&self.directory.join("t1.wav"), seconds, hz, amplitude);
    }

    /// What: `updates` is a copy of what the controller emitted so far.
    /// Why: assertions read it.
    fn updates(&self) -> Vec<Seen> {
        return self.seen.lock().expect("the update list should not be poisoned").clone();
    }
}

/// Removes the folder when the test ends.
impl Drop for Rig {
    /// What: `drop` removes the folder.
    /// Why: tests leave no files behind.
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.directory).ok();
    }
}

/// What: `pump_all` pumps until the controller has nothing to push.
/// Why: `pump_audio` returns false when the ring is full or the track is draining.
fn pump_all(controller: &mut Controller) {
    let mut guard = 0;
    while controller.pump_audio() {
        guard += 1;
        assert!(guard < 10_000, "pumping should settle");
    }
}

/// What: `consume` pops up to `samples` as the output would.
/// Why: the consumer's read index is the only thing the audible position depends on.
fn consume(consumer: &mut HeapCons<f32>, samples: usize) -> Vec<f32> {
    let mut taken = Vec::new();
    while taken.len() < samples {
        match consumer.try_pop() {
            Some(sample) => taken.push(sample),
            None => break,
        }
    }
    return taken;
}

/// What: `decode_all` decodes every sample of a file straight from the source.
/// Why: the reference the played samples are compared with.
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

#[test]
fn resume_position_keeps_the_place_clamps_a_playing_track_and_restarts_a_paused_one() {
    assert!((resume_position(1.5, 5.0, true) - 1.5).abs() < 1e-12);
    assert!((resume_position(1.5, 5.0, false) - 1.5).abs() < 1e-12);
    assert!((resume_position(4.0, 2.5, true) - 2.5).abs() < 1e-12, "a playing track ends where the shorter file ends");
    assert!(resume_position(4.0, 2.5, false).abs() < 1e-12, "a paused track whose place is not in the new file starts again");
    assert!((resume_position(4.0, 0.0, true) - 4.0).abs() < 1e-12, "an unknown length clamps nothing");
    assert!(resume_position(-1.0, 3.0, true).abs() < 1e-12);
}

#[test]
fn a_playing_file_that_is_replaced_goes_on_at_the_same_place_with_the_new_audio_and_says_nothing() {
    let mut rig = Rig::new("playing", true);
    rig.play_to(1.0);
    let before = rig.updates().len();
    let old_reference = decode_all(&rig.directory.join("t1.wav"));
    rig.replace(5, 220.3, 20_000.0);
    rig.controller.reload_current_when_its_file_changed();
    rig.flush();
    assert!(rig.controller.playing, "it keeps playing");
    assert!((rig.controller.audible_secs() - 1.0).abs() < 1e-6, "at the same place");
    let announced = rig.updates()[before..].to_vec();
    assert!(announced.contains(&Seen::Duration(5.0)), "the seek bar takes the new length: {announced:?}");
    assert!(!announced.iter().any(|seen| return matches!(seen, Seen::NowPlaying(_))), "the track is not announced again, so the page does not move");
    assert!(!announced.contains(&Seen::Unplayable), "no toast");
    assert!(!announced.contains(&Seen::Position(0.0)), "the seek bar does not flash to zero");
    let mut waited = 0;
    while rig.controller.pending_peak.is_some() {
        rig.controller.poll_pending_peak();
        std::thread::sleep(std::time::Duration::from_millis(10));
        waited += 1;
        assert!(waited < 500, "the new file's gain should arrive");
    }
    pump_all(&mut rig.controller);
    let played = consume(&mut rig.consumer, 2_000);
    let reference = decode_all(&rig.directory.join("t1.wav"));
    let gain = rig.controller.track_gain;
    // The decoder's seek lands on a packet boundary (the same granularity as the seek bar), so the audio is found in
    // the new file within one packet of the place, and it matches that file sample for sample.
    let error_at = |start: usize| -> f32 { return (0..200).map(|i| return (played[i] - reference[start + i] * gain).abs()).sum() };
    let best = (RATE as usize - 2_048..RATE as usize + 2_048).min_by(|a, b| return error_at(*a).partial_cmp(&error_at(*b)).expect("comparable")).expect("a range");
    assert!(error_at(best) < 1e-2, "the played samples are the new file's samples, found at frame {best}");
    let old_error: f32 = (0..200).map(|i| return (played[i] - old_reference[best + i] * gain).abs()).sum();
    assert!(old_error > 1.0, "and they are not the old file's");
}

#[test]
fn a_playing_file_replaced_by_a_shorter_one_ends_where_the_new_file_ends_and_the_next_track_follows() {
    let mut rig = Rig::new("shorter", true);
    rig.play_to(2.0);
    rig.replace(1, 330.0, 8_000.0);
    rig.controller.reload_current_when_its_file_changed();
    rig.flush();
    assert!((rig.controller.audible_secs() - 1.0).abs() < 1e-6, "clamped to the new length");
    assert!(rig.controller.playing);
    assert!(rig.updates().contains(&Seen::Duration(1.0)));
    let mut guard = 0;
    while rig.controller.queue.current_index() != Some(1) {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        rig.controller.tick_playback();
        guard += 1;
        assert!(guard < 1_000, "the next track should follow");
    }
    assert!(rig.updates().contains(&Seen::NowPlaying("t2.wav".to_string())));
}

#[test]
fn a_paused_selected_file_that_is_replaced_keeps_its_place_when_the_new_file_has_it() {
    let mut rig = Rig::new("paused-keeps", false);
    rig.controller.seek(1.5);
    rig.flush();
    rig.replace(5, 220.3, 20_000.0);
    rig.controller.reload_current_when_its_file_changed();
    rig.flush();
    assert!(!rig.controller.playing, "it stays paused");
    assert!((rig.controller.audible_secs() - 1.5).abs() < 1e-6);
    assert!(rig.updates().contains(&Seen::Duration(5.0)));
}

#[test]
fn a_paused_selected_file_that_is_replaced_by_one_without_that_place_starts_again() {
    let mut rig = Rig::new("paused-restarts", false);
    rig.controller.seek(2.5);
    rig.flush();
    rig.replace(1, 330.0, 8_000.0);
    rig.controller.reload_current_when_its_file_changed();
    rig.flush();
    assert!(!rig.controller.playing);
    assert!(rig.controller.audible_secs().abs() < 1e-6, "the place is not in the new file");
    assert_eq!(rig.updates().iter().rev().find_map(|seen| return if let Seen::Position(secs) = seen { Some(*secs) } else { None }), Some(0.0), "and the seek bar says so");
}

#[test]
fn a_replacement_that_does_not_open_yet_changes_nothing_and_is_tried_again_when_it_does() {
    let mut rig = Rig::new("unfinished", true);
    rig.play_to(1.0);
    let key_before = rig.controller.loaded_file.clone().expect("loaded");
    let before = rig.updates().len();
    std::fs::write(rig.directory.join("t1.wav"), b"RIFF half a file").expect("a partial file");
    rig.controller.reload_current_when_its_file_changed();
    assert_eq!(rig.controller.loaded_file, Some(key_before), "the old decoder stays");
    assert!(rig.controller.playing);
    assert_eq!(rig.updates()[before..], [] as [Seen; 0], "nothing is shown: no toast, no skip to the next track");
    assert_eq!(rig.controller.queue.current_index(), Some(0));
    rig.replace(4, 220.3, 20_000.0);
    rig.controller.reload_current_when_its_file_changed();
    assert!(rig.updates().contains(&Seen::Duration(4.0)), "the finished file is loaded at the next event");
}

#[test]
fn an_unchanged_file_is_not_loaded_again() {
    let mut rig = Rig::new("unchanged", true);
    rig.play_to(1.0);
    let before = rig.updates().len();
    rig.controller.reload_current_when_its_file_changed();
    assert_eq!(rig.updates().len(), before);
    assert!((rig.controller.audible_secs() - 1.0).abs() < 1e-6);
}

#[test]
fn the_peak_and_the_gain_follow_the_new_file() {
    let mut rig = Rig::new("gain", true);
    rig.play_to(0.5);
    let mut waited = 0;
    while !rig.updates().contains(&Seen::Peak) {
        rig.controller.poll_pending_peak();
        std::thread::sleep(std::time::Duration::from_millis(20));
        waited += 1;
        assert!(waited < 500, "the first file's peak should arrive");
    }
    let quiet_gain = rig.controller.track_gain;
    let peaks_before = rig.updates().iter().filter(|seen| return **seen == Seen::Peak).count();
    rig.replace(3, 440.7, 32_700.0);
    rig.controller.reload_current_when_its_file_changed();
    rig.flush();
    let mut waited_for_new = 0;
    while rig.updates().iter().filter(|seen| return **seen == Seen::Peak).count() == peaks_before {
        rig.controller.poll_pending_peak();
        std::thread::sleep(std::time::Duration::from_millis(20));
        waited_for_new += 1;
        assert!(waited_for_new < 500, "the new file's peak should arrive");
    }
    assert!(rig.controller.track_gain < quiet_gain, "the louder file gets a lower gain: {} then {}", quiet_gain, rig.controller.track_gain);
}

#[test]
fn a_file_replaced_while_the_next_track_is_staged_plays_on_from_its_place_and_stages_it_again() {
    let mut rig = Rig::new("staged", true);
    let mut guard = 0;
    while rig.controller.staged.is_none() {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 1_000);
        guard += 1;
        assert!(guard < 1_000, "the next track should be staged near the end");
    }
    let place = rig.controller.audible_secs();
    assert!(place > 1.0, "the first track has been playing for a while: {place}");
    rig.replace(6, 220.3, 20_000.0);
    rig.controller.reload_current_when_its_file_changed();
    rig.flush();
    assert!(rig.controller.staged.is_none(), "the early-opened next track is dropped");
    assert_eq!(rig.controller.spec.map(|spec| return spec.duration_secs), Some(6.0), "the format of the new file, not the old file's, is the audible one");
    assert_eq!(rig.controller.queue.current_index(), Some(0));
    assert!((rig.controller.audible_secs() - place).abs() < 1e-6);
    assert!(!rig.updates().contains(&Seen::NowPlaying("t2.wav".to_string())), "the next track has not been announced");
    let mut follow_guard = 0;
    while rig.controller.queue.current_index() != Some(1) {
        pump_all(&mut rig.controller);
        consume(&mut rig.consumer, 2_000);
        rig.controller.tick_playback();
        follow_guard += 1;
        assert!(follow_guard < 2_000, "the next track should follow the longer file");
    }
    assert_eq!(rig.updates().iter().filter(|seen| return **seen == Seen::NowPlaying("t2.wav".to_string())).count(), 1);
}
