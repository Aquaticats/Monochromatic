//! What:
//!  A path written into a record as hexadecimal bytes,
//! and the plain text record that a path was written as before.
//! Why:
//!  A record is text,
//! so a path that goes into one as `display()` loses every byte that is not UTF-8,
//! and a reader
//!  that compares the value with the directory it came from then refuses what the writer accepted.
//! A record now carries the path's bytes in hexadecimal under a second field,
//! and a reader prefers it;
//! the plain field stays beside it,
//! so a record written by an earlier release is still read,
//! and so is one written by this release for a repository whose path is text.
//! Log messages are not of this kind:
//! a byte lost in a line the person reads changes nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! record.gitDir = path.toString(); record.gitDirHex = Buffer.from(path).toString('hex')
//! ```

use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

/// What:
///  The end of the second field of a path:
/// the plain field's name and this make `gitDirHex`,
/// which holds the path's bytes.
/// Why:
///  `write_field` and `read_field` both build the name from the one suffix,
/// so a reader and a writer cannot disagree about where the bytes are.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const HEX_SUFFIX = 'Hex';
/// ```
pub const HEX_SUFFIX: &str = "Hex";

/// What:
///  The words of a line that names a Git directory by its bytes,
/// in a record of lines.
/// Why:
///   The work queue's journal,
/// the note that names an instance's worktree and a removal's snapshot all name one directory,
/// and each of them separates its lines by line breaks,
/// so the bytes take the same `key=value` form in all three.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const GITDIR_HEX_PREFIX = 'gitdirHex=';
/// ```
pub const GITDIR_HEX_PREFIX: &str = "gitdirHex=";

/// What:
///  The path's bytes as lowercase hexadecimal digits.
/// Why:
///  `write_field` stores this,
/// and it is the only form of a path that a record can hold without losing a byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathBytesHex = (path: string) => Buffer.from(path).toString('hex');
/// ```
pub fn hex_of_path(path: &Path) -> String {
    return hex_of_bytes(crate::path_bytes::path_bytes_of(path));
}

/// What:
///  The bytes as lowercase hexadecimal digits.
/// Why:
///   `hex_of_path` writes a path this way,
/// and a list of paths in one line of a record needs the same spelling for each of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathBytesHex = (bytes: Uint8Array) => Buffer.from(bytes).toString('hex');
/// ```
pub fn hex_of_bytes(bytes: &[u8]) -> String {
    // What: the one encoder of bytes as hexadecimal digits answers here too (issue 1672).
    return crate::hexadecimal_of_bytes::hexadecimal_of_bytes(bytes);
}

/// What:
///  The bytes the hexadecimal digits spell,
/// nothing when they are not an even number of hexadecimal digits.
/// Why:
///   A repository-relative path is bytes and not a native path,
/// so a record that holds one decodes those bytes without ever building a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const bytesOfHex = (hex: string) => hex.length % 2 === 0 && /^[0-9a-f]*$/i.test(hex) ? Buffer.from(hex, 'hex') : undefined;
/// ```
pub fn bytes_of_hex(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let mut bytes: Vec<u8> = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks(2) {
        bytes.push(u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?);
    }
    return Some(bytes);
}

/// What:
///  The path whose bytes the digits spell,
/// nothing when they are not an even number of hexadecimal digits.
/// Why:
///  `read_field` turns the second field back into the directory the writer named,
/// and a field that is not of that form is no path at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathOfHex = (hex: string) => hex.length % 2 === 0 && /^[0-9a-f]*$/i.test(hex) ? Buffer.from(hex, 'hex').toString() : undefined;
/// ```
pub fn path_of_hex(hex: &str) -> Option<PathBuf> {
    use std::os::unix::ffi::OsStringExt;
    return Some(PathBuf::from(std::ffi::OsString::from_vec(bytes_of_hex(hex)?)));
}

/// What:
///  The path whose bytes are given,
/// whatever bytes they are.
/// Why:
///   A value of a record's line is decoded to bytes before it is a path,
/// so the one place a byte string becomes a path answers here rather than at each reader.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathOfBytes = (bytes: Uint8Array) => Buffer.from(bytes).toString()
/// ```
pub fn path_of_bytes(bytes: &[u8]) -> PathBuf {
    return PathBuf::from(crate::path_bytes::os_from_path_bytes(bytes));
}

