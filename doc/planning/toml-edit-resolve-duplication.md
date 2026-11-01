# toml-edit resolver duplication (slopo cluster 385)

Status:
 proposal,
 awaiting decision.

Slopo cluster 385 (hash `482b1cf964fb`,
 score 0.92-0.95) flags four units:

- `package/toml-edit/core/src/resolve-block.ts` `locateBlock`
- `package/toml-edit/core/src/resolve-document.ts` `matchTables`
- `package/toml-edit/core/src/resolve-document.ts` `descendValue`
- `package/toml-edit/core/src/resolve-document.ts` `locateValueNode`

## Finding

The cluster is a true positive,
 and it sits on top of a package-wide pattern.

`package/toml-edit/core/src/path-prefix.ts` exports `isPrefix`,
`isStrictPrefix` and `segmentsEqual`
as the shared home for segment predicates.
`resolve-block.ts` imports from it.
Three other modules re-implement the same predicates locally instead:

- `resolve-document.ts` inlines all three as `.every(eq)` blocks inside `matchKeyValue` and `matchTables`.
- `toml-insert-comment-after.ts` carries local `segmentsEqual` and `strictPrefix` copies (clusters 262 and 323).
- `toml-insert-comment-before.ts` carries the same local `strictPrefix` copy (cluster 262).

Beyond the predicates,
`locateBlock` and `matchTables` run an identical table-scan algorithm:
exact-header filter yielding standard-table or array-of-tables,
then strict-prefix standard header,
 then descend into that parent's body.
Only the recursion target differs.

A `STRUCTURAL-IDIOM` dismissal does not hold for the cluster as reported,
because `resolve-block.ts` already imports from `path-prefix.ts`,
which proves extraction is the established pattern in this package rather than an unavoidable idiom.

## What was measured

All measurement ran in a throwaway worktree at `HEAD`,
 never in the main checkout.
The slopo index database was copied and its `source_dir` repointed,
so the embedding cache (keyed by `body_hash`) was reused and only changed bodies were re-embedded.

Control run reproduced cluster 385 with hash `482b1cf964fb` and the predicate cluster with hash `5a49fd1193d3`,
at the same cluster number as the committed report,
confirming the comparison is valid.

One caveat on the absolute figures.
The worktree checked out `HEAD`,
 so every run used the committed `slopo.conf.yaml`.
An uncommitted working-tree edit adds `'**/test-fixture/**'` to `source_dir_exclude`,
which would shrink the indexed unit count and shift the repo-wide percentages.
Cluster hashes cover unit bodies and are unaffected,
and both sides of the comparison used the same configuration,
so the A-versus-B delta holds either way.

Body node counts come from slopo's own parser
(`slopo/indexing/parsing/lang/typescript.py`,
 counting named tree-sitter nodes under the function `body` field).
Every unit involved sits far above `body_node_count_threshold: 13` before and after,
so node count is not the lever that retires a cluster.

### Variant A: descent instruction

`matchTableSection` returns either a table-section hit or a `{ kind: 'descend', blocks, path }` instruction
that each caller replays against its own entry point.
`matchTables` disappears;
`locateBlock` keeps only its exact-key-value step.

- Type check:
   no errors in any source file.
- Tests:
  `//package/toml-edit/core:test:unit` passes in full,
   all test groups green.
- Lint:
   see the lint section.
- `locateBlock` drops from 214 to 85 body nodes.
- Predicate cluster `5a49fd1193d3` disappears entirely.
- Repo-wide similarity ratio excluding exact copies falls from 10.88% (807 of 7419) to 10.81% (802 of 7417).
- `resolve-document.ts` grows from 277 to 293 total lines,
  which stays clear of the `eslint/max-lines` cap of 300 because that cap skips blanks and comments.
  No `max-lines` finding was emitted.

### Variant B: callback entry point

`matchTableSection<R>` takes a `descend: ResolveEntry<R>` callback,
collapsing each caller's tail to a single return statement.

- Type check:
   no errors in any source file.
- Tests:
   passes in full,
   exit status zero.
- `locateBlock` drops to 63 body nodes;
  `locateValueNode` to 71.
- Same clustering outcome and same repo-wide similarity ratio as variant A.

