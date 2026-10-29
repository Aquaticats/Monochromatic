# Are the dismissed clusters really non-actionable?

Written to answer one question about the four dismissal issues (#656,
 #657,
 #658,
 #659):
are the matched clusters really non-actionable,
or does dismissing them normalise duplication and miss shared utilities that are small but real?

Maintainer decision recorded while this investigation ran:
`body_node_count_threshold: 13` in `slopo.conf.yaml` is hand-tuned and is the floor.
Lowering it is rejected,
so every option below works at 13.

The answer has three parts.
The per-pair dismissals mostly survive a re-read.
The floor is not what hides the interesting families;
one large family is entirely above it and was still dismissed pair by pair.
And the cause is that the review judged clusters while the duplication lives in families.

## The floor is deliberate and mostly right

`slopo.conf.yaml` sets `body_node_count_threshold: 13`,
and `slopo/indexing/scanner.py:50` keeps a unit only when `body_node_count >= threshold`,
so bodies below 13 AST nodes are never indexed.
Measured on 2026-10-09 with a throwaway probe index at threshold 4
(`~/temp/agent/review-659/work/probe3/`,
`slopo index` only,
no embedding step and no API cost,
154 seconds):

- the current index at 13 holds 18440 units from 3136 files and 690 duplicated body hashes;
- the probe at 4 holds 23671 units from 3939 files and 1274 duplicated body hashes;
- so the floor excludes 5231 units,
   28% of the corpus,
   and 584 exact-duplicate body groups,
   46% of those that exist.

A sample of what the floor excludes argues for keeping it.
`module/hyperscript/src/css/values.constructors.ts` has 33 exported constructors.
At threshold 13 the index holds 8 of them;
the other 25 are 8-node one-liners such as `cssRem`,
 `cssEm`,
 `cssPercent`,
 `cssFr`,
 `cssLh`,
 `cssCh`,
 `cssVi`,
 `cssVb`,
 `cssCqi` and `cssCqb`.
Those are a named API surface where each function is the documentation of one CSS unit,
not waste.
A generic `cssUnit('rem', n)` would shorten the file and destroy the interface.

The genuine cases below the floor are smaller than they look.
`{ return line !== ''; }` appears at 16 sites across 7 packages and
`{ return line.length > 0; }` at 12 sites across 6,
so one predicate exists in two spellings at 28 sites.
`{ chunks.push(chunk,); }` appears at 16 sites across 4 packages as the callbacks
`collect`,
 `collectChunk`,
 `collectStdout`,
 `collectStderr` and `onData`.
Extracting either would not shorten any call site:
`isNonEmptyLine(line)` is longer than `line !== ''`.
What they cost is consistency,
not lines,
and the two spellings of one predicate are the only part worth acting on.

One configuration note,
not a change made here:
`slopo.conf.yaml` justifies the value with
"We're writing strict TypeScript,
 increased from default 10 until no bad reports remain",
but slopo `0.8.0` defaults `body_node_count_threshold` to 20
(`slopo/config.py:151-152`).
The comment describes a `0.4.0` default,
so the hand-tuned 13 is now more permissive than upstream's default rather than less.

## The family the floor does not hide

The error-class idiom is entirely above the floor,
which makes it the cleanest test of the question.

- 212 indexed units contain `this.name = `,
   across 147 distinct files,
   with a minimum `body_node_count` of exactly 13.
   Nothing about the threshold hides any of them.
- The idiom is
   `public constructor(message: string,) { super(message,); this.name = 'XError'; }`,
   concentrated in `git-policy/cli` (42 files),
   `module/deepmerge-ts.fuzz` (12),
   `open-code-review-issue/cli` (11) and `mvm/cli` (9).
- No base class sets the name automatically;
   `new.target.name` appears nowhere in the repository.
- All 209 assignments found by `rg` match their enclosing class name.
   The comparison was proven able to fail against a fixture carrying one planted mismatch
   before its zero-mismatch result was trusted,
   so the idiom has not drifted.
- `package/config/rolldown/src/index.ts:61-65` sets `mangle: false` with the comment
   "Mangle breaks func.name and makes output difficult for users to audit",
   so class names survive bundling and a base class assigning `this.name = new.target.name`
   would be safe in published artifacts.

What the review saw of it:
17 of the 1008 clusters contain an error-name constructor,
and they were split across four issues and four verdicts.

- 12 dismissed by #656 as `BOILERPLATE-TRIVIAL`
   (C139,
   C298,
   C311,
   C317,
   C341,
   C346,
   C432,
   C573,
   C907,
   C914,
   C922,
   C947),
   under the concepts `error-subclass-constructor` and `fork-error-classes`.
- 3 recorded `GENERATED-MIRROR` and owned by #612 (C26,
   C121,
   C127),
   which is correct:
   those are generated copies,
   not a design question.
- 1 recorded `UNCERTAIN` and owned by #642 (C302).
- 1 recorded `EXTRACT-SHARED` and owned by #624 (C373).

So 212 units presenting as 17 clusters in 4 issues were never counted as a family,
and the 12 dismissed clusters carry the reasoning
"two-line `super(message)` plus `this.name = '...'` bodies.
`PP4` requires a custom error class per failure mode,
so the repetition is the intended pattern."
`PP4` does require a class per failure mode.
It does not require each class to hand-assign a name that a base class can derive,
and at 212 sites the repetition is 212 chances for the name to stop matching the class.

