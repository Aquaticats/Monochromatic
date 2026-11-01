//! Shared filename prefixes: `かめりあ(Camellia) - Aegis` shows as `Aegis` when the whole folder starts the same way.
//!
//! What: `shared_prefix_len` finds the text that every title of one directory starts with, cut back to a separator
//! so no word is split, and `display_title` removes it (or, for a directory with one track, the directory's own name).
//! Why: D86 makes prefix stripping an always-on behavior: "titles are shown without a common filename prefix". No
//! desktop code did it (the note under D86 in `package/music-player/design/decisions.md` says so), so rows repeated
//! the artist and the deck read `folder / folder - title` (the polish review finding LS3).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const titles = stems.map((stem) => stem.slice(sharedPrefixLen(stems)));
//! ```

/// What:
///      `fn is_separator(character: char) -> bool`.
///  True for a character that can end a prefix: anything that is not a letter or a digit.
/// Why:
///       A prefix is cut only after a space or a punctuation mark, never inside a word, so `Aegis` and `Another` do not
///  lose their shared first letter.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isSeparator = (c: string) => !/[\p{L}\p{N}]/u.test(c);
/// ```
fn is_separator(character: char) -> bool {
    return !character.is_alphanumeric();
}

/// What:
///      `fn common_prefix_len(titles: &[&str]) -> usize`.
///  The byte length of the longest text that every title starts with, compared character by character.
/// Why:
///       The cut is made on whole characters, so the result is always a valid place to slice every title.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const commonPrefixLen = (titles: string[]) => { /* longest common prefix by code point */ };
/// ```
fn common_prefix_len(titles: &[&str]) -> usize {
    let Some(first) = titles.first() else {
        return 0;
    };
    let mut length: usize = 0;
    for (offset, character) in first.char_indices() {
        let end: usize = offset + character.len_utf8();
        if titles.iter().all(|title| return title.get(offset..end) == Some(&first[offset..end])) {
            length = end;
        } else {
            break;
        }
    }
    return length;
}

/// What:
///      `pub fn shared_prefix_len(titles: &[&str]) -> usize`.
///  The byte length of the prefix to remove from every title of one directory, or zero when nothing is removed.
///  The prefix is the longest shared start cut back to just after its last separator. Nothing is removed for fewer
///  than two titles, when the cut leaves no separator, when a title would be left empty or blank, when every title is the same, or when the removed
///  text holds no letter or digit.
/// Why:
///       `01 Song` and `02 Song` share only `0`, which holds no separator, so numbers stay. `Track 001` and
///  `Track 002` lose `Track ` because the cut falls after the space. A single title has nothing to share.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sharedPrefixLen(titles: string[]): number {
///   if (titles.length < 2) return 0;
///   const common = longestCommonPrefix(titles);
///   const cut = lastSeparatorEnd(common);
///   return cut > 0 && titles.every((t) => t.slice(cut).trim() !== "") && /[\p{L}\p{N}]/u.test(common.slice(0, cut)) ? cut : 0;
/// }
/// ```
pub fn shared_prefix_len(titles: &[&str]) -> usize {
    if titles.len() < 2 {
        return 0;
    }
    let Some(first) = titles.first() else {
        return 0;
    };
    if titles.iter().all(|title| return title == first) {
        tracing::trace!("every title is the same; nothing is removed");
        return 0;
    }
    let common: usize = common_prefix_len(titles);
    let shared: &str = &first[..common];
    let cut: usize = shared
        .char_indices()
        .filter(|(_offset, character)| return is_separator(*character))
        .map(|(offset, character)| return offset + character.len_utf8())
        .next_back()
        .unwrap_or(0);
    if cut == 0 {
        tracing::trace!(titles = titles.len(), "no shared filename prefix ends at a separator");
        return 0;
    }
    if !shared[..cut].chars().any(char::is_alphanumeric) {
        tracing::trace!("the shared start holds no letter or digit; nothing is removed");
        return 0;
    }
    if titles.iter().any(|title| return title[cut..].trim().is_empty()) {
        tracing::trace!("removing the shared prefix would leave a blank title; nothing is removed");
        return 0;
    }
    tracing::debug!(titles = titles.len(), prefix = &shared[..cut], "a shared filename prefix is removed");
    return cut;
}

/// What:
///      `pub fn strip_folder_name<'a>(title: &'a str, folder: &str) -> &'a str`.
///  Removes the directory's own name from the start of `title` when a separator and more text follow it, ignoring
///  case; otherwise returns `title` unchanged.
/// Why:
///       A directory with one track has no sibling to share a prefix with, but `Camellia / Camellia - Aegis` still
///  repeats the folder in the deck heading.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const stripFolderName = (title: string, folder: string) => { /* case-insensitive startsWith + separator + rest */ };
/// ```
pub fn strip_folder_name<'a>(title: &'a str, folder: &str) -> &'a str {
    let folder_text: &str = folder.trim();
    if folder_text.is_empty() {
        return title;
    }
    let mut title_characters = title.char_indices();
    for folder_character in folder_text.chars() {
        match title_characters.next() {
            Some((_offset, title_character)) if title_character.to_lowercase().eq(folder_character.to_lowercase()) => {}
            _ => return title,
        }
    }
    let after_name: usize = title_characters.next().map_or(title.len(), |(offset, _character)| return offset);
    let rest: &str = &title[after_name..];
    let remainder: &str = rest.trim_start_matches(is_separator);
    if remainder.len() == rest.len() || remainder.trim().is_empty() {
        return title;
    }
    return remainder;
}

/// What:
///      `pub fn display_title<'a>(title: &'a str, prefix_len: usize, folder: &str) -> &'a str`.
///  The title a row or the deck shows: the title without the shared prefix, or, when the directory shares none,
///  without its own folder name.
/// Why:
///       The caller finds the prefix once per directory (`shared_prefix_len`) and applies it to each title here, so a
///  list of thousands of rows is walked once per directory, not once per row.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const displayTitle = (title: string, prefixLen: number, folder: string) => prefixLen > 0 ? title.slice(prefixLen).trimStart() : stripFolderName(title, folder);
/// ```
pub fn display_title<'a>(title: &'a str, prefix_len: usize, folder: &str) -> &'a str {
    if prefix_len > 0 && title.is_char_boundary(prefix_len) {
        return title[prefix_len..].trim_start();
    }
    return strip_folder_name(title, folder);
}

#[cfg(test)]
#[path = "title_prefix_tests.rs"]
mod tests;
