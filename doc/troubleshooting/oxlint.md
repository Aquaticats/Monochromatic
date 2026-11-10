# Oxlint type-aware mode silently skips type checks when run from monorepo root, plus disable-comment prefix stripping accepting unknown plugin namespaces

This file groups two independent oxlint issues that surface across the
workspace.
 Each gets its own canonical section.

---

## Bug 1: `--type-aware` resolves the wrong `tsconfig.json` from monorepo root

### Symptom

Running `oxlint --type-aware .` from the monorepo root finishes with
no type-aware diagnostics for files that should clearly trigger them
(e.g. an obvious `no-floating-promises` violation).
 The same files
report the diagnostic when oxlint runs from inside the package
directory.

No error is printed;
 the type-aware check simply produces no findings.

### Root cause

Oxlint discovers the nearest `tsconfig.json` by walking upward from
the working directory.
 From the monorepo root the search resolves
`./tsconfig.json` (the root config),
 whose `include`/`references`
configuration does not enumerate every package's `src/` tree.
 Files
outside the root config's surface are not analysed by the type-aware
backend;
 oxlint treats them as untyped and skips the rules that
require type information.

This is a configuration-discovery choice in oxlint,
 not a defect:
 it
loads the config it finds and reports diagnostics for the files that
config covers.
 The trap is silence rather than error.

### Verification

Version under test:
 oxlint as pinned in `mise.toml` (commit `main` as
of 2026-03-13).

Reproduce:

```bash
# From the monorepo root: type-aware rules silently skip
mise run lint:oxlint

# Compare: run per-package; same files now flagged
mise run //package/<some-pkg>:lint:oxlint
```

The diagnostic count differs between the two invocations even though
the source tree is identical.

### Verified workaround

Run oxlint from each package independently.
 The `lint:oxlint` task
template in `mise.toml` already does this:
 the root `lint:oxlint` task
fans out across packages via `mise '/packages/...:lint:oxlint'`,
 so
every package executes with its own `tsconfig.json` in scope.

Tradeoff:
 parallel fan-out across many packages spends more
wall-clock startup time than a single invocation would;
 in practice
the per-package runs parallelise well and the additional cost is
small.
 Type correctness wins over throughput.

### What does not work

- Adding every package to the root `tsconfig.json`'s
  `include`/`references`:
   the workspace prefers per-package configs to
  avoid coupling unrelated packages' type checks.
   The change would
  invalidate isolated package builds.
- Passing `--tsconfig` on the root invocation:
   oxlint accepts the
  flag but the resolved config still misses package-local
  `paths`/`baseUrl` overrides,
   so per-package type info is incomplete.

### Why we do not file this upstream

1.  **Is it really upstream's fault?
    ** Borderline.
     Oxlint resolves
    what it is told to resolve;
     the silent-skip behaviour is a UX
    wart,
     not an algorithmic defect.
2.  **Can upstream fix it?
    ** They could emit a warning when running
    `--type-aware` and the discovered `tsconfig.json` does not cover
    some of the files under analysis.
     That is a UX patch.
3.  **Are they supporting this use case?
    ** Monorepos are a documented
    target;
     the per-package run is the documented pattern.
4.  **Will they likely fix it?
    ** Unknown.
     A "no type info for X files"
    warning would be a small change.
5.  **Have we prototyped a minimal fix?
    ** No.

Decision:
 no upstream report at this time.
 The per-package run pattern
is the documented escape hatch.

---

## Bug 2: Disable-comment prefix stripping is unguarded, so non-canonical and bogus prefixes silently suppress diagnostics

### Symptom

A disable comment using a non-canonical prefix appears to function:

```ts
// oxlint-disable-next-line eslint/no-await-in-loop
await something();
```

The reviewer expects oxlint to ignore the comment (because oxlint
rules do not use the `eslint/` prefix).
 It does not:
 the rule is
suppressed.
 The same is true for arbitrary prefixes:

```ts
// oxlint-disable-next-line xyzzy/no-await-in-loop
```

This is misleading rather than broken;
 it masks copy-paste mistakes
from ESLint configs and produces inconsistent codebases.

### Root cause

The substring `.contains()` form documented in the original
2026-03-13 snapshot of this file has since been closed upstream by an
explicit `==` comparison after stripping the plugin prefix from the
directive's stored rule name.
 A regression test at
`crates/oxc_linter/src/disable_directives.rs:2048-2086`
(`directive_rule_name_is_matched_on_full_rule_name_not_substring`)
locks the old `no-re-export` substring case shut.

