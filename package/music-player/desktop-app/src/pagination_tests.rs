// What:     Unit tests for `pagination.rs`, pulled in by
//           `#[cfg(test)] #[path = "pagination_tests.rs"] mod tests;` at the bottom of
//           `pagination.rs`. Compiles only under `cargo nextest run` / `cargo test`;
//           reaches the module items (including private ones) via `use super::*` because
//           this file is the `tests` CHILD of pagination.
// Why:      Keep the tests beside the code without inflating `pagination.rs` or its
//           max-lines budget (sibling `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;` imports everything from the parent module (this file) into
//           the test scope. `super` means "one level up".
// Why:      Tests need `paginate`, `page_of_index`, `Page`, `PageEntry`.
//
// In TS you'd write (pseudocode):
// ```ts
// import * as parent from "./pagination";
// ```
use super::*;

// What:     `fn names(list: &[&str]) -> Vec<String>` test helper: turn a slice of
//           string literals into the owned `Vec<String>` `paginate` takes.
// Why:      Tests are written with `&str` literals; `paginate` wants `String`s.
//
// In TS you'd write (pseudocode):
// ```ts
// function names(list: string[]): string[] { return [...list]; }
// ```
fn names(list: &[&str]) -> Vec<String> {
    // What:     `list.iter().map(|s| s.to_string()).collect()`. Borrow each `&&str`,
    //           `.to_string()` copies it into an owned `String`, `.collect()` gathers
    //           them. Tail expression -> return.
    // Why:      Build the owned input vector.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return list.map((s) => s);
    // ```
    return list.iter().map(|s| return s.to_string()).collect();
}

// What:     `#[test]` marks the next function as a test case.
// Why:      `cargo test` discovers and runs it.
//
// In TS you'd write (pseudocode):
// ```ts
// test("empty input yields no pages", () => { ... });
// ```
#[test]
fn empty_input_yields_no_pages() {
    // What:     `assert!(paginate(&[]).is_empty());`. `&[]` is an empty slice;
    //           `.is_empty()` is true when there are no pages. `assert!(cond)` panics
    //           (failing the test) when `cond` is false.
    // Why:      No names means no pages, not a single empty page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(paginate([]).length).toBe(0);
    // ```
    assert!(paginate(&[]).is_empty());
}

// What:     `#[test] fn same_top_folder_collapses_one_level()`. A test case.
// Why:      Prove paging uses one folder level only.
//
// In TS you'd write (pseudocode):
// ```ts
// test("same top folder collapses one level", () => { ... });
// ```
#[test]
fn same_top_folder_collapses_one_level() {
    // What:     `let pages = paginate(&names(&["Artist/Album1/01.flac", "Artist/Album2/01.flac"]));`.
    //           Two tracks in DIFFERENT deeper subfolders but the same top folder.
    // Why:      Prove paging uses one level only: both collapse onto `Artist`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["Artist/Album1/01.flac", "Artist/Album2/01.flac"]);
    // ```
    let pages = paginate(&names(&["Artist/Album1/01.flac", "Artist/Album2/01.flac"]));
    // What:     `assert_eq!(pages.len(), 1);`. One page expected.
    // Why:      Both share the `Artist` top-level folder despite differing albums.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages.length).toBe(1);
    // ```
    assert_eq!(pages.len(), 1);
    // What:     `assert_eq!(pages[0].label, "Artist");`. The label is the TOP folder
    //           only, not `Artist/Album1`.
    // Why:      Pages are limited to one folder level.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages[0].label).toBe("Artist");
    // ```
    assert_eq!(pages[0].label, "Artist");
    // What:     `let indices: Vec<usize> = pages[0].entries.iter().map(|e| e.index).collect();`.
    //           Pull just the load-order indices out of the page's entries.
    // Why:      Confirm the original positions survived grouping, in order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const indices = pages[0].entries.map((e) => e.index);
    // ```
    let indices: Vec<usize> = pages[0].entries.iter().map(|e| return e.index).collect();
    // What:     `assert_eq!(indices, vec![0, 1]);`. Indices preserved.
    // Why:      Clicking a filtered row must map back to the right queue index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(indices).toEqual([0, 1]);
    // ```
    assert_eq!(indices, vec![0, 1]);
}

