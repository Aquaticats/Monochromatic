//! Page index: the pages of a library, built once and extended as names are appended.
//!
//! What: `PageIndex` groups queue positions into the same pages `paginate` makes (folder pages, then A to Z pages,
//! then one `#` page, each sorted the way `paginate` sorts them) but stores positions, not name copies, and answers
//! "which page holds position i" without scanning.
//! Why: the window asked for the pages on every projection, and `paginate` clones every name into a map each time;
//! at one million tracks that was about half a second per call. The index is built when the library arrives (and
//! extended when more names arrive) and every later projection reads the selected page only.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! class PageIndex { groups: { label: string; identity: string; indices: number[] }[]; pageOf: number[]; extend(names) {} }
//! ```

use std::collections::HashMap;

/// What:
///      `const FOLDER_GROUP: u8 = 0`, `const LETTER_GROUP: u8 = 1`, `const CATCH_ALL_GROUP: u8 = 2`.
///  The group numbers `paginate` sorts by.
/// Why:
///       Folder pages come first, then letter pages, then the catch-all page.
const FOLDER_GROUP: u8 = 0;

/// The group number of an A to Z page.
const LETTER_GROUP: u8 = 1;

/// The group number of the `#` page.
const CATCH_ALL_GROUP: u8 = 2;

/// The label of the catch-all page.
const CATCH_ALL_LABEL: &str = "#";

/// What:
///      `pub struct PageGroup { pub label, pub identity, group, indices }`.
///  One page: its label, its stable identity and the queue positions it holds.
/// Why:
///       The window selects a page by identity and shows the positions of the selected one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageGroup {
    /// Folder name, letter, or `#`.
    pub label: String,
    /// `folder:{label}` for a folder page and `root:{label}` for a letter page, as `page_identity` formats it.
    pub identity: String,
    /// Group number used for ordering.
    group: u8,
    /// Queue positions on this page, ascending.
    indices: Vec<u32>,
}

/// Methods of `PageGroup`.
impl PageGroup {
    /// What: `indices` lists the queue positions on the page.
    /// Why: the selected page's rows are built from these positions.
    #[must_use]
    pub fn indices(&self) -> &[u32] {
        return &self.indices;
    }
}

/// What:
///      `pub struct PageIndex`.
///  All pages of a library in page order, with a position-to-page table.
/// Why:
///       See the module description.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PageIndex {
    /// Pages in creation order; `order` gives the display order.
    groups: Vec<PageGroup>,
    /// Group numbers in page order.
    order: Vec<usize>,
    /// Page position of each group.
    rank: Vec<usize>,
    /// Group number of each queue position.
    group_of: Vec<u32>,
    /// Folder page group by folder name.
    folder_slots: HashMap<String, usize>,
    /// A to Z page groups by letter index.
    letter_slots: [Option<usize>; 26],
    /// The `#` page group.
    catch_all_slot: Option<usize>,
}

/// What:
///      `fn page_sort_key(group: &PageGroup) -> (u8, String, &str)`.
///  The key `paginate` sorts pages by: group, upper-cased label, then label.
/// Why:
///       Equal keys must give the same page order as `paginate`.
fn page_sort_key(group: &PageGroup) -> (u8, String, &str) {
    return (group.group, group.label.to_uppercase(), group.label.as_str());
}

/// Methods of `PageIndex`.
impl PageIndex {
    /// What: `build` indexes a whole library.
    /// Why: the window builds it when a queue arrives.
    #[must_use]
    pub fn build(names: &[String]) -> PageIndex {
        let mut index: PageIndex = PageIndex::default();
        index.extend(names);
        return index;
    }

    /// What: `covered` is the number of names indexed.
    /// Why: callers compare it to the library length to see that the index is current.
    #[must_use]
    pub fn covered(&self) -> usize {
        return self.group_of.len();
    }

