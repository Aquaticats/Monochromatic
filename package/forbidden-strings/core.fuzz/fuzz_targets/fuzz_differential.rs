// What:  for a generated pattern in the shared dialect (no set algebra), compare our
//        byte verdict against the `regex` crate's on the same single-line content.
// Why:   `regex` is a mature, independent oracle. On the overlapping dialect (literals,
//        classes, bounded repetition, alternation, anchors) the two must agree byte for
//        byte; a disagreement is either a real bug in this engine or a documented
//        semantic gap to teach this target to skip.

#![no_main]

use forbidden_regex_fuzz::generators::PatternAndContent;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: PatternAndContent| {
    // What:  `regex` cannot express `&`/`~`, so skip algebra patterns.
    // Why:   there is no oracle for set algebra; those are covered by the engine's own
    //        differential tests against the counting simulation.
    if input.pattern.uses_algebra {
        return;
    }

    // What:  compile in our engine; skip rejections (e.g. empty-matchable, which we
    //        reject at compile but `regex` would accept).
    // Why:   only patterns BOTH engines accept can be compared.
    let Ok(ours) = forbidden_regex::compile(&input.pattern.text) else {
        return;
    };

    // What:  build `regex` over bytes with Unicode DISABLED.
    // Why:   our `\d \w \s \b` are ASCII and our classes are byte sets; `unicode(false)`
    //        makes `regex` match that (ASCII shorthands, byte `.` excluding `\n`), so the
    //        comparison is apples to apples on the shared dialect.
    let theirs = match regex::bytes::RegexBuilder::new(&input.pattern.text)
        .unicode(false)
        .build()
    {
        Ok(theirs) => theirs,
        Err(_) => return,
    };

    // What:  both are unanchored substring search over one line.
    // Why:   the content generator strips newlines, so `^`/`$`/multiline agree.
    let ours_match = ours.is_match(&input.content);
    let theirs_match = theirs.is_match(&input.content);

    if ours_match != theirs_match {
        panic!(
            "differential mismatch: ours={ours_match} regex={theirs_match}\npattern = {:?}\ncontent = {:?}",
            input.pattern.text, input.content,
        );
    }
});
