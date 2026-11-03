//! The unit tests of the CSV reader and the dump fill,
//! ported from `dump.unit.test.ts`,
//! `csv` cases and the tar cases against
//! an archive built in memory.

use std::collections::BTreeSet;
use std::io::{Cursor, Read, Write};

use flate2::Compression;
use flate2::write::GzEncoder;

use crate::csv::read_csv;
use crate::dump::{dump_timestamp, fill_histories_from_dump, needs_api};
use crate::policy::VersionRecord;
use crate::test_scratch::scratch;

fn rows_of(text: &str) -> Vec<Vec<String>> {
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut reader: Cursor<Vec<u8>> = Cursor::new(text.as_bytes().to_vec());
    read_csv(&mut reader, &mut |row: &[String]| {
        rows.push(row.to_vec());
        return true;
    })
    .expect("reads");
    return rows;
}

#[test]
fn plain_rows_split_on_commas_and_line_feeds_and_a_carriage_return_is_dropped() {
    assert_eq!(rows_of("a,b\r\nc,d\n"), vec![vec!["a", "b"], vec!["c", "d"]]);
}

#[test]
fn quoted_fields_keep_commas_line_breaks_and_doubled_quotes() {
    assert_eq!(rows_of("id,text\n1,\"a,b\nc \"\"q\"\"\"\n"), vec![vec!["id", "text"], vec!["1", "a,b\nc \"q\""]]);
}

#[test]
fn a_last_row_without_a_final_line_feed_and_empty_fields_are_read() {
    assert_eq!(rows_of("a,,c\nx,y,"), vec![vec!["a", "", "c"], vec!["x", "y", ""]]);
}

#[test]
fn the_callback_can_stop_the_read() {
    let mut seen: usize = 0;
    let mut reader: Cursor<Vec<u8>> = Cursor::new(b"a\nb\nc\n".to_vec());

    read_csv(&mut reader, &mut |_row: &[String]| {
        seen += 1;
        return seen < 2;
    })
    .expect("reads");

    assert_eq!(seen, 2);
}

#[test]
fn a_utc_dump_timestamp_becomes_rfc_3339_and_anything_else_is_refused() {
    assert_eq!(dump_timestamp("2026-10-29 12:00:00.123456+00").expect("utc"), "2026-10-29T12:00:00.123456Z");
    assert!(dump_timestamp("2026-10-29 12:00:00+02").is_err());
}

fn record(created_at: &str, publisher_id: &str) -> VersionRecord {
    return VersionRecord {
        num: "1.0.0".to_string(),
        created_at: created_at.to_string(),
        publisher_id: publisher_id.to_string(),
        trustpub_repository: String::new(),
        repository: String::new(),
    };
}

#[test]
fn only_a_publisher_less_release_since_trusted_publishing_began_needs_the_api() {
    assert!(!needs_api(&[record("2025-06-30T23:59:59Z", "")]));
    assert!(!needs_api(&[record("2026-01-01T00:00:00Z", "7")]));
    assert!(needs_api(&[record("2025-07-01T00:00:00Z", "")]));
}

/// A gzip tar with the given (path,
/// content) entries.
fn archive(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    for (path, content) in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append_data(&mut header, path, content.as_bytes()).expect("append");
    }
    let bytes: Vec<u8> = builder.into_inner().expect("finish the tar");
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&bytes).expect("compress");
    return encoder.finish().expect("finish the gzip");
}

fn wanted(names: &[&str]) -> BTreeSet<String> {
    return names.iter().map(|name| { return name.to_string(); }).collect();
}

const CRATES: &str = "id,name,other\n1,wanted,x\n2,unwanted,y\n3,trusted,z\n";
const VERSIONS: &str = "crate_id,num,created_at,published_by,repository\n1,1.0.0,2024-01-01 00:00:00.5+00,7,\"https://example.invalid/a,b\"\n1,1.0.1,2024-02-01 00:00:00+00,7,\n2,9.9.9,2024-01-01 00:00:00+00,1,\n3,1.0.0,2026-01-01 00:00:00+00,,\n";

