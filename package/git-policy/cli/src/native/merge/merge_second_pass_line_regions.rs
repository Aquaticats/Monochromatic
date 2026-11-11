//! What:
//!  How the line merge Git makes with `--zdiff3` is cut into the text Git settled and the regions it
//! left,
//! where a region sits in the three stages,
//! and which of a region's split-shaped lines is Git's own.
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
//!  The three markers that open a side carry labels made unguessable for every merge,
//! but the split marker carries none:
//! Git writes it as seven equals signs and a line end
//! (`xdl_fill_merge_buffer`),
//! so no label tells Git's split from a line of the person's file that looks the same.
//! A region is therefore cut with its split lines still unchosen,
//! and each region's split is then chosen by what the three stages hold
//! (`Cut::place`):
//! one split-shaped line after the common ancestor marker is Git's by being the only one,
//! and where a region holds more than one,
//! the split is the only one whose three sides are found in their own stages.
//! A region no single split explains is left to the person,
//! so a file is never merged on a guess.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const cut = cut(gitZdiff3Merge, labels); // settled.length === regions.length + 1
//! const regions = cut.place([base, ours, theirs]); // each region with the place of each side in its stage
//! ```

/// One conflict region of a `--zdiff3` merge,
/// cut into the three sides Git kept for the person,
/// with the place each side sits at in its stage.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Region {
    /// The common ancestor's text of the region,
    /// empty where both sides inserted at one point.
    pub base: Vec<u8>,
    /// The first side's text of the region.
    pub ours: Vec<u8>,
    /// The second side's text of the region.
    pub theirs: Vec<u8>,
    /// The byte offset each side's text sits at in its stage,
    /// in the order of the stages:
    /// base,
    /// ours,
    /// theirs.
    pub starts: [usize; 3],
}

/// The three labels the line merge's markers carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MarkerLabels<'a> {
    /// The first side's label,
    /// as `-L` gave it.
    pub ours: &'a [u8],
    /// The common ancestor's label.
    pub base: &'a [u8],
    /// The second side's label.
    pub theirs: &'a [u8],
}

/// A region Git left,
/// with the lines that could be its split marker.
/// Why the split is not chosen while the answer is read:
///   the three markers that open a side carry the labels the second pass made unguessable,
/// while the split marker carries none,
/// so a line of the person's file that is exactly seven equals signs cannot be told from Git's own by its bytes;
/// the region's lines are kept whole here,
/// and the split is chosen by what the stages hold (`Cut::place`).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RawRegion {
    /// The bytes the region holds in Git's answer,
    /// its markers included.
    pub markers: Vec<u8>,
    /// The byte offset in `markers` where each of its lines starts,
    /// in order,
    /// the opening marker's line first and the closing marker's line last.
    pub line_starts: Vec<usize>,
    /// The line the common ancestor's marker is on,
    /// when the region has one.
    pub base_line: Option<usize>,
    /// The lines shaped like a split marker,
    /// in order;
    /// Git's own is one of them.
    pub split_lines: Vec<usize>,
}

/// A `--zdiff3` merge cut apart,
/// with each region's split still to be chosen.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cut {
    /// The text Git settled,
    /// in order,
    /// one piece more than `regions`;
    /// the piece after a region follows it.
    pub settled: Vec<Vec<u8>>,
    /// The regions Git left,
    /// in file order,
    /// each with the split-shaped lines that could be Git's.
    pub regions: Vec<RawRegion>,
}

/// How many characters a conflict marker uses,
/// which is Git's default (`merge.conflictStyle` writes `<<<<<<<`).
const MARKER_WIDTH: usize = 7;

/// The byte that ends a line.
const LINE_END: u8 = b'\n';

/// The byte that comes before the line end of a file whose lines end with both.
const CARRIAGE_RETURN: u8 = b'\r';

