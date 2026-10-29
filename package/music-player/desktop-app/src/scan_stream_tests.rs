// What:     Tests for `scan_stream.rs`, pulled in by `#[cfg(test)] #[path = "scan_stream_tests.rs"] mod tests;`.
//           Each case walks a throwaway tree under the system temp directory (removed afterwards), and checks that the
//           streamed walk returns what `expand_paths` returns, in the same order, and that the batches it reports
//           concatenate to exactly that list.
// Why:      A batch is indexed by its position in the walk, and that position must be the track's position in the
//           queue, so the order and the coverage of the batches are the whole contract.
use super::*;
use crate::playback::expand_paths;

// What:     `fixture`, builds a tree of audio files, non-audio files, nested folders and a symlinked file; returns its root.
// Why:      Every rule of the walk (sorted files then sorted subfolders, extension filter, symlink handling) appears.
fn fixture(files_per_folder: usize) -> PathBuf {
    let nanos: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| return elapsed.as_nanos());
    let root: PathBuf = crate::test_scratch::scratch_root().join(format!("music-player-scan-{}-{nanos}", std::process::id()));
    for folder in ["B Artist", "A Artist", "A Artist/Album 1", "A Artist/Album 2", "C Artist/Deep/Deeper"] {
        std::fs::create_dir_all(root.join(folder)).expect("the fixture folder is created");
        for number in 0..files_per_folder {
            std::fs::write(root.join(folder).join(format!("{number:03}.flac")), b"x").expect("the fixture file is written");
        }
        std::fs::write(root.join(folder).join("cover.jpg"), b"x").expect("the fixture file is written");
    }
    std::fs::write(root.join("loose.mp3"), b"x").expect("the fixture file is written");
    std::fs::write(root.join("._loose.mp3"), b"x").expect("the fixture file is written");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(root.join("loose.mp3"), root.join("linked.mp3")).expect("the symlink is created");
        std::os::unix::fs::symlink(root.join("A Artist"), root.join("linked folder")).expect("the symlink is created");
    }
    return root;
}

// What:     `slash_joined` writes a relative path with `/` between its parts, whatever the platform separator is.
// Why:      The expectations below are written with `/`; Windows would show `\\`.
fn slash_joined(relative: &Path) -> String {
    return relative.components().map(|part| return part.as_os_str().to_string_lossy().into_owned()).collect::<Vec<String>>().join("/");
}

// What:     The streamed walk equals `expand_paths` for the same root.
// Why:      `expand_paths` is what the queue was always built from.
#[test]
fn the_streamed_walk_matches_expand_paths() {
    let root: PathBuf = fixture(7);
    let streamed: Vec<PathBuf> = scan_streaming(&root, usize::MAX, &mut |_batch| {});
    assert_eq!(streamed, expand_paths(vec![root.clone()]));
    // The fixture makes the symlinks on Unix only (creating one on Windows needs a privilege the test cannot assume).
    if cfg!(unix) {
        assert!(streamed.iter().any(|path| return path.ends_with("linked.mp3")), "a symlinked audio file is kept");
    }
    assert!(!streamed.iter().any(|path| return path.ends_with("cover.jpg") || path.ends_with("._loose.mp3")));
    std::fs::remove_dir_all(&root).expect("the fixture is removed");
}

// What:     Batches concatenate to the whole walk, in order, and a small threshold gives several batches.
// Why:      Batch positions are queue positions.
#[test]
fn batches_concatenate_to_the_walk() {
    let root: PathBuf = fixture(10);
    let mut batches: Vec<Vec<PathBuf>> = Vec::new();
    let all: Vec<PathBuf> = scan_streaming(&root, 8, &mut |batch| batches.push(batch.to_vec()));
    let joined: Vec<PathBuf> = batches.iter().flatten().cloned().collect();
    assert_eq!(joined, all);
    assert!(batches.len() >= 3, "a threshold of 8 over 50 files gives several batches, got {}", batches.len());
    assert!(batches.iter().all(|batch| return !batch.is_empty()));
    std::fs::remove_dir_all(&root).expect("the fixture is removed");
}

// What:     With no threshold the remainder is still reported once at the end.
// Why:      The last run of tracks must reach the index even when the walk never filled a batch.
#[test]
fn the_remainder_is_reported_at_the_end() {
    let root: PathBuf = fixture(3);
    let mut batches: Vec<usize> = Vec::new();
    let all: Vec<PathBuf> = scan_streaming(&root, usize::MAX, &mut |batch| batches.push(batch.len()));
    assert_eq!(batches.iter().sum::<usize>(), all.len());
    std::fs::remove_dir_all(&root).expect("the fixture is removed");
}