/// What:
///  The first line of a record of `key=value` lines whose values are written with [`encode_field`]:
///  the format it names is the one where every value of a line is escaped.
/// Why:
///   A reader has to know whether the values it reads were escaped,
///  and a record written before this format has no such line,
///  so its values are read as they are.
/// The line names a key no field of any record uses, so every reader of these records skips it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const FIELD_ENCODING_HEADER = 'format=2\n'
/// ```
pub const FIELD_ENCODING_HEADER: &str = "format=2\n";

/// What:
///  Whether a record's values were written with [`encode_field`],
///  which its first line says.
/// Why:
///   A record an earlier release wrote has no first line of this shape,
///  and its values are the text it wrote,
///  so decoding them would name another path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const carriesEncodedFields = (text: string) => text.startsWith(FIELD_ENCODING_HEADER)
/// ```
pub fn carries_encoded_fields(text: &str) -> bool {
    return text.starts_with(FIELD_ENCODING_HEADER);
}

/// What:
///  Whether a byte of a value is written as itself:
///  the bytes from space to `~`, except `%`,
///  which begins an escape.
/// Why:
///   Those are the bytes that keep a line a line and a field a field,
///  and a name of them reads as it is in a record a person looks at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isKeptVerbatim = (byte: number) => byte !== 0x25 && byte >= 0x20 && byte <= 0x7e
/// ```
fn is_kept_verbatim(byte: u8) -> bool {
    return byte != b'%' && (byte.is_ascii_graphic() || byte == b' ');
}

/// The byte a hexadecimal digit spells, nothing when the character is no digit.
fn byte_of_digit(digit: u8) -> Option<u8> {
    return match digit {
        b'0'..=b'9' => Some(digit - b'0'),
        b'A'..=b'F' => Some(digit - b'A' + 10),
        b'a'..=b'f' => Some(digit - b'a' + 10),
        _ => None,
    };
}

/// What:
///  One value of a line of a record,
///  as one line that holds every byte of it:
/// every byte a line can carry as itself stays,
/// and every other byte, `%` included,
///  is written as `%` and two uppercase hexadecimal digits.
/// Why:
///   A record of lines separates its lines by line breaks and its fields by tabs,
///  so a value that holds one of those would end its line or its field early,
///  and the text after it would be read as a field nobody wrote;
///  a commit's path may hold any byte but NUL,
///  and so may a message or a name a record keeps.
/// The escape is undone by [`decode_field`],
///  and it is applied at the one place a value becomes a line,
///  never to the bytes a reader acts on.
/// A record's hexadecimal fields stay beside it,
/// so a release that predates this format still reads every path by its bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const encodeField = (bytes: Uint8Array) => [...bytes].map((byte) => isKeptVerbatim(byte) ? String.fromCharCode(byte) : `%${byte.toString(16).toUpperCase().padStart(2, '0')}`).join('')
/// ```
pub fn encode_field(bytes: &[u8]) -> String {
    let mut encoded: String = String::with_capacity(bytes.len());
    for byte in bytes {
        if is_kept_verbatim(*byte) {
            encoded.push(char::from(*byte));
            continue;
        }
        encoded.push_str(format!("%{byte:02X}").as_str());
    }
    return encoded;
}

/// What:
///  The bytes a value written with [`encode_field`] spells,
///  and nothing when an escape is not two hexadecimal digits.
/// Why:
///   A value that does not decode is damage,
///  and a reader that took it for text would act on a path nobody wrote.
/// A `%` that begins no escape is damage too,
///  because the writer never leaves one unescaped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const decodeField = (text: string) => { const out = []; for (let i = 0; i < text.length; i += 1) { if (text[i] !== '%') { out.push(...Buffer.from(text[i])); continue } const pair = text.slice(i + 1, i + 3); if (!/^[0-9A-Fa-f]{2}$/.test(pair)) return undefined; out.push(parseInt(pair, 16)); i += 2 } return Buffer.from(out) }
/// ```
pub fn decode_field(text: &str) -> Option<Vec<u8>> {
    let bytes: &[u8] = text.as_bytes();
    let mut decoded: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut index: usize = 0;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }
        let pair: &[u8] = bytes.get(index + 1..index + 3)?;
        decoded.push(byte_of_digit(pair[0])? * 16 + byte_of_digit(pair[1])?);
        index += 3;
    }
    return Some(decoded);
}

