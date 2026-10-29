//! What:
//!     A byte cursor over the pattern that skips verbose-mode whitespace and comments.
//! Why:
//!      This file is the Rust module that groups the cursor implementation,
//!  so the
//!          compiler gives those items one namespace and sibling modules can import that name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // module cursor: see exported functions and types below.
//! ```

/// A forward cursor over the pattern bytes.
///
/// What:
///  holds the source bytes and a current offset,
///  and knows how to skip the
/// always-on verbose-mode noise (unescaped whitespace and first-column `#`
/// comments).
///  Why:
///  every grammar rule reads through this one cursor,
///  so the
/// verbose-mode rules live in exactly one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Cursor = {
///   // fields documented in Rust above
/// };
/// ```
pub struct Cursor<'a> {
    /// What:
    ///     The pattern as raw bytes;
    ///  matching and parsing are both byte-oriented.
    /// Why:
    ///      `src` stores the pattern as raw bytes;
    ///  matching and parsing are both
    ///          byte-oriented,
    ///  so matcher code reads that precomputed state by name instead of
    ///          recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// src: 'a [u8];
    /// ```
    src: &'a [u8],
    /// What:
    ///     Current read offset into `src`.
    /// Why:
    ///      `pos` stores current read offset into `src`,
    ///  so matcher code reads that
    ///          precomputed state by name instead of recomputing or passing it separately.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// pos: number;
    /// ```
    pos: usize,
}

/// What:
///     Reading and verbose-mode skipping over the pattern bytes.
/// Why:
///      The program attaches these functions to the named Rust type so callers can use
///          method syntax.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // Methods are written inside a class or as functions that take the value.
/// ```
impl<'a> Cursor<'a> {
    /// Builds a cursor at the start of `src`.
    ///
    /// What:
    ///  offset zero over the given bytes.
    ///  Why:
    ///  parsing begins at the front.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function new(src: 'a [u8]): Self {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn new(src: &'a [u8]) -> Self {
        return Cursor { src, pos: 0 }
    }

    /// Returns the current byte offset.
    ///
    /// What:
    ///  the index used for error positions.
    ///  Why:
    ///  `CompileError::Syntax`
    /// reports where a problem was found.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function pos(): number {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn pos(&self) -> usize {
        return self.pos
    }

    /// Reports whether the cursor is at or past the end.
    ///
    /// What:
    ///  `pos >= len`.
    ///  Why:
    ///  loops stop when input is exhausted.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function eof(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn eof(&self) -> bool {
        return self.pos >= self.src.len()
    }

    /// Returns the current byte without advancing.
    ///
    /// What:
    ///  `src.get(pos)`.
    ///  Why:
    ///  lookahead to decide which rule applies.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function peek(): number | null {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn peek(&self) -> Option<u8> {
        return self.src.get(self.pos).copied()
    }

    /// Returns the byte `offset` ahead without advancing.
    ///
    /// What:
    ///  `src.get(pos + offset)`.
    ///  Why:
    ///  range detection in classes and the
    /// `{n,` form need one byte of extra lookahead.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function peek_at(offset: number): number | null {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn peek_at(&self, offset: usize) -> Option<u8> {
        return self.src.get(self.pos + offset).copied()
    }

    /// Returns the current byte and advances past it.
    ///
    /// What:
    ///  reads then increments `pos`.
    ///  Why:
    ///  the consuming primitive every
    /// rule builds on.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function bump(): number | null {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn bump(&mut self) -> Option<u8> {
        let b = self.peek();
        if b.is_some() {
            self.pos += 1;
        }
        return b
    }

    /// Reports whether the cursor sits at the first column of a line.
    ///
    /// What:
    ///  true at offset zero or right after a newline.
    ///  Why:
    ///  a `#` comment is
    /// only a comment when it is the first character of its line.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function at_line_start(): boolean {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    fn at_line_start(&self) -> bool {
        return self.pos == 0 || self.src.get(self.pos - 1) == Some(&b'\n')
    }

    /// Skips verbose-mode whitespace and first-column comments.
    ///
    /// What:
    ///  advances over ASCII whitespace and,
    ///  at a line start,
    ///  over a `#` and
    /// the rest of that line.
    ///  Why:
    ///  verbose mode is always on,
    ///  so a rule may span
    /// many lines and carry first-column comment lines;
    ///  this is where both are
    /// consumed before any token is read.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// function skip_ignorable(): void {
    ///   // Rust body below is the implementation.
    /// }
    /// ```
    pub fn skip_ignorable(&mut self) {
        // What: loop until neither whitespace nor a comment is next.
        // Why: whitespace and a following comment line can alternate.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // Same step as the Rust statement below, written with ordinary TS objects/functions.
        // ```
        loop {
            match self.peek() {
                Some(b) if b.is_ascii_whitespace() => {
                    self.pos += 1;
                }
                Some(b'#') if self.at_line_start() => {
                    // What: drop bytes up to but not including the newline.
                    // Why: the newline is left for the whitespace arm to consume,
                    // keeping `at_line_start` correct for the next line.
                    //
                    // In TS you'd write (pseudocode):
                    // ```ts
                    // // Same step as the Rust statement below, written with ordinary TS objects/functions.
                    // ```
                    while let Some(c) = self.peek() {
                        if c == b'\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                _ => break,
            }
        }
    }
}
