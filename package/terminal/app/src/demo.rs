//! Demo VT stream for the first prototype checkpoint.

/// What:
///      `pub fn demo_vt() -> Vec<u8>` returns owned bytes.
///  `Vec<u8>` is a
///           growable byte array;
///  siblings are borrowed slices `&[u8]` and fixed
///           arrays `[u8; N]`.
/// Why:
///       The prototype feeds deterministic VT content until PTY I/O is added.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function demoVt(): Uint8Array {
///   return new TextEncoder().encode(lines.join("\r\n"));
/// }
/// ```
pub fn demo_vt() -> Vec<u8> {
    // What:     `let mut bytes = Vec::new()` creates a mutable owned byte array.
    //           `Vec` is chosen over `String` because VT streams are bytes, not
    //           guaranteed UTF-8 text.
    // Why:      Escape sequences and future PTY chunks are byte-oriented.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const chunks: string[] = [];
    // ```
    let mut bytes = Vec::new();
    // What:     `for index in 0..160` loops over a half-open integer range.
    //           `0..160` yields 0 through 159.
    // Why:      Enough rows are generated to make scrollback visible immediately.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (let index = 0; index < 160; index += 1) { ... }
    // ```
    for index in 0..160 {
        // What:     `let color = ...` chooses an ANSI color number as a string slice.
        //           `&str` is a borrowed string view; `String` would allocate.
        // Why:      Reusing static color snippets keeps demo generation cheap.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const color = ["31", "32", "33", "34", "35", "36"][index % 6];
        // ```
        let color = match index % 6 {
            0 => "31",
            1 => "32",
            2 => "33",
            3 => "34",
            4 => "35",
            _ => "36",
        };
        // What:     `format!(...)` builds one owned `String` from literals and
        //           values. The `\x1b` bytes are ANSI escape introducers.
        // Why:      Each line demonstrates colors, bold text, wrapping, and scrollback.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const line = `\x1b[${color}m${index}\x1b[0m ...\r\n`;
        // ```
        let line = format!(
            "\x1b[{color}mrow {index:03}\x1b[0m  libghostty-vt scrollback  \x1b[1mslint owns pixels\x1b[0m  fractional row demo\r\n",
        );
        // What:     `bytes.extend_from_slice(line.as_bytes())` appends borrowed bytes
        //           from the owned `String`. `.as_bytes()` borrows UTF-8 storage.
        // Why:      The terminal engine accepts bytes, not Rust strings.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // chunks.push(line);
        // ```
        bytes.extend_from_slice(line.as_bytes());
    }
    // What:     `bytes` without a trailing semicolon is the function's return value.
    // Why:      Hand the complete VT stream to the caller.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return new TextEncoder().encode(chunks.join(""));
    // ```
    return bytes
}