// What:     A missing root gives an empty walk and no batch.
// Why:      The engine opens whatever folder the session remembers, and it may be gone.
#[test]
fn a_missing_root_gives_an_empty_walk() {
    let mut reported: usize = 0;
    let all: Vec<PathBuf> = scan_streaming(Path::new("/nonexistent/music-player/root"), 8, &mut |batch| reported += batch.len());
    assert!(all.is_empty());
    assert_eq!(reported, 0);
}

// What:     Batch names are relative to the root, joined with `/`.
// Why:      The Search index shows these names before the queue's display names exist.
#[test]
fn batch_names_are_root_relative() {
    let root: PathBuf = PathBuf::from("/lib");
    let names: Vec<String> = batch_names(&root, &[PathBuf::from("/lib/A/b/c.flac"), PathBuf::from("/lib/loose.mp3")]);
    assert_eq!(names, vec!["A/b/c.flac".to_string(), "loose.mp3".to_string()]);
}

// What:     Scanning a root into a controller's queue reports batches whose positions are the queue positions.
// Why:      D111: the Search index is built from these batches while the walk runs, and a result's position must be the
//           position the engine later plays; the final queue must be the concatenation of the batches.
#[test]
fn controller_scan_reports_batches_that_concatenate_to_the_queue() {
    use crate::command::Update;
    use crate::controller::Controller;
    use crate::peakcache::CacheHandle;
    use std::sync::{Arc, Mutex};

    let root: PathBuf = fixture(2500);
    let seen: Arc<Mutex<Vec<(usize, Vec<String>)>>> = Arc::new(Mutex::new(Vec::new()));
    let sink: Arc<Mutex<Vec<(usize, Vec<String>)>>> = Arc::clone(&seen);
    let mut controller: Controller = Controller::new(
        Box::new(move |update| {
            if let Update::ScanBatch { first_index, names } = update {
                sink.lock().expect("the sink is not poisoned").push((first_index, names));
            }
        }),
        None,
        CacheHandle::open_degraded(),
    );
    controller.scan_root_into_queue(root.clone());
    let batches: Vec<(usize, Vec<String>)> = seen.lock().expect("the sink is not poisoned").clone();
    assert!(batches.len() >= 2, "a threshold of five thousand over thirteen thousand files gives several batches");
    let mut expected_first: usize = 0;
    for (first_index, names) in &batches {
        assert_eq!(*first_index, expected_first, "each batch continues the previous one");
        expected_first += names.len();
    }
    let joined: Vec<String> = batches.into_iter().flat_map(|(_, names)| return names).collect();
    assert_eq!(joined, batch_names(&root, controller.queue.tracks()));
    std::fs::remove_dir_all(&root).expect("the fixture is removed");
}

// What:     A pause value means whole milliseconds, and anything else means no pause.
// Why:      A normal run must never be slowed, and a test or an end-to-end run needs a scan slow enough to type in (D111).
#[test]
fn the_scan_pause_is_whole_milliseconds_or_nothing() {
    assert_eq!(parse_pause(None), Duration::ZERO, "no pause unless asked for");
    assert_eq!(parse_pause(Some("25")), Duration::from_millis(25));
    assert_eq!(parse_pause(Some(" 40 ")), Duration::from_millis(40));
    assert_eq!(parse_pause(Some("not a number")), Duration::ZERO);
    assert_eq!(parse_pause(Some("-5")), Duration::ZERO);
    assert_eq!(pause_from_environment(), Duration::ZERO, "the test process sets no pause");
}

// What:     Files and folders of the walk come in natural order: `Loose 2` before `Loose 10`, `Album 2` before `Album 10`.
// Why:      LS5 of the second polish review: loose tracks sorted as text (`Loose 1`, `Loose 10`, `Loose 11`, `Loose 2`).
#[test]
fn the_walk_lists_numbered_names_in_natural_order() {
    let nanos: u128 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| return elapsed.as_nanos());
    let root: PathBuf = crate::test_scratch::scratch_root().join(format!("music-player-natural-{}-{nanos}", std::process::id()));
    for folder in ["Album 10", "Album 2"] {
        std::fs::create_dir_all(root.join(folder)).expect("the fixture folder is created");
        std::fs::write(root.join(folder).join("x.flac"), b"x").expect("the fixture file is written");
    }
    for name in ["Loose 11.wav", "Loose 2.wav", "Loose 1.wav", "Loose 10.wav"] {
        std::fs::write(root.join(name), b"x").expect("the fixture file is written");
    }
    let walked: Vec<PathBuf> = scan_streaming(&root, usize::MAX, &mut |_batch| {});
    let shown: Vec<String> = walked
        .iter()
        .map(|path| return slash_joined(path.strip_prefix(&root).expect("every track is under the root")))
        .collect();
    assert_eq!(
        shown,
        vec!["Loose 1.wav", "Loose 2.wav", "Loose 10.wav", "Loose 11.wav", "Album 2/x.flac", "Album 10/x.flac"]
    );
    std::fs::remove_dir_all(&root).expect("the fixture is removed");
}