A second family sits above the floor too:
11 indexed units contain `attachShadow(`,
all at 20 nodes or more,
and `rg` finds 22 sites in total.
The other 11 are in `webapp-productivity/done-postcss`,
which the `done-*` pattern in `slopo.conf.yaml` excludes from indexing,
verified with `pathspec`'s gitignore matcher.
That exclusion is deliberate:
`done-postcss/package.json` calls it a PostCSS variant
"kept for side-by-side CSS framework comparison",
and #69 tracks consolidating the fork.
What nothing owns is a shared base class or `defineComponent` helper for the 11 components
inside each tree.

## Why clustering cannot see families

slopo clusters by embedding similarity above `analyze_similarity_threshold: 0.93`
and `analyze_rerank_threshold: 0.95`,
then drops overlapping units.
Two error constructors whose bodies differ only in a class-name literal are near-duplicates,
not exact ones,
so they form a cluster only when the similarity is high enough,
and a 212-member idiom surfaces as 17 clusters or fewer.
Exact-body grouping sees only identical bodies,
which is why the 690 and 1274 figures above undercount idiom families and overcount
copy-pasted one-liners.

The consequence for the review is structural rather than a tuning complaint:
the unit of judgement was the cluster,
and every verdict in the ledger is recorded as `full-read` of the reported bodies,
with no caller,
ownership,
or existing-utility check.
"Extraction costs more than it saves" was asserted per pair and never tested against the size of
the family the pair belongs to.

## What I read, and what it showed

Two kinds of evidence,
 because reading 147 clusters by hand is not what was missing.

Mechanical,
 over all 147 proposed entries.
The report grammar puts one body-hash group per `### ______ N ______` section,
so a single-section cluster means every unit has byte-identical body text.
Bodies were then tokenised with identifiers and literals normalised to placeholders,
so "same skeleton,
 different names" separates from "structurally different".

- 9 clusters have one body shared by every unit:
   8 with 2 units and 1 with 11 units.
- 39 clusters have the same token skeleton with different names or literals.
- 99 differ structurally,
   and 6 of those differ only by `async` and `await` plus names.

Hand reads,
 50 clusters,
 chosen as the largest and the highest-risk families.

- 24 deliberate-mirror clusters in #657 (`sync-async-twins`,
   `complementary-predicate-twins`,
   `differential-fuzz-adapters`,
   `or-throw-sibling-guards`,
   `perf-benchmark-arms`,
   `sync-async-test-twins`),
   diffed line by line.
   Every difference is on the intended axis:
   `await` and `Promise<T>` for the twins,
   `readFile` against `readFileSync`,
   `<=` against `<` for `isPrefix` and `isStrictPrefix`,
   swapped ternary branches and a De Morgan swap for the complements,
   `peek` against `get` and object-form against positional arguments for the fork adapters.
   No drift in any of them.
- 11 coverage-probe clusters in #658.
   Each unit is a distinct invalid input:
   `parseCss` on `.a {` against `.a` against `.a ; {}`,
   `pify` on `null` against `'nope'`,
   `config.set` on `__internal__` against `__internal__.x` against a nested value.
   Compressing them into a table would hide which branch each input reaches,
   which is the file's purpose.
- 4 of the 13 i18n clusters in #658.
   `en` and `ca` select a complement form through `nonFiniteSurface({ entry, form })`
   while `zh` reads `.surface`;
   `en` passes `{ sentence, deps }` to `renderWhSubject` while `zh` passes `sentence`;
   `ca` reads `entry.infinitive` where `en` reads the verb entry whole.
   The differences are grammatical,
   as claimed.
- The 11 judgement-heavy clusters in #659,
   read against their source.

Among the clusters whose bodies differ,
 the hand reads found no counterexample.
Every counterexample is in the identical-body group.

## Nine proposed dismissals are byte-identical copies

Bodies quoted exactly as indexed.

- C13,
   #656,
   `BOILERPLATE-TRIVIAL`,
   11 identical units:
   `{ return formatter.write_str(self.message.as_str()); }`.
   Eleven byte-identical Rust `Display` impls,
   out of 50 `Display` impls in the repository.
   The recorded reason is "idiomatic per-type Rust `Display` impls".
   Idiomatic and identical are different claims:
   11 copies of one line is what a `macro_rules!` in a `rust-module/*` crate removes,
   and `thiserror` would too,
   though it appears in no `Cargo.toml` in the repository and only in third-party audits
   (`doc/audit/turso-cargo-toml-0.6.1.md` and two vetting docs),
   so adopting it is a `choosing-technology` decision rather than a fix.
- C617,
   #657,
   `INTENTIONAL-VARIANT`,
   2 identical units:
   `{ if (getToolMode() === 'zoom') event.preventDefault(); }`.
   The recorded reason is "mirror keydown/keyup handlers,
   opposite cursor".
   The bodies are identical,
   so nothing about them is opposite;
   the difference lives in which event each is registered for.
   One handler function registered for both events would be behaviour-preserving.
- C665,
   #658,
   `STRUCTURAL-IDIOM`,
   2 identical units:
   `{ return dispatch({ model, context, ...(options === undefined ? {} : { options, }), },); }`.
   `STRUCTURAL-IDIOM` is defined in `slopo.ignore.txt` as
   "shared skeleton where each body's actual logic differs".
   Here the logic is byte-identical and the difference is in the parameter types,
   which is a real distinction but not that category's.