What survives is a different defect with the same surface symptom:
the prefix is stripped **unconditionally**,
 without checking that the
prefix names a recognized oxlint plugin.
 So `eslint/no-await-in-loop`,
`xyzzy/no-await-in-loop`,
 and any other made-up prefix still strip
down to the bare rule name and equality-check successfully.

Source citations against oxc commit
`e182aee2599c275dc0bd93f52b4ddda70ff2c93b` (HEAD of `main` on
2026-05-17,
 the snapshot used for the prototype below):

- `crates/oxc_linter/src/disable_directives.rs:281-340`:
   `contains()`
  method.
   Lines 304-306 contain an explicit comment about the
  substring fix;
   the active match arm at lines 307-313 reads
  `name.rsplit_once('/').map_or(name.as_str(), |(_, rule)| rule) ==
  rule_name`,
   which strips any prefix without validating it.
- `crates/oxc_linter/src/disable_directives.rs:832-871`:
  `get_rule_names()` parser (splits on `,`,
   trims whitespace,
   strips
  `--`/`-` description suffixes);
   produces the `name` values that
  feed into the prefix-strip above.
- `crates/oxc_linter/src/tsgolint.rs:1158-1185`:
  `should_skip_diagnostic()` calls the same `contains()` method,
   so a
  fix at the method body automatically reaches tsgo rules.
   The
  surrounding `typescript-eslint/{rule}` and `@typescript-eslint/{rule}`
  fallback calls remain belt-and-braces against canonical-prefix
  authoring.
- `crates/oxc_linter/src/config/plugins.rs:134-167`:
  `LintPlugins::try_from` is the authoritative parsed prefix table.
  Maps recognized plugin namespaces (canonical names plus the
  `eslint-plugin-` and `@scope/eslint-plugin-foo` alias normalizations)
  to the bitflag set.
   `LintPlugins::ESLINT` is the empty bitflag
  because eslint-core rules are addressed without a prefix in oxlint's
  canonical naming.

Any token in the comment whose suffix (after stripping the first
`/`-separated prefix) equals the bare rule name satisfies the check,
regardless of what prefix was used.

### Verification

Version under test:
 oxc commit
`e182aee2599c275dc0bd93f52b4ddda70ff2c93b` (HEAD of `main`,
2026-05-17),
 built with the toolchain pinned in
`rust-toolchain.toml` (rustc 1.95.0).

Reproduce:

```ts
// File A: canonical prefix
// oxlint-disable-next-line no-await-in-loop
await x();

// File B: ESLint-style prefix
// oxlint-disable-next-line eslint/no-await-in-loop
await x();

// File C: nonsense prefix
// oxlint-disable-next-line xyzzy/no-await-in-loop
await x();
```

Running oxlint over all three produces no diagnostics.
 The diagnostic
returns only when the comment is removed entirely or when the rule
name itself is mistyped (the unguarded prefix-strip then has nothing
to compare against).

Canonical prefixes (matching `parse_rule_key` in `config/rules.rs`):

- eslint core:
   bare name,
   no prefix (e.g. `no-await-in-loop`,
  `require-await`)
- TypeScript:
   `typescript/` (e.g.
  `typescript/no-unsafe-type-assertion`)
- Import:
   `import/` (e.g. `import/no-unassigned-import`)
- Promise:
   `promise/` (e.g. `promise/avoid-new`)
- Unicorn:
   `unicorn/` (e.g. `unicorn/prefer-top-level-await`)
- Node:
   `node/` (e.g. `node/no-sync`)

### Verified workaround

When authoring a new disable comment,
 use the canonical prefix list
above.
 When reviewing existing code,
 normalise non-canonical prefixes
to the canonical form during the next touch of the file.

Tradeoff:
 the existing tree contains many non-canonical prefixes
inherited from the pre-oxlint era (ESLint-style,
 parentheses-style
such as `eslint(rule-name)`).
 A sweep-replace is feasible but would
churn the history of every package;
 normalising opportunistically is
the lower-risk path.

### What does not work

- Configuring oxlint to enforce canonical prefixes:
   there is no such
  option (the prefix-strip behaviour is built into the directive
  matcher,
   not a setting).
- Writing a project linter rule to flag non-canonical prefixes:
  possible,
   but adds yet another tool.
   Not justified for the current
  rate of new disable comments.

### Why we file this upstream

1.  **Is it really upstream's fault?
    ** Yes;
     unconditional prefix
    stripping is the source of the wart.
     Validating the prefix against
    the parsed plugin table would be the correct behaviour.
