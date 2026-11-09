//! What:
//!  Controls for reading a file's extended attributes and giving them to its replacement:
//!  the value the person stored comes back,
//!  a file with none reads as none,
//!  and an attribute the system will not set stops the write.
//! Why:
//!  These are the calls that decide whether a fix keeps the person's data or deletes it,
//! and a namespace the system refuses must read as a refusal rather than as "nothing to carry".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await attributesOf(file, 'reading')).toEqual([{ name: 'user.draft', value: 'x' }]);
//! ```

/// The module under test.
use super::{ATTRIBUTE_ATTEMPTS, Attribute, AttributeRefusal, attributes_of, give_to, read_until_it_fits};
/// A disposable fixture and its removal.
use crate::test_support::{fixture, remove};
/// `CString` names the attribute the control plants.
use std::ffi::CString;
/// `OsStrExt` lends the path's bytes.
use std::os::unix::ffi::OsStrExt;
/// `MetadataExt` and `PermissionsExt` describe the file the control plants on.
use std::os::unix::fs::PermissionsExt;
/// `Path` and `PathBuf` are filesystem paths.
use std::path::{Path, PathBuf};

/// What:
///  Give `path` the extended attribute `name` with `value`;
///  false when the filesystem refuses it.
/// Why:
///  A control that plants an attribute must know whether the filesystem can hold one,
///  so a refusal is reported as a control that did not run rather than as a defect.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const planted = (path, name, value) => { try { setxattrSync(path, name, value); return true } catch { return false } };
/// ```
fn set_attribute(path: &Path, name: &[u8], value: &[u8]) -> bool {
    let (Ok(name_text), Ok(path_text)) = (CString::new(name), CString::new(path.as_os_str().as_bytes())) else {
        return false;
    };
    // SAFETY: both strings are NUL-terminated and outlive the call; `value` holds `value.len()` initialised bytes.
    let result: libc::c_int = unsafe {
        libc::lsetxattr(
            path_text.as_ptr(),
            name_text.as_ptr(),
            value.as_ptr().cast(),
            value.len(),
            0,
        )
    };
    return result == 0;
}

/// What:
///  A fresh fixture and the one file in it,
///  carrying the attribute the control planted,
///  or nothing when the filesystem refuses that attribute.
/// Why:
///  Every control here plants one,
///  and a filesystem without them has nothing to check;
///  the control then says so rather than passing on an empty list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // const planted = plant('xattr-read', 'user.draft', 'only-copy-of-draft');
/// // if (planted === undefined) return; // XATTR CONTROL NOT RUN
/// ```
fn plant(name: &str, attribute: &[u8], value: &[u8]) -> Option<(PathBuf, PathBuf)> {
    let root: PathBuf = fixture(name);
    let path: PathBuf = root.join("kept.txt");
    std::fs::write(&path, b"x\n").expect("a file");
    if !set_attribute(path.as_path(), attribute, value) {
        eprintln!(
            "XATTR CONTROL NOT RUN: this filesystem refuses user extended attributes ({})",
            std::io::Error::last_os_error()
        );
        remove(root.as_path());
        return None;
    }
    return Some((root, path));
}

/// The attribute a control plants,
///  and the value that is the person's only copy.
const DRAFT: &[u8] = b"user.draft";
/// The value of `DRAFT`.
const DRAFT_VALUE: &[u8] = b"only-copy-of-draft";

/// What:
///  The attributes of a file are read back with the name and value the person stored.
/// Why:
///  This is the read the installer relies on before it writes anything.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(await attributesOf(file, 'reading')).toContainEqual({ name: 'user.draft', value: 'only-copy-of-draft' });
/// ```
#[test]
fn an_attribute_is_read_with_the_name_and_value_it_was_given() {
    let Some((root, path)) = plant("xattr-read", DRAFT, DRAFT_VALUE) else {
        return;
    };
    let found: Vec<Attribute> = attributes_of(path.as_path(), "reading a file").expect("read");
    let carried: Option<&Attribute> = found.iter().find(|attribute| { return attribute.name == DRAFT; });
    let Some(draft) = carried else {
        panic!("the attribute the file carries is listed: {found:?}");
    };
    assert_eq!(draft.value, DRAFT_VALUE);
    assert_eq!(draft.display_name(), "user.draft");
    // The list is ordered by name, so two files' lists compare as one sequence.
    let mut sorted: Vec<&Attribute> = found.iter().collect();
    sorted.sort();
    assert_eq!(sorted, found.iter().collect::<Vec<&Attribute>>());
    remove(root.as_path());
}