// What:     `#[test] fn distinct_folders_sorted_by_path()`. A test case.
// Why:      Prove separate pages, sorted by folder path.
//
// In TS you'd write (pseudocode):
// ```ts
// test("distinct folders sorted by path", () => { ... });
// ```
#[test]
fn distinct_folders_sorted_by_path() {
    // What:     `let pages = paginate(&names(&["Pop/b.flac", "Jazz/a.flac"]));`. Two
    //           tracks in different folders, given out of sorted order.
    // Why:      Prove separate pages, sorted by folder path.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["Pop/b.flac", "Jazz/a.flac"]);
    // ```
    let pages = paginate(&names(&["Pop/b.flac", "Jazz/a.flac"]));
    // What:     `let labels: Vec<String> = pages.iter().map(|p| p.label.clone()).collect();`.
    //           Collect the labels in page order (`.clone()` copies each owned label).
    // Why:      Compare the sorted sequence at once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const labels = pages.map((p) => p.label);
    // ```
    let labels: Vec<String> = pages.iter().map(|p| return p.label.clone()).collect();
    // What:     `assert_eq!(labels, vec!["Jazz", "Pop"]);`. `Jazz` sorts before `Pop`.
    // Why:      Folder pages order by path regardless of input order.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(labels).toEqual(["Jazz", "Pop"]);
    // ```
    assert_eq!(labels, vec!["Jazz", "Pop"]);
    // What:     `assert_eq!(pages[0].entries[0].index, 1);`. The `Jazz` page holds the
    //           second input (`Jazz/a.flac`, load index 1).
    // Why:      Sorting reorders pages but each entry keeps its real index.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages[0].entries[0].index).toBe(1);
    // ```
    assert_eq!(pages[0].entries[0].index, 1);
}

// What:     `#[test] fn folder_pages_sort_case_insensitively()`. A test case.
// Why:      Prove folder pages interleave case-insensitively (the reported bug fix).
//
// In TS you'd write (pseudocode):
// ```ts
// test("folder pages sort case insensitively", () => { ... });
// ```
#[test]
fn folder_pages_sort_case_insensitively() {
    // What:     `let pages = paginate(&names(&["Zedd/a.flac", "daniwellP/b.flac", "Reol/c.flac", "r-906/d.flac"]));`.
    //           Four folder tracks whose top folders mix upper- and lowercase first
    //           letters, given in an order a raw codepoint sort would mangle (`Zedd`
    //           before `daniwellP`, `r-906` after `Z`).
    // Why:      Reproduce the reported ordering and prove the fix interleaves them
    //           case-insensitively.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["Zedd/a.flac", "daniwellP/b.flac", "Reol/c.flac", "r-906/d.flac"]);
    // ```
    let pages = paginate(&names(&[
        "Zedd/a.flac",
        "daniwellP/b.flac",
        "Reol/c.flac",
        "r-906/d.flac",
    ]));
    // What:     `let labels: Vec<String> = pages.iter().map(|p| p.label.clone()).collect();`.
    //           Collect the labels in page order.
    // Why:      Compare the full ordered sequence at once.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const labels = pages.map((p) => p.label);
    // ```
    let labels: Vec<String> = pages.iter().map(|p| return p.label.clone()).collect();
    // What:     `assert_eq!(labels, vec!["daniwellP", "r-906", "Reol", "Zedd"]);`.
    //           Case-folded order: `DANIWELLP` < `R-906` < `REOL` < `ZEDD` (the `-` at
    //           0x2D sorts before `E`, so `r-906` precedes `Reol`). The displayed
    //           labels keep their ORIGINAL casing.
    // Why:      Confirm lowercase-led folders no longer trail after `Z`, and the label
    //           text is not uppercased for display.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(labels).toEqual(["daniwellP", "r-906", "Reol", "Zedd"]);
    // ```
    assert_eq!(labels, vec!["daniwellP", "r-906", "Reol", "Zedd"]);
}

