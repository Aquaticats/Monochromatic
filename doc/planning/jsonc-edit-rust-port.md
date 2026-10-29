# Native Rust port of jsonc-edit

## Status

Requirements confirmed by the user on 2026-09-24 ("Do it.").
 Implementation is authorized,
 subject to a separate foundation-adoption gate before work depends on an unselected external component.
 Crates.io publication is explicitly requested.

Foundation adoption was granted by the user on 2026-09-24 (option A):
 the repository-owned parser,
 emitter and exact-number identity,
 with no third-party parser or numeric dependency.
 Both vet reports record the evidence,
 scores and sensitivity behind that recommendation,
 and
 `doc/decision/jsonc-edit-rust-foundation.md` is the accepted decision record.

Progress since adoption:

- `package/rust-module/jsonc-edit` holds the crate `monochromatic-jsonc-edit` 0.1.0:
   scanner,
   iterative depth-bounded parser,
   canonical emitter,
   comment merging,
   UTF-16 text conversion,
   exact-number identity,
   immutable edit state,
   address reads,
   set and delete,
   and the comment query and edit surface.
   54 tests pass in debug and release,
   `cargo clippy --release --all-targets -- -D warnings` is clean,
   and `monochromatic-rust-linter` reports no findings.
   Edits rebuild on an explicit spine because a recursive rebuild overflowed a debug test thread's
   stack at the accepted 512-container depth.
- The maintained TypeScript package's confirmed defects are fixed,
   each with a guard test shown
   failing before the fix and passing after:
   a separator following trivia on a later line,
   comment ownership around a later-line comma,
   CR and CRLF line-comment termination,
   unedited number spelling on clean input,
   and the clean-input nesting-limit bypass.
   The last two required removing the native `JSON.parse` fast-path,
   which `doc/decision/jsonc-edit-parser-foundation.md` now records as an amendment with the
   measured clean-input cost (11055 to 2794 ops/s) and the comparison-library numbers.
   The package unit suite,
   the conformance corpus and the fuzz property suites pass after the change.

- Language-neutral fixtures are shared:
   `fixtures/jsonc-conformance.json` exists as byte-identical copies in
   `package/module/jsonc-edit.conformance` and `package/rust-module/jsonc-edit`,
   both suites read it,
   and a `test:shared-fixtures` task in each package fails on drift.
   The fixture covers accepted and rejected sources,
   root shape,
   comment ownership on the root,
   keys and values,
   preserved number spelling,
   mathematical equality,
   and the nesting boundary.
   Cases needing exact mathematical identity are flagged,
   because the TypeScript package stores binary64 values and asserts those through preserved
   spelling instead.

- The packaged artifact was verified from a disposable consumer at
   `~/temp/agent/jsonc-packaged-consumer-2026-09-25`:
   `cargo package` produced 31 files,
   the extracted `.crate`'s own 61 tests passed,
   and the consumer exercised parsing,
   reads,
   immutable set and delete,
   comment queries and attachment,
   canonical emission,
   number spelling and identity,
   lone-surrogate retention,
   the nesting boundary and the rejection set.
- `.github/workflows/cargo-publish.yml` gained `je-detect` and `je-publish-crate`,
   a dispatch choice for `monochromatic-jsonc-edit` and a push trigger on the crate's manifest,
   mirroring the `forbidden-regex` library-crate jobs.
- `cargo publish --dry-run --no-verify` packaged and reached the upload step locally.

- Published:
   the user minted a bootstrap token after the stored one answered
   `status 403 Forbidden` with `authentication failed`,
   and `cargo publish --no-verify` uploaded `monochromatic-jsonc-edit` 0.1.0 on 2026-09-26.
   Verified from the registry side:
   `max_stable_version` is `0.1.0`,
   not yanked,
   `license` is `LGPL-3.0-or-later`,
   `crate_size` is `72070` bytes compressed against the `286.9KiB` that
   `cargo package` reports uncompressed,
   and the description and repository metadata match the manifest.
   `https://docs.rs/monochromatic-jsonc-edit/0.1.0/monochromatic_jsonc_edit/` serves `200`
   with `parse_jsonc`,
   `emit_jsonc_value`,
   `jsonc_set_comment` and `JsoncNumberIdentity` listed.
