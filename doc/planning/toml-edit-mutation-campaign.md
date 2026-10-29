# TOML edit mutation campaign

## Goal and scope

The user requested mutation testing against `package/toml-edit/core` and fixes for actionable findings.
Use the opt-in `test:mutation` task with `--full-suite` and reports under private scratch.
The package task scans `src/conformance/` by default,
 but the upstream conformance runner does not execute inside the mutation container.
The targeted campaigns here name runtime source files explicitly.

## Completed evidence

- Initial container smoke on `src/toml-has.ts`:
  one killed,
  five compile errors,
  no infrastructure errors.
- Accessor batch on `empty-toml-edit.ts`,
  `toml-get-value.ts`,
  `toml-set-header-comment.ts`,
  `toml-keys.ts`,
  and `toml-stringify.ts`:
  nine killed,
  three confirmed survivors,
  and 42 compile errors.
  Added a canonical-options test and scalar-path key test in `canonical.unit.test.ts` and `toml-keys.unit.test.ts`.
  Recheck killed both actionable mutants.
- The remaining `toml-keys.ts` condition mutant skips `result === MISSING`.
  `MISSING` is a symbol (`document-materialize.ts`),
  so the same input falls through to the final empty-array return.
  Keep the explicit guard as documentation of missing-path handling.
- A direct built-artifact probe found that `trailingNewline: false`
  did not remove final newlines from canonical output.
  Canonical parsed output without a newline also did not gain one when enabled.
  Added red tests in `canonical.unit.test.ts`,
  fixed the final-output boundary in `emit-document.ts`,
  and verified through `buildAndTest` and a built-artifact call.
- Core batch on `path.ts`,
  `path-prefix.ts`,
  `resolve-block.ts`,
  and `build-value.ts`:
  12 killed,
  19 confirmed survivors,
  and 96 compile errors.
  Added built-artifact assertions for `_formatPath` and `_isStrictPrefix`.
  Recheck on the path files produced 26 killed,
  one confirmed equivalent survivor,
  and 31 compile errors.
- The `path-prefix.ts` equivalent mutant forces the non-strict length check to true.
  For valid string or numeric path segments,
  any candidate past the end compares against `undefined` and fails the `every` check anyway.
- The `build-value.ts` survivor fills `commentsBefore` on inline-table entries.
  An earlier classification called it equivalent because serialization and accessors do not read that field.
  That was too broad:
  `TomlEditState.blocks` exposes the parsed entry structure,
  so a caller can distinguish the extra comment directly.
  Added `inline-entry-state.unit.test.ts` to assert the public state has no phantom comments.
  The recheck killed that mutant and reported no surviving `build-value.ts` mutants
  (one killed,
  34 compile errors,
  no infrastructure errors).
- Package wrappers named in the unit tests were missing from the published entrypoint.
  A package-root import failed for `tomlLocalDate` before `index.ts` exported the wrappers.
  Unit tests now import the built package,
  and the fuzz coverage driver reaches the exported wrapper factories.
- Package and sidecar type checks and oxlint are clean.
  The package build and unit suite,
  sidecar bounded property suite,
  TOML 1.0 and 1.1 conformance,
  and the refrozen fuzz coverage gate have passed after the runtime changes.

## Emitter campaign and removal ledger

The container run on `src/emit-document.ts` and `src/emit-value.ts` completed with
 87 killed,
 59 confirmed survivors,
 one timeout,
 147 compile errors,
 and no infrastructure errors.
The timeout is a forced-true loop condition at `emit-document.ts:40`,
 not a successful assertion.
The remaining loop-bound mutants forcing `end > 0` to true or changing `>` to `>=`
 are equivalent because an out-of-range index is `undefined` and fails the newline comparison.

A repository-wide symbol and subpath search found no callers of these exports outside `emit-value.ts` itself.
`package/toml-edit/core/package.json` still exposes `./ts/*` and ships `src`,
 so these helpers were reachable through source-subpath imports despite not being root exports.
