// What:  library target for the forbidden-regex fuzz package, hosting the structured
//        pattern/content generator the roundtrip and differential targets share.
// Why:   the no-panic targets (fuzz_compile, fuzz_from_bytes) feed raw bytes, but the
//        roundtrip and differential targets need VALID dialect patterns to be
//        meaningful, so the generator lives here and is imported by both.

pub mod generators;