/// What:
///  A file nothing was planted on reads with no attribute of the person's.
/// Why:
///  The negative control of the reading:
///  the values the control plants are found because they are there,
///  not because the reading invents them,
///  and a system-assigned label is not mistaken for one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect((await attributesOf(plain, 'reading')).filter((a) => a.name.startsWith('user.'))).toEqual([]);
/// ```
#[test]
fn a_file_nothing_was_planted_on_reads_no_user_attribute() {
    let root: PathBuf = fixture("xattr-none");
    let path: PathBuf = root.join("plain.txt");
    std::fs::write(&path, b"x\n").expect("a file");
    let found: Vec<Attribute> = attributes_of(path.as_path(), "reading a file").expect("read");
    let planted: Vec<&Attribute> = found
        .iter()
        .filter(|attribute| { return attribute.name.starts_with(b"user."); })
        .collect();
    assert_eq!(planted, Vec::<&Attribute>::new(), "{found:?}");
    // A name nothing is at is a refusal, not an empty list: the fix must not carry on as if the file held nothing.
    let absent: String = attributes_of(root.join("gone.txt").as_path(), "reading a file").expect_err("no file");
    assert!(absent.starts_with("reading a file failed ("), "{absent}");
    remove(root.as_path());
}

/// What:
///  An attribute the system will not set stops the write,
///  and the refusal names the attribute.
/// Why:
///  A namespace this system does not know is the one refusal a control can produce everywhere,
///  and the installer must treat it as "I cannot keep your data" rather than as "nothing to keep".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(giveTo(file, [{ name: 'bogus.never', value: 'x' }])).toEqual({ name: 'bogus.never', value: 'x' });
/// ```
#[test]
fn an_attribute_the_system_will_not_set_is_refused_by_name() {
    let root: PathBuf = fixture("xattr-refused");
    let path: PathBuf = root.join("kept.txt");
    std::fs::write(&path, b"x\n").expect("a file");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("mode");
    let file: std::fs::File = std::fs::OpenOptions::new().write(true).open(&path).expect("open");
    let impossible: Attribute = Attribute {
        name: b"bogus.never".to_vec(),
        value: b"x".to_vec(),
    };
    let refusal: AttributeRefusal =
        give_to(&file, [impossible.clone()].as_slice()).expect_err("no such namespace");
    let AttributeRefusal::NotGiven { attribute: refused, reason: why } = refusal else {
        panic!("the refusal names the attribute the system would not set: {refusal:?}");
    };
    assert_eq!(refused, impossible);
    assert_ne!(why.raw_os_error(), None, "the system's own reason is kept: {why}");
    assert_eq!(refused.display_name(), "bogus.never");
    // Positive control: the call does set an attribute this system knows.
    let Some((known_root, known_path)) = plant("xattr-given", DRAFT, b"first") else {
        remove(root.as_path());
        return;
    };
    let known: std::fs::File = std::fs::OpenOptions::new().write(true).open(&known_path).expect("open");
    let wanted: Attribute = Attribute {
        name: DRAFT.to_vec(),
        value: b"second".to_vec(),
    };
    // The list is the file's own with the wanted value in place of the one it holds, as the fix's list is the
    // original's: every attribute the file carries is named, so nothing is taken off it.
    let mut asked: Vec<Attribute> = attributes_of(known_path.as_path(), "reading a file").expect("read");
    asked.retain(|attribute| {
        return attribute.name != wanted.name;
    });
    asked.push(wanted.clone());
    assert!(give_to(&known, asked.as_slice()).is_ok(), "the value is given");
    let read_back: Vec<Attribute> = attributes_of(known_path.as_path(), "reading a file").expect("read");
    assert!(
        read_back.contains(&wanted),
        "the value the replacement was given is the one it holds: {read_back:?}"
    );
    remove(root.as_path());
    remove(known_root.as_path());
}