The unauthenticated public npm registry returned `E404` for this package on 2026-09-24;
 that alone does not prove that no authenticated or Git-based consumer exists.
Their removal is a deliberate source-API design change,
 not proof of backwards compatibility.

- `emitArrayWithoutIndex`:
  previously described array-element omission from a parser AST.
  Live owner is `delete-value.ts`'s immutable tree update followed by
  `renderValueNode` in `emit-value-node.ts` and `assembleArrayParts` in `emit-value.ts`.
  The nested-array deletion cases in `toml-delete.unit.test.ts` cover the live edit path.
  The array-content property separately covers re-emission,
  but the mutation runner does not include that sidecar as a killer.
  Retire the direct AST-helper contract,
  including its array-input and index behavior.
- `emitArrayWithSkipPath`:
  previously described recursive nested-array omission,
  with no non-recursive callers.
  The same live tree-update and rendering path owns nested deletion;
  `toml-delete.unit.test.ts` covers nested deletion,
  including deeper nesting and inline-table elements.
  Retire the direct AST-helper contract and its empty-path/non-array diagnostics.
- `emitInlineTableWithExtra`:
  previously described adding an entry to a parser AST inline table.
  Live owner is `set-value-inline.ts`'s immutable entry append,
  rendered by `renderValueNode` and `assembleInlineTableParts`.
  `toml-set.unit.test.ts` covers inline-table extension and conflicts.
  Retire the direct AST-helper contract and its error diagnostic.

`assembleArrayParts`,
 `assembleInlineTableParts`,
 `emitContentNode`,
 and the live `emitInlineTable` path remain consumed.
Most `emit-value.ts` survivors in the former AST-only functions cannot be killed by package behavior;
 removal avoids mislabeling this dead code as weak assertions.

## Verification and remaining limits

- Removed `emitArrayWithoutIndex`,
  `emitArrayWithSkipPath`,
  and `emitInlineTableWithExtra` from `emit-value.ts` after the coverage ledger.
  The package build,
  unit suite,
  type check,
  and oxlint passed after removal.
- Added built-artifact tests for parsed boolean values,
  inclusive array layout thresholds,
  nested multiline indentation,
  empty containers,
  quoted inline-table keys,
  exact multi-line headers,
  and trailing comments at EOF.
- The post-removal emitter recheck produced 113 killed,
  14 confirmed survivors,
  one timeout,
  and 117 compile errors.
  A further `emit-value.ts` run after adding built-artifact formatting tests produced
  41 killed,
  one confirmed survivor,
  and 57 compile errors.
  The two live dotted-key and nested-depth mutations were killed.
- A package-produced `TOMLTable` passed to the exposed `_emitContentNode` seam reaches the invalid-node diagnostic.
  A built-artifact test now checks its error class and exact message.
  The final `emit-value.ts` run reported 42 killed,
  no survivors,
  57 compile errors,
  and no infrastructure errors;
  the diagnostic mutant was killed.
- Remaining `emit-document.ts` survivors were traced to package-factory construction invariants,
  not universal equivalence for caller-composed structural states passed to `_emitDocument`:
  `build-document.ts` creates parsed key-values and tables with clean origins,
  so `emit-document.ts` returns their original source before synthetic comment rendering.
  `set-create.ts`,
  `build-input.ts`,
  and `set-aot.ts` create synthetic nodes with empty `commentsBefore` and no `commentAfter`;
  `tomlInsertCommentBefore` inserts filler blocks instead,
  and `tomlInsertCommentAfter` uses `trailingCommentAppend`.
  `set-value.ts`,
  `set-replace.ts`,
  and `delete-value.ts` preserve enclosing key-value/header origins when replacing nested values.
  `set-aot.ts` is the only synthetic table-header constructor and makes array tables,
  so the standard synthetic bracket spelling is unreachable.
  Forcing the `trailingCommentAppend` guard false passes `append: undefined`,
  which `appendTrailing` handles identically to an omitted field.
  The two surviving loop-bound changes still stop when `text[-1]` is `undefined`;
  the forced-true entire loop condition times out and is not a survivor.
