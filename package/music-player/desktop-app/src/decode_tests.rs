// What:     Unit tests for `decode.rs`, pulled in by
//           `#[cfg(test)] #[path = "decode_tests.rs"] mod tests;` at
//           the bottom of `decode.rs`. Compiles only under
//           `cargo nextest run` / `cargo test`; reaches the module items
//           (including private ones) via `use super::*` because this file is
//           the `tests` CHILD of decode.
// Why:      Keep the tests beside the code without inflating
//           `decode.rs` or its max-lines budget (sibling
//           `*_tests.rs` files are exempt from the linter).

// What:     `use super::*;`. Import everything from the PARENT module (this
//           file) into the test module. `super` = one level up.
// Why:      So tests can call `decode_all`, name `AudioSpec`, etc.
use super::*;

// What:     `fn decode_all(path: &Path) -> Result<(AudioSpec, Vec<f32>), PlayerError>`.
//           A TEST HELPER that opens a file and drains it fully to one big
//           interleaved `Vec<f32>`, returning it with the spec. Defined
//           inside `#[cfg(test)]` so it never ships.
// Why:      Lets each codec test assert on the full decoded output simply.
fn decode_all(path: &Path) -> Result<(AudioSpec, Vec<f32>), PlayerError> {
    // What:     `let mut source = open(path)?;`. Open the file; `mut` because
    //           draining mutates the source. `?` unwraps/returns.
    // Why:      Get a decoder for the file.
    let mut source = open(path)?;

    // What:     `let spec = source.spec();`. Snapshot the stream shape.
    // Why:      Return it alongside the samples.
    let spec = source.spec();

    // What:     `let mut all: Vec<f32> = Vec::new();`. An empty, growable f32
    //           array we will extend with each chunk. Explicit type annotation
    //           because it starts empty (type cannot be inferred yet).
    // Why:      Accumulate all decoded samples.
    let mut all: Vec<f32> = Vec::new();

    // What:     `loop { ... }`. Repeat until we see the empty-Vec EOF signal.
    // Why:      Pull every chunk to the end.
    loop {
        // What:     `let chunk = source.next_chunk()?;`. Decode one block; `?`
        //           propagates a real error.
        // Why:      Advance through the stream.
        let chunk = source.next_chunk()?;

        // What:     `if chunk.is_empty() { break; }`. Empty chunk == EOF; `break`
        //           exits the loop.
        // Why:      Stop at end of stream.
        if chunk.is_empty() {
            break;
        }

        // What:     `all.extend(chunk);`. Append all elements of `chunk` to
        //           `all`, MOVING `chunk`'s contents in.
        // Why:      Build the full sample vector.
        all.extend(chunk);
    }

    // What:     `Ok((spec, all))`. Wrap a tuple of the spec and samples as
    //           success. Tail -> return.
    // Why:      Hand both back to the test.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return [spec, all];
    // ```
    return Ok((spec, all));
}

// What:     `fn check_fixture(name: &str)`. Shared assertion routine: decode
//           a fixture file and check the output is sane.
// Why:      One body reused by every per-codec test below.
fn check_fixture(name: &str) {
    // What:     `let path = Path::new("fixture").join(name);`. `Path::new`
    //           borrows a `&str` as a `&Path`; `.join(name)` returns an owned
    //           `PathBuf` ("fixture/<name>").
    // Why:      Build the path to the committed test file.
    let path = Path::new("fixture").join(name);

    // What:     `let (spec, samples) = decode_all(&path).expect(...)`. Call the
    //           helper (lending `&path`); `.expect(msg)` UNWRAPS `Ok`, or
    //           PANICS with `msg` on `Err` (panicking fails the test). The
    //           tuple is destructured into `spec` and `samples`.
    // Why:      Decode the fixture or fail loudly with context.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const [spec, samples] = decodeAll(path);
    // ```
    let (spec, samples) = decode_all(&path).expect("fixture should decode");

    // What:     `assert!(spec.rate > 0, ...)`. `assert!(cond, msg)` panics
    //           (fails the test) if `cond` is false.
    // Why:      A real stream must report a positive sample rate.
    assert!(spec.rate > 0, "{name}: rate should be positive");

    // What:     `assert!(spec.channels >= 1, ...)`. At least one channel.
    // Why:      Every audio stream has >= 1 channel.
    assert!(spec.channels >= 1, "{name}: should have >=1 channel");

    // What:     `assert!(!samples.is_empty(), ...)`. `!` negates; require some
    //           decoded samples.
    // Why:      A 0.3s tone must decode to non-empty PCM.
    assert!(!samples.is_empty(), "{name}: should decode some samples");

    // What:     `assert_eq!(samples.len() % spec.channels as usize, 0, ...)`.
    //           `assert_eq!(a, b)` panics unless `a == b`. `samples.len()` is
    //           `usize`; `spec.channels as usize` widens `u16` to `usize`; `%`
    //           is modulo. Interleaved data length must be a whole number of
    //           frames.
    // Why:      Catch interleaving/channel-count mismatches.
    assert_eq!(
        samples.len() % spec.channels as usize,
        0,
        "{name}: interleaved length should be a multiple of channels"
    );
}