- C15,
   #656,
   `BOILERPLATE-TRIVIAL`,
   2 identical units:
   `{ return ConfigError { message: String::from(message), }; }`.
   Same Rust error-boilerplate family as C13.
- C629,
   #656,
   2 identical units:
   `return w.toplevel().unwrap().wl_surface() == &root`.
- C636,
   #656,
   2 identical units:
   `{ send_and_wake(&self.tx, &self.worker, command); }`.
   Both already delegate to one shared function,
   so nothing is left to extract.
   This dismissal is correct.
- C639,
   #657,
   2 identical units:
   the `expiresIn` fork and upstream adapters.
   Defensible for differential fuzzing,
   where both sides are written out on purpose,
   but the recorded reason describes an API difference that the body does not contain.
- C669,
   #656,
   2 identical units:
   `{ self.playing.store(on, Ordering::Relaxed); }`.
- C688,
   #656,
   2 identical units:
   `{ return Rc::new(VecModel::from(Vec::new())); }`.

The last four are one-line bodies where extraction saves nothing and the dismissal stands.
C13,
 C15,
 C617,
 C639 and C665 are the ones whose recorded reasoning does not match their evidence,
and C13 is the one with a real remedy.

## Retractions from the first pass of this investigation

- The suggestion that C866 and C989 could share an in-file helper is withdrawn.
   `package/module/hyperscript/README.md` documents the family as the design:
   "Branded value constructors replace raw strings,
   preventing invalid units and disallowed color functions at the type level",
   and lists `cssRem`,
   `cssEm`,
   `cssCh`,
   `cssLh`,
   `cssVi`,
   `cssVb`,
   `cssCqi`,
   `cssCqb`,
   `cssDvi`
   as named exports.
   `cssMin` and `cssMax` returning a branded `CssValue` is the point,
   not the duplication.
- Handing measured site lists to #617 and #618 is redundant.
   #617 already enumerates roughly 70 call sites across 16 clusters and records real drift:
   the whole-string digit predicates disagree about the empty string and about leading zeros.
   #618 already enumerates its guard family,
   including `isStringArray` at six sites with two different narrowings,
   and proposes `isNonEmptyString`.
   Those two owners did family-level work;
   the dismissal issues did not.
- The "119 lower-risk against 28 judgement-heavy" split was a heuristic built from package
   placement and never tested.
   The identical-body test replaces it,
   because it is mechanical,
   covers all 147,
   and found the counterexamples the heuristic missed.
- The 50-token shared-run cut that replaced it was also wrong,
   for a different reason:
   it measured the pair instead of the family.
   That retraction and the corrected partition are in
   "Why \"under 50 tokens\" was the wrong excuse".

## The error-name family, measured properly

- 212 indexed units contain `this.name = `,
   across 147 files,
   minimum `body_node_count` 13.
- 154 are the pure two-statement idiom,
   `super(message,); this.name = 'XError';`.
- 58 do more:
   extra fields such as `this.status`,
   `this.url` and `this.reason`,
   a tagged logger built in the constructor,
   or a multi-line message.
   A base class removes the name line from those but keeps their constructors.
- 21 forward an `options` argument to `super`,
   so any base class must forward it too.
- 209 assign a string literal and 3 already assign `X.name`,
   as in `this.name = SkillMirrorManifestError.name` in root `file-enforcer.config.ts`.
   The pattern that makes the literal redundant already exists in the repository.
- All 209 literals found by `rg` match their enclosing class name,
   checked by a comparison proven able to fail against a planted mismatch.

So a base class assigning `this.name = new.target.name` would delete 154 constructors and one line
from 58 more,
with `mangle: false` in all three rolldown configs keeping the names intact in published artifacts.
It has not drifted yet,
it touches 147 files,
and published `module/*` packages would gain a dependency edge.

## What slopo cannot tell you

- No stale-entry reporting exists.
   Its nine commands are `init`,
   `show-config`,
   `index`,
   `embed`,
   `analyze`,
   `review`,
   `agent-configs`,
   `agent-review` and `agent-analyze`;
   none compares `slopo.ignore.txt` against the current report,
   and nothing in `slopo/result/analysis/` mentions unused,
   stale or orphaned entries.
- Clustering cannot size a family.
   Two error constructors differing only in a class-name literal are near-duplicates,
   not exact ones,
   so a 212-member idiom surfaces as 17 clusters or fewer,
   split across four issues and four verdicts.
- What can size one is already in the index.
   `code_units.body`,
   `body_node_count` and `body_hash` are populated at threshold 13,
   so one query counts a family with no re-index,
   no embedding and no API cost:
   that is how 212 was produced.

## The test that should have been used first

"Bodies differ" does not settle whether an extraction is demanded.
Two bodies can differ in their data while sharing one policy,
so the right instrument measures the shared part and asks what kind of thing varies.