- The deterministic coverage gate reported `emit-value.ts` falling from 191 to 133 covered lines
  after removing the unconsumed helper bodies.
  The baseline was refrozen with `fuzz:coverage --write` after inspecting that change;
  the surviving file now has 133 covered lines out of 218 code lines.
  The coverage gate then passed in check mode.
- Final `buildAndTest`,
  package and sidecar types and oxlint,
  sidecar bounded property tests,
  and the coverage gate passed.
  TOML 1.0 and 1.1 conformance passed after retiring the unused emitter helpers.
  A separate sidecar consumer imported `tomlFloat` from the built package and observed
  `trailingNewline: false` output `ratio = 1.0` without a final newline.
- Additional tests now pin empty and header-only canonical output,
  multiple terminal newlines,
  multiline string contents,
  empty-source splice behavior,
  and edited nonempty splice preservation.
  `emit.property.unit.test.ts` compares reparsed nested array and inline-table contents
  through the built `_emitContentNode` seam.
  The targeted tests passed;
  rerun full scoped verification after triaging emitter mutants.
- Never delete or stage unrelated changes under other packages;
  concurrent agents have been editing `deepmerge-ts.fuzz` and other areas.

## Continuing runtime scan

The initial named campaigns did not exhaust the runtime source list.
A further container run is now active on read,
 error,
 and comment sources:
 `basic-escape.ts`,
 `keys.ts`,
 `errors.ts`,
 `toml-get-node.ts`,
 `toml-get-raw.ts`,
 `toml-get-comments.ts`,
 `toml-get.ts`,
 `types.ts`,
 `wrappers.ts`,
 and `comments.ts`.
The batch reported 96 killed,
 46 confirmed survivors,
 172 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-remaining-a.json`.
New package tests cover named and unnamed basic-string escapes,
 key alphabet endpoints,
 error names,
 raw table/header slices,
 and the canonical key-creation default.
The survivor-bearing recheck reported 133 killed,
 eight confirmed survivors confined to `comments.ts`,
 91 compile errors,
 and no infrastructure errors.
The other rechecked files (`basic-escape.ts`,
 `errors.ts`,
 `keys.ts`,
 `toml-get-raw.ts`,
 `types.ts`) had no survivors.
Adding a parsed adjacent-hash case also exposed the real trailing-comment defect.
The final `comments.ts` recheck reported 25 killed,
 four confirmed survivors,
 15 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-comment-boundaries.json`.
The remaining comparisons are equivalent only for parser-produced,
 non-overlapping comment ranges:
 an attached comment ends strictly before its key,
 a prior comment ends strictly before the retreating cursor,
 and a trailing comment cannot start at the newline byte delimiting its line.
`TomlEditState.comments` is structural,
 so caller-composed overlapping or zero-gap ranges were not established as equivalent.
Package type and oxlint checks and the fuzz coverage gate passed after the comment fix.

A direct consumer probe of `key = 1#tail` found a real defect:
 `tomlGetCommentAfter` returned no comment because `comments.ts` required the hash offset
 to be strictly greater than the value end.
A regression test failed before the change;
 changing that comparison to inclusive made the package build and unit suite pass.
The sidecar bounded property suite,
 types,
 and package oxlint also passed after the fix.

The parser/document batch is now running in bounded containers:
 `build-comments.ts`,
 `build-document.ts`,
 `build-input.ts`,
 and `parse-toml-edit.ts` write `/var/home/user/temp/agent/toml-mutation-parser-builder.json` (`proc_8259`);
 `document-materialize.ts`,
 `value-materialize.ts`,
 `emit-value-node.ts`,
 and `emit-value-string.ts` wrote `/var/home/user/temp/agent/toml-mutation-materialization.json`.
That batch reported 44 killed,
 22 confirmed survivors,
 119 compile errors,
 and no infrastructure errors.
The parser/build batch completed with 59 killed,
 91 confirmed survivors,
 two timeouts,
 178 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-parser-builder.json`.
A structural-state test for `key = 1#tail` failed before `build-comments.ts`
 adopted the same inclusive boundary as the read accessor;
 the package build and unit suite passed after the change.
