// What:  compile a generated valid pattern, serialize it, reload it, and assert the
//        reloaded matcher gives the SAME verdict on the generated content.
// Why:   the scanner builds the matcher once and ships the serialized bytes; if
//        `to_bytes` -> `from_bytes` ever changed a verdict, a deployed scanner would
//        silently disagree with the one that built it, which for a secret scanner can
//        mean a missed leak. Serialization must be verdict-preserving.

#![no_main]

use forbidden_regex_fuzz::generators::PatternAndContent;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: PatternAndContent| {
    // What:  only proceed on patterns this dialect accepts.
    // Why:   rejections are fine here; this target is about the serialize roundtrip.
    let Ok(regex) = forbidden_regex::compile(&input.pattern.text) else {
        return;
    };

    // What:  the verdict from the freshly compiled engine.
    // Why:   the reference the reloaded engine must reproduce.
    let before = regex.is_match(&input.content);

    // What:  a compiled engine must serialize.
    // Why:   `to_bytes` only fails on an encoder error, which would itself be a bug.
    let bytes = match regex.to_bytes() {
        Ok(bytes) => bytes,
        Err(error) => panic!("compiled pattern failed to serialize: {error:?}\npattern = {:?}", input.pattern.text),
    };

    // What:  our own bytes must reload (validation must accept what we emitted).
    // Why:   a self-rejecting roundtrip is a bug in encode or in validate.
    let reloaded = match forbidden_regex::Regex::from_bytes(&bytes) {
        Ok(reloaded) => reloaded,
        Err(error) => panic!("self-serialized pattern failed to reload: {error:?}\npattern = {:?}", input.pattern.text),
    };

    // What:  verdicts must match.
    // Why:   the core roundtrip invariant.
    let after = reloaded.is_match(&input.content);
    if before != after {
        panic!(
            "roundtrip verdict mismatch: before={before} after={after}\npattern = {:?}\ncontent = {:?}",
            input.pattern.text, input.content,
        );
    }
});