For all 147 proposed entries,
the longest common token subsequence between each cluster's bodies was computed,
sized against `body_node_count_threshold: 13`
(the repository's own bar for "this is a function worth indexing"),
and the variant tokens were classified as names,
literals,
punctuation,
`async` and `await` markers,
or polarity operators.

- 9 clusters have byte-identical bodies,
   27 units.
- 34 clusters share a run of 50 tokens or more,
   roughly five lines,
   across 97 units.
- 104 clusters share less than 50 tokens,
   across 291 units.

The last group is where this investigation first claimed dismissal was safe on measurement alone,
on the reasoning that a helper would be longer than what it replaces.
That claim is withdrawn in "Why \"under 50 tokens\" was the wrong excuse":
pair size does not decide whether a utility is worth extracting,
family size does,
and the 104 clusters were then sized by pattern instead.
The first two groups were read individually.

## Verdicts on the 34 clusters with a function-sized shared part

Held back,
 because the recorded reason does not survive the measurement:

- Seven i18n clusters in #658:
   C327 (168 of 169 tokens shared across 8 units),
   C353 (150 of 152),
   C348 (397 of 407),
   C321 (135 of 145),
   C582 (172 of 186 across 6 units),
   C328 (103 of 121) and C336 (60 of 75).
   The grammar difference is real and is expressed as which field to read:
   `en/render-sentence-core.ts` selects `entry.imperative ?? entry.base` where
   `ca/render-sentence-core.ts` selects `.imperative ?? .infinitive`,
   and the adverbial renderers differ in `locationPreposition(adv.relation)` against
   `locativeCoverb(adv.relation)` against a bare `${adv.relation}`.
   That is a handful of tokens inside 60 to 140-line functions that are otherwise identical.
   #639 already proposes the mechanism for this package,
   "the renderers and case invariants injected",
   and its criterion names `renderOptionalComplement`,
   `renderOptionalObject`,
   `renderPart`,
   `renderTimeOperand` and `capitalizeBody`.
   These seven are the same design question one level up,
   so they belong in #639's scope rather than in a dismissal.
   C353's three bodies also each open with a byte-identical `renderTimeOperand`,
   which #639 lists as its own cluster C331,
   so dismissing C353 hides code an open extraction issue already claims.
- C604 in #657,
   8 units sharing 67 of 68 tokens.
   These are `batch_sheng`,
   `batch_sheng2`,
   `is_match_batch_scalar`,
   `is_match_batch_interleaved`,
   `batch_inter_w::<N>` and `batch_tight_w::<N>`,
   and every body is the same three-step wrapper:
   allocate `vec![false; lines.len()]`,
   match on `self.engine.table_dfa()`,
   call one kernel or fall back to `self.engine.is_match_batch`.
   Only the kernel name varies.
   The recorded reason,
   "the `cfg` and feature-detection ladder has to be written per kernel because each calls its own
   intrinsics",
   describes the kernels in `dfa/sheng.rs` and `dfa/sheng2.rs`,
   not these dispatch wrappers,
   which contain no intrinsics and no `cfg`.
   A `macro_rules!` or one generic wrapper taking the kernel as a parameter removes all eight bodies.
- C963 and C906 in #657,
   `runPipe` against `runPipeAsync`,
   sharing 519 of 575 tokens and 361 of 415,
   with the only variance being 54 `async` and `await` markers.
   This is the largest duplication in the whole dismissal set.
   The recorded reason is that unrolling preserves the per-arity types,
   which is true of the signatures and says nothing about the bodies.
   The open question is whether one side can be generated from the other,
   or share a core,
   without losing those types.
- C896 in #657,
   `stream` against `streamSimple`,
   sharing 68 of 70 tokens with four name variants.
- C977 in #657,
   `embedAll` against `embedBatchAll`,
   sharing 87 of 93 tokens with eight name and four literal variants.
   The batch form can share a core with the single form even though their public contracts differ.

Kept dismissed,
 because the variance is the point of the code:

- C744,
   376 of 406 tokens shared,
   the fork and upstream adapters in `p-map-fork.fuzz`.
   Differential fuzzing needs both sides written out so that an observation difference means a
   behaviour difference.
   The duplication is the method.
- C771,
   C935 and C873,
   the SIMD kernels themselves,
   sharing 48%,
   78% and 42% with heavy operator and intrinsic variance.
   These are different machine code,
   unlike C604's wrappers.
- C853,
   C616,
   C784,
   C892,
   C900,
   C911 and C979,
   the coverage probes.
   Each body is one invalid input,
   and the input is the test.
   A table would hide which branch each reaches.
- C753,
   C758 and C910,
   sync and async twins sharing 95%,
   94% and 79%,
   where every variant is `async`,
   `await`,
   `Promise<T>` or `readFile` against `readFileSync`.
   The duplication is the deliberate cost of two typed entry points,
   and no drift was found in any of them.
- C700 (`prependComments` against `appendComments`,
   97% shared) and
   C764,
   C751 and C903 (complementary predicates).
   A shared version needs a polarity parameter that restates what the two names already carry.
   This is a values call the repository has already made in `slopo.ignore.txt`.
- C814,
   the `or-throw` sibling guards:
   the kind of iterable is the API.
- C1003,
   the two `p-map-fork` engines,
   29% shared and already verified against the package README.
- C976,
   the record and array branches of one recursive JSONC delete.
- C745,
   `get` against `peek` in quick-lru:
   recency updating is the difference.
- C973,
   the four `createOnce` rule factories,
   whose shared scaffolding the plugin already extracted.
- C993 and C808,
   coverage-fixture iterators whose yield schedules are the fixture.
- C865,
   two hidden subcommand branches.
   C770,
   six per-API mutation-differential cases.
   C804,
   mirror key handlers.
   C599,
   the shell against git-config quoting pair,
   where the shared 62 tokens are `Array.from(...).map(...).join(...)`
   and the escape tables are the whole content.

## Why "under 50 tokens" was the wrong excuse

The partition above ended by accepting 104 clusters because their shared run was under 50 tokens.
That reasoning is withdrawn.
Pair size does not determine whether a utility is worth extracting;
family size does.
A two-line predicate repeated at 28 sites is a better extraction candidate than a 60-line function
duplicated once,
because the value of a shared utility is one place to hold a policy times the number of consumers,
not lines saved per pair.
This repository is full of that shape:
`module/or-throw`,
`module/numeric-format`,
`module/throws` and `agent-harness-shared/text-scan` are all small utilities with many consumers.

So the test is three questions,
none of which is "how much code do the two sides share":

1. How many units repo-wide instantiate this idiom?
2. Does the shared part hold a policy that can be wrong,
   or drift,
   such as a comparison,
   a boundary condition,
   a format string,
   an escape table or an error kind?
3. Does an owner already exist?

Sizing families by skeleton grouping does not work on its own.
Normalising identifiers and literals collapses every one-expression template return into one
skeleton,
which produced a spurious 127-unit family across 40 packages for four unrelated clusters
(C342,
 C706,
 C731,
 C999 and C994).
The reliable instrument is a targeted pattern per idiom,
which is how the numbers below were produced.

## Families behind the dismissals, measured by pattern

- Error-class naming:
   `extends Error` at 241 declarations across 181 files,
   and `this.name = ` at 210 sites across 146 files.
   The policy is "the error's name matches its class",
   currently enforced by hand at 210 sites.
   A base class assigning `this.name = new.target.name` moves it to one.
   Measured drift today:
   none,
   verified by a comparison proven able to fail against a planted mismatch.
   Twelve proposed dismissals belong to this family:
   C139,
   C298,
   C311,
   C317,
   C341,
   C346,
   C432,
   C573,
   C907,
   C914,
   C922 and C947.
- Rust one-line `Display` forwarding a message field:
   `write_str(self.message` at 11 sites
   across 11 files,
   out of 50 `Display` impls across 32 files.
   C13 is this family and its 11 units are byte-identical.
- Rust one-line error constructors:
   `String::from(message` at 17 sites across 16 files.
   C15 belongs here.
- Rust error kinds:
   `io::Error::new(io::ErrorKind::` at 15 sites across 8 files,
   11 of them in `wg/cli-quicker-exempt`.
   The policy is which kind maps to which failure.
   C998 belongs here.
- Web-component shadow roots:
   `attachShadow(` at 22 sites across 22 files,
   of 25 `extends HTMLElement` declarations.
   The policy is the shadow mode and where the root is stored.
   C305 is 9 of them;
   the other 11 are in `done-postcss`,
   which `done-*` excludes from indexing and #69 tracks as a fork.
- Null-prototype records:
   `Object.create(null` at 21 sites across 14 files.
   C355 belongs here.
- Comparators:
   `.localeCompare(` at 40 sites across 37 files,
   with no shared comparator utility.
   #659 dismisses C315 and C358 as coincidental comparators while its own caveat 1 proposes
   `compareBy([...keyFns])`.
   The family measurement supports the caveat rather than the dismissal.
- Not families:
   `Array.isArray(` at 296 hits and `Promise.resolve(` at 350 hits are language
   primitives used inline.
   Counting them overstates duplication,
   which is why #618's proposal is about named guards such as `isRecord` and `isStringArray`
   rather than every call.

## Why family size was also the wrong test

Family size is a magnitude,
 and the question is generative.
A pattern instantiated twice is a template for the third copy,
so a family of 2 and a family of 210 have the same implication:
if nobody owns the pattern,
 it keeps being copied.
This repository is the evidence.
The error-name constructor was 2 sites when it was first written and is 210 today;
`attachShadow` is 22;
`.localeCompare(` is 40;
`Object.create(null` is 21.
None of those grew because someone decided to duplicate;
each grew because the previous copy was the nearest thing to a template.

So the deciding question for a dismissal is not how much code the two sides share
and not how many copies exist today.
It is whether future work will instantiate the pattern again,
and if so what should own it.
That produces three buckets,
and every one of the 86 concepts behind the 147 proposed entries was classified.

## Bucket A: propagating patterns that need an owner

40 clusters in 17 concepts.
The shared part holds a policy,
and new instances appear whenever the repository grows in that direction.

Eleven of the seventeen have no owner today:

- Error-class naming,
   12 clusters (C139,
   C298,
   C311,
   C317,
   C341,
   C346,
   C432,
   C573,
   C907,
   C914,
   C922,
   C947).
   210 `this.name = ` sites across 146 files of 241 `extends Error` declarations.
   Owner:
   a base class assigning `this.name = new.target.name`.
- Rust one-line `Display`,
   C13,
   11 byte-identical impls of 50.
   Owner:
   a `display_message!` macro in a `rust-module/*` crate.
- Rust one-line error constructors,
   C15,
   17 `String::from(message` sites across 16 files.
   Owner:
   the same macro family.
- Rust error kinds,
   C998,
   15 `io::Error::new(io::ErrorKind::` sites across 8 files,
   11 of them in `wg/cli-quicker-exempt`.
   Owner:
   one helper that chooses the kind.
- Web-component shadow roots,
   C305,
   22 `attachShadow(` sites of 25 `extends HTMLElement`.
   Owner:
   a base element class or a `defineComponent` helper.
   #69 owns the `done` against `done-postcss` fork,
   not the base.
- Null-prototype records,
   C355,
   21 `Object.create(null` sites across 14 files.
   Owner:
   a `nullRecord()` helper or a lint rule.
- Comparators,
   C315 and C358,
   40 `.localeCompare(` sites across 37 files.
   Owner:
   `compareBy([...keyFns])`,
   which #659's own caveat 1 proposes.
- Module-id query stripping,
   C187.
   Two sites today,
   both in plugins that parse module ids,
   and every future id-parsing plugin is a
   third.
   Owner:
   one stripper taking the marker explicitly.
- Batch dispatch wrappers,
   C604,
   eight wrappers sharing 67 of 68 tokens.
   Owner:
   one macro or a generic wrapper taking the kernel as a parameter.

Six have an owner that does not yet know about these clusters:

- i18n per-locale renderers,
   all 13 clusters (C321,
   C322,
   C327,
   C328,
   C336,
   C348,
   C353,
   C361,
   C364,
   C450,
   C504,
   C582,
   C597).
   Owner:
   #639,
   which already proposes "the renderers and case invariants injected" for 11 sibling
   clusters.
   A fourth locale copies the whole tree,
   so the propagation is per locale rather than per helper.
- Shell and git-config quoting,
   C588 and C599.
   Owner:
   #650,
   which covers html,
   xml,
   markdown and JSON escaping but not these two grammars.
- Line-ending normalisation,
   C295.
   Owner:
   the text module #617 proposes.
- Abort-controller disposables,
   C351.
   Owner:
   #620.
- Path segments,
   C833.
   Owner:
   #619,
   which is consolidating `module/fs-path`.
- Manifest license readers,
   C878.
   Owner:
   #633,
   which owns the file,
   while `dev-script/deps-cube` already has the reader that also unwraps the object form.

## Bucket B: propagating habits that need a policy decision

40 clusters in 20 concepts.
The pattern here is not a utility anyone forgot to write;
it is the habit of writing both sides by hand,
and it propagates across the repository even though each instance is a closed pair.

- Differential-fuzz adapters,
   10 clusters.
   Each new forked package needs an adapter pair,
   and there are 13 sidecars today.
   The two sides must stay explicit,
   because identical adapter code is what makes an observation
   difference mean a behaviour difference,
   so the owner is the scaffolding in #613's shared harness
   rather than a merged adapter.
- Sync and async twins,
   7 clusters plus 1 test cluster,
   and 22 `*Async` function names today.
   Hand-writing both sides is the current policy and nothing records it as a choice.
- Pipe arity unrolling,
   C906 and C963,
   sharing 519 and 361 tokens with only async markers varying.
   Each new arity adds another unrolled copy.
- Complementary predicates,
   3 clusters,
   plus the mirror families:
   per-event handlers (2),
   key handlers,
   navigation,
   stream pause and resume,
   priority providers,
   root markers,
   column close,
   edit conflicts,
   single against batch,
   get against peek,
   platform sink sets,
   fixture compilers,
   filter composition and type-case emitters.
   Each is a closed pair,
   and each teaches the next author to write two.
- The repository has already made one of these calls explicitly:
   `slopo.ignore.txt` records that a polarity parameter would restate what two names carry.
   That is a policy,
   and it should be stated as one for the whole bucket rather than re-argued
   cluster by cluster.

## Bucket C: closed patterns and data

67 clusters in 49 concepts.
Either the pattern cannot grow,
or each instance is data rather than code that could be owned:
coverage probes where the invalid input is the test,
benchmark arms where the difference is the experiment,
fuzz arbitraries and fixtures per grammar construct,
per-node-type visitors,
protocol line formats fixed by git,
two-CLI output contracts,
two payload conventions,
two enums,
two unrelated frameworks,
and the coincidental shapes whose shared part is a language idiom carrying no policy.
Four of these already have their pattern owned in place,
which is why the dismissal is safe:
`one-line-forwarders` and `engine-send-wrapper` both delegate to a shared function,
`oxlint-createonce-scaffolding` uses the `createOnce` the plugin already extracted,
and `or-throw-sibling-guards` lives in `module/or-throw`.

## What this says about the taxonomy itself

Every category in `slopo.ignore.txt` is a magnitude claim about a pair:
`BOILERPLATE-TRIVIAL` says extraction costs more than it saves,
`COINCIDENTAL-SHAPE` says the units do unrelated work,
`STRUCTURAL-IDIOM` says the skeleton is shared but the logic differs,
`INTENTIONAL-VARIANT` says the two sides are meant to mirror each other.
None of them asks whether the pattern will be instantiated again,
and none names an owner.
That is why a review using only these categories can dismiss 210 copies of one idiom as twelve
unrelated instances of trivial boilerplate.

The missing question is the one this document now asks first:
who owns this pattern,
and if nobody does,
what stops the next copy.

## Judgements adopted

These are calls,
 not options,
 and each is recorded here so it can be vetoed rather than rediscovered.

1. Hold 80 of the 147 proposed entries,
    the 40 in bucket A and the 40 in bucket B,
   and accept the 67 in bucket C.
   A bucket C entry is accepted because the pattern cannot grow or because each instance is data,
   never because the shared code is small.
2. Add the propagation question to the dismissal convention in
   `doc/troubleshooting/slopo-threshold-tuning.md`:
   an entry must name the owner of the pattern,
   or state that the pattern is closed,
   or state the policy that accepts the copies.
   "Too small to extract" is not a reason on its own.
3. Hand the six owned bucket-A concepts to their issues with the measured site lists:
   the 13 i18n clusters to #639,
   the two quoting clusters to #650,
   C295 to #617,
   C351 to #620,
   C833 to #619 and C878 to #633.
4. File the eleven ownerless bucket-A patterns as proposals rather than dismissals,
   grouped by remedy:
   one base error class for the 210 sites;
   one Rust macro family for the 11 `Display` impls and 17 one-line constructors;
   one `io::ErrorKind` helper for the 15 sites;
   one base element for the 22 shadow roots;
   one `nullRecord()` helper for the 21 sites;
   one `compareBy` for the 40 comparator sites;
   one module-id stripper;
   one dispatch macro for the eight forbidden-regex wrappers.
5. Put bucket B to you as one policy decision rather than 40 dismissals:
   either the repository adopts a rule for the two-sided habit
   (share a core,
   generate one side,
   or parameterise)
   or it records the opposite choice as policy,
   the way `slopo.ignore.txt` already records the polarity-parameter call.
   Until one of those is written down,
   each new pair is authored without a decision behind it.
6. Correct the identical-body clusters whose recorded reasoning claims a difference the bodies do
   not contain:
   C617 (one handler serves both events),
   C665 (identical bodies cannot be `STRUCTURAL-IDIOM`),
   C639 (the reason is about signatures),
   and C629,
    C636,
    C669 and C688 (the reason is that the pattern is closed at two,
   which is now measured rather than assumed).
7. Record three recipes in the tuning doc:
   the family-count query over `code_units`,
   the shared-run comparison over report bodies,
   and the targeted-pattern family count,
   with the warning that skeleton grouping alone produces artifacts
   (it collapsed every one-expression template return into a spurious 127-unit family).
8. Give the three held-back `UNCERTAIN` clusters a home:
   C562 joins #660 as an eighth judgement call,
   and C622 and C721 get one focused issue.
9. Copy the review ledger into `doc/artifact/` as a dated record.
10. Leave `slopo.conf.yaml` and `mise.toml` untouched;
   13 stays.
   Two wording fixes are proposals only:
   the conf comment "increased from default 10" describes a `0.4.0` default,
   since `0.8.0` defaults to 20 at `slopo/config.py:151-152`;
   and `mise.toml:213` installs `pipx:slopo` at `latest` while the tuning doc is verified against
   two specific versions whose source paths already moved.

## Executed on 2026-10-09

All fourteen judgements were carried out.
The record:

- The four dismissal issues were rewritten around the partition:
   #656 proposes 16 and holds 17,
   #657 proposes 8 and holds 42,
   #658 proposes 30 and holds 15,
   #659 proposes 13 and holds 9.
   67 entries are proposed in total,
   each under the heading its recorded verdict names,
   and 80 are held with the owner or decision each waits on named in its entry.
   Verified at the tracker boundary:
   proposed and held hash sets do not overlap,
   fences are balanced,
   and no comment is truncated.
- C665 was re-filed under `INTENTIONAL-VARIANT` with a hand-written comment,
   because its two bodies are byte-identical and `STRUCTURAL-IDIOM` requires differing logic.
- Six issues received the measured site lists for patterns they own and did not know about:
   #639 (13 i18n clusters with their shared-run figures,
   plus the note that C353 contains the `renderTimeOperand` it claims as C331),
   #650 (the shell and git-config quoting grammars),
   #617 (line-ending normalisation and the 28-site non-empty-line predicate in two spellings),
   #620 (the `AbortController` disposable pair),
   #619 (`splitSegments` for `module/fs-path`) and #633 (the two license readers and the third
   reader in `deps-cube` that unwraps the object form).