Additional tests now cover parsed block metadata,
 array navigation bounds,
 nested input and wrapper reads,
 parsed string styles,
 and synthetic array/inline-table indentation.
A prior test also exposed a stale TSDoc claim about `toml-eslint-parser@1.0.3`:
 upstream defaults to TOML 1.1,
 not 1.0.
The test now selects 1.0 explicitly and checks the current default;
 `doc/troubleshooting/toml-eslint-parser-default-version.md` traces the installed and tagged source.

The materialization recheck completed on the shared revision with 59 killed,
 seven confirmed survivors,
 119 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-materialization-recheck.json`.
Parsed-string style tests killed every `emit-value-string.ts` mutant that compiled.
Array-index bounds tests killed the actionable `document-materialize.ts` bounds mutants.
Its remaining survivor changes only the `MISSING` symbol's description,
 not sentinel identity or root API results;
 source-subpath consumers can inspect that description.
`value-materialize.ts` retains five survivors in null classification,
 empty-path `updateDeep`,
 and `Object.hasOwn` guards.
Package constructors reject null and supply nonempty header/value paths;
 direct source-subpath calls,
 caller-composed states,
 or prototype hooks could distinguish those mutations,
 so they are not claimed universally equivalent.
A new test now checks unchanged hexadecimal array siblings after another element is set,
 targeting the remaining `emit-value-node.ts` clean-node block mutant.
The parser/build recheck completed with 111 killed,
 39 confirmed survivors,
 two timeouts,
 178 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-parser-builder-recheck.json`.
A further guard run after state and diagnostic tests reported 132 killed,
 18 confirmed survivors,
 two timeouts,
 and 178 compile errors;
 `build-document.ts` had no survivors.
Tests added since that run pin exact parser rejection text,
 EOF comment metadata,
 deeper array-table indices,
 and a structurally tagged float string's numeric read.
`build-input.ts`'s duplicate null guard was removed after the shared value encoder proved the same rejection;
 package build/tests,
 types,
 oxlint,
 and bounded sidecar properties passed.
The final parser-boundary recheck reported 128 killed,
 12 confirmed survivors,
 two confirmed timeouts,
 177 compile errors,
 and no infrastructure errors.
`build-document.ts` has no survivors.
The remaining `build-comments.ts` comparisons are equivalent only for parser-produced,
 non-overlapping comment ranges.
The `build-input.ts` survivors are the optional `existing: undefined` forwarding branch
 and a number-type branch whose non-number inputs already fall through to the same kind.
The `parse-toml-edit.ts` surviving range check adds one harmless out-of-range iteration;
 other survivors affect warnings or the non-`ParseError` error path not driven by the package unit suite.
The two reversed/forced loop mutants were confirmed timeouts,
 not survivors.
The `build-input.ts` coverage baseline was refrozen from 130 to 128 covered lines
 after deleting the duplicate guard;
 the gate passed in check mode.
A previous review mistakenly treated `parse-toml-edit.ts:53`'s `<` to `>=` mutant as a survivor.
The actual final report and raw singleton shard record both classify it as a confirmed timeout;
 there is no verdict mismatch.
Built-artifact state tests now assert nested array-of-tables indices,
 EOF and adjacent comment metadata,
 synthetic scalar kinds,
 precise nested-input errors,
 and preservation of clean array-sibling spelling.
The package build and unit suite,
 types,
 oxlint,
 and deterministic fuzz coverage gate passed after these changes.
The clean-sibling `emit-value-node.ts` recheck reported five killed,
 no survivors,
 26 compile errors,
 and no infrastructure errors;
 its former clean-node block survivor was killed.
The parser/build recheck completed at
 `/var/home/user/temp/agent/toml-mutation-parser-guard-final.json`.
The edit-path campaigns completed on the same source snapshot:

- `delete-value.ts`,
  `resolve-document.ts`,
  `toml-delete.ts`,
  and `toml-set.ts` wrote `/var/home/user/temp/agent/toml-mutation-edit-a.json`:
  65 killed,
  55 confirmed survivors,
  274 compile errors,
  no infrastructure errors.
- `set-aot.ts`,
  `set-create.ts`,
  `set-replace.ts`,
  `set-value-inline.ts`,
  and `set-value.ts` wrote `/var/home/user/temp/agent/toml-mutation-edit-b.json`:
  95 killed,
  85 confirmed survivors,
  302 compile errors,
  no infrastructure errors.

A built-artifact test demonstrated that `tomlGetValue` resolves clean indexed array-of-tables paths
 while `tomlGetNode` and `tomlGetRaw` incorrectly threw `TomlPathNotFoundError`.
`resolve-document.ts` now selects the indexed table and descends its body.
The package build and tests passed after the change,
 and tests cover indexed node/raw/comment views and out-of-range rejection.
Additional deletion and set tests cover indexed instances,
 nested table/inline-table edits,
 AOT replacement among siblings,
 implicit dotted-key replacement,
 path-create placement,
 and exact rejection diagnostics.
The delete/resolve recheck completed with 106 killed,
 28 confirmed survivors,
 292 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-edit-a-recheck.json`.
The set-internals recheck wrote `/var/home/user/temp/agent/toml-mutation-edit-b-recheck.json`
 with 142 killed,
 38 confirmed survivors,
 302 compile errors,
 and one infrastructure error.
The CLI exited nonzero because `rsync` code 24 reported a vanishing
 `deepmerge-ts.fuzz/.cache/typescript/tsconfig.tsbuildinfo` file while copying a shard.
That is not a TOML mutant verdict.
`package/mutation-test/cli` now excludes `**/.cache` from shard copies and image context;
 its unit test was red before this fix,
 then build/tests,
 types,
 oxlint,
 and a disposable rsync copy probe passed.
`doc/troubleshooting/mutation-test-rsync-cache-churn.md` records the source trace.
The clean `src/set-replace.ts` rerun completed with 21 killed,
 four confirmed survivors,
 57 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-set-replace-infra-recheck.json`.
A package-root test then exposed an unnamed empty-path error (`tomlSet at  requires ...`);
 `set-replace.ts` now reports `document root` and `document body` for root replacement.
Additional tests pin sibling isolation for set/delete,
 indexed and nested AoT read views,
 implicit replacement under mixed parents,
 and synthetic inline-entry comment state.
The expanded package build,
 unit suite,
 types,
 and oxlint passed after these changes.
The edit-path rechecks completed without infrastructure errors:

- `/var/home/user/temp/agent/toml-mutation-edit-a-final.json`:
  111 killed,
  23 confirmed survivors,
  292 compile errors.
- `/var/home/user/temp/agent/toml-mutation-edit-b-final.json`:
  156 killed,
  34 confirmed survivors,
  301 compile errors.

Built-artifact clean-reader tests now cover nested standard tables,
 array elements,
 inline-table values,
 and nested indexed array-of-tables paths.
Edit tests cover reversed parent/table declaration order,
 deeper nested AOT headers,
 implicit-parent comment fillers,
 root replacement diagnostics,
 mixed-parent rejection,
 and numeric path-create diagnostics.
The expanded package build,
 unit suite,
 types,
 and oxlint passed.
The last edit-path mutation rechecks completed with no infrastructure errors:

- `/var/home/user/temp/agent/toml-mutation-edit-a-last.json`:
  124 killed,
  10 confirmed survivors,
  292 compile errors.
- `/var/home/user/temp/agent/toml-mutation-edit-b-last.json`:
  166 killed,
  24 confirmed survivors,
  301 compile errors.

Remaining survivors include sentinel descriptions exposed by source subpaths;
 guards rendered redundant by earlier dispatch,
 valid-path type or length invariants,
 and `tomlDelete`'s missing-path state identity.
A forced-true implicit-parent check may return a fresh but value-equivalent no-op state;
 identity was not promised as a stable contract.