- Verified as a downstream dependency:
   a disposable consumer at `~/temp/agent/jsonc-published-consumer-2026-09-26` resolves
   `monochromatic-jsonc-edit = "0.1.0"` from
   `registry+https://github.com/rust-lang/crates.io-index` per its `Cargo.lock`,
   passes the whole API exercise,
   and the registry-downloaded source passes its own 61 tests including the shared fixtures.

- The release workflow integration was exercised,
   not just parsed:
   dispatching `cargo-publish.yml` with `crate=monochromatic-jsonc-edit` and `dry-run=true`
   (run `36217693067`,
   2026-09-26) concluded `success`,
   with `je-detect` success,
   `je-publish-crate` success including the `Build .crate package (verifies by compile)` and
   `Publish (dry run)` steps,
   and the attestation,
   Trusted Publishing authentication and real publish steps correctly skipped.
   The other three crates' detect jobs reported no bump and their publish jobs stayed skipped.
   The OIDC publish path itself is still unexercised,
   because it only runs for a version that is not yet on crates.io.

- Trusted Publishing is configured and proven.
   The user saved the crate's publisher row,
   and dispatching the workflow with **dry-run** unchecked
   (run `36217828047`,
   2026-09-26) concluded `success` with `Authenticate with crates.io (Trusted Publishing)`
   succeeding,
   which only happens when the OIDC claims match the configured owner,
   repository,
   workflow filename and environment.
   The publish step then logged
   `monochromatic-jsonc-edit 0.1.0 already on crates.io; skipping publish.`,
   so no upload occurred:
   the registry still lists exactly one version,
   `0.1.0`.
   `Attest .crate provenance` also succeeded on the same run.

- The bootstrap token was revoked by the user after Trusted Publishing was configured,
   so the crate now publishes through the workflow's OIDC route only.
- Property fuzzing exists for the Rust crate:
   `package/rust-module/jsonc-edit.fuzz` holds four libFuzzer targets
   (round-trip canonical stability and comment preservation,
   one-mutation rejection without panic or hang,
   the exact nesting envelope on both sides,
   and immutable edits at fuzzer-drawn resolving addresses),
   a structured `arbitrary` generator,
   shared invariant checks with negative controls,
   and thirteen unit tests.
   The clean bounded campaign ran 13184,
   160114,
   625656 and 452889 executions per target inside a 2 GiB and 2 CPU container with no crash
   artifacts.
   Five crashes found across the campaigns are triaged in
   `doc/handover/jsonc-edit-rust-fuzzing.md`:
   two were product defects fixed in **both** implementations
   (a comment body carrying a bare CR was emitted as a `//` line that ended early,
   and setting the root to a scalar produced a document the parser rejects),
   and three were defects in the harness's own invariants.
- Mutation testing ran over the crate:
   485 mutants,
   397 caught by a failing test,
   34 caught by timeout,
   52 unviable,
   and 2 survivors now excluded with measured proofs in
   `package/rust-module/jsonc-edit/.cargo/mutants.toml`.
   Twenty-one of the original twenty-four survivors were real test gaps and are now covered by
   assertions that name the expected refusal message,
   indentation level or emission text.
- Canonical layout is shared contract:
   the fixture corpus gained `roundTrip`,
   `rootShape` and `canonicalLayout` sections,
   the last measured from the TypeScript package and then verified byte for byte against the Rust
   emitter,
   so indentation,
   separators,
   trailing commas and comment placement cannot drift between the two implementations unnoticed.

Nothing in the port plan is outstanding.
Future versions publish by bumping `version` in the crate manifest and pushing to `main`,
or by dispatching `cargo-publish.yml` with **dry-run** unchecked.

