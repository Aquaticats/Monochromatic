// What:     Unit tests for `relpath.rs`, pulled in by
//           `#[cfg(test)] #[path = "relpath_tests.rs"] mod tests;` at
//           the bottom of `relpath.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of relpath.
// Why:      Keep the tests beside the code without inflating
//           `relpath.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module (this
//           file) into the test scope. `super` means "one level up".
// Why:      Tests need `relative_display_paths`, `PathBuf`, etc.
use super::*;

// What:     `fn paths(list: &[&str]) -> Vec<PathBuf>` test helper: turn string
//           literals into the owned `Vec<PathBuf>` the function takes.
// Why:      Tests read more clearly written with `&str` path literals.
fn paths(list: &[&str]) -> Vec<PathBuf> {
    // What:     `list.iter().map(PathBuf::from).collect()`. Borrow each `&&str`;
    //           `PathBuf::from` wraps it as an owned path (passing the function
    //           by name is the closure shorthand); `.collect()` gathers them.
    //           Tail expression -> return.
    // Why:      Build the owned input vector.
    return list.iter().map(PathBuf::from).collect()
}

// What:     `#[test]` marks the next function as a test case.
// Why:      `cargo test` discovers and runs it.
#[test]
fn empty_input_yields_empty() {
    // What:     `assert!(relative_display_paths(&[]).is_empty());`. `&[]` is an
    //           empty slice; `assert!(cond)` panics (fails the test) when false.
    // Why:      No tracks means no rows.
    assert!(relative_display_paths(&[]).is_empty());
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A single track has its whole directory chain stripped.
#[test]
fn single_track_keeps_only_filename() {
    // What:     `let got = relative_display_paths(&paths(&["/music/Artist/Album/01.flac"]));`.
    //           Relativize a one-element queue. `&paths(...)` lends the vector.
    // Why:      With one track every segment is "common", so the cap leaves only
    //           the filename.
    let got = relative_display_paths(&paths(&["/music/Artist/Album/01.flac"]));
    // What:     `assert_eq!(got, vec!["01.flac"]);`. `assert_eq!(a, b)` fails
    //           unless `a == b`; `vec![...]` builds the expected vector.
    // Why:      Only the filename remains.
    assert_eq!(got, vec!["01.flac"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Two albums under one root keep their artist/album folders.
#[test]
fn distinct_albums_keep_relative_folders() {
    // What:     relativize two tracks in different artist folders.
    // Why:      Their common root is `/music`; the rest is the relative path.
    let got = relative_display_paths(&paths(&[
        "/music/A/Alb/01.flac",
        "/music/B/Alb/01.flac",
    ]));
    // What:     `assert_eq!(got, vec!["A/Alb/01.flac", "B/Alb/01.flac"]);`.
    // Why:      Only `/music` is stripped; folders survive, order preserved.
    assert_eq!(got, vec!["A/Alb/01.flac", "B/Alb/01.flac"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      One opened album (single folder) reduces to bare filenames.
#[test]
fn single_folder_yields_bare_filenames() {
    // What:     relativize two tracks sharing the same album folder.
    // Why:      The cap stops stripping one short of the filename, leaving names.
    let got = relative_display_paths(&paths(&[
        "/m/A/Alb/01.flac",
        "/m/A/Alb/02.flac",
    ]));
    // What:     `assert_eq!(got, vec!["01.flac", "02.flac"]);`.
    // Why:      The whole `/m/A/Alb` chain is the common prefix.
    assert_eq!(got, vec!["01.flac", "02.flac"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      A loose root file and a foldered file share only the top folder.
#[test]
fn mixed_depth_strips_only_shared_top() {
    // What:     relativize a root-level file beside a nested one.
    // Why:      Common prefix is just `/m`; one stays a bare name, one keeps its
    //           folders, so pagination can route them to a letter vs a folder page.
    let got = relative_display_paths(&paths(&["/m/loose.flac", "/m/A/Alb/01.flac"]));
    // What:     `assert_eq!(got, vec!["loose.flac", "A/Alb/01.flac"]);`.
    // Why:      `loose.flac` has no folder; `A/Alb/01.flac` does.
    assert_eq!(got, vec!["loose.flac", "A/Alb/01.flac"]);
}

// What:     `#[test]` marks the next function as a test case.
// Why:      Identical paths must not strip away the filename.
#[test]
fn duplicate_paths_keep_filename() {
    // What:     relativize the same path twice.
    // Why:      Every segment is "common", so the cap must leave the filename.
    let got = relative_display_paths(&paths(&["/m/A/x.flac", "/m/A/x.flac"]));
    // What:     `assert_eq!(got, vec!["x.flac", "x.flac"]);`.
    // Why:      Filename survives even for duplicates.
    assert_eq!(got, vec!["x.flac", "x.flac"]);
}

// What:     `assert_scope_matches_pagination`, compares `page_scope_for_anchor` with the page that `paginate` gives each anchor.
// Why:      Playback scope and the visible page must agree for every track, whatever the folder shape.
fn assert_scope_matches_pagination(tracks: &[PathBuf]) {
    let names = relative_display_paths(tracks);
    let pages = crate::pagination::paginate(&names);
    for anchor in 0..tracks.len() {
        let expected: Vec<usize> = crate::pagination::page_of_index(&pages, anchor)
            .map(|page| return pages[page].entries.iter().map(|entry| return entry.index).collect())
            .expect("every track is on a page");
        assert_eq!(page_scope_for_anchor(tracks, anchor), Some(expected), "anchor {anchor} of {names:?}");
    }
    assert_eq!(page_scope_for_anchor(tracks, tracks.len()), None);
}

// What:     A queue with folders, subfolders, letter files, digit files, symbol files and case-different folders.
// Why:      Each shape decides a page rule: folder page, letter page, catch-all page.
#[test]
fn page_scope_for_anchor_matches_paginate_on_a_mixed_queue() {
    assert_scope_matches_pagination(&paths(&[
        "/lib/Abba/one.mp3",
        "/lib/Abba/Sub/two.mp3",
        "/lib/b.mp3",
        "/lib/B2.mp3",
        "/lib/1.mp3",
        "/lib/#x.mp3",
        "/lib/Zed/z.mp3",
        "/lib/abba/lower.mp3",
        "/lib/\u{00E9}clair.mp3",
    ]));
}

// What:     A queue that lives in one folder, so the folder itself is part of the shared prefix.
// Why:      The display names then have no folder part, and every track is a root-level name.
#[test]
fn page_scope_for_anchor_matches_paginate_in_a_single_folder() {
    assert_scope_matches_pagination(&paths(&["/lib/Only/a.mp3", "/lib/Only/b.mp3", "/lib/Only/1.mp3", "/lib/Only/A2.mp3"]));
}

// What:     A single track.
// Why:      The shared prefix never swallows the last component, so one track still has a name and a page.
#[test]
fn page_scope_for_anchor_handles_a_single_track() {
    assert_scope_matches_pagination(&paths(&["/lib/Only/a.mp3"]));
}

// What:     With the opened folder known, a library with one top-level folder keeps that folder in its names.
// Why:      LS1: the shared directory was stripped, so `Music/Artist/*.flac` looked like loose files and lost the folder.
#[test]
fn a_single_top_level_folder_keeps_its_name_relative_to_the_opened_folder() {
    let tracks = paths(&["/lib/Music/Artist/01.flac", "/lib/Music/Artist/Disc 2/02.flac"]);
    let got = relative_display_paths_in(&tracks, Some(Path::new("/lib/Music")));
    assert_eq!(got, vec!["Artist/01.flac", "Artist/Disc 2/02.flac"]);
    assert_eq!(relative_display_paths(&tracks), vec!["01.flac", "Disc 2/02.flac"], "without a root the shared directory is stripped");
}

// What:     Loose files of the opened folder have no folder part, next to folder tracks.
// Why:      They sit in the virtual letter folders.
#[test]
fn loose_files_have_bare_names_relative_to_the_opened_folder() {
    let tracks = paths(&["/lib/Music/loose.flac", "/lib/Music/A/x.flac"]);
    assert_eq!(relative_display_paths_in(&tracks, Some(Path::new("/lib/Music"))), vec!["loose.flac", "A/x.flac"]);
}

// What:     A track outside the opened folder, or an opened folder with no named component, falls back to the shared directory.
// Why:      The names must still be unambiguous when the root does not describe every track.
#[test]
fn a_root_that_does_not_hold_every_track_falls_back_to_the_shared_directory() {
    let tracks = paths(&["/lib/Music/A/x.flac", "/elsewhere/y.flac"]);
    assert_eq!(relative_display_paths_in(&tracks, Some(Path::new("/lib/Music"))), relative_display_paths(&tracks));
    let one = paths(&["/a/b/c.flac"]);
    assert_eq!(relative_display_paths_in(&one, Some(Path::new("/"))), relative_display_paths(&one));
}

// What:     A track equal to the root path itself is not under it, so the fallback applies.
// Why:      A zero-length relative name would be blank.
#[test]
fn a_track_that_is_the_root_itself_falls_back() {
    let tracks = paths(&["/lib/Music"]);
    assert_eq!(relative_display_paths_in(&tracks, Some(Path::new("/lib/Music"))), relative_display_paths(&tracks));
}

// What:     The playback scope of an anchor equals the page of the grouping made from root-relative names.
// Why:      The window groups the names and the engine plays the scope; the two must agree for every anchor.
#[test]
fn page_scope_with_a_root_matches_paginate_of_the_root_relative_names() {
    let tracks = paths(&[
        "/lib/Music/Artist/one.mp3",
        "/lib/Music/Artist/Disc 2/two.mp3",
        "/lib/Music/b.mp3",
        "/lib/Music/1.mp3",
        "/lib/Music/Zed/z.mp3",
    ]);
    let root = Some(Path::new("/lib/Music"));
    let names = relative_display_paths_in(&tracks, root);
    let pages = crate::pagination::paginate(&names);
    for anchor in 0..tracks.len() {
        let expected: Vec<usize> = crate::pagination::page_of_index(&pages, anchor)
            .map(|page| return pages[page].entries.iter().map(|entry| return entry.index).collect())
            .expect("every track is on a page");
        assert_eq!(page_scope_for_anchor_in(&tracks, anchor, root), Some(expected), "anchor {anchor} of {names:?}");
    }
}

// What:     A single-folder library gives a folder page for its folder and a letter page for a loose file.
// Why:      The finding's own example: one top-level folder plus loose tracks.
#[test]
fn a_folder_and_a_loose_file_scope_separately() {
    let tracks = paths(&["/lib/Music/Artist/one.mp3", "/lib/Music/two.mp3", "/lib/Music/tango.mp3"]);
    let root = Some(Path::new("/lib/Music"));
    assert_eq!(page_scope_for_anchor_in(&tracks, 0, root), Some(vec![0]));
    assert_eq!(page_scope_for_anchor_in(&tracks, 1, root), Some(vec![1, 2]), "both start with T: one virtual folder");
}