/// How many split-shaped lines of one file are searched for against the three stages in all.
/// Why:
///   A region whose answer holds one split-shaped line behind its common ancestor marker is Git's,
/// because Git writes one split per region,
/// so that line costs nothing;
/// a region with more than one is only explained by finding its sides in their stages,
/// which costs one search of each stage per line tried.
/// An 8 MiB stage is the largest the pass takes and holds about four hundred thousand lines,
/// so this many tries is about six hundred searches of the stages,
/// about 2 GB of comparisons,
/// and only a file whose answer holds more split-shaped lines than this inside regions Git left is refused
/// and kept as Git left it.
const SPLIT_TRIES: usize = 256;

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
/// so the run and the label together are what a parser can trust,
/// not the run alone.
/// The second pass makes the labels unguessable for every merge
/// (`merge_second_pass_git_lines.rs`),
/// so a line of the person's file that happens to look like a marker is the file's own text.
/// A file whose lines end with a carriage return and a line feed gets that ending in its markers too,
/// so both endings are a marker's own.
/// The split line carries no label,
/// so a line of that shape inside a region is only ever a candidate for Git's split
/// (`RawRegion::split_lines`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const marker = (line, labels) => [[b'<', labels.ours], [b'|', labels.base], [b'=', b''], [b'>', labels.theirs]].find(([c, label]) => line.equals(Buffer.concat([Buffer.alloc(7, c), label.length ? Buffer.from(` ${label}\n`) : Buffer.from('\n')])))?.[0];
/// ```
fn marker_of(line: &[u8], labels: &MarkerLabels<'_>) -> Option<u8> {
    let character: u8 = *line.first()?;
    let label: &[u8] = match character {
        b'<' => labels.ours,
        b'|' => labels.base,
        b'=' => b"",
        b'>' => labels.theirs,
        _ => return None,
    };
    if line.len() < MARKER_WIDTH || line[..MARKER_WIDTH] != [character; MARKER_WIDTH] {
        return None;
    }
    let mut rest: &[u8] = &line[MARKER_WIDTH..];
    if !label.is_empty() {
        rest = rest.strip_prefix(b" ")?;
    }
    let tail: &[u8] = rest.strip_prefix(label)?;
    // Git writes the marker of a file whose lines end with a carriage return and a line feed with that
    // ending too, so both endings are a marker's own.
    return match tail {
        [] | [LINE_END] | [CARRIAGE_RETURN, LINE_END] => Some(character),
        _ => None,
    };
}

impl RawRegion {
    /// Add one line of the answer to the region,
    /// remembering where in `markers` it starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const pushLine = (region, line) => { region.lineStarts.push(region.markers.length); region.markers = Buffer.concat([region.markers, line]); };
    /// ```
    fn push_line(&mut self, line: &[u8]) {
        self.line_starts.push(self.markers.len());
        self.markers.extend_from_slice(line);
    }

    /// What:
    ///  Whether the region holds a split-shaped line behind its common ancestor marker,
    /// where Git writes its split.
    /// Why:
    ///   Git writes the split after the common ancestor's section,
    /// so a split-shaped line before that marker is the first side's own text,
    /// and a region with none behind it is one this answer cannot hold.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const splitFollowsBase = (region) => region.splitLines.some((line) => line > (region.baseLine ?? -1));
    /// ```
    fn split_follows_base(&self) -> bool {
        let first: usize = match self.base_line {
            Some(base) => base + 1,
            None => 0,
        };
        for line in &self.split_lines {
            if *line >= first {
                return true;
            }
        }
        return false;
    }

    /// The byte offset in `markers` where `line` starts,
    /// or the end of the region for a line past its last.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const startOf = (region, line) => region.lineStarts[line] ?? region.markers.length;
    /// ```
    fn start_of(&self, line: usize) -> usize {
        return self.line_starts.get(line).copied().unwrap_or(self.markers.len());
    }

    /// The byte offset in `markers` where the closing marker's line starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const closingStart = (region) => region.lineStarts[region.lineStarts.length - 1];
    /// ```
    fn closing_start(&self) -> usize {
        return self.start_of(self.line_starts.len().saturating_sub(1));
    }