`set-aot.ts`'s length check still short-circuits on an out-of-bounds segment,
 and the first-instance fallback is unreachable after the caller selected an existing AOT.
`set-create.ts`'s deepest-table scan has defensive checks for a missing best block that
 cannot arise from the factory-produced block list.
These are conditional classifications over package-created states,
 not universal claims for caller-composed `TomlEditState` or direct `/ts/*` imports.
The indexed array-of-tables read fix passed the rebuilt package suite,
 types,
 oxlint,
 and the deterministic fuzz coverage gate.
TOML 1.0 and 1.1 conformance and the sidecar bounded property suite passed after that change.
The remaining runtime comment API and value-encoding files are now being scanned
 in bounded containers on the same source revision:

- `toml-get-comment-after.ts`,
  `toml-get-comments-before.ts`,
  `toml-insert-comment-after.ts`,
  and `toml-insert-comment-before.ts` wrote
  `/var/home/user/temp/agent/toml-mutation-comment-api.json`:
  14 killed,
  34 confirmed survivors,
  148 compile errors,
  no infrastructure errors.
  Missing-path diagnostics and comment insertion inside standard tables need tests.
- `value-encoders.ts` and `values.ts` wrote
  `/var/home/user/temp/agent/toml-mutation-value-encoders.json`:
  80 killed,
  83 confirmed survivors,
  143 compile errors,
  no infrastructure errors.

A built-artifact test demonstrated that `tomlGetCommentAfter` missed a trailing
 comment on a standard or indexed array-of-tables header.
`toml-get-comment-after.ts` had scanned from a stored header span that includes the newline;
 it now anchors to the parse-time header key before the comment.
The test failed before the fix and the rebuilt package suite passed afterward.
Additional comment tests cover nested insertion and missing-path diagnostics.
Direct built-artifact value tests now pin tagged nonfinite numbers,
 false booleans,
 empty and populated arrays/tables,
 threshold-boundary layout,
 and null-prototype tables.
`value-encoders.ts` now shares one integer-wrapper spelling path,
 while `values.ts` delegates array and table assembly to the existing emitters.
The package build and unit suite,
 type check,
 oxlint,
 and bounded sidecar property suite passed after this refactor.
`values.ts` covered-line count fell from 332 to 304 because duplicated array/table assembly was removed;
 the fuzz coverage gate requires an intentional baseline refreeze after mutation rechecks.
The comment API recheck completed with 31 killed,
 17 confirmed survivors,
 149 compile errors,
 and no infrastructure errors.
Its report is `/var/home/user/temp/agent/toml-mutation-comment-api-recheck.json`.
The remaining comment findings are concentrated in synthetic-node `-1` offsets,
 sentinel descriptions,
 and array-table descent guards.
The value encoding recheck completed at
 `/var/home/user/temp/agent/toml-mutation-value-encoders-recheck.json`:
 111 killed,
 12 confirmed survivors,
 141 compile errors,
 no infrastructure errors.
The comment recheck's remaining synthetic-header cases are now covered by a test
 with an unrelated preceding comment.
Direct built-artifact value tests now distinguish equal versus changed literal/numeric spelling,
 finite float markers,
 and nested array depth.
The package build,
 unit suite,
 types,
 and oxlint passed after these assertions.
The comment/value rechecks completed with no infrastructure errors:

- `/var/home/user/temp/agent/toml-mutation-comment-api-final.json`:
  35 killed,
  13 confirmed survivors,
  149 compile errors.
- `/var/home/user/temp/agent/toml-mutation-value-final.json`:
  116 killed,
  seven confirmed survivors,
  141 compile errors.

The comment survivors are confined to synthetic-node offset sentinels,
 private `NOT_FOUND` symbol descriptions in the insertion modules,
 and AoT body descent guards.
Those symbols are module-local and never leave the exported insertion functions;
 the `/ts/*` source subpaths do not export them.
