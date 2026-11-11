//! What:
//!  How the line merge Git makes with `--zdiff3` is cut into the text Git settled and the regions it
//! left,
//! and where a region sits in the three stages.
//! Why:
//!   The word pass runs on the regions Git could not settle and on nothing else,
//! so a region has to be split out of Git's answer with its three
//!  sides and its place in the file.
//! `zdiff3` is the style that keeps the lines both sides wrote identically out of a conflict
//! (`git help merge`,
//! `merge.conflictStyle`;
//! `xdl_refine_conflicts` and `XDL_MERGE_ZEALOUS_DIFF3` in Git 2.56.0's `xdiff/xmerge.c`),
//! so a region is the lines that really are a choice,
//! and the word pass has less room to join one side's clause to the other side's.
//!  A region's place is found by searching its text forward from the end of the region before it,
//! because a `--zdiff3` answer holds its regions in file order;
//! the place feeds the report's line numbers only,
//! never the merged bytes,
//! so a repeated passage can at worst name the earlier of two identical places.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { settled, regions } = cut(gitZdiff3Merge); // settled.length === regions.length + 1
//! ```

/// One conflict region of a `--zdiff3` merge.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Region {
    /// The common ancestor's text of the region,
    /// empty where both sides inserted at one point.
    pub base: Vec<u8>,
    /// The first side's text of the region.
    pub ours: Vec<u8>,
    /// The second side's text of the region.
    pub theirs: Vec<u8>,
    /// The bytes the region holds in Git's answer,
    /// its markers included.
    pub markers: Vec<u8>,
}

/// A `--zdiff3` merge cut apart.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cut {
    /// The text Git settled,
    /// in order,
    /// one piece more than `regions`;
    /// the piece after a region follows it.
    pub settled: Vec<Vec<u8>>,
    /// The regions Git left,
    /// in file order.
    pub regions: Vec<Region>,
}

/// How many characters a conflict marker uses,
/// which is Git's default (`merge.conflictStyle` writes `<<<<<<<`).
const MARKER_WIDTH: usize = 7;

/// The byte that ends a line.
const LINE_END: u8 = b'\n';

/// Whether one byte ends a line.
fn is_line_end(byte: &u8) -> bool {
    return *byte == LINE_END;
}

/// What:
///  Which marker line this is:
/// `<` for the first side,
/// `|` for the common ancestor,
/// `=` for the split,
/// `>` for the second side,
/// or nothing for any other line.
/// Why:
///   Git writes a marker as its character repeated `MARKER_WIDTH` times and then a space and a label
/// (`xdl_fill_merge_buffer`,
/// `xdiff/xmerge.c`),
/// so the run is what a parser can trust,
/// not the label the caller chose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marker = (line) => /^(<{7}|={7}|\|{7}|>{7})( |$)/.exec(line)?.[1][0];
/// ```
fn marker_of(line: &[u8]) -> Option<u8> {
    let character: u8 = *line.first()?;
    if !matches!(character, b'<' | b'|' | b'=' | b'>') {
        return None;
    }
    if line.len() < MARKER_WIDTH || line[..MARKER_WIDTH] != [character; MARKER_WIDTH] {
        return None;
    }
    match line.get(MARKER_WIDTH) {
        None | Some(b' ') | Some(b'\n') => return Some(character),
        Some(_) => return None,
    }
}

/// What:
///  Cut a merge into the text Git settled and the conflict regions it left.
/// Why:
///   The word pass takes each region whole or not at all,
/// and the settled text is written exactly as Git wrote it.
/// An answer that ends inside a region,
/// or that nests one region inside another,
/// is refused rather than guessed:
/// a half-read region would lose a side.
/// A line that only looks like a marker outside a region is the file's own text,
/// because Git does not escape the markers it writes;
/// the caller compares the region count with the conflict count Git reported,
/// so an answer that was cut apart wrongly is refused as well.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cut(merge: Uint8Array): { settled: Uint8Array[]; regions: Region[] } | { error: string };
/// ```
pub fn cut(merge: &[u8]) -> Result<Cut, String> {
    let mut settled: Vec<Vec<u8>> = vec![Vec::new()];
    let mut regions: Vec<Region> = Vec::new();
    let mut open: Option<Region> = None;
    // 0 outside a region, 1 the first side, 2 the common ancestor, 3 the second side.
    let mut part: u8 = 0;
    for line in merge.split_inclusive(is_line_end) {
        let marker: Option<u8> = marker_of(line);
        if part == 0 {
            match marker {
                Some(b'<') => {
                    let mut region: Region = Region::default();
                    region.markers.extend_from_slice(line);
                    open = Some(region);
                    part = 1;
                }
                _ => settled.last_mut().expect("a piece for every region plus one").extend_from_slice(line),
            }
            continue;
        }
        let region: &mut Region = open.as_mut().expect("a region is open");
        region.markers.extend_from_slice(line);
        match (marker, part) {
            (Some(b'|'), 1) | (Some(b'|'), 2) => part = 2,
            (Some(b'='), 1) | (Some(b'='), 2) => part = 3,
            (Some(b'>'), 3) => {
                regions.push(open.take().expect("a region is open"));
                settled.push(Vec::new());
                part = 0;
            }
            (Some(_), _) => return Err(String::from("a conflict marker of the line merge is out of order")),
            _ => match part {
                1 => region.ours.extend_from_slice(line),
                2 => region.base.extend_from_slice(line),
                _ => region.theirs.extend_from_slice(line),
            },
        }
    }
    if open.is_some() {
        return Err(String::from("the line merge ends inside a conflict region"));
    }
    return Ok(Cut { settled, regions });
}

/// What:
///  The byte offset of `needle` in `text` at or after `from`,
/// searched only at the start of a line.
/// Why:
///   A region's text is a run of whole lines of the stage it came from,
/// so a search that lands mid-line would name a place the region cannot start at.
/// An empty needle stands for a region with no line on that side,
/// which sits exactly where the search starts.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function offsetFromLine(text: Uint8Array, needle: Uint8Array, from: number): number | undefined;
/// ```
pub fn offset_from_line(text: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    if needle.is_empty() {
        return Some(from);
    }
    if needle.len() > text.len() {
        return None;
    }
    let mut at: usize = 0;
    while at < text.len() {
        if at >= from && text.len() - at >= needle.len() && &text[at..at + needle.len()] == needle {
            return Some(at);
        }
        let offset: usize = text[at..].iter().position(is_line_end)?;
        at += offset + 1;
    }
    return None;
}

/// What:
///  How many lines of `text` come before byte `start`.
/// Why:
///   A region's line span inside the region,
/// moved to the file,
/// needs the number of lines the region starts after.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const linesBefore = (text, start) => text.subarray(0, start).filter((b) => b === 10).length;
/// ```
pub fn lines_before(text: &[u8], start: usize) -> usize {
    let mut lines: usize = 0;
    for byte in &text[..start.min(text.len())] {
        if *byte == LINE_END {
            lines += 1;
        }
    }
    return lines;
}

/// The cutting of an answer and the placing of a region in its stage.
#[cfg(test)]
#[path = "merge_second_pass_line_regions_tests.rs"]
mod tests;