    /// What: `group_for` finds or creates the page of a name.
    /// Why: the page comes from the text before the first `/`, or from the first letter for a root file.
    fn group_for(&mut self, name: &str) -> usize {
        if let Some(slash) = name.find('/') {
            let folder: &str = &name[..slash];
            if let Some(slot) = self.folder_slots.get(folder) {
                return *slot;
            }
            let slot: usize = self.add_group(folder.to_string(), FOLDER_GROUP, "folder");
            self.folder_slots.insert(folder.to_string(), slot);
            return slot;
        }
        return match name.chars().next() {
            Some(first) if first.is_ascii_alphabetic() => {
                let letter: usize = usize::from(first.to_ascii_uppercase() as u8 - b'A');
                if let Some(slot) = self.letter_slots[letter] {
                    return slot;
                }
                let slot: usize = self.add_group(first.to_ascii_uppercase().to_string(), LETTER_GROUP, "root");
                self.letter_slots[letter] = Some(slot);
                slot
            }
            _ => {
                if let Some(slot) = self.catch_all_slot {
                    return slot;
                }
                let slot: usize = self.add_group(CATCH_ALL_LABEL.to_string(), CATCH_ALL_GROUP, "root");
                self.catch_all_slot = Some(slot);
                slot
            }
        };
    }

    /// What: `add_group` appends an empty page.
    /// Why: its display position is settled after the batch, in `extend`.
    fn add_group(&mut self, label: String, group: u8, kind: &str) -> usize {
        let identity: String = format!("{kind}:{label}");
        self.groups.push(PageGroup {
            label,
            identity,
            group,
            indices: Vec::new(),
        });
        return self.groups.len() - 1;
    }

    /// What: `extend` indexes the names after the ones already covered.
    /// Why: a library that grows by appending only costs the new names, plus a re-sort of the pages when a new page
    /// appeared.
    ///
    /// # Panics
    /// When the library passes four billion names, which is far above the supported size.
    pub fn extend(&mut self, names: &[String]) {
        let started: usize = self.group_of.len();
        let groups_before: usize = self.groups.len();
        self.group_of.reserve(names.len().saturating_sub(started));
        for (position, name) in names.iter().enumerate().skip(started) {
            let slot: usize = self.group_for(name);
            self.groups[slot].indices.push(u32::try_from(position).expect("a library holds under four billion names"));
            self.group_of.push(u32::try_from(slot).expect("a library holds under four billion pages"));
        }
        if self.groups.len() != groups_before || self.order.len() != self.groups.len() {
            let mut order: Vec<usize> = (0..self.groups.len()).collect();
            order.sort_by(|left, right| return page_sort_key(&self.groups[*left]).cmp(&page_sort_key(&self.groups[*right])));
            let mut rank: Vec<usize> = vec![0; order.len()];
            for (page, slot) in order.iter().enumerate() {
                rank[*slot] = page;
            }
            self.order = order;
            self.rank = rank;
        }
    }

    /// What: `page_count` is the number of pages.
    /// Why: the rail and the selection need it.
    #[must_use]
    pub fn page_count(&self) -> usize {
        return self.order.len();
    }

    /// What: `page` is the page at display position `position`.
    /// Why: the window reads the selected page and the labels in page order.
    #[must_use]
    pub fn page(&self, position: usize) -> Option<&PageGroup> {
        return self.order.get(position).map(|slot| return &self.groups[*slot]);
    }

    /// What: `pages` lists the pages in display order.
    /// Why: the rail is built from all labels.
    pub fn pages(&self) -> impl Iterator<Item = &PageGroup> + '_ {
        return self.order.iter().map(|slot| return &self.groups[*slot]);
    }

    /// What: `position_of_identity` finds a page by identity.
    /// Why: the selection is stored as an identity so it survives a rebuild.
    #[must_use]
    pub fn position_of_identity(&self, identity: &str) -> Option<usize> {
        return self.pages().position(|page| return page.identity == identity);
    }

    /// What: `position_of_label` finds the first page with a label.
    /// Why: choosing a folder from Search or the picker names the folder, not its identity.
    #[must_use]
    pub fn position_of_label(&self, label: &str) -> Option<usize> {
        return self.pages().position(|page| return page.label == label);
    }

    /// What: `page_of_index` is the display position of the page that holds queue position `index`.
    /// Why: following the playing track must not scan the library.
    #[must_use]
    pub fn page_of_index(&self, index: usize) -> Option<usize> {
        return self.group_of.get(index).map(|slot| return self.rank[*slot as usize]);
    }
}

/// Unit tests in `page_index_tests.rs`,
///  pulled in at this point.
#[cfg(test)]
#[path = "page_index_tests.rs"]
mod tests;