// What:     `#[test] fn case_variant_folders_stay_distinct_pages()`. A test case.
// Why:      Prove case-folding orders pages but does not MERGE distinct folders.
//
// In TS you'd write (pseudocode):
// ```ts
// test("case variant folders stay distinct pages", () => { ... });
// ```
#[test]
fn case_variant_folders_stay_distinct_pages() {
    // What:     `let pages = paginate(&names(&["REOL/a.flac", "Reol/b.flac"]));`. Two
    //           folders whose names differ only in case (`Reol` vs `REOL`).
    // Why:      Case-folding orders pages but must not MERGE genuinely distinct folders;
    //           the original label rides in the sort key as a tiebreaker.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["REOL/a.flac", "Reol/b.flac"]);
    // ```
    let pages = paginate(&names(&["REOL/a.flac", "Reol/b.flac"]));
    // What:     `assert_eq!(pages.len(), 2);`. Two separate pages, not one merged.
    // Why:      `REOL` and `Reol` are different directories on disk.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages.length).toBe(2);
    // ```
    assert_eq!(pages.len(), 2);
    // What:     `let labels: Vec<String> = pages.iter().map(|p| p.label.clone()).collect();`.
    //           Collect the labels; both case variants survive, ordered by the original
    //           label after the shared case-folded key (`REOL` < `Reol` because
    //           uppercase letters precede lowercase in codepoint order).
    // Why:      Deterministic, stable order for equal-fold labels.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const labels = pages.map((p) => p.label);
    // ```
    let labels: Vec<String> = pages.iter().map(|p| return p.label.clone()).collect();
    // What:     `assert_eq!(labels, vec!["REOL", "Reol"]);`. Both case variants present,
    //           uppercase-led first.
    // Why:      Confirm the tiebreaker keeps them separate and ordered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(labels).toEqual(["REOL", "Reol"]);
    // ```
    assert_eq!(labels, vec!["REOL", "Reol"]);
}

// What:     `#[test] fn root_letters_merge_case_insensitively()`. A test case.
// Why:      Prove root-level names merge onto one letter page case-insensitively.
//
// In TS you'd write (pseudocode):
// ```ts
// test("root letters merge case insensitively", () => { ... });
// ```
#[test]
fn root_letters_merge_case_insensitively() {
    // What:     `let pages = paginate(&names(&["apple.flac", "Apricot.flac", "AVOCADO.flac"]));`.
    //           Three root-level (no `/`) names starting with the same letter in
    //           different cases.
    // Why:      Prove case-folding merges them onto one `A` letter page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["apple.flac", "Apricot.flac", "AVOCADO.flac"]);
    // ```
    let pages = paginate(&names(&["apple.flac", "Apricot.flac", "AVOCADO.flac"]));
    // What:     `assert_eq!(pages.len(), 1);`. One page expected.
    // Why:      All three share the `A` bucket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages.length).toBe(1);
    // ```
    assert_eq!(pages.len(), 1);
    // What:     `assert_eq!(pages[0].label, "A");`. The bucket label.
    // Why:      Letter pages caption with the uppercased letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages[0].label).toBe("A");
    // ```
    assert_eq!(pages[0].label, "A");
    // What:     `let indices: Vec<usize> = pages[0].entries.iter().map(|e| e.index).collect();`.
    //           Indices preserved in load order.
    // Why:      Clicking maps back correctly.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages[0].entries.map((e) => e.index)).toEqual([0, 1, 2]);
    // ```
    let indices: Vec<usize> = pages[0].entries.iter().map(|e| return e.index).collect();
    // What:     `assert_eq!(indices, vec![0, 1, 2]);`. All three indices in order.
    // Why:      Nothing dropped or reordered.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(indices).toEqual([0, 1, 2]);
    // ```
    assert_eq!(indices, vec![0, 1, 2]);
}

