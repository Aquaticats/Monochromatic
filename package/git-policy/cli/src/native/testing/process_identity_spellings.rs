//! What:
//!  One process's identity written the ways a writer that is not this release writes it:
//!  a start tick with a leading zero,
//!  the platform's tag in capitals,
//!  a space before the birth,
//!  a leading zero on a process number,
//!  and a leading zero on a PID namespace.
//! Why:
//!  Every record of a live process is written in one spelling and read by comparing the recorded text with the identity
//!  the platform gives now, so a control of what a reader does with a second spelling of one live owner's identity must
//!  keep every other byte the writer wrote.
//!  The spellings are made here once, so each control respells the same value the same way and a reader that answers
//!  "gone" for one of them is named by the field it was given.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const padded = (text: string) => `0${text}`; // one number, two spellings
//! ```

/// What:
///  The birth `linux:8423337` with a leading zero on its start tick,
///  which is the same tick as a program that pads numbers writes it.
/// Why:
///   A start tick is a number,
///  and one number has one value however many zeros lead it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${tag}:0${tick}` // 'linux:8423337' -> 'linux:08423337'
/// ```
///
/// # Panics
///
/// When the text names no platform tag and start tick to respell,
/// or the tick already leads with a zero.
pub(crate) fn birth_with_a_leading_zero(birth: &str) -> String {
    let Some((tag, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} names no platform tag and start tick to respell");
    };
    assert!(
        !tick.starts_with('0'),
        "the birth {birth} already leads its start tick with a zero"
    );
    return format!("{tag}:0{tick}");
}

/// What:
///  The birth `linux:8423337` with its platform tag in capitals,
///  which is the same platform and the same start tick in another case.
/// Why:
///   The tag names a platform,
///  and a name read without regard to case is the same name;
///  a reader that takes a tag in another case for another platform reads the birth as text it cannot compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${tag.toUpperCase()}:${tick}` // 'linux:8423337' -> 'LINUX:8423337'
/// ```
///
/// # Panics
///
/// When the text names no platform tag,
/// or its tag has no letter whose case can change.
pub(crate) fn birth_with_its_platform_tag_in_capitals(birth: &str) -> String {
    let Some((tag, tick)) = birth.split_once(':') else {
        panic!("the birth {birth} names no platform tag to respell");
    };
    let capitals: String = tag.to_uppercase();
    assert!(
        capitals != tag,
        "the platform tag of the birth {birth} has no letter whose case can change"
    );
    return format!("{capitals}:{tick}");
}

/// What:
///  The birth `linux:8423337` behind one space,
///  which is the same birth written by a writer that pads the fields beside it.
/// Why:
///   A field of a record is separated from the next by a space in the records this wrapper reads from other programs,
///  and a writer that copies a field with that space attached writes the same birth.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ` ${birth}`
/// ```
pub(crate) fn birth_behind_a_space(birth: &str) -> String {
    return format!(" {birth}");
}

/// What:
///  The process number `4242` with a leading zero,
///  which is the same number as a program that pads numbers writes it.
/// Why:
///   A process number is a number,
///  and one number has one value however many zeros lead it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `0${pid}`
/// ```
pub(crate) fn process_number_with_a_leading_zero(number: i64) -> String {
    return format!("0{number}");
}

/// What:
///  The PID namespace `pid:[4026531836]` with a leading zero on its number,
///  which is the same namespace as a program that pads numbers writes it.
/// Why:
///   The namespace is the number inside the `/proc/self/ns/pid` link,
///  and one number has one value however many zeros lead it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `pid:[0${number}]` // 'pid:[4026531836]' -> 'pid:[04026531836]'
/// ```
///
/// # Panics
///
/// When the text is not the `/proc` link's `pid:[...]` spelling,
/// or its number already leads with a zero.
pub(crate) fn namespace_with_a_leading_zero(namespace: &str) -> String {
    let Some(number) = namespace
        .strip_prefix("pid:[")
        .and_then(|rest| { return rest.strip_suffix(']'); })
    else {
        panic!("the namespace {namespace} is not the pid:[...] text the /proc link reads");
    };
    assert!(
        !number.starts_with('0'),
        "the namespace {namespace} already leads its number with a zero"
    );
    return format!("pid:[0{number}]");
}
