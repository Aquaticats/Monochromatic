//! Display order of sibling names in the file tree:
//!  mixed folders and files,
//! case folded,
//! digit runs compared as numbers.

/// Native names are compared through their lossy text,
///  with the raw name last so equal-looking names keep one fixed order.
use std::{
    cmp::Ordering,
    ffi::{OsStr, OsString},
};

/// One comparable piece of a name.
/// A digit run stands where the digit zero stands among characters,
///  so a dot or a dash comes before numbers and letters come after them.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Piece {
    /// A run of decimal digits without its leading zeros:
    ///  the longer run is the larger number,
    /// and equal lengths compare digit by digit.
    Number {
        /// Digit count after leading zeros are dropped.
        length: usize,
        /// The digits themselves.
        digits: String,
    },
    /// One character folded to lower case,
    /// so `B` and `b` compare equal and `a` sorts before `B`.
    Character(
        /// The character after lower-casing.
        char,
    ),
}

/// A piece compares by the character it stands for,
/// then by value among numbers.
impl Piece {
    /// The character a piece takes its place from;
    ///  no `Character` is ever a digit,
    /// because digits always become a `Number`.
    fn rank(&self) -> char {
        return match self {
            Piece::Number { .. } => '0',
            Piece::Character(character) => *character,
        };
    }
}

/// Pieces order by rank first;
///  two numbers then order by length and digits.
impl Ord for Piece {
    /// Rank decides first;
    ///  only two numbers fall through to length and digits.
    fn cmp(&self, other: &Self) -> Ordering {
        let by_rank = self.rank().cmp(&other.rank());
        if by_rank != Ordering::Equal {
            return by_rank;
        }
        return match (self, other) {
            (
                Piece::Number { length, digits },
                Piece::Number {
                    length: other_length,
                    digits: other_digits,
                },
            ) => {
                // Digits are compared only when the lengths are equal.
                let by_length = length.cmp(other_length);
                if by_length == Ordering::Equal {
                    digits.cmp(other_digits)
                } else {
                    by_length
                }
            }
            _ => Ordering::Equal,
        };
    }
}

/// Ordering must agree with `Ord`.
impl PartialOrd for Piece {
    /// Delegate to `cmp` so the two orders cannot disagree.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        return Some(self.cmp(other));
    }
}

/// Sort key of one name;
///  keys compare in the order the tree shows the names.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NameKey {
    /// Case folded pieces with numbers compared by value.
    pieces: Vec<Piece>,
    /// Names that fold to the same pieces (`A` and `a`,
    /// `01` and `1`) fall back to their raw bytes.
    raw: OsString,
}

/// Split a name into pieces in one pass over its characters.
fn pieces(text: &str) -> Vec<Piece> {
    let mut result = Vec::new();
    let mut characters = text.chars().peekable();
    while let Some(character) = characters.next() {
        if character.is_ascii_digit() {
            let mut digits = String::from(character);
            while let Some(next) = characters.next_if(char::is_ascii_digit) {
                digits.push(next);
            }
            let trimmed = digits.trim_start_matches('0').to_owned();
            result.push(Piece::Number {
                length: trimmed.len(),
                digits: trimmed,
            });
        } else {
            for folded in character.to_lowercase() {
                result.push(Piece::Character(folded));
            }
        }
    }
    return result;
}

/// The key that orders `name` among its siblings.
pub fn name_key(name: &OsStr) -> NameKey {
    return NameKey {
        pieces: pieces(&name.to_string_lossy()),
        raw: name.to_owned(),
    };
}