// What:     `#[test] fn non_letter_root_names_go_to_catch_all()`. A test case.
// Why:      Prove digits/CJK/symbols/accented letters land on the `#` page.
//
// In TS you'd write (pseudocode):
// ```ts
// test("non letter root names go to catch all", () => { ... });
// ```
#[test]
fn non_letter_root_names_go_to_catch_all() {
    // What:     `let pages = paginate(&names(&["1 song.flac", "初音.flac", "#tag.flac", "élan.flac"]));`.
    //           Four root-level names whose first char is a digit, CJK, symbol, and an
    //           accented (non-English) letter respectively.
    // Why:      Prove all four land on the single `#` catch-all page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["1 song.flac", "初音.flac", "#tag.flac", "élan.flac"]);
    // ```
    let pages = paginate(&names(&[
        "1 song.flac",
        "初音.flac",
        "#tag.flac",
        "élan.flac",
    ]));
    // What:     `assert_eq!(pages.len(), 1);`. One page expected.
    // Why:      None of the four is a plain A-Z letter.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages.length).toBe(1);
    // ```
    assert_eq!(pages.len(), 1);
    // What:     `assert_eq!(pages[0].label, "#");`. The catch-all caption.
    // Why:      Confirm the catch-all collects them all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages[0].label).toBe("#");
    // ```
    assert_eq!(pages[0].label, "#");
    // What:     `let indices: Vec<usize> = pages[0].entries.iter().map(|e| e.index).collect();`.
    //           All four indices present, in order.
    // Why:      Nothing dropped.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pages[0].entries.map((e) => e.index)).toEqual([0, 1, 2, 3]);
    // ```
    let indices: Vec<usize> = pages[0].entries.iter().map(|e| return e.index).collect();
    // What:     `assert_eq!(indices, vec![0, 1, 2, 3]);`. All four, in order.
    // Why:      Confirm none of the non-letter names was lost.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(indices).toEqual([0, 1, 2, 3]);
    // ```
    assert_eq!(indices, vec![0, 1, 2, 3]);
}

// What:     `#[test] fn folders_precede_letters_precede_catch_all()`. A test case.
// Why:      Prove the sort GROUP, not the label text, orders the three axes.
//
// In TS you'd write (pseudocode):
// ```ts
// test("folders precede letters precede catch all", () => { ... });
// ```
#[test]
fn folders_precede_letters_precede_catch_all() {
    // What:     `let pages = paginate(&names(&["Zed/x.flac", "apple.flac", "1.flac"]));`.
    //           One folder track, one letter track, one catch-all track, given so that
    //           the folder's label (`Zed`) sorts AFTER the letter's (`A`).
    // Why:      Prove the sort group, not the label text, orders the axes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["Zed/x.flac", "apple.flac", "1.flac"]);
    // ```
    let pages = paginate(&names(&["Zed/x.flac", "apple.flac", "1.flac"]));
    // What:     `let labels: Vec<String> = pages.iter().map(|p| p.label.clone()).collect();`.
    //           Collect the labels in page order.
    // Why:      Compare the full ordered sequence.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const labels = pages.map((p) => p.label);
    // ```
    let labels: Vec<String> = pages.iter().map(|p| return p.label.clone()).collect();
    // What:     `assert_eq!(labels, vec!["Zed", "A", "#"]);`. Folder first (group 0),
    //           then the `A` letter page (group 1), then `#` (group 2), even though
    //           `Zed` > `A` as text.
    // Why:      Confirm group ordering dominates.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(labels).toEqual(["Zed", "A", "#"]);
    // ```
    assert_eq!(labels, vec!["Zed", "A", "#"]);
}

