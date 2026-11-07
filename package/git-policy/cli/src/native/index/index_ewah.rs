//! What:
//!  Git's EWAH compressed bitmap (`ewah/ewah_bitmap.c`,
//! `ewah/ewah_io.c`):
//! read from bytes,
//! listed as bit positions,
//!  built by setting bits in increasing order,
//! written back.
//! Why:
//!  The untracked-cache and fsmonitor index extensions store bitmaps indexed by directory or entry position.
//! Rewriting an
//!  index changes those positions,
//! so each bitmap is decoded to its set bits and encoded again by the same algorithm
//!  Git uses,
//! which gives the same bytes for the same bits (`ewah_set` is ported step for step,
//! including its
//!  run-length corner cases).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const bitmap = Ewah.read(bytes); const bits: number[] = bitmap.bits(); const out = Ewah.fromBits(bits).serialize();
//! ```

/// The bits in one word of the bitmap.
const WORD_BITS: usize = 64;

/// The most a run-length word counts as running words (32 bits).
const LARGEST_RUN: u64 = (1 << 32) - 1;

/// The most a run-length word counts as literal words (31 bits).
const LARGEST_LITERAL: u64 = (1 << 31) - 1;

/// An EWAH bitmap:
/// the compressed words,
/// the position of the last run-length word,
/// and the bit length.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ewah {
    /// One more than the highest bit position set so far,
    /// as the file's bit size.
    bit_size: usize,
    /// Run-length words,
    /// each followed by its literal words.
    buffer: Vec<u64>,
    /// The index in `buffer` of the run-length word the next word extends.
    rlw: usize,
}

/// The run bit of a run-length word.
fn run_bit(word: u64) -> bool {
    return word & 1 == 1;
}

/// The number of running words a run-length word counts.
fn running_len(word: u64) -> u64 {
    return (word >> 1) & LARGEST_RUN;
}

/// The number of literal words that follow a run-length word.
fn literal_words(word: u64) -> u64 {
    return word >> 33;
}

/// The run-length word with its run bit set to `value`.
fn with_run_bit(word: u64, value: bool) -> u64 {
    return (word & !1) | u64::from(value);
}

/// The run-length word with its running length set to `value`.
fn with_running_len(word: u64, value: u64) -> u64 {
    return (word & !(LARGEST_RUN << 1)) | (value << 1);
}

/// The run-length word with its literal count set to `value`.
fn with_literal_words(word: u64, value: u64) -> u64 {
    return (word & ((1 << 33) - 1)) | (value << 33);
}

impl Ewah {
    /// An empty bitmap,
    /// as `ewah_new`.
    pub fn new() -> Ewah {
        return Ewah { bit_size: 0, buffer: vec![0], rlw: 0 };
    }

    /// Append a fresh run-length word and make it the current one (`buffer_push_rlw`).
    fn push_rlw(&mut self, value: u64) {
        self.buffer.push(value);
        self.rlw = self.buffer.len() - 1;
    }

    /// Add `number` empty words of `value` bits (`add_empty_words`).
    fn add_empty_words(&mut self, value: bool, count: u64) {
        let mut number: u64 = count;
        let current: u64 = self.buffer[self.rlw];
        let size: u64 = running_len(current) + literal_words(current);
        if run_bit(current) != value && size == 0 {
            self.buffer[self.rlw] = with_run_bit(current, value);
        } else if literal_words(current) != 0 || run_bit(current) != value {
            self.push_rlw(with_run_bit(0, value));
        }
        let run: u64 = running_len(self.buffer[self.rlw]);
        let can_add: u64 = number.min(LARGEST_RUN - run);
        self.buffer[self.rlw] = with_running_len(self.buffer[self.rlw], run + can_add);
        number -= can_add;
        while number >= LARGEST_RUN {
            self.push_rlw(with_running_len(with_run_bit(0, value), LARGEST_RUN));
            number -= LARGEST_RUN;
        }
        if number > 0 {
            self.push_rlw(with_running_len(with_run_bit(0, value), number));
        }
    }

    /// Add one literal word (`add_literal`).
    fn add_literal(&mut self, data: u64) {
        let current: u64 = literal_words(self.buffer[self.rlw]);
        if current >= LARGEST_LITERAL {
            self.push_rlw(with_literal_words(0, 1));
        } else {
            self.buffer[self.rlw] = with_literal_words(self.buffer[self.rlw], current + 1);
        }
        self.buffer.push(data);
    }

    /// Add one empty word of `value` bits (`add_empty_word`).
    fn add_empty_word(&mut self, value: bool) {
        let current: u64 = self.buffer[self.rlw];
        let no_literal: bool = literal_words(current) == 0;
        let run: u64 = running_len(current);
        if no_literal && run == 0 {
            self.buffer[self.rlw] = with_run_bit(current, value);
        }
        if no_literal && run_bit(self.buffer[self.rlw]) == value && run < LARGEST_RUN {
            self.buffer[self.rlw] = with_running_len(self.buffer[self.rlw], run + 1);
        } else {
            self.push_rlw(with_running_len(with_run_bit(0, value), 1));
        }
    }