/// What:
///  The text a path's bytes spell in the plain field of a record,
///  with a byte that is no text replaced by the character every reader of text replaces it with.
/// Why:
///   A record written before this format kept its paths as plain text,
///  and that text was the lossy spelling of the bytes beside it;
/// a reader that compares the two fields can tell whether the text names the same value,
///  and refuses the record when it does not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const lossyTextOf = (bytes: Uint8Array) => new TextDecoder('utf-8').decode(bytes)
/// ```
pub fn lossy_text_of(bytes: &[u8]) -> String {
    return String::from_utf8_lossy(bytes).into_owned();
}

/// What:
///  The path as text when one `key=value` line of a line-oriented record can hold it,
/// nothing when it cannot.
/// Why:
///   A record of lines separates them by line breaks and fields by tabs,
/// so a path that holds one of those would end its line or its field early and name elsewhere;
/// `hex_of_path` is the form such a record always takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const textInOneLine = (path: string) => /[\n\r\t]/.test(path) ? undefined : path;
/// ```
pub fn text_in_one_line(path: &Path) -> Option<&str> {
    let text: &str = path.to_str()?;
    if text.contains(['\n', '\r', '\t']) {
        return None;
    }
    return Some(text);
}

/// What:
///  The path a record of lines holds for one name:
/// its bytes from the field that holds them,
/// its text from the field that holds that,
/// and nothing when the record has neither.
/// Why:
///   Every reader of these records keeps one rule:
/// a field of bytes that is absent is a record an earlier release wrote,
/// which holds the text,
/// and a field of bytes that is there but does not decode is damage;
/// a reader that takes damage for absence acts on a different path than the writer recorded.
/// A record whose bytes field is damaged is therefore an error,
/// which its reader rejects.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathOfTwoFields = (bytes, text) => bytes === undefined ? text : (pathOfHex(bytes) ?? damaged())
/// ```
pub fn path_of_the_two_fields(bytes: Option<&str>, text: Option<&str>) -> Result<Option<PathBuf>, String> {
    if let Some(hexadecimal) = bytes {
        return match path_of_hex(hexadecimal) {
            Some(path) => Ok(Some(path)),
            None => Err(String::from("the field of bytes is not the bytes of a path")),
        };
    }
    return Ok(text.map(PathBuf::from));
}

/// What:
///  Put `path` into the record `fields` under `key` as plain text and under `key` followed by
/// `HEX_SUFFIX` as its bytes.
/// Why:
///   A reader that predates the second field reads the plain text,
/// which is the whole path whenever the path is text,
/// and this release's reader takes the bytes,
///  which are the whole path always.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// fields[key] = path.toString(); fields[key + HEX_SUFFIX] = pathBytesHex(path);
/// ```
pub fn write_field(fields: &mut Map<String, Value>, key: &str, path: &Path) {
    fields.insert(String::from(key), Value::from(path.display().to_string()));
    fields.insert(format!("{key}{HEX_SUFFIX}"), Value::from(hex_of_path(path)));
}

/// What:
///  The path a record holds under `key`:
/// its bytes from the second field when the record has them,
/// the plain text otherwise.
/// Why:
///   A record written before the second field existed holds only text,
/// and so does one whose name the plain field spells exactly
/// (a change of a commit request,
/// which leaves the bytes out for names that are text);
/// reading the bytes first means the plain text is never used for a path that has them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readPathField = (record, key) => record[key + HEX_SUFFIX] !== undefined ? pathOfHex(record[key + HEX_SUFFIX]) : record[key];
/// ```
pub fn read_field(value: &Value, key: &str) -> Result<PathBuf, String> {
    match value.get(format!("{key}{HEX_SUFFIX}")) {
        Some(Value::String(hex)) => {
            return match path_of_hex(hex.as_str()) {
                Some(path) => Ok(path),
                None => Err(format!("the {key} field is not the bytes of a path")),
            };
        }
        Some(_not_text) => return Err(format!("the {key} field is not the bytes of a path")),
        None => {}
    }
    return match value.get(key).and_then(Value::as_str) {
        Some(text) => Ok(PathBuf::from(text)),
        None => Err(format!("the record has no {key} field")),
    };
}