    /// What:
    ///  The three sides of the region when its split marker is `split`,
    /// as slices of `markers`,
    /// in the order of the stages:
    /// base,
    /// ours,
    /// theirs.
    /// Why:
    ///   The sides are searched in the stages before any of them is copied,
    /// so choosing among the split-shaped lines of one region costs no copy of the region per candidate.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const sidesAt = (raw, split) => {
    ///   const at = (line) => raw.lineStarts[line] ?? raw.markers.length;
    ///   const baseLine = raw.baseLine ?? split;
    ///   const base = raw.baseLine === undefined ? raw.markers.subarray(at(split), at(split)) : raw.markers.subarray(at(baseLine + 1), at(split));
    ///   return [base, raw.markers.subarray(at(1), at(baseLine)), raw.markers.subarray(at(split + 1), at(raw.lineStarts.length - 1))];
    /// };
    /// ```
    fn sides_at(&self, split: usize) -> [&[u8]; 3] {
        let closing: usize = self.closing_start();
        let split_start: usize = self.start_of(split);
        let theirs_start: usize = self.start_of(split + 1);
        let (ours_end, base_start): (usize, usize) = match self.base_line {
            Some(base) => (self.start_of(base), self.start_of(base + 1)),
            None => (split_start, split_start),
        };
        return [&self.markers[base_start..split_start], &self.markers[self.start_of(1)..ours_end], &self.markers[theirs_start..closing]];
    }

    /// The region cut at `split`,
    /// with no place in its stages yet.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const regionAt = (raw, split) => { const [base, ours, theirs] = sidesAt(raw, split); return { base: Buffer.from(base), ours: Buffer.from(ours), theirs: Buffer.from(theirs), starts: [0, 0, 0] }; };
    /// ```
    fn region_at(&self, split: usize) -> Region {
        let [base, ours, theirs] = self.sides_at(split);
        return Region { base: base.to_vec(), ours: ours.to_vec(), theirs: theirs.to_vec(), starts: [0, 0, 0] };
    }
}

/// What:
///  Cut a merge into the text Git settled and the conflict regions it left,
/// taking a marker line only when it carries the label `labels` names,
/// and keeping every split-shaped line of a region as a candidate.
/// Why:
///   The word pass takes each region whole or not at all,
/// and the settled text is written exactly as Git wrote it.
/// An answer that ends inside a region,
/// or that nests one region inside another,
/// is refused rather than guessed:
/// a half-read region would lose a side.
/// A line that only looks like a marker outside a region is the file's own text,
/// because Git does not escape the markers it writes and the labels are the second pass's own;
/// the caller compares the region count with the conflict count Git reported,
/// so an answer that was cut apart wrongly is refused as well.
/// The split of each region is chosen by `Cut::place`,
/// because the split marker carries no label and its bytes alone cannot be told from the file's own text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cut(merge: Uint8Array, labels: MarkerLabels): { settled: Uint8Array[]; regions: RawRegion[] } | { error: string };
/// ```
pub fn cut(merge: &[u8], labels: &MarkerLabels<'_>) -> Result<Cut, String> {
    let mut settled: Vec<Vec<u8>> = vec![Vec::new()];
    let mut regions: Vec<RawRegion> = Vec::new();
    let mut open: Option<RawRegion> = None;
    for line in merge.split_inclusive(is_line_end) {
        let marker: Option<u8> = marker_of(line, labels);
        if open.is_none() {
            if marker == Some(b'<') {
                let mut started: RawRegion = RawRegion::default();
                started.push_line(line);
                open = Some(started);
                continue;
            }
            settled.last_mut().expect("a piece for every region plus one").extend_from_slice(line);
            continue;
        }
        if marker == Some(b'>') {
            let mut done: RawRegion = open.take().expect("a region is open");
            done.push_line(line);
            if !done.split_follows_base() {
                return Err(String::from("a conflict marker of the line merge is out of order"));
            }
            regions.push(done);
            settled.push(Vec::new());
            continue;
        }
        let region: &mut RawRegion = open.as_mut().expect("a region is open");
        match marker {
            // A region inside a region is not an answer Git writes.
            Some(b'<') => return Err(String::from("a conflict marker of the line merge is out of order")),
            Some(b'|') => {
                if region.base_line.is_some() {
                    return Err(String::from("a conflict marker of the line merge is out of order"));
                }
                region.push_line(line);
                region.base_line = Some(region.line_starts.len() - 1);
            }
            Some(b'=') => {
                region.push_line(line);
                region.split_lines.push(region.line_starts.len() - 1);
            }
            _ => region.push_line(line),
        }
    }
    if open.is_some() {
        return Err(String::from("the line merge ends inside a conflict region"));
    }
    return Ok(Cut { settled, regions });
}