#[test]
fn the_wanted_histories_are_written_and_a_possibly_trusted_one_is_left_to_the_api() {
    let cache = scratch();
    let bytes: Vec<u8> = archive(&[("2026-10-29/data/crates.csv", CRATES), ("2026-10-29/data/versions.csv", VERSIONS), ("2026-10-29/data/later.csv", "x\n")]);

    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted", "trusted"]), cache.path()).expect("fills");

    assert_eq!(result.found, 2);
    assert_eq!(result.written, 1);
    assert_eq!(result.left_to_api, vec!["trusted".to_string()]);
    let mut text: String = String::new();
    std::fs::File::open(cache.path().join("wanted.json")).expect("cache file").read_to_string(&mut text).expect("read");
    assert!(text.contains("\"num\":\"1.0.1\"") && text.contains("2024-01-01T00:00:00.5Z") && text.contains("https://example.invalid/a,b"));
    assert!(!cache.path().join("unwanted.json").exists());
    assert!(!cache.path().join("trusted.json").exists());
}

#[test]
fn an_existing_cache_file_is_kept() {
    let cache = scratch();
    std::fs::write(cache.path().join("wanted.json"), "[]\n").expect("existing");
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", VERSIONS)]);

    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path()).expect("fills");

    assert_eq!((result.kept, result.written), (1, 0));
    assert_eq!(std::fs::read_to_string(cache.path().join("wanted.json")).expect("read"), "[]\n");
}

#[test]
fn a_dump_with_versions_before_crates_a_missing_column_or_no_versions_is_refused_naming_why() {
    let cache = scratch();
    let reversed = archive(&[("d/data/versions.csv", VERSIONS), ("d/data/crates.csv", CRATES)]);
    let renamed = archive(&[("d/data/crates.csv", "id,title\n1,x\n"), ("d/data/versions.csv", VERSIONS)]);
    let truncated = archive(&[("d/data/crates.csv", CRATES)]);

    let first = fill_histories_from_dump(Box::new(Cursor::new(reversed)), &wanted(&["wanted"]), cache.path()).expect_err("refused");
    let second = fill_histories_from_dump(Box::new(Cursor::new(renamed)), &wanted(&["wanted"]), cache.path()).expect_err("refused");
    let third = fill_histories_from_dump(Box::new(Cursor::new(truncated)), &wanted(&["wanted"]), cache.path()).expect_err("refused");

    assert!(first.message.contains("versions.csv before crates.csv"));
    assert!(second.message.contains("\"name\" column"));
    assert!(third.message.contains("ended before versions.csv"));
}

#[test]
fn the_registry_reads_the_dump_only_above_the_break_even_and_the_api_serves_a_crate_the_dump_left() {
    use crate::http::{DumpSource, RegistryError, TextFetcher};
    use crate::registry::{CratesIo, Registry};
    use std::cell::Cell;
    use std::rc::Rc;

    struct OneArchive {
        bytes: Vec<u8>,
        opened: Rc<Cell<u32>>,
    }
    impl DumpSource for OneArchive {
        fn open(&self, _url: &str) -> Result<Box<dyn Read>, RegistryError> {
            self.opened.set(self.opened.get() + 1);
            return Ok(Box::new(Cursor::new(self.bytes.clone())));
        }
    }
    struct NoNetwork;
    impl TextFetcher for NoNetwork {
        fn get(&self, url: &str) -> Result<String, RegistryError> {
            return Err(RegistryError { message: format!("no network: {url}"), status: None, retry_after_seconds: 0 });
        }
    }
    let root = scratch();
    let opened: Rc<Cell<u32>> = Rc::new(Cell::new(0));
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", VERSIONS)]);
    let registry: CratesIo = CratesIo::new(root.path(), Box::new(NoNetwork), std::time::Duration::ZERO, std::time::Duration::ZERO)
        .with_dump(Box::new(OneArchive { bytes, opened: Rc::clone(&opened) }), 2);

    registry.prefetch(&wanted(&["wanted"])).expect("below the break-even: nothing is read");
    assert_eq!(opened.get(), 0);
    registry.prefetch(&wanted(&["wanted", "trusted"])).expect("at the break-even the dump is read");

    assert_eq!(opened.get(), 1);
    let history = registry.history("wanted", &["1.0.1".to_string()]).expect("served from the cache the dump filled, no network");
    assert_eq!(history.len(), 2);
    assert!(registry.history("trusted", &["1.0.0".to_string()]).is_err(), "a crate left to the API needs the network");
}