// What:     `#[test] fn page_of_index_finds_and_misses()`. A test case.
// Why:      Cover both the hit and miss branches of `page_of_index`.
//
// In TS you'd write (pseudocode):
// ```ts
// test("page of index finds and misses", () => { ... });
// ```
#[test]
fn page_of_index_finds_and_misses() {
    // What:     `let pages = paginate(&names(&["A/x.flac", "B/y.flac", "c.flac"]));`.
    //           Two folder tracks and one root-level letter track.
    // Why:      A fixture spanning both axes to look indices up in.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const pages = paginate(["A/x.flac", "B/y.flac", "c.flac"]);
    // ```
    let pages = paginate(&names(&["A/x.flac", "B/y.flac", "c.flac"]));
    // What:     `assert_eq!(page_of_index(&pages, 2), Some(2));`. Load index 2 (`c.flac`)
    //           lives on the third page (the `C` letter page, after the two folder
    //           pages). `Some(2)` wraps the found page position.
    // Why:      Auto-follow must locate the playing track's page.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pageOfIndex(pages, 2)).toBe(2);
    // ```
    assert_eq!(page_of_index(&pages, 2), Some(2));
    // What:     `assert_eq!(page_of_index(&pages, 99), None);`. An out-of-range index
    //           belongs to no page; `None` is the empty `Option`.
    // Why:      Robustness: a missing index yields no page, not a panic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(pageOfIndex(pages, 99)).toBe(null);
    // ```
    assert_eq!(page_of_index(&pages, 99), None);
}

// What:     `#[test] fn row_display_trims_only_folder_tab_prefix()`. A test case for the
//           `row_display` helper.
// Why:      Prove folder tabs drop the `<label>/` prefix while letter / `#` tabs keep the
//           whole name (the reported display change).
//
// In TS you'd write (pseudocode):
// ```ts
// test("row display trims only folder tab prefix", () => { ... });
// ```
#[test]
fn row_display_trims_only_folder_tab_prefix() {
    // What:     `assert_eq!(row_display("Ado", "Ado/B/C.opus"), "B/C.opus");`. A folder
    //           page (label `Ado`, name nested under `Ado/`): the `Ado/` prefix is
    //           stripped.
    // Why:      The `Ado` tab already names the folder; the row shows only the path below
    //           it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(rowDisplay("Ado", "Ado/B/C.opus")).toBe("B/C.opus");
    // ```
    assert_eq!(row_display("Ado", "Ado/B/C.opus"), "B/C.opus");
    // What:     `assert_eq!(row_display("A", "Apple.flac"), "Apple.flac");`. A LETTER page
    //           (label `A`) whose root file merely starts with `A` but has no `/`: it is
    //           returned UNCHANGED.
    // Why:      Loose files grouped by first letter have no folder to trim; the bare `A`
    //           must not be chopped off the filename.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(rowDisplay("A", "Apple.flac")).toBe("Apple.flac");
    // ```
    assert_eq!(row_display("A", "Apple.flac"), "Apple.flac");
    // What:     `assert_eq!(row_display("#", "#tag.flac"), "#tag.flac");`. The `#`
    //           catch-all page: a root file starting with `#` is returned unchanged
    //           (after stripping `#` there is no `/`).
    // Why:      The catch-all is a letter-style tab; its loose files keep their names.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(rowDisplay("#", "#tag.flac")).toBe("#tag.flac");
    // ```
    assert_eq!(row_display("#", "#tag.flac"), "#tag.flac");
    // What:     `assert_eq!(row_display("A", "A/song.flac"), "song.flac");`. A FOLDER
    //           literally named `A` (its names are `A/...`): this IS a folder tab, so the
    //           `A/` prefix is stripped.
    // Why:      The distinction is the `/` after the label, not the label's length: a
    //           one-letter FOLDER still trims, unlike a one-letter LETTER bucket.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // expect(rowDisplay("A", "A/song.flac")).toBe("song.flac");
    // ```
    assert_eq!(row_display("A", "A/song.flac"), "song.flac");
}
