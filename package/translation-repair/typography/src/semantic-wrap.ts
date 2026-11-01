import { LINT_BINARY_NAME, } from './semantic-wrap-binary-constants.ts';
import { wrapWithLintBinary, } from './semantic-wrap-binary.ts';

//region Semantic wrap
// PUTS THE LINE BREAKS BACK that a model does not emit.
//
// A model returns a passage as one long line, or as whatever wrapping it felt
// like. Measured over the pool settled 2026-08-18, 58 of 64 shipped passages
// carried at least one `semantic-line-breaks` finding, 326 findings in all, and
// 17 of them flattened a multi-line archive passage to a single line. The
// grader called this out unprompted at two separate items and stated the policy
// behind it: semantic wrapping is paramount for maintainability, whether or not
// the Chinese original had it. `MD1` says the same thing.
//
// THE RULE IS THIS REPOSITORY'S OWN, and it is chosen rather than reimplemented
// for one property: it only ever ADDS BREAKS. It turns the space after a
// break-point character that ends a written word into a newline plus the
// block's continuation prefix, and it never joins, moves or removes a break. So
// it cannot destroy a break a model got right, which is what makes running it
// over every shipped passage safe without inspecting them first.
//
// IT RUNS IN THE RUST LINTER, `monochromatic-lint`, started once per passage by
// `semantic-wrap-binary.ts` (how it is found, its pinned version and its
// failures are stated there). It was a TypeScript rule of the repository's
// former Markdown linter package, and the two agreed byte for byte on every
// test input and on all but one class of passage: one with a character outside
// the Basic Multilingual Plane (an emoji, say) ahead of a comma that follows
// an emphasis or link span, or ahead of a quotation, made the TypeScript rule
// skip the break or write the quotation's continuation with two spaces for
// `> `. The Rust output is the one the rule's own tests describe.
//
// THE SPACE WAS ONCE LEFT IN PLACE, which opened every inserted continuation
// with a stray one: `>  text` inside a blockquote, and a three-space indent
// under a `- ` marker. Passages settled before 2026-08-20 carry that shape, and
// the rule leaves it exactly as it is rather than re-wrapping it, which is what
// keeps a cache replay from returning text no lane produced.
//
// MEASURED BEFORE IT WAS WIRED IN, over all 64 shipped passages of that pool:
// findings go from 326 to 0, no passage loses a non-newline character, the ten
// that gain characters gain continuation prefixes inside blockquotes and lists,
// and a second application changes nothing. The probe is written up in
// `doc/audit/line-structure-loss-when-a-replacement-ships.md`.
//
// IDEMPOTENCE IS WHAT LETS THIS RUN ON A CACHE REPLAY. A resumed slice is
// wrapped on the way out of the cache rather than on the way in, so a pool
// holding entries written before this existed needs no migration and no key
// change, and a slice that was already wrapped is left exactly as it was.

/**
 Wraps one passage at its semantic boundaries.

 NEVER APPLIED TO TEXT A LANE DECIDED TO KEEP. Wrapping a retained passage
 would turn a decision to change nothing into a change, which the delivery
 coherence check and the assembly assertion both refuse by design, and it
 would also manufacture the one kind of edit a human grader cannot usefully
 judge. Callers that ship the result pass only wording a lane produced;
 `wordingKey` wraps both sides of a comparison, retained text included, and
 ships neither (ledger B26).

 @param text - passage as the lane produced it

 @returns Same passage with semantic line breaks inserted

 @throws {@link LintBinaryMissingError} when the pinned linter is not on the
 PATH, {@link LintBinaryVersionError} when the one found is another version,
 {@link LintBinaryRunError} when it fails, and {@link LintBinaryTextError} when
 the passage holds an unpaired surrogate, which standard input cannot carry

 @example
 ```ts
 const wrapped = wrapReplacementText({ text: 'It naps. It wakes.', },);
 ```
 */
export function wrapReplacementText({ text, }: { readonly text: string; },): string {
  return wrapWithLintBinary({
    binary: LINT_BINARY_NAME,
    text,
  },);
}

//endregion Semantic wrap