fn read_all(text: &str) -> std::io::Result<Vec<Vec<String>>> {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let mut rows: Vec<Vec<String>> = Vec::new();
    // What:     `.to_vec()` copies a borrowed slice (`&[T]`) into a new owned growable `Vec<T>`.
    // Why:      The result can be stored or modified independently of the slice it came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // [...slice]
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `reader: Cursor<Vec<u8>>`.
    // What:     Integer types are fixed width: `i32`/`i64` signed, `u32`/`u64` unsigned, `usize` as wide as a memory address (siblings of each other); there is no automatic widening.
    // Why:      The width matches the API the value comes from or goes to: `usize` for indexes and lengths, `i64` for milliseconds, `i32` for process exit codes, `u32` for process ids and counts.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: number
    // ```
    let mut reader: Cursor<Vec<u8>> = Cursor::new(text.as_bytes().to_vec());
    // What:     `|a, b| body` is a closure, an anonymous function (a TS arrow function); `|_|` ignores its argument. Closures can use variables from the enclosing function.
    // Why:      Short callbacks for iterator steps, error conversion and lazy defaults stay next to where they are used.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // (a, b) => body
    // ```
    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    read_csv(&mut reader, &mut |row: &[String]| {
        // `.to_vec()`: copies the slice into a new owned `Vec`.
        rows.push(row.to_vec());
        return true;
    // What:     `expr?` is the propagation operator: if `expr` is `Err` (or `None`) the whole function returns that failure at once, otherwise the success value is unwrapped and used.
    // Why:      A failure stops the step at its source and reaches the caller as a returned value, with no `try`/`catch` in between.
    // Gotcha:   `?` only works in a function whose own return type is a `Result` (or `Option`) of a compatible kind.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const value = await expr;   // throws to the caller on failure
    // ```
    })?;
    // What:     `Ok(value)` and `Err(error)` build the two variants of `Result<T, E>`, Rust's success-or-failure value (siblings: `Option<T>`, a value or nothing; a TS function that throws has no value form of this).
    // Why:      Failure is returned as an ordinary value, so the caller must handle or forward it; nothing is thrown.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Ok(value)  is  return value;
    // // Err(error) is  throw error;   (the caller sees it as a returned value)
    // ```
    return Ok(rows);
}

/// Review finding (malformed EOF):
/// a stream that ends inside a quoted field is cut off,
/// not finished.
#[test]
fn a_stream_that_ends_inside_a_quoted_field_is_an_error() {
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    assert!(read_all("a,b\n1,\"never closed").is_err());
}

/// Review finding (malformed quoting):
/// text after a closing quote,
/// and a quote inside a bare field,
/// are not CSV.
#[test]
fn text_after_a_closing_quote_and_a_quote_inside_a_bare_field_are_errors() {
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_all("a,b\n\"x\"y,2\n").is_err());
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_all("a,b\nx\"y,2\n").is_err());
}

/// Review finding (bounded memory):
/// one field larger than the bound is refused instead of grown.
#[test]
fn a_field_over_the_bound_is_refused() {
    // Typed binding: `text: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let text: String = format!("a\n\"{}\"\n", "x".repeat(crate::csv::MAX_FIELD_BYTES + 1));

    // Borrow `&text`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_all(&text).is_err());
}