2.  **Can upstream fix it?
    ** Yes;
     replace the unguarded
    `name.rsplit_once('/').map_or(...)` strip with a guarded
    `name.split_once('/')` plus `LintPlugins::try_from(prefix)` check.
    The change is local to the `contains()` method in
    `disable_directives.rs`;
     `tsgolint::should_skip_diagnostic` calls
    the same method and inherits the fix for free.
3.  **Are they supporting this use case?
    ** Yes;
     canonical prefixes are
    the authoritative configuration form (`LintPlugins::try_from` and
    `parse_rule_key` both reject unknown plugin names with an explicit
    error).
     The disable-directive matcher is the only path that
    silently accepts unknown prefixes.
4.  **Will they likely fix it?
    ** Probably worth filing.
     The closed
    substring regression (`directive_rule_name_is_matched_on_full_rule_name_not_substring`,
    line 2048) shows the maintainers care about disable-directive
    correctness.
     The remaining permissive behaviour is the natural
    next correction,
     but it does shift semantics for any disable
    comment that relies on a bogus prefix today.
5.  **Have we prototyped a minimal fix?
    ** Yes;
     patch + tests below.

#### Prototype result

Clone (private,
 throwaway):
 `/tmp/oxc-bug2-prototype.*/oxc`.
Origin verified at `https://github.com/oxc-project/oxc.git`,
 HEAD
`e182aee2599c275dc0bd93f52b4ddda70ff2c93b`.

Patch file:
 [oxlint.patch](oxlint.patch).
The diff is 110 lines (one source hunk,
 one tests hunk);
 the file
header in the patch records the apply-and-verify commands.

Test verification:

- Added four targeted tests (the four `*_post_fix` names listed in
  the patch).
   Pre-patch run:
   2 pass (`bare_canonical_prefix_*` and
  `canonical_typescript_prefix_*`),
   2 fail
  (`non_canonical_eslint_prefix_*`,
   `bogus_xyzzy_prefix_*`),
  reproducing the bug.
   Post-patch run:
   4 pass.
- Full suite:
   `cargo test -p oxc_linter --lib` → 1129 passed,
   1
  failed (1 ignored,
   unchanged).
   The single casualty is
  `directive_rule_lists_parse_rules_and_descriptions`,
   which uses a
  `// oxlint-disable-next-line @scope/plugin/rule-name no-debugger`
  directive in `test_directive` and relies on the lenient invariant
  that any `/`-containing rule's bare suffix also suppresses the rule.
  After the patch,
   only canonical-plugin prefixes may strip;
   the
  `@scope/plugin` namespace is not a recognized oxlint plugin,
   so the
  bare `rule-name` suffix no longer matches.
   The casualty is intended
  semantically (it is the very behaviour the fix corrects).
   Upstream
  should either drop that case from the `cases` array or replace it
  with a canonical-prefix example such as
  `typescript/unbound-method`,
   which already appears earlier in the
  same array.

Decision:
 file upstream with the patch and the candidate test edit.
The 5-constraint audit now reads all-yes.

### Draft upstream issue

Do not file as-is;
 review against the latest upstream `main` before
opening,
 and re-run the verification harness against that HEAD to
confirm the patch still applies and the casualty list has not grown.

````md
Title: linter: disable-comment prefix is stripped without validation, so unknown plugin namespaces silently suppress diagnostics

Labels: A-linter, C-bug

### Summary

`DisableDirectives::contains` strips the first `/`-separated prefix
from the directive's stored rule name unconditionally before
comparing the bare suffix to the rule the linter is reporting on.
That means comments such as `// oxlint-disable-next-line
eslint/no-await-in-loop` or `// oxlint-disable-next-line
xyzzy/no-await-in-loop` suppress oxlint's `no-await-in-loop` rule
even though `eslint/` and `xyzzy/` are not recognized oxlint plugin
namespaces.

The substring-`.contains()` form that the
`directive_rule_name_is_matched_on_full_rule_name_not_substring`
regression test (`crates/oxc_linter/src/disable_directives.rs:2048`)
locks shut is closed. The remaining defect is unconditional prefix
stripping, which is closely related but distinct.

### Repro (oxc commit `e182aee2599c275dc0bd93f52b4ddda70ff2c93b`)

```ts
// oxlint-disable-next-line eslint/no-await-in-loop
await x();

// oxlint-disable-next-line xyzzy/no-await-in-loop
await y();
```

Running oxlint over either source emits no `no-await-in-loop`
diagnostic. The diagnostic only returns when the comment is removed
entirely or when the rule name itself is mistyped.

### Root cause