/// What:
///  An extended attribute the replacement carries and the original never did is taken off it.
/// Why:
///  A directory's default access-control list is copied onto every file created inside it,
///  and the replacement is created inside the Git directory:
///  a replacement that kept the inherited list would grant an account read access the original
///       never gave,
///  which no comparison of the displaced original can see.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // setxattr(replacement, 'user.inherited', 'from the directory'); giveTo(replacement, []); expect(getxattr(replacement, 'user.inherited')).toBeUndefined()
/// ```
#[test]
fn an_attribute_the_original_never_had_is_taken_off_the_replacement() {
    const INHERITED: &[u8] = b"user.inherited";
    let root: PathBuf = fixture("xattr-not-inherited");
    let path: PathBuf = root.join("replacement.txt");
    std::fs::write(&path, b"x\n").expect("a file");
    if !set_attribute(path.as_path(), INHERITED, b"from the directory") {
        eprintln!(
            "XATTR CONTROL NOT RUN: this filesystem refuses user extended attributes ({})",
            std::io::Error::last_os_error()
        );
        remove(root.as_path());
        return;
    }
    let file: std::fs::File = std::fs::OpenOptions::new().write(true).open(&path).expect("open");
    // The list is what the original carried: every attribute of the file but the one the directory planted on it.
    let kept: Vec<Attribute> = attributes_of(path.as_path(), "reading a file")
        .expect("read")
        .into_iter()
        .filter(|attribute| {
            return attribute.name != INHERITED;
        })
        .collect();
    give_to(&file, kept.as_slice()).expect("every attribute the original never had is taken off");
    let found: Vec<Attribute> = attributes_of(path.as_path(), "reading a file").expect("read");
    let user: Vec<&Attribute> = found
        .iter()
        .filter(|attribute| {
            return attribute.name.starts_with(b"user.");
        })
        .collect();
    assert_eq!(user, Vec::<&Attribute>::new(), "the inherited attribute is gone: {found:?}");
    remove(root.as_path());
}

/// What:
///  One answer as another process can leave it between the length question and the reading call:
///  empty when
///       the length is asked,
///  and holding the person's bytes by the time the reading runs.
/// Why:
///  A value that grows after the size question is exactly what makes a zero-sized buffer come back with a
///  length instead of bytes,
///  and the control plants that growth rather than racing a real writer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GrowingAnswer = { after: Buffer; asked: number };
/// ```
struct GrowingAnswer {
    /// The bytes the answer holds once it has grown.
    after: Vec<u8>,
    /// How many times the length was asked.
    asked: u32,
}

/// What:
///  The length question of a growing answer:
///  nothing before the first reading,
///  the grown length after it.
/// Why:
///  This is what the kernel answers when the attribute was empty at the first question and nonempty at the
///  second.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const growingLength = (answer) => ++answer.asked === 1 ? 0 : answer.after.length;
/// ```
fn growing_length(answer: &mut GrowingAnswer) -> std::io::Result<usize> {
    answer.asked += 1;
    if answer.asked == 1 {
        return Ok(0);
    }
    return Ok(answer.after.len());
}

/// What:
///  The reading of a growing answer:
///  a buffer that cannot hold it copies nothing and reports the length it needs.
/// Why:
///  A zero-sized buffer is another length question to the kernel,
///  which is the answer the loop must not
///  mistake for an empty value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const growingRead = (answer, buffer) => buffer.length < answer.after.length ? answer.after.length : copy(buffer);
/// ```
fn growing_read(answer: &mut GrowingAnswer, buffer: &mut [u8]) -> std::io::Result<usize> {
    if buffer.len() < answer.after.len() {
        return Ok(answer.after.len());
    }
    buffer[..answer.after.len()].copy_from_slice(answer.after.as_slice());
    return Ok(answer.after.len());
}