// What:     `#[test]`. Marks the next function as a test case.
// Why:      `cargo test` runs every `#[test]` function.
#[test]
// What:     `fn decodes_wav() { check_fixture("tone.wav"); }`. WAV/PCM path.
// Why:      Verify the PCM decoder.
fn decodes_wav() {
    check_fixture("tone.wav");
}

// What:     `#[test]` FLAC path.
// Why:      Verify the FLAC decoder.
#[test]
fn decodes_flac() {
    check_fixture("tone.flac");
}

// What:     `#[test]` MP3 path.
// Why:      Verify the MP3 decoder.
#[test]
fn decodes_mp3() {
    check_fixture("tone.mp3");
}

// What:     `#[test]` Vorbis (in Ogg) path.
// Why:      Verify the Vorbis decoder.
#[test]
fn decodes_ogg_vorbis() {
    check_fixture("tone.ogg");
}

// What:     `#[test]` Opus path (routes through `opus.rs`, libopus).
// Why:      Verify the Opus decode path end to end.
#[test]
fn decodes_opus() {
    check_fixture("tone.opus");
}

// What:     `#[test]` regression for seeking to the very beginning of an Ogg/Opus
//           stream whose first audio frame is NOT frame 0.
// Why:      `fixture/offset.opus` is an Opus file remuxed with a +0.5s timestamp
//           offset, so its track `start_ts` is non-zero (~23352 frames). The old
//           seek used `SeekTo::Time { 0s }`, which maps to frame 0; the Ogg
//           demuxer rejects any frame below `start_ts` with
//           "requested seek timestamp is out-of-range for stream", so dragging the
//           bar to the beginning errored. The fix (`seek_format`) seeks to
//           `start_ts + 0`, the real first frame. This test fails (the original
//           bug) if the seek is ever reverted to `SeekTo::Time`.
#[test]
fn seek_to_start_of_offset_opus_succeeds() {
    // What:     `let path = Path::new("fixture").join("offset.opus");`. Build the
    //           path to the committed offset fixture (see `gen:fixtures`).
    // Why:      This is the only fixture with a non-zero start frame.
    let path = Path::new("fixture").join("offset.opus");

    // What:     `let mut source = open(&path).expect("offset.opus should open");`.
    //           `open(&path)` lends the path and returns `Result<Box<dyn Source>>`;
    //           `.expect(msg)` unwraps `Ok` or panics (failing the test). `mut`
    //           because seeking and decoding mutate the source.
    // Why:      Drive the real decode path the player uses, not a hand-rolled one.
    let mut source = open(&path).expect("offset.opus should open");

    // What:     `source.seek(0.0).expect(...)`. Seek to second 0 (the beginning),
    //           unwrapping the `Result<(), PlayerError>` and panicking on error.
    // Why:      This is the exact action that errored before the fix; it must now
    //           succeed.
    source
        .seek(0.0)
        .expect("seek to the beginning must not be out-of-range");

    // What:     `let chunk = source.next_chunk().expect(...)`. Pull one decoded
    //           block after the seek, unwrapping the `Result<Vec<f32>>`.
    // Why:      Confirm the post-seek position is actually decodable, not just that
    //           `seek` returned `Ok`.
    let chunk = source
        .next_chunk()
        .expect("decoding after seek-to-start should yield samples");

    // What:     `assert!(!chunk.is_empty(), ...)`. `!` negates; require a non-empty
    //           block.
    // Why:      An empty first block would mean the seek landed at end-of-stream.
    assert!(
        !chunk.is_empty(),
        "first chunk after seeking to the start should not be empty"
    );
}

// What:     `#[test]` AAC-LC (in MP4) path.
// Why:      Verify the AAC decoder.
#[test]
fn decodes_aac() {
    check_fixture("tone.aac.m4a");
}

// What:     `#[test]` ALAC (in MP4) path.
// Why:      Verify the ALAC decoder.
#[test]
fn decodes_alac() {
    check_fixture("tone.alac.m4a");
}