A test now asserts that a synthetic key-value cannot borrow an earlier trailing comment.
The five surviving `values.ts` mutants consist of four kind checks
 (`e4b52b79592e2b76`, `b992d0d0b8bcf9a8`, `0d8050ba970c0c31`, `c37fdfd7a39e0b8c`)
 redundant with strict value equality on parser-produced string or numeric nodes,
 plus `33ed844fb684807c`, a nonfinite `> 0` boundary.
After NaN is handled,
 only positive or negative infinity reaches that boundary.
The tagged integer branch returned the same `String(value)` as the default branch,
 so it was removed.
The package build and unit suite,
 type check,
 oxlint,
 and bounded sidecar properties passed after those changes.
The focused rechecks completed without infrastructure errors:

- `/var/home/user/temp/agent/toml-mutation-synthetic-comment.json`:
  12 killed,
  two confirmed survivors,
  29 compile errors.
  The synthetic key-value offset mutant was killed;
  the remaining sentinel mutants require an impossible factory-produced AST mismatch
  or a caller-bypassed not-located state.
- `/var/home/user/temp/agent/toml-mutation-tagged-encoder.json`:
  61 killed,
  one confirmed survivor,
  26 compile errors.
  The remaining `>` to `>=` change has the same result on the reachable nonfinite values
  after NaN and finite numbers are handled.
TOML 1.0 and 1.1 conformance passed after the shared value-assembly refactor.
Do not claim a full-runtime verdict before those campaigns and survivor rechecks finish.

## Shared value assembly refactor

The value-encoding campaign surfaced repeated array and inline-table layout logic in `values.ts`.
Before replacing it,
 the consumed responsibilities are mapped:

- `values.ts:encodeArray` still recursively encodes JS elements and carries equal-valued parse-time element nodes.
  Its final inline-vs-multiline assembly can use the existing `emit-value.ts:assembleArrayParts`,
  already consumed by parsed AST emission and synthetic value-node rendering.
  `js-value-text.unit.test.ts` pins empty,
  inline,
  threshold-boundary,
  multiline,
  nested-indentation,
  and raw-element spelling paths.
  The duplicated final formatting branch is retired;
  recursive coercion remains.
- `values.ts:encodeInlineTable` still builds per-entry encoded keys and recursive values.
  Its final punctuation can use `emit-value.ts:assembleInlineTableParts`,
  already consumed by parsed and synthetic table rendering.
  `js-value-text.unit.test.ts` pins empty and populated table output.
  The duplicated final formatting branch is retired;
  entry coercion remains.
- `values.ts:encodeNumber` has identical `String(value)` returns for safe and other finite numbers.
  The numeric guard adds no behavior,
  so it can be removed while the nonfinite branch stays.
  The direct built-artifact tests pin finite and nonfinite outputs.

This is a parity-preserving owner change over the tested formatter paths,
 not evidence that formatting did not need tests or that source-subpath consumers are unchanged.

## Remaining scope

The `emit-document.ts` recheck retains equivalent or factory-state-unreachable mutants,
 plus one forced-true-loop timeout.
`TomlEditState` is structural and `_emitDocument` is exposed as an unstable seam;
 a caller-composed state could distinguish some synthetic comment/header mutants.
The campaign does not claim universal equivalence for such inputs.
These are not counted as killed assertions.
The `emit-value.ts` final recheck has no survivors.
A current dry run selected 45 runtime files with mutants,
 and a comparison against the saved targeted reports found all 45 represented.
These runs were not one atomic full-package campaign at a single final revision;
 only the changed and survivor-bearing files were rechecked after edits.
Conformance adapters under `src/conformance/` were not included as mutant targets,
 and sidecar properties were not mutant killers in the current CLI.
The `values.ts` coverage baseline fell from 332 to 304 covered lines.
The `6a1852183` source diff deletes its duplicate finite-number check and array/inline-table formatting branches,
 replacing the latter with shared assembler calls;
 the retained input-coercion and recursive paths are unchanged.
The final coverage gate was run without a pipeline so its exit status was checked directly;
 it passed after the intentional baseline refreeze.
Commit only explicit paths in scope.
