//! Controls for the one encoder of bytes as hexadecimal digits.

use super::*;

/// What:
///  Every byte is spelled as the two lowercase digits `format!` gives it.
/// Why:
///  The encoder replaced copies whose spelling `format!("{byte:02x}")` was,
///  and a record,
///  a claim's name or a hash that another program reads must keep the same digits:
///  lowercase,
///  two characters,
///  and a leading zero where the byte is below sixteen.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let byte = 0; byte <= 255; byte += 1) expect(hexOfByte(byte)).toBe(byte.toString(16).padStart(2, '0'));
/// ```
#[test]
fn every_byte_keeps_the_two_lowercase_digits_it_had() {
    for value in 0..=u8::MAX {
        let digits: String = hexadecimal_of_byte(value);
        assert_eq!(digits, format!("{value:02x}"), "byte {value} keeps its spelling");
        assert_eq!(digits.len(), 2, "byte {value} is two digits");
        let mut written: String = String::new();
        write_hexadecimal(&mut written, value);
        assert_eq!(written, digits, "writing the digits and asking for them agree for byte {value}");
    }
}

/// What:
///  A run of bytes is the digits of each byte,
///  in order,
///  and nothing for an empty run.
/// Why:
///  A path's bytes,
///  a digest and an index checksum are spelled by the same function,
///  and a reader splits the result into pairs:
///  a missing or extra digit would name another byte.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(hexOfBytes(Buffer.from([0x00, 0x0f, 0x10, 0xff]))).toBe('000f10ff');
/// ```
#[test]
fn a_run_of_bytes_is_each_byte_in_order() {
    assert_eq!(hexadecimal_of_bytes(&[]), "", "an empty run has no digits");
    assert_eq!(hexadecimal_of_bytes(&[0x00]), "00", "a leading zero is kept");
    assert_eq!(hexadecimal_of_bytes(&[0x0f, 0x10]), "0f10", "the order of the bytes is the order of the digits");
    assert_eq!(hexadecimal_of_bytes(&[0xff]), "ff", "the digits are lowercase");
    let bytes: Vec<u8> = (0..=u8::MAX).collect();
    assert_eq!(hexadecimal_of_bytes(bytes.as_slice()), bytes.iter().map(|byte| return format!("{byte:02x}")).collect::<String>(), "a whole run is the digits of each byte");
}