/// What:
///  A value that grows between the length question and the reading call is read whole.
/// Why:
///  Truncating the empty buffer to the length the second call reports cannot grow it,
///  so the person's
///       nonempty value would come back as empty,
///  and a displaced file holding newer data would compare equal to the
///       state the fix recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(await readUntilItFits({ after: 'only-copy-of-draft', asked: 0 })).toEqual('only-copy-of-draft')
/// ```
#[test]
fn a_value_that_grows_between_the_length_question_and_the_read_is_read_whole() {
    let mut answer: GrowingAnswer = GrowingAnswer {
        after: DRAFT_VALUE.to_vec(),
        asked: 0,
    };
    let read: Vec<u8> = read_until_it_fits(
        &mut answer,
        "its value kept changing while it was read",
        growing_length,
        growing_read,
    )
    .expect("the value is read once its length fits");
    assert_eq!(read, DRAFT_VALUE, "the grown value is the one that comes back");
    assert_eq!(answer.asked, 2, "the length was asked again after the buffer did not fit");
}

/// What:
///  A list that grows between the length question and the reading call is read whole.
/// Why:
///  The names of a file's attributes are read with the same two calls,
///  and an empty answer that grows would
///  otherwise read as a file with no attributes at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(await readUntilItFits({ after: 'user.draft\0', asked: 0 })).toEqual('user.draft\0')
/// ```
#[test]
fn a_list_that_grows_between_the_length_question_and_the_read_is_read_whole() {
    const LISTED: &[u8] = b"user.draft\0";
    let mut answer: GrowingAnswer = GrowingAnswer {
        after: LISTED.to_vec(),
        asked: 0,
    };
    let read: Vec<u8> = read_until_it_fits(
        &mut answer,
        "the list of its extended attributes kept changing while it was read",
        growing_length,
        growing_read,
    )
    .expect("the list is read once its length fits");
    assert_eq!(read, LISTED, "the grown list is the one that comes back");
    assert_eq!(answer.asked, 2, "the length was asked again after the buffer did not fit");
}

/// What:
///  The length question of an answer that is always larger than the buffer it was given.
/// Why:
///  A file whose attributes never settle has no list and no value this process can name,
///  so the reading
///       must stop rather than ask for ever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const neverFittingLength = () => 0;
/// ```
fn never_fitting_length(_answer: &mut GrowingAnswer) -> std::io::Result<usize> {
    return Ok(0);
}

/// The reading of an answer that is always larger than the buffer it was given.
fn never_fitting_read(answer: &mut GrowingAnswer, buffer: &mut [u8]) -> std::io::Result<usize> {
    answer.asked += 1;
    return Ok(buffer.len() + answer.after.len() + 1);
}

/// What:
///  An answer that never fits its buffer is refused after the attempts the reading allows.
/// Why:
///  A file whose attributes keep changing must cost a refusal,
///  never a loop;
///  the count is the constant the
///       reading is built on,
///  so a change to it shows here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // expect(await readUntilItFits(neverFitting)).rejects.toThrow('kept changing'); expect(attempts).toBe(ATTEMPTS)
/// ```
#[test]
fn an_answer_that_never_fits_is_refused_after_the_attempts_it_allows() {
    let mut answer: GrowingAnswer = GrowingAnswer {
        after: b"still growing".to_vec(),
        asked: 0,
    };
    let refusal: std::io::Error = read_until_it_fits(
        &mut answer,
        "its value kept changing while it was read",
        never_fitting_length,
        never_fitting_read,
    )
    .expect_err("an answer that never fits is refused");
    assert_eq!(refusal.to_string(), "its value kept changing while it was read");
    assert_eq!(answer.asked, ATTRIBUTE_ATTEMPTS, "the reading stops at its own bound");
}
