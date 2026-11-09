//! What:
//!  Bytes as lowercase hexadecimal digits,
//!  and the two digits of one byte.
//! Why:
//!  Records,
//!  claim names,
//!  audit lines and index checksums all spell bytes this way,
//!  and each of them carried its own copy of the alphabet and the nibble split
//!  (issue 1672:
//!  seven sites across four packages),
//!  so one change of the spelling had to find every copy.
//!  One encoder answers for the whole crate;
//!  a place that must spell digits some other way records why beside its own table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const hexOfByte = (byte: number) => byte.toString(16).padStart(2, '0');
//! const hexOfBytes = (bytes: Uint8Array) => Buffer.from(bytes).toString('hex');
//! ```

/// The lowercase digits a byte is spelled with,
///  indexed by its value.
const HEXADECIMAL_DIGITS: &[u8; 16] = b"0123456789abcdef";

/// What:
///  Add the two lowercase hexadecimal digits of `byte` to `written`.
/// Why:
///  A caller that fills one string with many bytes asks for the digits and not for a string apiece,
///  which is what the alphabet and the nibble split cost when each copy kept them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeHex(written: string[], byte: number): void; // `${byte >> 4}` then `${byte & 15}`
/// ```
pub fn write_hexadecimal(written: &mut String, byte: u8) {
    written.push(char::from(HEXADECIMAL_DIGITS[usize::from(byte >> 4)]));
    written.push(char::from(HEXADECIMAL_DIGITS[usize::from(byte & 0x0f)]));
}

/// What:
///  The two lowercase hexadecimal digits of one byte.
/// Why:
///  A record that spells a single byte as an escape needs the digits of that one byte,
///  and not those of a run.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hexOfByte(0x0a); // '0a'
/// ```
pub fn hexadecimal_of_byte(byte: u8) -> String {
    let mut digits: String = String::with_capacity(2);
    write_hexadecimal(&mut digits, byte);
    return digits;
}

/// What:
///  The bytes as lowercase hexadecimal digits.
/// Why:
///   A record,
///  a claim's name and an audit line keep a byte string this way,
///  and every one of them asks the same question of it.
///  The capacity is the exact length,
///  so a long run is spelled in one pass over the bytes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// hexOfBytes(Buffer.from('ab')); // '6162'
/// ```
pub fn hexadecimal_of_bytes(bytes: &[u8]) -> String {
    let mut digits: String = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write_hexadecimal(&mut digits, *byte);
    }
    return digits;
}

/// The encoder's own controls hold it to the spelling `format!` gives.
#[cfg(test)]
#[path = "hexadecimal_of_bytes_tests.rs"]
mod tests;