- #665 was opened with the eight drafted owner designs,
   each carrying its measurement,
   code shape,
   migration,
   risk and falsifier,
   plus the two proposed `AGENTS.md` rules with tag codes checked against both forbidden-strings
   appendixes.
- #664 was opened for the two held-back `UNCERTAIN` clusters in the quick-lru family,
   and C562 became item 8 of #660,
   which resolves the pointer #647 made to "the remaining-uncertain issue".
- Reading the source for #664 corrected the review's own evidence:
   the coverage driver's eviction text is not differently formatted,
   because `renderPair` returns the same `${key}=${value}`,
   and the real finding is that the driver discards its array with `void evicted;` and nothing reads
   it.
   The same file holds six iteration generators rather than the three the cluster reports,
   with one expiry-filter policy written identically three times.
   #657's body carries the correction inline.
- `doc/troubleshooting/slopo-threshold-tuning.md` gained rule 9 of the recording convention,
   requiring a dismissal to name the pattern's owner or state that it is closed or state the policy
   that accepts the copies,
   and a "Measuring a family instead of a pair" section with the three instruments and the
   skeleton-grouping trap.
- The review ledger is now a repository artifact,
   `doc/artifact/slopo-cluster-review-ledger-2026-10-07.jsonl` with a companion explaining its
   fields,
   distributions and limits.
