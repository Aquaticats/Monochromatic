// What:     Unit tests for `playback.rs`, pulled in by
//           `#[cfg(test)] #[path = "playback_tests.rs"] mod tests;` at
//           the bottom of `playback.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of playback.
// Why:      Keep the tests beside the code without inflating
//           `playback.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module into the
//           test scope. `super` means "one level up".
// Why:      Tests need `expand_paths`, `process_sample`, `PathBuf`, etc.
use super::*;
// What:     `use std::fs;` brings the filesystem module into scope.
// Why:      The test builds a real directory tree to walk.
use std::fs;
// What:     `use std::time::{SystemTime, UNIX_EPOCH};`. A clock reading and the
//           1970 epoch reference point.
// Why:      Build a unique temp-dir name so reruns do not collide.
use std::time::{SystemTime, UNIX_EPOCH};

// What:     `fn unique_temp_dir() -> PathBuf` test helper: make and return a
//           fresh throwaway directory under the system temp dir.
// Why:      Verify on a disposable fixture, never real state.
fn unique_temp_dir() -> PathBuf {
    // What:     Read nanoseconds elapsed since the Unix epoch.
    // Why:      A high-resolution component keeps the directory name unique.
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after Unix epoch")
        .as_nanos();
    // What:     `let dir = crate::test_scratch::scratch_root().join(format!("music-player-expand-{}-{}", std::process::id(), nanos));`.
    //           Build the path: system temp dir + a name carrying the process
    //           id and the nanosecond stamp. `std::process::id()` is this
    //           process's pid (a `u32`).
    // Why:      Unique per process and per call.
    let dir = crate::test_scratch::scratch_root().join(format!(
        "music-player-expand-{}-{}",
        std::process::id(),
        nanos
    ));
    // What:     Create the directory and any missing parents.
    // Why:      The fixture root must exist before we populate it.
    fs::create_dir_all(&dir).expect("fixture directory should be created");
    // What:     `dir`. Tail expression -> return the created path.
    // Why:      Hand the fixture root to the caller.
    return dir
}