/// What:
///  One field of a path,
/// as the pair of `key` and `keyHex` fragments that fit into a hand-built record:
/// the text when text spells every byte of the path,
/// its bytes otherwise.
/// Why:
///   A repository path appears in a record for every entry a copy installs,
/// so each one takes one field:
/// a name that is text is written as it reads,
/// and a name that is not text only as its bytes,
/// which a reader that knows the bytes form takes and which one that does not refuses.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const field = (key, path) => isText(path) ? `"${key}":${JSON.stringify(text)}` : `"${key}${HEX_SUFFIX}":"${hex}"`;
/// ```
pub fn field_fragment(key: &str, path: &[u8]) -> String {
    return match std::str::from_utf8(path) {
        Ok(text) => format!("\"{key}\":{}", crate::json_record::quote(text)),
        Err(_not_text) => format!("\"{key}{HEX_SUFFIX}\":{}", crate::json_record::quote(hex_of_bytes(path).as_str())),
    };
}

/// What:
///  One list of repository paths,
/// as the pair of `key` and `keyHex` fragments that fit into a hand-built record,
/// under the same rule as [`field_fragment`]:
/// the text array when text spells every byte of every path,
/// the array of bytes otherwise.
/// Why:
///   A journal and an install log name every claimed and every created path,
/// so the text form is kept for the lists whose bytes it spells exactly,
/// and only a list that holds a name that is not text takes the bytes form.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const list = (key, paths) => paths.every(isText) ? `"${key}":${JSON.stringify(texts)}` : `"${key}${HEX_SUFFIX}":${JSON.stringify(hexes)}`;
/// ```
pub fn list_fragment(key: &str, paths: &[Vec<u8>]) -> String {
    let mut all_text: Vec<&str> = Vec::with_capacity(paths.len());
    let mut every_name_is_text: bool = true;
    for path in paths {
        let Ok(text) = std::str::from_utf8(path) else {
            every_name_is_text = false;
            break;
        };
        all_text.push(text);
    }
    // The text array is kept for a list whose every name it spells; one name that is not text moves the list to bytes.
    if !every_name_is_text {
        let mut hexes: Vec<String> = Vec::with_capacity(paths.len());
        for path in paths {
            hexes.push(crate::json_record::quote(hex_of_bytes(path).as_str()));
        }
        return format!("\"{key}{HEX_SUFFIX}\":[{}]", hexes.join(","));
    }
    let mut quoted: Vec<String> = Vec::with_capacity(all_text.len());
    for text in all_text {
        quoted.push(crate::json_record::quote(text));
    }
    return format!("\"{key}\":[{}]", quoted.join(","));
}

/// What:
///  One list of repository paths read from a record,
/// its bytes from the field that holds them,
/// its text from the field that holds that,
/// and nothing when the record has neither.
/// Why:
///   A list written before the bytes form existed holds text and is read as it is,
/// a list of bytes is read as those bytes,
/// and a list of bytes that does not decode is damage,
/// which no reader takes for text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readPathList = (record, key) => record[key + HEX_SUFFIX] !== undefined ? decodeEach(record[key + HEX_SUFFIX]) : texts(record[key]);
/// ```
pub fn read_path_list(value: &Value, key: &str) -> Option<Vec<Vec<u8>>> {
    if let Some(hexadecimal) = value.get(format!("{key}{HEX_SUFFIX}")) {
        let mut paths: Vec<Vec<u8>> = Vec::with_capacity(hexadecimal.as_array()?.len());
        for item in hexadecimal.as_array()? {
            paths.push(bytes_of_hex(item.as_str()?)?);
        }
        return Some(paths);
    }
    let mut paths: Vec<Vec<u8>> = Vec::with_capacity(value.get(key)?.as_array()?.len());
    for item in value.get(key)?.as_array()? {
        paths.push(item.as_str()?.as_bytes().to_vec());
    }
    return Some(paths);
}

/// What:
///  One repository path read from a record's `key` and `keyHex` fields,
/// under the same rule as [`read_path_list`].
/// Why:
///   An install log names one created path per entry,
/// so each entry carries one field of one of the two forms.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const readPathField = (record, key) => record[key + HEX_SUFFIX] !== undefined ? bytesOfHex(record[key + HEX_SUFFIX]) : Buffer.from(record[key]);
/// ```
pub fn read_path_field(value: &Value, key: &str) -> Option<Vec<u8>> {
    if let Some(hexadecimal) = value.get(format!("{key}{HEX_SUFFIX}")) {
        return bytes_of_hex(hexadecimal.as_str()?);
    }
    return Some(value.get(key)?.as_str()?.as_bytes().to_vec());
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "path_bytes_in_records_tests.rs"]
mod tests;