In `crates/oxc_linter/src/disable_directives.rs:281-340`, the
`DisableRule::Single` match arm at lines 307-313:

```rust
DisabledRule::Single { rule_name: name, .. } => {
    if rule_name.contains('/') {
        name == rule_name
    } else {
        name.rsplit_once('/').map_or(name.as_str(), |(_, rule)| rule) == rule_name
    }
}
```

The `else` branch strips any prefix off `name` and compares the
suffix to `rule_name`. There is no check that the stripped prefix
identifies a real oxlint plugin. `LintPlugins::try_from`
(`crates/oxc_linter/src/config/plugins.rs:134-167`) is the
authoritative parsed plugin table and would reject `xyzzy`, but the
disable-directive matcher does not consult it.

### Suggested fix

Replace the unguarded strip with a guarded `split_once` plus
`LintPlugins::try_from(prefix)` check. The eslint pseudo-plugin
(`LintPlugins::ESLINT`, the empty bitflag) is intentionally rejected
because eslint-core rules are addressed without a prefix in oxlint's
canonical naming. Concretely:

```rust
match name.split_once('/') {
    None => name == rule_name,
    Some((prefix, suffix)) => {
        suffix == rule_name
            && LintPlugins::try_from(prefix).is_ok_and(|p| !p.is_empty())
    }
}
```

Full diff (single hunk in `contains()` plus four new test cases) is
attached as `oxlint.patch` in the linked downstream
repository; it applies cleanly with `git apply` against HEAD
`e182aee2599c275dc0bd93f52b4ddda70ff2c93b`.

### Test coverage

The patch adds four tests next to the existing
`directive_rule_name_is_matched_on_full_rule_name_not_substring`
test:

- `bare_canonical_prefix_still_suppresses_post_fix`: bare-name
  directive still suppresses (no regression).
- `canonical_typescript_prefix_still_suppresses_post_fix`:
  `typescript/no-floating-promises` still suppresses bare
  `no-floating-promises` (cross-plugin strip-and-match remains for
  canonical plugins).
- `non_canonical_eslint_prefix_must_not_suppress_post_fix`:
  `eslint/no-await-in-loop` no longer suppresses (eslint-core has no
  prefix in oxlint canonical naming).
- `bogus_xyzzy_prefix_must_not_suppress_post_fix`:
  `xyzzy/no-await-in-loop` no longer suppresses.

Pre-patch, the first two pass and the last two fail. Post-patch all
four pass.

### Breaking-change concern

This is a behaviour-correcting change with a known migration cost:
any existing disable comment in the wild that uses a prefix outside
the recognized `LintPlugins` namespace (or that uses the
non-canonical `eslint/` prefix) will start to surface the underlying
diagnostic again. The migration path is to rewrite those comments
to use a canonical prefix or the bare rule name. The set of
recognized prefixes is exactly the set listed in
`LintPlugins::try_from`, which downstream consumers can grep for.

The existing in-tree test
`directive_rule_lists_parse_rules_and_descriptions` exercises a
`@scope/plugin/rule-name` directive whose `test_directive` helper
asserts that the bare `rule-name` suffix suppresses regardless of
prefix. That assertion is the very lenient behaviour this patch
corrects, so the case needs to be either dropped or rewritten to use
a canonical-plugin prefix. The remaining 1129 unit tests in
`oxc_linter` continue to pass.

### Alternatives considered