### Lint

`//package/toml-edit/core:lint:oxlint` with type-aware rules,
comparing unmodified `HEAD` against variant A in the same worktree.

Absolute totals are polluted by test-file noise:
the worktree cannot resolve the package's own name (`@monochromatic-dev/toml-edit-core`),
so type-aware rules see `error` types throughout the test files.
Only the delta and the per-file findings are meaningful.

- Control:
   326 warnings,
   146 errors.
- Variant A:
   326 warnings,
   143 errors.
- Findings citing `resolve-block.ts` or `resolve-document.ts`:
  seven `prefer-readonly-parameter-types` before,
   four after,
   and no new rule classes.

The first draft of variant A did introduce one finding,
`eslint(no-duplicate-imports)`,
by importing `TableSectionHit` from `resolve-document.ts` on a separate `import type` line
from the `matchTableSection` and `NOT_LOCATED` value import.
The fix is a single combined import using an inline `type` specifier,
matching the existing convention in `parse-toml-edit.ts`.
All figures reported here are after that fix.

### Residual in both variants

Neither variant silences slopo for these files.
A three-member cluster survives at score 0.94,
covering `locateBlock`,
`locateValueNode` and `descendValue`:
variant A hash `534c966e3b99`,
 variant B hash `43ca8a078e7e`.

What remains is the sentinel-guard plus early-return plus tail-recursion shell
mandated by the repo conventions (`ST9` named parameters,
`TQ3` early return,
`PP8`,
 the `NOT_LOCATED` sentinel),
with each body resolving a different thing.
That residual is genuinely dismissible under `STRUCTURAL-IDIOM`,
in the same class as the existing entries `5bab4aae8252` (mutually-recursive AST walkers)
and `40d7339c9902` (no-sync provenance resolver chain).
The rationale is honest only because the shared algorithm was actually extracted first.

Read the dismissal hash from a fresh `slopo analyze` after the refactor lands.
The hashes recorded here come from this investigation's worktree,
and `hash_body` covers the whitespace-normalized,
 comment-stripped body,
so any token or path difference between the candidate and what actually lands changes them.

## Recommendation

Land variant A,
 then dismiss the residual hash.

Variant B measures marginally better:
score 0.94-0.94 against 0.94-0.95,
and smaller bodies (`locateBlock` 63 against 85 nodes,
`locateValueNode` 71 against 93).
Both produce a three-member residual and the same repo-wide similarity ratio,
so that margin is within noise and the choice falls to readability.

On readability variant A wins:
it stays first-order,
 while variant B introduces a generic parameter
and passes each resolver to itself as a callback,
 which reads less directly at the call site.

One rule tension is worth naming rather than leaving for a later reviewer.
`AD3` prefers direct execution over descriptor and interpreter patterns,
and variant A's `{ kind: 'descend' }` value is a small descriptor that variant B avoids.
The judgement here is that a single-case instruction consumed by its immediate caller
is not the interpreter pattern `AD3` targets,
and that avoiding a generic higher-order parameter is the larger readability gain.
If a reviewer disagrees,
 variant B is measured,
 type-clean and test-passing,
 and can be taken instead.

Do not add any ignore entry before the refactor lands.
A hash covers the unit body,
 so it changes when the code changes,
and dismissing a body that is about to be rewritten is churn.

## Adjacent cheap wins

Clusters 262 (`ab81684eac0e`) and 323 (`cf34f1b3eb81`) are untouched by this change
and share the same root cause.
Replacing the local `segmentsEqual` and `strictPrefix` copies in
`toml-insert-comment-after.ts` and `toml-insert-comment-before.ts`
with imports from `path-prefix.ts` should be mechanical and behaviour-preserving.

Reasoned,
 not measured:
unlike the resolver case,
 this deletes duplicate units outright rather than reshaping them,
so it removes cluster members by construction.
The local copies were read and differ from the `path-prefix.ts` exports only in parameter names,
but no build,
 test or slopo run was performed against that change.

## Reproducing

```bash
# From a throwaway worktree with the slopo index copied in and source_dir repointed.
slopo index
slopo embed
slopo analyze
```

Candidate sources for both variants were kept outside the repository during this investigation
and are not checked in.