- `slopo.conf.yaml` and `mise.toml` were left untouched.
   The two wording fixes remain proposals recorded in the judgements.
- `slopo.ignore.txt` was restructured earlier the same day into three explicit states,
   because all 55 of its keys were dead:
   23 annotated with the successors the dismissal issues propose,
   23 commented out as stale with no recorded successor,
   and 9 deleted where the successor is owned by a code-change issue.

## Decisions taken and applied on 2026-10-09

The maintainer reviewed the partition and the drafted designs and decided all of it in one pass.
What was applied:

- The 67 bucket C entries were accepted and written into `slopo.ignore.txt`,
   each under the heading its recorded verdict names.
   Verified with slopo itself against a copy of the 2026-10-07 index,
   so the reference report was left intact:
   the run logged `Ignored 67 previously reviewed clusters.`,
   reported 941 clusters instead of 1008,
   and the removed hash set equalled the accepted set exactly with no additions.
   The heading counts are now `INTENTIONAL-VARIANT` 9 live plus 10 dead pending,
   `STRUCTURAL-IDIOM` 30 live,
   `COINCIDENTAL-SHAPE` 11 live,
   `BOILERPLATE-TRIVIAL` 16 live plus 1 dead pending,
   and `TEST-OR-GENERATED` 1 live.
   Twelve dead keys whose successors were accepted were removed,
   eleven whose successors are held were kept and annotated,
   and the 23 stale keys stay commented out with their rationale.