impl Cut {
    /// What:
    ///  Choose each region's split marker and where its three sides sit in their stages,
    /// or nothing when no single choice explains the answer.
    /// Why:
    ///   The split marker carries no label,
    /// so a line of the person's file that is exactly seven equals signs reads as a split
    /// (`merge_second_pass_line_regions.rs`,
    /// the module's own words).
    /// Git writes one split per region,
    /// always behind the common ancestor marker when the region has one,
    /// so a region whose answer holds exactly one split-shaped line there is explained without asking the stages.
    /// Where more than one line could be it,
    /// the stages are what tells them apart:
    /// each side of the region is a run of whole lines of its own stage,
    /// so the split Git meant is the only one whose three sides are found at or after the place the region before it
    /// ended.
    /// When none of them is found that way,
    /// or when two are,
    /// the file is left to the person rather than merged on a guess:
    /// a wrong split would join one side's text to another side's and name it resolved.
    /// The tries are bounded by `SPLIT_TRIES`,
    /// so a file whose answer holds more split-shaped lines than that inside its regions is left as well.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// place(sides) { let cursors = [0,0,0]; let tried = 0; return this.regions.map((raw) => { ... }); }
    /// ```
    pub fn place(&self, sides: [&[u8]; 3]) -> Option<Vec<Region>> {
        let mut cursors: [usize; 3] = [0, 0, 0];
        let mut placed: Vec<Region> = Vec::with_capacity(self.regions.len());
        let mut tried: usize = 0;
        for raw in &self.regions {
            let first: usize = match raw.base_line {
                Some(base) => base + 1,
                None => 0,
            };
            let mut candidates: Vec<usize> = Vec::new();
            for line in &raw.split_lines {
                if *line >= first {
                    candidates.push(*line);
                }
            }
            // One candidate is Git's by being the only one: Git always writes a split, and never two in one region.
            let ambiguous: bool = candidates.len() > 1;
            let mut chosen: Option<(usize, [usize; 3])> = None;
            let mut fitting: usize = 0;
            for split in candidates {
                if ambiguous {
                    if tried >= SPLIT_TRIES {
                        return None;
                    }
                    tried += 1;
                }
                let plain: [&[u8]; 3] = raw.sides_at(split);
                let mut starts: [usize; 3] = [0, 0, 0];
                let mut found: bool = true;
                for side in 0..3 {
                    match offset_from_line(sides[side], plain[side], cursors[side]) {
                        Some(at) => starts[side] = at,
                        None => {
                            found = false;
                            break;
                        }
                    }
                }
                if !found {
                    continue;
                }
                fitting += 1;
                chosen = Some((split, starts));
                if !ambiguous {
                    break;
                }
            }
            if fitting != 1 {
                return None;
            }
            let (split, starts) = chosen.expect("one split fits");
            let mut region: Region = raw.region_at(split);
            // A region with no text on any side is not an answer Git writes, and cutting it would drop the text it holds.
            if region.base.is_empty() && region.ours.is_empty() && region.theirs.is_empty() {
                return None;
            }
            region.starts = starts;
            for side in 0..3 {
                cursors[side] = starts[side] + [region.base.len(), region.ours.len(), region.theirs.len()][side];
            }
            placed.push(region);
        }
        return Some(placed);
    }
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

/// The cutting of an answer into regions with their splits to choose,
/// and the placing of a region in its stage.
#[cfg(test)]
#[path = "merge_second_pass_line_regions_tests.rs"]
mod tests;