    /// Set bit `position`,
    /// which must not be below the bit size (`ewah_set`).
    pub fn set(&mut self, position: usize) {
        assert!(position >= self.bit_size, "bits are set in increasing order");
        let dist: usize = (position + 1).div_ceil(WORD_BITS) - self.bit_size.div_ceil(WORD_BITS);
        self.bit_size = position + 1;
        let bit: u64 = 1 << (position % WORD_BITS);
        if dist > 0 {
            if dist > 1 {
                self.add_empty_words(false, (dist - 1) as u64);
            }
            self.add_literal(bit);
            return;
        }
        if literal_words(self.buffer[self.rlw]) == 0 {
            let run: u64 = running_len(self.buffer[self.rlw]);
            self.buffer[self.rlw] = with_running_len(self.buffer[self.rlw], run - 1);
            self.add_literal(bit);
            return;
        }
        let last: usize = self.buffer.len() - 1;
        self.buffer[last] |= bit;
        if self.buffer[last] == u64::MAX {
            self.buffer.pop();
            let literals: u64 = literal_words(self.buffer[self.rlw]);
            self.buffer[self.rlw] = with_literal_words(self.buffer[self.rlw], literals - 1);
            self.add_empty_word(true);
        }
    }

    /// The bitmap with exactly the bits of `positions` (increasing) set.
    pub fn from_bits(positions: &[usize]) -> Ewah {
        let mut bitmap: Ewah = Ewah::new();
        for position in positions {
            bitmap.set(*position);
        }
        return bitmap;
    }

    /// The bit length the file stores.
    pub fn bit_size(&self) -> usize {
        return self.bit_size;
    }

    /// The set bits in increasing order (`ewah_each_bit`).
    pub fn bits(&self) -> Vec<usize> {
        let mut found: Vec<usize> = Vec::new();
        let mut position: usize = 0;
        let mut pointer: usize = 0;
        while pointer < self.buffer.len() {
            let word: u64 = self.buffer[pointer];
            let run: usize = running_len(word) as usize * WORD_BITS;
            if run_bit(word) {
                found.extend(position..position + run);
            }
            position += run;
            pointer += 1;
            for _ in 0..literal_words(word) {
                let literal: u64 = self.buffer[pointer];
                for shift in 0..WORD_BITS {
                    if literal & (1 << shift) != 0 {
                        found.push(position + shift);
                    }
                }
                position += WORD_BITS;
                pointer += 1;
            }
        }
        return found;
    }

    /// Append the bitmap in the file's form:
    /// bit size,
    /// word count,
    /// words,
    /// position of the last run-length word.
    pub fn serialize(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&(self.bit_size as u32).to_be_bytes());
        out.extend_from_slice(&(self.buffer.len() as u32).to_be_bytes());
        for word in &self.buffer {
            out.extend_from_slice(&word.to_be_bytes());
        }
        out.extend_from_slice(&(self.rlw as u32).to_be_bytes());
    }

    /// Read one bitmap from the start of `bytes` and the number of bytes it used,
    /// or `None` for a truncated or inconsistent one
    /// (`ewah_read_mmap`) and for one of more than `max_bits` bits.
    ///
    /// The checks are the ones that keep a small input from describing a large bitmap:
    /// every run-length word's literal words
    /// are in the buffer,
    /// the last run-length word is where the position says,
    /// and the words the runs and literals add up to
    /// are exactly the words a bitmap of that bit size takes (what `ewah_set` builds).
    /// The caller names the most bits it can
    /// use (the entries of the index,
    /// the directories of the cache),
    /// so the set bits that `bits` lists are bounded by it.
    pub fn read(bytes: &[u8], max_bits: usize) -> Option<(Ewah, usize)> {
        let bit_size: usize = u32::from_be_bytes(bytes.get(0..4)?.try_into().ok()?) as usize;
        let count: usize = u32::from_be_bytes(bytes.get(4..8)?.try_into().ok()?) as usize;
        if bit_size > max_bits {
            return None;
        }
        let words_end: usize = 8usize.checked_add(count.checked_mul(8)?)?;
        let mut buffer: Vec<u64> = Vec::with_capacity(count.min(1 << 20));
        let (chunks, leftover) = bytes.get(8..words_end)?.as_chunks::<8>();
        if !leftover.is_empty() {
            return None;
        }
        for chunk in chunks {
            buffer.push(u64::from_be_bytes(*chunk));
        }
        let rlw: usize = u32::from_be_bytes(bytes.get(words_end..words_end + 4)?.try_into().ok()?) as usize;
        let (mut at, mut last, mut logical): (usize, usize, u64) = (0, 0, 0);
        while at < buffer.len() {
            last = at;
            let word: u64 = buffer[at];
            logical = logical.checked_add(running_len(word))?.checked_add(literal_words(word))?;
            at = at.checked_add(1)?.checked_add(usize::try_from(literal_words(word)).ok()?)?;
        }
        if buffer.is_empty() || at != buffer.len() || rlw != last || logical != bit_size.div_ceil(WORD_BITS) as u64 {
            return None;
        }
        return Some((Ewah { bit_size, buffer, rlw }, words_end + 4));
    }
}

/// An empty bitmap is the default.
impl Default for Ewah {
    fn default() -> Ewah {
        return Ewah::new();
    }
}

#[cfg(test)]
#[path = "index_ewah_tests.rs"]
mod tests;
