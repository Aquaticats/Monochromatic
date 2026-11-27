//! What:
//!  Whether a character prints as nothing a reader can see.
//! Why:
//!  A message names a folder, a file or a line so its reader can find it,
//!  and a character that prints as nothing turns the name the message shows into a different name.
//!  A zero-width space between two letters, a left-to-right mark copied from a web page in front of a word,
//!  a byte order mark at the start of a file:
//!  each is invisible in a terminal and each changes what the line says.
//!  So every message that quotes such a character spells it as `\u{<code point>}`,
//!  and this module answers which characters a reader cannot see.
//!  Three properties answer it, and each has a source rather than a list written by hand:
//!  the general category `Cf` (format) and `Cc` (control) come from `unicode-properties`,
//!  whose tables say which Unicode release they hold,
//!  and a code point Unicode's `Default_Ignorable_Code_Point` names while it is neither `Cf` nor `Cc`
//!  is written out in `DEFAULT_IGNORABLE_CODE_POINTS`, beside the data file and release it comes from.
//!  Whitespace is not one of them:
//!  a space between two words is visible,
//!  and a tab or a no-break space is spelled by the rule that names it, wherever it stands.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const printsAsNothing = (letter) =>
//!   !/\s/.test(letter) &&
//!   (/[\p{Cf}\p{Cc}]/u.test(letter) || DEFAULT_IGNORABLE_CODE_POINTS.some(([a, b]) => letter >= a && letter <= b))
//! ```

/// Import the general category query, whose tables this crate already links for the parser stack.
use unicode_properties::UnicodeGeneralCategory;

/// What:
///  The code points Unicode's `Default_Ignorable_Code_Point` names
///  that are neither `Cf` (format) nor `Cc` (control),
///  as inclusive ranges, in ascending order.
/// Why:
///  The property is derived:
///  it holds every format character, and beside those a set of combining marks, Hangul fillers and
///  code points no Unicode release has assigned yet,
///  and none of those is a category the general-category query answers as invisible.
///  Without them a variation selector, a Hangul filler or a language tag would print raw in a message,
///  and a reader would see a name that looks right and is not on disk.
///  Source:
///  `DerivedCoreProperties-17.0.0.txt`, the property `Default_Ignorable_Code_Point`,
///  from https://www.unicode.org/Public/17.0.0/ucd/DerivedCoreProperties.txt,
///  the same release as the general-category tables this crate links,
///  taking that property's code points and leaving out every `Cf` and `Cc` code point
///  (`DerivedGeneralCategory-17.0.0.txt`) and every whitespace code point (`PropList-17.0.0.txt`).
///  The ranges are the same code points the release review of 0.5.2-alpha.21 counted as candidates
///  beside `Cf` and `Cc`.
///  A release of the data file that moves a code point between these ranges and a category
///  changes only which of the two answers spells it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DEFAULT_IGNORABLE_CODE_POINTS = [[0x34f, 0x34f], [0x115f, 0x1160], ...];
/// ```
pub(crate) const DEFAULT_IGNORABLE_CODE_POINTS: &[(char, char)] = &[
    ('\u{34f}', '\u{34f}'),
    ('\u{115f}', '\u{1160}'),
    ('\u{17b4}', '\u{17b5}'),
    ('\u{180b}', '\u{180d}'),
    ('\u{180f}', '\u{180f}'),
    ('\u{2065}', '\u{2065}'),
    ('\u{3164}', '\u{3164}'),
    ('\u{fe00}', '\u{fe0f}'),
    ('\u{ffa0}', '\u{ffa0}'),
    ('\u{fff0}', '\u{fff8}'),
    ('\u{e0000}', '\u{e0000}'),
    ('\u{e0002}', '\u{e001f}'),
    ('\u{e0080}', '\u{e0fff}'),
];

/// What:
///  Whether a character prints as nothing wherever it stands,
///  so a name holding one is no name a reader can see.
/// Why:
///  A byte order mark before a line's first word makes that word no word the reader knows,
///  a zero-width space inside a name is no separator it reads,
///  and a right-to-left override reorders every character after it on a terminal that draws bidirectional text:
///  each must be spelled, or the message shows a line other than the one the file holds.
///  A whitespace character is not one of these:
///  a space between two words is visible,
///  and the rule that names a separator spells a tab or a no-break space wherever it stands.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const printsAsNothing = (letter) => !/\s/u.test(letter) && (isFormatOrControl(letter) || isDefaultIgnorable(letter))
/// ```
pub(crate) fn prints_as_nothing(letter: char) -> bool {
    // What: whether the character is one the line may set a word off with, which a reader can see between two words.
    // Why: a space between two words is visible, and every other whitespace character is spelled by the rule that names it.
    if letter.is_whitespace() {
        return false;
    }
    return is_format_or_control(letter) || is_default_ignorable(letter);
}

/// What:
///  Whether the Unicode general category of a character is one that has no glyph of its own:
///  `Cf` (format) or `Cc` (control).
/// Why:
///  The category is the property a reader's terminal acts on,
///  and the query answers it from the tables this crate already links,
///  so no list of format characters is written by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isFormatOrControl = (letter) => /\p{Cf}|\p{Cc}/u.test(letter)
/// ```
fn is_format_or_control(letter: char) -> bool {
    // What: the general category the linked Unicode tables give this character.
    // Why: `Cf` and `Cc` are exactly the two categories whose code points print as nothing of their own.
    let category: unicode_properties::GeneralCategory = letter.general_category();
    return category == unicode_properties::GeneralCategory::Format
        || category == unicode_properties::GeneralCategory::Control;
}

/// What:
///  Whether a character is one of the code points `DEFAULT_IGNORABLE_CODE_POINTS` writes out.
/// Why:
///  The table holds the ordered ranges of a property the general-category query does not answer,
///  and one lookup tells a variation selector from an accent a reader can see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isDefaultIgnorable = (letter) => DEFAULT_IGNORABLE_CODE_POINTS.some(([from, to]) => letter >= from && letter <= to)
/// ```
fn is_default_ignorable(letter: char) -> bool {
    return DEFAULT_IGNORABLE_CODE_POINTS
        .iter()
        .any(|(from, to)| return *from <= letter && letter <= *to);
}

/// What:
///  The Unicode release the linked general-category tables hold, as major, minor and update.
/// Why:
///  The ranges above come from one release of the data file,
///  and a reader checking them needs the release of the tables they are read beside:
///  two releases that disagree would spell one character twice or not at all.
///  The crate's own tables answer this, so no version number is written by hand twice.
///
/// Only a control asks this question, so it is built only for one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unicodeRelease = () => UNICODE_VERSION
/// ```
#[cfg(test)]
pub(crate) fn unicode_release() -> (u64, u64, u64) {
    return unicode_properties::UNICODE_VERSION;
}

/// Controls of which characters a reader cannot see stay outside release artifacts.
#[cfg(test)]
#[path = "layout_unseen_characters_tests.rs"]
mod tests;
