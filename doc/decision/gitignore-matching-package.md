# Keep ignore for gitignore matching

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Status

Accepted,
 2026-07-05.

## Context

The workspace uses `ignore` for gitignore-style matching in three places:

- `package/pi-plugin/guardrail/src/path-guard.ts` protects paths before pi edit/write tools run.
- `package/watch-restart/cli/src/filters/gitignore.ts` filters watched file events.
- `package/cli/markdown-lint/src/walk-files.ts` applies nested `.gitignore` layers while walking Markdown files.

The guardrail caller is the decision driver.
It runs inside a coding-agent tool boundary,
controls writes to protected paths,
and needs both final gitignore state and rule provenance for useful refusal messages.
The current guardrail implementation uses `ignore().add({ pattern, mark })` and `.test(relativePath)`;
`package/pi-plugin/guardrail/src/path-guard.ts` also rebuilds single-pattern matchers to select the last matching
message rule.

The replacement constraints were:

- Gitignore parity,
 including negation,
 directory rules,
 anchoring,
 and nested `.gitignore` behavior where relevant.
- Node ESM consumption,
 preferably native ESM with bundled TypeScript declarations.
- High human-auditability for agent guardrail code.
- No loss of rule provenance where the guardrail needs a refusal reason.

Open-source candidates were surveyed through npm keyword search and GitHub topic search.
The serious packages audited were `ignore`,
 `fast-ignore`,
 `gitignore-matcher`,
 `cspell-gitignore`,
 and `ignore-walk`.
Metadata-only packages and generators were rejected during screening because they were glob-only,
 ignore-file template tools,
 CLI generators,
 or wrappers around a filesystem walk rather than direct matchers.

## Decision

Keep `ignore` as the gitignore matcher for guardrail and other parity-sensitive callers.
Do not replace it with a native-ESM package at this time.

`ignore` is CommonJS,
 but Node's ESM interop works for default import in the workspace target:
 a scratch package installed `ignore@7.0.5` and ran `import ignore from "ignore"`,
 then `ignore({ ignorecase: false }).add("*.log").test("debug.log").ignored` returned `true`.
The published package ships `index.d.ts`,
 has no runtime dependencies,
 and exposes the `mark` field the guardrail uses for rule provenance.

Source and validation evidence from the audit:

- `/tmp/agent/node-ignore-20260705-1783268149/index.js` stores `mark` on `IgnoreRule`,
 accepts `{ pattern, mark }` in `createRule`,
 returns `{ ignored, unignored, rule }` from the rule manager,
 and exports the CommonJS factory plus `factory.default`.
- `/tmp/agent/node-ignore-20260705-1783268149/test/git-check-ignore.test.js` compares fixtures with `git check-ignore`.
- `/tmp/agent/node-ignore-20260705-1783268149/.github/workflows/nodejs.yml` runs Linux,
 macOS,
 and Windows jobs on Node 20.
- Container validation ran with
`podman run --memory=2g --cpus=2 --rm --security-opt label=disable --userns=keep-id --user $(id --user):$(id --group) --volume /tmp/agent/node-ignore-20260705-1783268149:/work --workdir /work docker.io/library/node:24-bookworm sh -lc 'node --version && npm install --ignore-scripts --no-audit --no-fund && npm test'`.
  It passed `922/922` tests with `100%` statement,
 branch,
 function,
 and line coverage in the tap report.
- Human-auditability surface is small for the trust boundary:
 one production file,
 no runtime dependencies,
 and no native or Wasm boundary.

The package stays despite not being native ESM because exact semantics,
 rule provenance,
 mature tests,
 and a smaller dependency surface matter more for agent guardrail code than package-module format.

## Rejected alternatives

### fast-ignore

`fast-ignore@2.0.0` is the closest native-ESM alternative.
It ships native ESM,
 bundled declarations,
 and a compact implementation.
`/tmp/agent/fast-ignore-20260705-1783268149/src/ignore/matcher.ts` compiles ignore tiers into a tree
and returns a boolean matcher.
`/tmp/agent/fast-ignore-20260705-1783268149/src/glob/parse.ts` delegates segment parsing to `grammex`,
 and `/tmp/agent/fast-ignore-20260705-1783268149/src/glob/grammar.ts` defines the supported star,
 question,
 and character-class grammar.
The production surface measured as ten source files and roughly `256` code lines.

Rejected for guardrail use.
It has no `mark`,
 explain,
 or final-rule API,
 so the protected-path refusal message path would need extra local matcher logic.
It also extends the audit surface through same-author runtime dependencies:
 `grammex` and `string-escape-regex`.
Those dependencies were cloned at `/tmp/agent/grammex-20260705-1783268149`
and `/tmp/agent/string-escape-regex-20260705-1783268149`.
`string-escape-regex` is trivial,
 but `grammex` is a separate parser engine with about `426` nonblank source lines,
 so using `fast-ignore` means auditing more than the candidate package itself.

Validation found a reproducible packaging friction:
 `npm test` failed on a clean clone with `ERR_MODULE_NOT_FOUND` because `test/index.js` imports `dist/` before it
 exists.