// What:     `#[test]` marks the next function as a test case.
// Why:      `cargo test` discovers and runs it.
#[test]
fn expand_paths_walks_directories_recursively_and_sorts() {
    // What:     `let root = unique_temp_dir();`. Make the throwaway fixture.
    // Why:      A real tree to expand.
    let root = unique_temp_dir();

    // What:     Create a file named `b.flac` in the fixture root.
    // Why:      Two root files created out of alphabetical order prove the walk sorts them.
    fs::write(root.join("b.flac"), b"x").expect("root fixture should be written");
    // What:     create the second root file.
    // Why:      Out-of-order sibling.
    fs::write(root.join("a.flac"), b"x").expect("root fixture should be written");

    // What:     `let sub = root.join("sub");`. A subfolder path.
    // Why:      Prove the walk descends one level.
    let sub = root.join("sub");
    // What:     create the subfolder.
    // Why:      It must exist before adding files.
    fs::create_dir_all(&sub).expect("subdirectory fixture should be created");
    // What:     a file inside the subfolder.
    // Why:      Expected after the root files.
    fs::write(sub.join("c.flac"), b"x").expect("subdirectory fixture should be written");

    // What:     `let nested = sub.join("nested");`. A deeper folder.
    // Why:      Prove the walk descends more than one level.
    let nested = sub.join("nested");
    // What:     create the nested folder.
    // Why:      Needed before its file.
    fs::create_dir_all(&nested).expect("nested fixture directory should be created");
    // What:     a file two levels down.
    // Why:      Expected last.
    fs::write(nested.join("d.flac"), b"x").expect("nested fixture should be written");

    // What:     `let got = expand_paths(vec![root.clone()]);`. Expand the root
    //           folder. `vec![...]` wraps it in a one-element vector;
    //           `root.clone()` copies the path (we reuse `root` afterwards).
    // Why:      Exercise the recursive walk.
    let got = expand_paths(vec![root.clone()]);

    // What:     `let expected = vec![ ... ];`. The order the walk must produce:
    //           a folder's files (sorted) before its subfolders' files,
    //           depth-first.
    // Why:      Pin the deterministic ordering.
    let expected = vec![
        root.join("a.flac"),
        root.join("b.flac"),
        sub.join("c.flac"),
        nested.join("d.flac"),
    ];
    // What:     `assert_eq!(got, expected);`. Panics (failing the test) unless
    //           the two vectors are equal.
    // Why:      Confirm recursive collection and ordering.
    assert_eq!(got, expected);

    // What:     `let single = expand_paths(vec![root.join("a.flac")]);`. Expand a
    //           plain FILE path (not a directory).
    // Why:      A file should pass through unchanged.
    let single = expand_paths(vec![root.join("a.flac")]);
    // What:     `assert_eq!(single, vec![root.join("a.flac")]);`. One element, the
    //           file itself.
    // Why:      Files are not expanded.
    assert_eq!(single, vec![root.join("a.flac")]);

    // What:     create an AppleDouble sidecar file beside the direct-file case.
    // Why:      The direct sidecar test should exercise an existing file, not a missing path.
    fs::write(root.join("._a.flac"), b"x").expect("sidecar fixture should be written");
    // What:     `let direct_sidecar = expand_paths(vec![root.join("._a.flac")]);`. Expand a
    //           directly supplied AppleDouble sidecar path.
    // Why:      Direct file-open paths need the same `._` rejection as directory scans.
    let direct_sidecar = expand_paths(vec![root.join("._a.flac")]);
    // What:     `assert!(direct_sidecar.is_empty());` fails unless the expanded list has zero
    //           entries.
    // Why:      A direct sidecar must be ignored instead of passed to the decoder.
    assert!(direct_sidecar.is_empty());

    // What:     `let _ = fs::remove_dir_all(&root);`. Delete the throwaway tree;
    //           `let _ =` discards the result (cleanup is best-effort).
    // Why:      Leave no fixture behind.
    let _ = fs::remove_dir_all(&root);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Cover the audio-extension predicate directly.
#[test]
fn is_audio_file_matches_extensions_case_insensitively() {
    // What:     `assert!(is_audio_file(Path::new("a.flac")));`. `Path::new(s)`
    //           wraps a `&str` as a `&Path` with no allocation. A `.flac` file
    //           is audio.
    // Why:      Baseline positive.
    assert!(is_audio_file(Path::new("a.flac")));
    // What:     uppercase extension still matches.
    // Why:      The check is case-insensitive.
    assert!(is_audio_file(Path::new("A.FLAC")));
    // What:     a mixed-case extension on a nested path matches.
    // Why:      Confirm path components do not affect the extension test.
    assert!(is_audio_file(Path::new("/x/y/b.OpUs")));
    // What:     `assert!(!is_audio_file(Path::new("cover.jpg")));`. `!` negates;
    //           a non-audio extension is rejected.
    // Why:      Cover art must not enter the queue.
    assert!(!is_audio_file(Path::new("cover.jpg")));
    // What:     a dotfile has no extension, so it is rejected.
    // Why:      System files like `.DS_Store` must be skipped.
    assert!(!is_audio_file(Path::new(".DS_Store")));
    // What:     an extensionless name is rejected.
    // Why:      Nothing identifies it as audio.
    assert!(!is_audio_file(Path::new("noext")));
    // What:     an AppleDouble sidecar with an audio-looking extension is rejected.
    // Why:      `._song.mp3` is metadata for `song.mp3`, not a playable track.
    assert!(!is_audio_file(Path::new("._song.mp3")));
    // What:     the sidecar marker is checked on the final path component.
    // Why:      Nested sidecars must be ignored the same way as root-level sidecars.
    assert!(is_apple_double_sidecar(Path::new("/music/._Track.FLAC")));
    // What:     an ordinary track is not classified as a sidecar.
    // Why:      The helper must not reject real files beside AppleDouble metadata.
    assert!(!is_apple_double_sidecar(Path::new("/music/Track.FLAC")));
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Prove a folder scan keeps only audio files and skips junk.
#[test]
fn expand_paths_keeps_only_audio_files_and_skips_junk() {
    // What:     `let root = unique_temp_dir();`. A throwaway fixture directory.
    // Why:      A real folder to scan.
    let root = unique_temp_dir();

    // What:     create two audio files, deliberately out of alphabetical order.
    // Why:      Confirm they survive and come back sorted.
    fs::write(root.join("song.mp3"), b"x").expect("MP3 fixture should be written");
    fs::write(root.join("tune.flac"), b"x").expect("FLAC fixture should be written");
    // What:     create non-audio and hidden/system files that must be skipped.
    // Why:      These are exactly the kinds of files that leaked into the queue.
    fs::write(root.join("cover.jpg"), b"x").expect("cover fixture should be written");
    fs::write(root.join("playlist.m3u"), b"x").expect("playlist fixture should be written");
    fs::write(root.join(".DS_Store"), b"x").expect("Finder fixture should be written");
    fs::write(root.join(".nomedia"), b"x").expect("media marker fixture should be written");
    fs::write(root.join(".database_uuid"), b"x").expect("database fixture should be written");
    // What:     create an AppleDouble sidecar with an audio-looking extension.
    // Why:      This is the regression case: extension-only filtering would keep it.
    fs::write(root.join("._song.mp3"), b"x").expect("sidecar fixture should be written");

    // What:     `let got = expand_paths(vec![root.clone()]);`. Scan the folder.
    // Why:      Exercise the filtered walk.
    let got = expand_paths(vec![root.clone()]);
    // What:     `let expected = vec![root.join("song.mp3"), root.join("tune.flac")];`.
    //           Only the two audio files, sorted (`s` before `t`).
    // Why:      Pin the filtered, ordered result.
    let expected = vec![root.join("song.mp3"), root.join("tune.flac")];
    // What:     `assert_eq!(got, expected);`. Fail unless equal.
    // Why:      Confirm junk is dropped and audio kept in order.
    assert_eq!(got, expected);

    // What:     `let _ = fs::remove_dir_all(&root);`. Best-effort cleanup.
    // Why:      Leave no fixture behind.
    let _ = fs::remove_dir_all(&root);
}

// What:     `fn approx_eq(a: f32, b: f32) -> bool` test helper: true when two
//           floats sit within a tiny tolerance of each other.
// Why:      Float math is not bit-exact, and a direct `==` on floats is both
//           fragile and flagged by clippy; compare distances instead.
fn approx_eq(a: f32, b: f32) -> bool {
    // What:     `const TOLERANCE: f32 = 1e-6;`. Largest allowed difference;
    //           `1e-6` is scientific notation for 0.000001.
    // Why:      Far below any audible difference, loose enough for f32 rounding.
    const TOLERANCE: f32 = 1e-6;
    // What:     `(a - b).abs() < TOLERANCE`. Subtract, take the magnitude with
    //           `.abs()`, then compare with `<`. Tail expression -> return.
    // Why:      Distance-based equality avoids the float `==` trap.
    return (a - b).abs() < TOLERANCE
}

// What:     `#[test]` marks the next function as a test case.
// Why:      `cargo test` discovers and runs it.
#[test]
fn process_sample_applies_gain_then_clamps() {
    // What:     `assert!(approx_eq(process_sample(0.0, 1.0), 0.0));`. `assert!(cond)`
    //           panics (failing the test) when `cond` is false.
    // Why:      Silence in must stay silence out, whatever the gain.
    assert!(approx_eq(process_sample(0.0, 1.0), 0.0));

    // What:     unity gain passes a below-range sample through unchanged.
    // Why:      No clamp when within range.
    assert!(approx_eq(process_sample(0.5, 1.0), 0.5));

    // What:     the gain multiplies the sample (0.8 * 0.5 = 0.4).
    // Why:      Prove the combined gain is applied.
    assert!(approx_eq(process_sample(0.8, 0.5), 0.4));

    // What:     a result above 1.0 (1.5 * 1.0) is clamped to exactly 1.0.
    // Why:      The clamp backstops anything that would exceed full scale.
    assert!(approx_eq(process_sample(1.5, 1.0), 1.0));
    // What:     a result below -1.0 (-2.0 * 1.0) is clamped to exactly -1.0.
    // Why:      Clamp is symmetric.
    assert!(approx_eq(process_sample(-2.0, 1.0), -1.0));
}