- **Strict equality only** (no strip-and-match even for canonical
  prefixes): would also fix the bug but breaks the documented
  cross-plugin compatibility for canonical rules
  (`typescript/no-foo` suppressing oxlint's `no-foo`). Rejected.
- **Maintain a separate disable-comment prefix table**: duplicates
  `LintPlugins::try_from` and drifts. Rejected; the existing table
  is already the source of truth.
- **Lint disable comments instead of fixing the matcher**: requires
  an additional rule and does not retroactively fix existing
  permissive comments. Acceptable as a complement but not a
  substitute.

### Environment

- oxc commit: `e182aee2599c275dc0bd93f52b4ddda70ff2c93b` (HEAD of
  `main`, 2026-05-17).
- Toolchain: rustc 1.95.0 (per `rust-toolchain.toml`).
- Verified on Linux x86_64.
````

Decision:
 ready to file upstream once a maintainer re-runs the
verification harness against the latest `main` (no issue has been
opened yet from this repository).
 The casualty is documented and
intended.
 Re-evaluate constraint 4 if the maintainers reject the
breaking-change cost;
 in that case the lint-disable-comments
alternative becomes the next probe.

---

## Cross-reference

ESLint was fully replaced by oxlint on 2026-03-13.
 The project will not
re-adopt ESLint;
 the disable-comment prefix sweep applies only to the
non-canonical residue from the pre-oxlint era.

## Upstream bug: ignorePatterns resolved from CWD

**Note:
** The monorepo has since migrated from `.oxlintrc.json` to `oxlint.config.ts`.
The bug report below is preserved as-is since it documents a real upstream issue.

---

## oxlint bug: `ignorePatterns` resolved from CWD instead of config file directory

**oxlint version:
** 1.55.0
**Severity:
** incorrect file filtering when `-c` config is in a different directory than CWD

### Summary

`ignorePatterns` in `.oxlintrc.json` are documented as
"resolved from the configuration file path"
([config-file-reference](https://oxc.rs/docs/guide/usage/linter/config-file-reference)),
but the CLI resolves them from CWD instead.
This causes patterns like `**/test-fixture/**` to silently stop working
when oxlint is invoked from a subdirectory with `-c` pointing to a config elsewhere.

### Minimal reproduction

```text
repo/
  .oxlintrc.json          # { "ignorePatterns": ["**/test-fixture/**"] }
  package/
    test-fixture/
      my-pkg/
        src/
          file.ts         # has a lint violation
```

```bash
## From repo root -- works correctly (file ignored, 0 files linted)
cd repo
oxlint -c .oxlintrc.json package/test-fixture/my-pkg/src/file.ts
## -> Found 0 warnings and 0 errors. Finished on 0 files.

## From subdirectory with -c -- BROKEN (file linted, violation reported)
cd repo/packages/test-fixture/my-pkg
oxlint -c /absolute/path/to/repo/.oxlintrc.json src/file.ts
## -> Found errors.

## Auto-discovery from subdirectory is also broken
cd repo/packages/test-fixture/my-pkg
oxlint -c /absolute/path/to/repo/.oxlintrc.json
## -> Lints all files, ignorePatterns has no effect
```

### Root cause

`apps/oxlint/src/lint.rs:326` passes `&self.cwd` (CWD from `env::current_dir()`)
as the root directory for `LintIgnoreMatcher`:

```rust
// lint.rs:61
cwd: env::current_dir().expect("Failed to get current working directory"),

// lint.rs:326
let ignore_matcher =
    { LintIgnoreMatcher::new(&base_ignore_patterns, &self.cwd, nested_ignore_patterns) };
```

`LintIgnoreMatcher::new` builds a `GitignoreBuilder` rooted at `base_root`:

```rust
// crates/oxc_linter/src/config/ignore_matcher.rs:22
let mut builder = GitignoreBuilder::new(base_root);
```

`GitignoreBuilder::new(root)` resolves all patterns relative to `root`.
When `root` is CWD (e.g. `package/test-fixture/my-pkg/`),
the discovered file path `src/file.ts` is matched against `**/test-fixture/**`
relative to that CWD.
 Since `src/file.ts` does not contain `test-fixture/`,
the pattern never matches.

When `root` is the config file's parent directory (the repo root),
the path becomes `package/test-fixture/my-pkg/src/file.ts`,
which correctly matches `**/test-fixture/**`.

The **LSP implementation gets this right**:
`apps/oxlint/src/lsp/server_linter.rs:225` uses `root_path` (workspace root,
which is where the config lives) instead of CWD:

```rust
// server_linter.rs:225
LintIgnoreMatcher::new(&base_patterns, &root_path, nested_ignore_patterns),
```

### Suggested fix

Use the config file's parent directory instead of CWD:

```rust
// lint.rs:325-326, before:
let ignore_matcher =
    { LintIgnoreMatcher::new(&base_ignore_patterns, &self.cwd, nested_ignore_patterns) };

// after:
let config_dir = root_config
    .path
    .parent()
    .unwrap_or(&self.cwd);
let ignore_matcher =
    { LintIgnoreMatcher::new(&base_ignore_patterns, config_dir, nested_ignore_patterns) };
```

Falls back to CWD when no explicit config file is provided
(in which case `root_config.path` defaults to the auto-discovered config in CWD,
so `parent()` already equals CWD).

### Impact

Any monorepo that runs per-package lint tasks from subdirectories
(e.g. mise,
 moon,
 turborepo,
 nx) with a shared root `.oxlintrc.json`
will have all `ignorePatterns` silently ignored.
This is especially common in monorepo setups like:

```toml
## mise task template
[task_templates."lint:oxlint"]
run = "oxlint --type-aware -c {{vars.monorepo_root}}/.oxlintrc.json"
```

where each package's lint task runs from the package directory.