- The eight owner designs became eight issues,
   one per pattern rather than one bundled proposal:
   #666 base error class,
   #667 Rust display and constructor macros,
   #668 `io::ErrorKind` helper,
   #669 shadow-element factory,
   #670 `nullRecord()`,
   #671 `compareBy`,
   #672 module-id query stripper,
   #673 batch dispatch macro.
   #665 is now their index with the recommended sequencing.
- The two rules were redirected out of `AGENTS.md` into
   `.agents/skills/project-code-review/SKILL.md` as `GRW` and `TWS`,
   broadened so they fire on any diff rather than only on duplicate-code dismissals.
   The narrower dismissal-side form also landed as rule 9 of the recording convention in
   `doc/troubleshooting/slopo-threshold-tuning.md`.
- #664's two clusters and #660's item 8 have recorded decisions,
   both consolidating rather than dismissing.
- Both wording fixes were applied:
   `slopo.conf.yaml` now says that 13 is a hand-tuned floor and that `0.8.0` defaults to 20,
   so the value is more permissive than upstream rather than less;
   and `mise.toml` pins `pipx:slopo` to `0.8.0` with the reason,
   since an unpinned upgrade can invalidate every suppression silently and slopo has no dead-key
   detection.
   The `mise.toml` pin is applied in the working tree but was left uncommitted,
   because that file carries an unrelated uncommitted change to `_.path` that is not this work's to
   commit.