## Existing boundaries

- `package/module/jsonc-edit` is a TypeScript JSONC parser,
   canonical emitter,
   immutable edit API,
   and attached-comment API.
   Its public exports are in `package/module/jsonc-edit/src/index.ts`.
- `doc/decision/jsonc-edit-parser-foundation.md` accepts the comment-as-data model,
   canonical serialization,
   and browser availability without WebAssembly for the TypeScript package.
- The TypeScript package has unit,
   property,
   conformance,
   and benchmark sidecars.
   A search of package and workflow source found no production import outside those sidecars;
   this does not establish that external consumers do not exist.
- The separate monorepo-manager design selects a Rust `jsonc-parser` wrapper for meow in
   `doc/planning/monorepo-manager-from-scratch-design.md`,
   under "Managed file editing".
   This port does not silently change that decision.
- `package/forbidden-strings/core/Cargo.toml` is a standalone,
   published Rust-library precedent,
   while `.github/workflows/cargo-publish.yml` currently handles only the named crates in its dispatch and push paths.
- `mise exec rust -- cargo info jsonc-edit` and the same probe for `monochromatic-jsonc-edit` both reported
   that the package could not be found in the registry;
   neither name is reserved by that probe.
   `cargo search serde_json --limit 2` returned results as a positive control for registry queries.
- `scanString` uses `JSON.parse(raw)` in `package/module/jsonc-edit/src/scan.ts`,
   so escaped unpaired UTF-16 surrogates are accepted.
   A direct call through the existing neutral bundle parsed `{"s":"\uD800"}`
   and `{"s":"\uD800",} // c` as a string whose first code unit is 55296,
   and re-emitted each.
   Rust `String` cannot represent this decoded code unit alone.
   Its native value representation requires a decision.
- A direct bundle probe showed a clean `{"n":1e0}` emits a number spelled `1`,
   while a commented `{"n":1e0,} // c` preserves `1e0`.
   This matches the `JSON.parse` clean fast path and structured parser implementation.
   Q6's spelling preservation is a stronger contract than current clean-path behavior,
   and Q8 requires a corresponding TypeScript fix if that contract remains shared.