/// A row with more fields than the bound is refused.
#[test]
fn a_row_over_the_field_bound_is_refused() {
    // Typed binding: `text: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let text: String = format!("a\n{}\n", ",".repeat(crate::csv::MAX_ROW_FIELDS + 1));

    // Borrow `&text`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_all(&text).is_err());
}

/// Review finding (2b 11):
/// many fields that each stay under the field bound still must not add up to a row the process cannot hold.
#[test]
fn a_row_whose_fields_add_up_over_the_row_budget_is_refused() {
    // Typed binding: `field: String`.
    let field: String = "x".repeat(crate::csv::MAX_FIELD_BYTES);
    // Typed binding: `text: String`.
    // Macro `format!`: builds or prints text (or a list) from a template.
    let text: String = format!("a,b,c\n\"{field}\",\"{field}\",y\n");

    // Macro `assert!`: builds or prints text (or a list) from a template.
    const { assert!(crate::csv::MAX_ROW_BYTES < 2 * crate::csv::MAX_FIELD_BYTES + 1) };
    // Borrow `&text`: lends the value without copying or giving it away.
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(read_all(&text).is_err());
}

/// Review finding (padding of short rows):
/// a row narrower than the header is a damaged table,
/// and nothing may be cached
/// from it.
#[test]
fn a_short_row_refuses_the_table_and_nothing_is_cached() {
    let cache = scratch();
    // Typed binding: `short_versions: &str`.
    let short_versions: &str = "crate_id,num,created_at,published_by,repository\n1,1.0.0,2024-01-01 00:00:00+00,7\n";
    // Borrow `&[`: lends the value without copying or giving it away.
    // Typed binding: `bytes: Vec<u8>`.
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", short_versions)]);

    // Borrow `&wanted`: lends the value without copying or giving it away.
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err());
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!cache.path().join("wanted.json").exists());
}

/// Review finding:
/// a versions table cut off inside a quoted field caches nothing.
#[test]
fn a_versions_table_cut_off_inside_a_quote_caches_nothing() {
    let cache = scratch();
    // Typed binding: `cut: &str`.
    let cut: &str = "crate_id,num,created_at,published_by,repository\n1,1.0.0,2024-01-01 00:00:00+00,7,\"https://example.invalid/unfinished";
    // Borrow `&[`: lends the value without copying or giving it away.
    // Typed binding: `bytes: Vec<u8>`.
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", cut)]);

    // Borrow `&wanted`: lends the value without copying or giving it away.
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err());
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!cache.path().join("wanted.json").exists());
}

/// Review finding (retained history):
/// one crate's history is bounded.
#[test]
fn more_releases_of_one_crate_than_the_bound_are_refused() {
    let cache = scratch();
    // What:     `.to_string()` (also `.to_owned()`, `String::from`) copies borrowed or displayable text into a new owned `String` (sibling: `&str`, a borrowed view that owns nothing).
    // Why:      A struct field or a returned value must own its text because it may outlive the place the text came from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // String(value)  // JS strings are always owned values
    // ```
    // `let mut`: a binding that may change.
    // Typed binding: `versions: String`.
    let mut versions: String = "crate_id,num,created_at,published_by,repository\n".to_string();
    for number in 0..=crate::dump::MAX_VERSIONS_PER_CRATE {
        // Borrow `&format`: lends the value without copying or giving it away.
        // Macro `format!`: builds or prints text (or a list) from a template.
        versions.push_str(&format!("1,1.0.{number},2024-01-01 00:00:00+00,7,\n"));
    }
    // Borrow `&[`: lends the value without copying or giving it away.
    // Typed binding: `bytes: Vec<u8>`.
    let bytes: Vec<u8> = archive(&[("d/data/crates.csv", CRATES), ("d/data/versions.csv", &versions)]);

    // Borrow `&wanted`: lends the value without copying or giving it away.
    // Constructor `Box::new(`: an empty owned collection or a boxed value.
    let result = fill_histories_from_dump(Box::new(Cursor::new(bytes)), &wanted(&["wanted"]), cache.path());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(result.is_err());
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!cache.path().join("wanted.json").exists());
}