What was not done,
 deliberately:
no production source changed.
Implementation of the eight owners belongs to the issues that now track them.

## Verification limits

- The bucket A,
   B and C classification is a hand judgement per concept,
   made for all 86 concepts
   behind the 147 entries.
   It is the weakest link in this document:
   the site counts and shared-run figures are measured,
   but "will this pattern be instantiated again" is a forecast.
   Bucket C is where a wrong call hides a propagating pattern,
   and its 49 concepts are listed by name in "Bucket C:
   closed patterns and data" so each can be
   challenged individually.
- A pattern's propagation can also be closed by an owner that already exists in place,
   which is why four bucket C concepts are named as already owned.
   That check was made by reading the bodies,
   not by tracing every call site.

- The probe index at threshold 4 used a throwaway database and did not touch `.slopo.local.dir/`.
   It indexed only;
   it did not embed,
   so no clustering ran on the new units.
   The duplicate counts are exact-body-hash groups,
   not slopo clusters.
- The identical-body test relies on the report's section grammar,
   one body-hash group per section,
   which the triage handover validated against `index.md` for all 1008 clusters.
- The skeleton comparison that produced the first 99-cluster "structurally different" figure
   normalises identifiers and literals but not operator or keyword choice,
   so it reported sync/async twins as structurally different.
   It was replaced by the shared-run measurement in
   "The test that should have been used first",
   which sizes the common part and classifies the variance instead of asking whether the bodies
   match.
- The shared-run figure is a longest-common-token-subsequence length,
   not an AST node count,
   so "50 tokens" is an approximation of the 13-node floor rather than the same unit.
   It also measures the best pair in a cluster,
   so a cluster of eight units is summarised by its most similar two.
- The 34 clusters with a function-sized shared part were read individually.
   The 104 clusters sharing under 50 tokens were not read,
   and their acceptance rests on the measurement that a helper would be longer than what it
   replaces.
- Site counts are `rg` and index queries over tracked TypeScript and Rust at HEAD on 2026-10-09.
   slopo indexes transpiled `.js` that the repository's blanket `*.js` ignore hides from `rg`,
   which is why build output appears in some duplicate groups.
- Whether a 147-file mechanical change is worth making to prevent drift that has not happened is a
   values judgement about depth against governance,
   so judgement 6 is a recommendation and not a fact.
   The same applies to the polarity pairs in C700,
   C751,
   C764 and C903,
   where the repository has already chosen two names over one parameter.