- The [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html) says published versions
   cannot be overwritten or deleted.
   The [crates.io Trusted Publishing guide](https://crates.io/docs/trusted-publishing) says initial publication
   requires an API token before trusted publishing can be configured for the crate.
   Existing workflow comments on the library's first release describe manual bootstrap.

## Decisions from the user

- Q1:
   Build for native Rust callers only.
   Do not require JavaScript bindings or browser execution of the Rust code.
- Q2:
   Maintain the TypeScript package alongside the Rust implementation.
   Do not freeze or retire it as part of this port.
- Q3:
   Expose an idiomatic Rust API with the equivalent public capabilities,
   rather than copying TypeScript call syntax.
- Q4:
   Expose exact JSON numeric values rather than JavaScript-number semantics.
   Q6 strengthens source-spelling preservation for unedited literals,
   including clean input where the TypeScript fast path currently loses it.
- Q5:
   Duplicate object keys are user error and outside the supported behavioral contract.
   Interpret "undefined behavior" as unspecified library results for unsupported input,
   never permission for Rust memory unsafety or an unsafe-language contract;
   do not promise a particular parse or edit result for duplicates.
- Q6:
   Exact numeric values compare mathematically:
   `1`,
   `1.0`,
   and `1e0` compare equal.
   An unedited number retains its source spelling on output.
- Q7:
   Each edit returns a new state;
   the previous state remains usable.
   Do not make in-place mutation the public editing contract.
- Q8:
   Share supported-behavior fixtures across Rust and TypeScript,
   and fix confirmed unintended discrepancies in both during the port.
   The exact Rust number API is an explicit,
   tested exception to JavaScript-number parity.
- Q9:
   Port supported behavior,
   not necessarily the TypeScript parser algorithm.
   Vetted Rust parser components are allowed only if they preserve the attached-comment and edit contract;
   no dependency is selected yet.
- Q10:
   Publish the native Rust crate as part of this port,
   rather than keeping it private or only publication-ready.
- Q11:
   Name the published crate `monochromatic-jsonc-edit`,
   not `jsonc-edit`.
- Q12:
   Preserve support for escaped unpaired UTF-16 surrogates in Rust string values.
   Provide a representation that retains the code units and a fallible conversion to Rust `String` for values
   that cannot be expressed in UTF-8.
- Q13:
   The user directed us to cull anything using regex early.
   Operational interpretation:
   reject regex-defined production parsers or lexers,
   including generated lexers and required production/build dependencies.
   Dev-only usage does not itself prove production regex use.
   This boundary is our screening interpretation,
   not a separate user endorsement.

## Working baseline

A port retains the documented identity of the TypeScript library:
 JSONC container-root parsing,
 queryable attached comments,
 immutable edits,
 and canonical output.
 Documented behavior is the baseline,
 not unintended defects.
 Historical defect reports in `doc/planning/monorepo-manager-route-research/rust-structured-edits.md` need reproduction
 before they become acceptance criteria;
 that document also records later corrections to its research.

## Open decisions

- First-publish authentication and workflow path,
   without asking for or exposing secrets.
   A local Cargo credential file exists,
   and `gh api user --jq .login` returned `Aquaticat`;
   neither proves crates.io publication rights or token validity.
- Native dependency foundation:
   research and verification are **complete** under the repository's choosing-technology gates.
   Both vet reports record saturated discovery for their frozen schedules,
   equal-depth validation of every survivor,
   frozen-rubric scores,
   sensitivity reruns,
   full rankings and recommendations:
   `doc/audit/tech-jsonc-edit-rust-parser-foundation-vet-2026-09-24-63342231.md` and
   `doc/audit/tech-jsonc-edit-rust-exact-number-foundation-vet-2026-09-24-e8e0034a.md`.
   Both recommend the repository-owned parser/emitter translation plus the repository-owned raw-token
   and mathematical-identity number representation.
   What remains open is the **adoption decision itself**,
   which this plan reserves to the user;
   no dependency,
   product crate,
   decision record or publication exists yet.
- Recheck name availability immediately before publish;
   registry lookup does not reserve names.

## Implementation baseline for confirmation

- Place the standalone crate in `package/rust-module/jsonc-edit`,
   with package name `monochromatic-jsonc-edit`,
   using the repository's Rust package tasks and license precedent.
   Use `0.1.0` for its first release,
   following existing Rust-library versions;
   preserve the current library's LGPL-3.0-or-later license.
   These are repository defaults,
   not separate changes to the user's requirements.
- Port the public parser,
   emitter,
   edit,
   navigation,
   errors,
   node and comment capabilities.
   Internal artifact-test helpers need tests,
   not identical public Rust exports.
   Preserve container-root JSONC syntax,
   escaped UTF-16 code units,
   and canonical formatting.
   Edits are immutable.
   Document and test the exact-number divergence.
- Reuse the existing TypeScript conformance and property cases through language-neutral fixtures,
   adding focused differential tests for comments,
   edits,
   and output.
   When a supported-behavior defect is confirmed with a failing test,
   fix both maintained implementations,
   with documented exceptions for exact Rust numbers.
- Evaluate parser or numeric dependencies under the choosing-technology gates.
   Present the evidence,
   ranking,
   and risks;
   get explicit adoption of the foundation before dependent implementation or a decision record.
   Do not adopt meow's selected editor merely because it exists,
   and do not replace meow's editor as part of this port.
- Verify the packaged crate by building and calling it from a disposable Rust consumer,
   then publish the checked artifact using an authorized token route.
   Extend the existing release workflow for subsequent releases only after the first publication,
   when the crate can be configured for trusted publishing.

## Foundation evaluation queries

Freeze this numeric-foundation discovery schedule before running it:

- Registry:
   `cargo search exact decimal --limit 100`,
   `cargo search bigint --limit 100`,
   `cargo search arbitrary precision json --limit 100` and `cargo info` for identified serious candidates.
   Record result counts and incomplete registry enumeration honestly.
- Repository host:
   GitHub repository searches for `rust arbitrary precision decimal json` and `rust bigint`.
- Web:
   `Rust exact JSON number decimal exponent arbitrary precision source crate`
   and `Rust JSON number lexeme exact comparison library`.
- In-repo:
   inspect existing Cargo dependencies and `doc/planning/monorepo-manager-route-research/rust-structured-edits.md` for
   number equality and raw-token precedents.

One de-duplicated expansion round after initial taxonomy findings:
 registry `cargo search 'json number' --limit 100` and `cargo search 'decimal arbitrary exponent' --limit 100`;
 repository host `gh search repos 'rust json number' --limit 100 --json fullName,url,description,updatedAt`;
 web `Rust lexical JSON number numeric equality arbitrary exponent crate`.
 Freeze after this round.
 Registry pagination uses the crates.io API through the rendered web-fetch transport when direct `curl` cannot read it;
 inspect whole pages until two consecutive pages add no screening survivor or the provider is exhausted.

Compare a verified raw-token representation owned by the crate with available exact-decimal components.
 Cull components whose production path uses regex.
 A component must support exact value comparison over the admitted JSON-number grammar,
 preserve original literal spelling on unedited output,
 and avoid silent rounding or unbounded recursion.
 Maintain the parser-foundation query record separately when its research returns.
 No candidate is recommended from these queries alone.

## Parser foundation discovery expansion

The delegated parser survey froze and ran its initial queries before source reads;
 their exact text and blocked local searches are recorded in
 `doc/audit/tech-jsonc-edit-rust-parser-foundation-vet-2026-09-24.md`.
 Freeze this single de-duplicated expansion before running it:

- Registry:
   crates.io search `jsonc comments` and `lossless json parser` with full pagination
   until the two-page survivor condition or exhaustion.
- Repository host:
   `gh search repos 'jsonc rust lossless' --limit 100 --json fullName,url,description,updatedAt`
   and `gh search repos 'rust json comments parser' --limit 100 --json fullName,url,description,updatedAt`.
- Web:
   `Rust JSONC lossless CST raw token UTF-16 unpaired surrogate crate`
   and `Rust JSON with comments lexer preserve raw escape JSONC parser alternative`.
- In-repo taxonomy:
   `doc/decision/jsonc-edit-parser-foundation.md`,
   `package/module/jsonc-edit/src/`,
   and `doc/planning/monorepo-manager-route-research/rust-structured-edits.md`.

Later taxonomy terms are recorded without scheduling recursive expansion.
Discovery metadata is not a library recommendation.
The early regex gate excludes `edikt-jsonc`:
 its [0.4.0 lexer](https://docs.rs/crate/edikt-jsonc/0.4.0/source/src/lexer.rs)
uses `logos` with `#[regex(...)]` token patterns at lines 47 to 69.
The [JSON CST example](https://docs.rs/crate/tokora/0.11.0/source/examples/json_cst.rs) for `tokora`
also uses `logos` regex patterns and is excluded as an approach,
 not proof that the combinator library requires regex.
`json-five` lists `regex` only under dev-dependencies and has no regex use in its inspected runtime `src/`;
this alone does not cull it.
Direct source searches of `fjson`,
 `jsonc-parser`,
 `biome_json_parser`,
 `jwc`,
 `hifijson`,
 and `json-number`
found no regex calls or macros.
 Selected normal/build dependency trees were checked;
source-level transitive clearance remains open where required.

## Current gate status

- `edikt-jsonc` and the regex-backed `tokora` JSON CST example are excluded.
   Disposable pinned Cargo trees of the remaining listed candidates showed no regex-named normal/build dependency,
   while the `edikt-jsonc` positive control exposed `logos`,
   `regex`,
   and related crates.
   Source audit for hidden regex use in mandatory dependencies remains open;
   a delegated Biome source pass did not establish complete clearance.
- `jsonc-parser` and `jwc` reject unpaired surrogate escapes as-is.
   `fjson` and `json-five` expose raw tokens but need owned adapters for the comment model and strict syntax.
   `momoa` 3.2.6 is excluded separately on a source-proven invalid-`char` safety path.
   A checked pre-fix instrument and patched public-entry test passed their expected fail/pass controls in a bounded
   container,
   and the patched upstream Rust suite passed.
   The original unsafe parser was never run on the suspect input.
   See `doc/troubleshooting/momoa-rust-unpaired-surrogate.md`.
- A dependency-free Rust exact-number prototype under private scratch passes its unit suite,
   Clippy with warnings denied,
   and a separate consuming-crate call.
   An independent review led to private identity fields,
   strict valid-fixture checks,
   long-exponent tests,
   and a hash/equality check.
   No product crate was built or published.
- An owned parser/emitter scratch crate passed initial syntax,
   comment,
   exact-number,
   UTF-16,
   Clippy,
   and consuming-crate probes.
   A new 513-level nested-array case aborted its 2 GiB/2 CPU isolated test process with a stack overflow
   before the intended 512-depth error.
   A separate positive control with **valid 512-level nesting** also aborted with a stack overflow in the same bounds.
   An uncaptured stage marker printed before parsing but not after;
   the observed overflow occurs during parse execution.
   Merely checking depth before recursive descent cannot preserve the accepted input domain.
   A scratch rewrite with explicit container frames then passed both boundary cases
   in the same bounded container;
   the valid 512-level test also dropped its parsed node.
   Further bounded debug tests then passed emission,
   reparse,
   drop,
   clone/equality,
   nested-record lifecycle,
   and error cleanup at the accepted depth.
   The bounded debug and optimized release suites both passed with explicit phase markers.
   Non-Linux builds,
   full conformance,
   fuzzing,
   and the public edit interface remain unverified.
   The candidate is **not yet validated** for adoption.
- A separate TypeScript source probe found that clean `[1\n,2]` parses through the `JSON.parse` fast path,
   but `[1\n,2,]` fails in the structured path with `JsoncParseError: expected , or ] in array (at offset 3)`;
   a commented input and object with a newline before the comma fail analogously.
   Treat syntax parity here as a separate issue from stack safety;
   add shared fixtures and fix both implementations after foundation adoption.
- A separate public-boundary depth probe used the existing TypeScript node bundle from a disposable,
   2 GiB/2 CPU Node 26 container (`~/temp/agent/jsonc-ts-depth-envelope/probe.mjs`).
   Clean arrays at depths 512 **and 513** both returned a `plainJson` node;
   the same 512-depth array with a trailing comment returned a structured `array`,
   while the commented 513-depth array threw `JsoncParseError: nesting too deep (at offset 513)`.
   `src/parse-jsonc.ts:40-44,108-121` returns `JSON.parse` results before calling `parseValue`,
   bypassing the depth guard documented in `src/parse.ts:22-31,64-70`.
   The accepted parser-foundation decision states:
   the fast path must not change public behavior (`doc/decision/jsonc-edit-parser-foundation.md`,
   "Consequences").
   Treat the clean 513-depth acceptance as a confirmed fast-path bypass,
   not a promise of unbounded nesting;
   align both maintained implementations to the structured path's explicit 512-container boundary
   and add shared clean/commented boundary fixtures after foundation adoption.
   This is distinct from the comma-grammar fast-path discrepancy and the Rust stack-lifecycle issue.
- A bounded owner-sensitive Rust differential found one accepted-input difference on `{"a":1\n, //inline\n"b":2}`:
   the owned parser places `inline` on key `b`,
   but the Biome prototype places it on value `a`.
   Named object and array tests independently failed that ownership case.
   The scratch Biome adapter now routes a later-line comma's following comment to the next child;
   both named regressions passed in bounded debug and optimized release runs.
   Positive-controlled debug and optimized differentials found no syntax
   or node difference within the 5635 generated inputs,
   of which both parsers admitted 1577.
   This corpus result does not prove parity for every JSONC input.
   The TypeScript structured path still rejects this input today;
   the accepted same-line trailing rule guides the scratch correction,
   not an assertion that TypeScript already parses it.
- A multi-line value comment in `{"k":/*value\ncomment*/1}` moves from the value to the key after canonical emission
   and reparse.
   A direct TypeScript bundle probe measured `COMMENT_ABSENT` on the key before emission and on the value afterward;
   an isolated scratch Rust test failed with the same migration.
   The scratch Rust emitter was changed to put that comment after the colon;
   its parse/emit/reparse regression passed in the bounded container.
   The maintained TypeScript emitter still needs the same correction and shared fixture after foundation adoption.
   This is a separate comment-ownership incident,
   not the stack or comma failure.
- A direct TypeScript bundle probe of a line comment before CRLF returned text ending in `\r`;
   the CR-only variant threw `JsoncParseError: unterminated object (at offset 0)`.
   Microsoft's `node-jsonc-parser` scanner treats both CR and LF as line breaks
   (`~/temp/agent/node-jsonc-parser-2026-09-24/src/impl/scanner.ts:251-261,399-401`,
   checkout `dba4356`).
   Isolated scratch Rust tests also failed independently:
   CRLF kept `" x\r"` instead of `" x"`,
   and CR-only returned `ParseError` at byte offset 18 rather than the next member.
   A bounded debug Biome adapter fixture accepted both endings with comment text `" x"`
   and reparsed its canonical output.
   The owned scratch scanner now stops at either CR or LF;
   bounded debug and optimized regressions,
   Clippy,
   and a separate consumer's parse/emit/reparse for both endings passed.
   The Biome CR fixture also passed its bounded optimized run.
   Maintained TypeScript changes still wait for foundation adoption.
- A separate published `biome_json_parser` 0.5.7 syntax probe passed bounded debug and optimized release runs for
   escaped lone surrogate text,
   raw number tokens,
   comment trivia,
   strict JSON5 rejection,
   and 512-level array/record parse and cleanup.
   Upstream also accepts a 513th container;
   a scratch postparse adapter guard passed 512-accepted and 513-rejected controls.
   Comment trivia lives on punctuation tokens,
   so mapping it to separately queryable key and value owners remains unvalidated.
   This is an unadopted alternative with a larger build dependency surface.
   A separate bounded debug and release syntax comparison exercised 5635 fixed mutations,
   confirmed a deliberately mismatching BOM control first,
   and found no admission difference within that corpus;
   both boundaries admitted 1577 inputs.
   This does not validate general comment attachment,
   exact-value projection,
   or the published package integration.
- Current reports are `doc/audit/tech-jsonc-edit-rust-parser-foundation-vet-2026-09-24-63342231.md`
   and `doc/audit/tech-jsonc-edit-rust-exact-number-foundation-vet-2026-09-24-e8e0034a.md`.
   Their no-regex predecessors are superseded.
   Other worktree changes are concurrent and out of scope.

## Next action

Finish parser and exact-number candidate discovery and compare surviving foundations at equal depth.
 Compare the owned iterative parser and any viable syntax adapter against the maintained TypeScript conformance corpus,
 including the measured comma and value-comment cases.
 Add comma-after-trivia and comment-owner fixtures to the maintained TypeScript implementation at the appropriate
 adoption stage;
 its value-comment emitter remains unfixed.
 Then rerun parser syntax,
 comment,
 UTF-16,
 numeric,
 and consumer tests.
 Finish candidate comparison and present a vetted foundation recommendation for adoption.
 Only then implement the product crate,
 align TypeScript behavior,
 verify through a consuming project,
 and publish with an authorized credential route.
