//! What:
//!  The one doc block each item of `merge_isolation_apply_save.rs` carries,
//!  and the one item of that file whose doc must be its own.
//! Why:
//!   Helper functions were inserted between the doc of `save_for_a_later_finish` and the function,
//!  so the doc sat above `branch_word` and `save_for_a_later_finish` had none
//!  (alpha.47's release pass, Low 5).
//!  A doc block belongs to the item directly under it,
//!  so a block that names two subjects is filed under one item while describing another,
//!  and the release's own lint reads no doc text and cannot see it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(docBlocksWithTwoSubjects(source)).toEqual([]); // one line per second `What:` in one doc block
//! ```

/// What:
///  `const SUBJECT: &str` is the text of `merge_isolation_apply_save.rs`, the module these controls read.
///  `&str` is a borrowed view of text (siblings: `String`, the owned buffer; `&[u8]`, the same bytes with no promise of UTF-8).
/// Why:
///  The two controls read the file this one sits beside, and `include_str!` embeds it at compile time,
///  so no control opens a file and none can wait on a name that is not a plain one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const SUBJECT = readFileSync(new URL('./merge_isolation_apply_save.rs', import.meta.url), 'utf8');
/// ```
const SUBJECT: &str = include_str!("merge_isolation_apply_save.rs");

/// What:
///  `fn doc_blocks_with_two_subjects(source: &str) -> Vec<usize>` is the one-based line numbers at which a
///  second `What:` header begins inside one unbroken run of `///` lines.
///  `Vec<usize>` is a growable list of line numbers (siblings: `&[usize]`, a borrowed view of the same list;
///  `[usize; N]`, a fixed run whose length is part of the type).
/// Why:
///  A finding names a line a person opens,
///  and this crate gives every item one `What:` header,
///  so a second header in the same run is a second doc with no item of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function docBlocksWithTwoSubjects(source: string): number[] {
///   const found: number[] = [];
///   let headers = 0;
///   source.split('\n').forEach((line, index) => {
///     if (!line.trimStart().startsWith('///')) { headers = 0; return; }
///     if (line.trimStart().slice(3).trim() === 'What:') { headers += 1; if (headers === 2) found.push(index + 1); }
///   });
///   return found;
/// }
/// ```
fn doc_blocks_with_two_subjects(source: &str) -> Vec<usize> {
    let mut found: Vec<usize> = Vec::new();
    // How many `What:` headers the run of doc lines this line belongs to has held so far.
    let mut headers: usize = 0;
    for (index, line) in source.lines().enumerate() {
        let trimmed: &str = line.trim_start();
        let Some(after_marker) = trimmed.strip_prefix("///") else {
            // A line that is no doc line ends the run, and the next doc block starts its own count.
            headers = 0;
            continue;
        };
        if after_marker.trim() != "What:" {
            continue;
        }
        headers += 1;
        if headers == 2 {
            // The line number a reader opens, counted from one as an editor counts them.
            found.push(index + 1);
        }
    }
    return found;
}

/// What:
///  `fn doc_above<'a>(source: &'a str, declaration: &str) -> Vec<&'a str>` is the doc block directly above the
///  first line of `source` that begins with `declaration`, each line without its `///` marker.
///  `&'a str` items are borrowed from `source` (sibling: `String`, which would copy each line here).
/// Why:
///  The control below names its item by the line that declares it,
///  so renaming the item moves no line of this file and a doc that wanders away from it is caught.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function docAbove(source: string, declaration: string): string[] {
///   const lines = source.split('\n');
///   const at = lines.findIndex((line) => line.startsWith(declaration));
///   const above: string[] = [];
///   for (let index = at - 1; index >= 0 && lines[index].trimStart().startsWith('///'); index -= 1) above.unshift(lines[index].trimStart().slice(3).trim());
///   return above;
/// }
/// ```
fn doc_above<'a>(
    source: &'a str,
    declaration: &str,
) -> Vec<&'a str> {
    let lines: Vec<&str> = source.lines().collect();
    let at: usize = match lines
        .iter()
        .position(|line| return line.starts_with(declaration))
    {
        Some(found) => found,
        // The item this control guards is absent, which the caller's assertion reports as an empty doc.
        None => return Vec::new(),
    };
    let mut above: Vec<&str> = Vec::new();
    let mut index: usize = at;
    while index > 0 {
        index -= 1;
        let trimmed: &str = lines[index].trim_start();
        let Some(after_marker) = trimmed.strip_prefix("///") else {
            // The first line above the run that is no doc line is where the block ends.
            break;
        };
        above.push(after_marker.trim());
    }
    above.reverse();
    return above;
}

/// No doc block of the file holds two `What:` headers,
/// so every doc in it is filed under the one item it describes.
#[test]
fn no_doc_block_of_the_saving_module_names_two_subjects() {
    let found: Vec<usize> = doc_blocks_with_two_subjects(SUBJECT);
    assert!(
        found.is_empty(),
        "a `What:` header begins at each of these lines inside a doc block that already had one, so \
         the doc before it documents no item of its own: {found:?}"
    );
}

/// The item that saves a merge for a later finish carries the doc written for it,
/// rather than a helper above it holding that doc.
#[test]
fn the_item_that_saves_a_merge_for_a_later_finish_carries_its_own_doc() {
    let doc: Vec<&str> = doc_above(SUBJECT, "pub(crate) fn save_for_a_later_finish(");
    assert!(
        doc.iter()
            .any(|line| return line.contains("Save a merge that met an index lock")),
        "the item that saves a merge is described by the doc written for it, and no doc sits above it now: {doc:?}"
    );
}