`npm run compile && npm test` then passed with `4` tests.
No GitHub workflow,
 fuzzing,
 property-based testing,
 or mutation testing evidence was found by searching for `fast-check`,
 `fuzz`,
 `mutation`,
 `stryker`,
 `proptest`,
 and `quickcheck`.

`fast-ignore` remains acceptable for a future non-guardrail boolean filter if the caller only needs flat in-memory
matching.
It should not replace `ignore` where rule provenance or high-assurance gitignore parity is required.

### gitignore-matcher

`gitignore-matcher@1.0.0` is native ESM and has no runtime dependencies.
It was rejected because source audit and integration testing found incorrect git semantics.

`/tmp/agent/condu-20260705-1783268149/packages/generic/gitignore-matcher/gitignore-matcher.ts` converts patterns to
regular expressions and exposes `isAccepted`,
 `isIgnored`,
 and `explain`.
The implementation measured as one production file and roughly `219` code lines.
Its npm package has no license metadata,
 had `35` downloads for 2026-06-05 to 2026-07-04,
 and the upstream repo had no public issue history when checked.

Behavior check:
 a scratch package installed `gitignore-matcher@1.0.0`,
 created `new GitIgnore("foo/bar\n")`,
 and `isIgnored("x/foo/bar")` returned `true`.
A disposable Git repository with the same `.gitignore` pattern reported `?? x/foo/bar` from
`git status --ignored --short --untracked-files=all`,
 so Git did not ignore that path.
That mismatch disqualifies it for gitignore parity.

### cspell-gitignore

`cspell-gitignore@10.0.1` is ESM,
 actively released,
 and has source-level support for returning matching file,
 line,
 and glob information.
It was rejected because it is the wrong abstraction and has a nested-negation mismatch against Git.

`/tmp/agent/cspell-20260705-1783268149/packages/cspell-gitignore/src/GitIgnoreFile.ts` builds a `GlobMatcher` from each
`.gitignore` file and `GitIgnoreHierarchy.isIgnored()` returns on the first matching layer.
`/tmp/agent/cspell-20260705-1783268149/packages/cspell-gitignore/src/GitIgnore.ts` walks ancestor directories
and caches hierarchies.
The package depends on `@cspell/url`,
 `cspell-glob`,
 and `cspell-io`,
 which in turn pull in `@cspell/cspell-service-bus` and `picomatch` in the resolved runtime closure.

Behavior check:
 a scratch repository used root `.gitignore` content `*.log` and nested `sub/.gitignore` content `!keep.log`.
`cspell-gitignore` reported `sub/keep.log` as ignored by the root pattern.
Git reported `?? sub/keep.log` from `git status --ignored --short --untracked-files=all`,
 so the nested negation re-included the file.
That mismatch disqualifies it for gitignore parity.

Maintenance is strong at the monorepo level,
 with frequent releases and recent maintainer PR activity,
 but the semantics mismatch matters more than release cadence for this decision.

### ignore-walk

`ignore-walk@9.0.0` is rejected because it is a filesystem walker,
 not a direct in-memory matcher.
`/tmp/agent/ignore-walk-20260705-1783268149/lib/index.js` reads ignore files during traversal,
 applies `minimatch`,
 and returns walked file lists.
The API shape does not replace `ignore().add(...).test(path)` in the guardrail.
It is also CommonJS and requires Node `^22.22.2 || ^24.15.0 || >=26.0.0`,
 which is a stricter engine floor than the incumbent.

### Picomatch and converters

`picomatch` is a glob matcher,
 not a gitignore-rule manager.
`@humanwhocodes/gitignore-to-minimatch` converts patterns but does not provide final gitignore-state management,
 nested directory semantics,
 or rule provenance.
`gitignore-fs` was rejected during metadata screening because it depends on `ignore`
and shells out through a filesystem-oriented API,
 so it does not remove the incumbent risk.

## Ranking

1. `ignore`,
 chosen.
2. `fast-ignore`,
 acceptable only for non-guardrail boolean matching.
3. `ignore-walk`,
 useful walker but wrong API.
4. `cspell-gitignore`,
 active but git-incompatible for tested nested negation.
5. `gitignore-matcher`,
 native ESM but git-incompatible for tested slash anchoring.

`ignore` beats `fast-ignore` because the guardrail needs rule provenance,
 a mature parity test suite,
 and the smallest runtime audit surface.
`fast-ignore` beats `ignore-walk` because it is at least an in-memory matcher.
`ignore-walk` beats `cspell-gitignore` for this ranking only because its wrong abstraction is explicit;
 `cspell-gitignore` looked closer but failed a parity check.
`cspell-gitignore` beats `gitignore-matcher` because it has stronger maintenance signals and richer source tests,
 even though both are rejected for parity-sensitive use.

## Consequences

- Keep `ignore` in the pnpm catalog and package manifests that need gitignore semantics.
- Keep the existing guardrail use of `mark` and `.test(relativePath)`.
- Do not propose `fast-ignore` as a drop-in replacement unless the caller explicitly does not need rule provenance
and accepts a smaller validation base.
- Any future replacement proposal must include a disposable Git fixture comparing candidate behavior with
`git status --ignored` or `git check-ignore`,
 plus source audit and full package validation.
