# Audit: unused oxlint-disable directives across the workspace

## Supersession

This report preserves the measured pre-remediation directive baseline.
The later semantic-readonly migration retired the native readonly rule and removed its remaining authored directives,
including ignored `package-paused/` sources.
Current code must use the project-owned rule's honest types and effect contracts instead of suppression.

## Summary

oxlint reports 301 unused oxlint-disable directives spread across 38 packages,
out of 1442 directive occurrences total (about 21 percent unused).
They are genuinely unused,
 not false positives:
 each one suppresses a rule that
reports nothing at that location.

Roughly three-quarters target type-aware (tsgolint) rules that the adjacent code
does not actually trigger,
 mostly `typescript/no-unsafe-type-assertion`,
`typescript/prefer-readonly-parameter-types`,
 and the `typescript/no-unsafe-*` family.
The non-type-aware remainder is about 75 to 80 occurrences across rules like
`no-bitwise`,
 `import/no-namespace`,
 `no-await-in-loop`,
 and `no-magic-numbers`.
The directives are not careless:
 most carry thoughtful justification comments.
They are unnecessary because the rules are more lenient than the author assumed,
the rule is turned off in config,
 or the directive sits on a callback parameter the
rule exempts.

This is a workspace-debt finding,
 not an oxlint bug.
oxlint's unused-directive reporting behaves correctly,
 including across the type-aware pass.

All 301 were removed (see "Remediation (applied)" below);
 the findings here describe the
pre-remediation state.

## How they surface

`package/config/oxlint/src/index.ts:40` sets `reportUnusedDisableDirectives: 'warn'`,
and `index.ts:39` sets `denyWarnings: true`.
Together,
 every unused directive is a warning that `denyWarnings` promotes to a failure.

The real per-package task confirms this.
Running `mise run //package/module/es:lint:oxlint` reports 19 unused directives and ends with
`ERROR task failed` (253 warnings,
 0 errors,
 denied).
So `mise run lint` is currently red on each package that owns unused directives.

The schema at `node_modules/oxlint/configuration_schema.json:2925` notes that
`reportUnusedDisableDirectives` is "Only supported in the root configuration file.
"
In practice it is active in normal per-package runs as well:
 the workspace has a single
`oxlint.config.ts` at the root,
 so oxlint treats it as the root config regardless of the
directory the lint runs from.
This differs from `typeAware` (same options group,
 `index.ts:62` comment),
 which the mise
task template still passes explicitly as `--type-aware` (`mise.toml:222`).
No CLI flag is needed for unused-directive reporting;
 the config value already takes effect.

## Evidence the 301 are genuinely unused

Three independent lines of evidence rule out the false-positive hypothesis,
 that
type-aware suppressions are flagged because the unused-check cannot see tsgolint diagnostics.

1.  oxlint's own diagnostic text is `Unused oxlint-disable directive (no problems were reported).`
    The check ran the rule at that scope and the rule reported nothing.
2.  A controlled fixture proves the check accounts for tsgolint.
    Two identical `value as string` assertions on an `unknown`,
     one with a
    `typescript/no-unsafe-type-assertion` directive and one without:
     the bare one produced
    `typescript(no-unsafe-type-assertion): Unsafe type assertion`,
     the suppressed one produced
    nothing,
     and its directive was not flagged unused.
    A used type-aware directive is correctly recognized as used.
3.  The flagged set is type-independent.
    Running the whole tree with `--type-aware` and without it both flag exactly the same 301
    locations (byte-for-byte identical file:
    line sets),
     even though the type-aware run emits
    1133 real `! typescript(...)` diagnostics elsewhere.
    The 301 do not depend on type information;
     the rules simply do not fire there.

## Root-cause categories

Canonical rule breakdown (type-aware rules marked TA,
 prefix variants merged).
Counts are approximate;
 a single directive line can list several rules,
 and
block disables span an `oxlint-disable` and an `oxlint-enable` line.

```text
TA  no-unsafe-type-assertion          ~74   (typescript/, typescript-eslint/, bare, and enable markers)
TA  prefer-readonly-parameter-types   ~79
TA  no-unnecessary-condition           24   (also turned off in config; see below)
TA  no-unsafe-assignment               14
TA  no-unsafe-return                   13
TA  no-unsafe-member-access            18
TA  no-unsafe-call                     16
TA  strict-boolean-expressions          8
TA  require-await                      13
TA  no-unsafe-argument                  4
    no-await-in-loop                   15
    no-magic-numbers                   12
    no-bitwise                         11
    prefer-destructuring               10
    import/no-namespace                 8
    no-restricted-syntax (+/no-function-root-let)  13
    import/no-named-as-default          5
    no-var                              4
```

The categories that explain them:

1.  Defensive type-aware suppressions on safe code.
    The bulk.
     Authors disabled `prefer-readonly-parameter-types` or `no-unsafe-type-assertion`
    expecting a complaint that never comes.
     Example:
    `package/module/es/src/types/t function/f/t function/memoize/r a/p n/index.ts:223`
    disables `no-unsafe-type-assertion` above `return memoized as MemoizedAsyncFunction<...>`,
    but tsgolint considers that assertion safe after the property assignments above it.
    Many sit on `Array.prototype.filter` and `.map` callbacks whose parameter the rule exempts
    (for example `package/webapp-forge/server/src/worker/render.ts:234`).
2.  Disabling a rule that is turned off in config.
    `typescript/no-unnecessary-condition` is `'off'` at
    `package/config/oxlint/src/rule/correctness.ts:44`,
     so all 24 directives targeting it are dead.
    Example:
     `package/config/tofu/fetch_ips.ts:76` disables it above `while (true)`.
3.  Dead `oxlint-disable`/`oxlint-enable` block pairs where the rule never fires inside.
    72 of the 301 are `/* oxlint-enable ... */` lines (232 are `oxlint-disable` lines).
    Example:
     `package/webapp-forge/server/src/storage/adapter.ts:78`.
    oxlint flags only the enable for such a block;
     the paired disable reports unused only after
    the enable is gone (removing the enable extends the disable to end-of-file,
     then the disable
    itself is flagged).
     A controlled test confirmed the disable suppresses nothing:
     deleting it
    alone surfaced no new diagnostic,
     only `Unused oxlint-enable directive (no matching
    oxlint-disable directives were found)`.
     Both lines must be removed together.
4.  Non-canonical prefixes copied from ESLint configs.
    `typescript-eslint/...`,
     `typescript-eslint(...)`,
     and bare `no-unsafe-type-assertion`
    forms still match the rule (see `doc/troubleshooting/oxlint.md` on prefix stripping),
    so the prefix is not why they are unused;
     the underlying rule simply does not fire.

## The tofu exception

`package/config/tofu` is an OpenTofu/Terraform package with `.tf` files plus a couple of
root-level TypeScript scripts and no `mise.toml` and no `src/`.
It has no per-package lint task,
 so `mise run //package/config/tofu:lint:oxlint` does nothing
and its `fetch_ips.ts` directives surface only in a root-level whole-tree scan.
This is the one place where a per-package task and a root scan disagree on the count.

## Reproduce

```bash
# whole-tree count (301), from repo root
task-oxlint --report-unused-disable-directives 2>&1 | rg -c "Unused oxlint-disable directive"

# identical with type-aware on
task-oxlint --type-aware --report-unused-disable-directives 2>&1 | rg -c "Unused oxlint-disable directive"

# a real per-package task, already failing
mise run //package/module/es:lint:oxlint
```

## Remediation (applied)

Removed across two commits,
 each verified behaviorally neutral by capturing the full set of
2352 real (non-unused) diagnostics before and after and confirming they are byte-identical:

1.  `oxlint --fix` does not remove unused directives.
    Unlike ESLint,
     oxlint's `--fix` leaves them in place even with
    `--report-unused-disable-directives` and `--type-aware`;
     verified by running it against a
    known dead directive and seeing no change.
     Removal edits the flagged spans directly.
2.  Single-line and file-level directives (231,
     commit `b89e4f96`).
    `oxlint-disable-next-line` comments and file-level `oxlint-disable` blocks with no matching
    enable:
     delete the whole line,
     or drop the dead rule from a multi-rule list.
3.  Dead block pairs (63) and dead rules in live blocks (7),
     commit `a9a3e479`.
    For each flagged `oxlint-enable`,
     the matching `oxlint-disable` (located by per-file LIFO
    pairing) is also dead,
     so both lines are removed together;
     doing so in one pass avoids the
    cascade that removing only the enable would trigger.
     For a rule dead in an otherwise-live
    block,
     the rule is dropped from both the disable and its enable.

After remediation,
 zero unused oxlint-disable directives remain across the workspace.
The affected packages still fail `mise run lint` on pre-existing warnings unrelated to
directives (for example `typescript/prefer-readonly-parameter-types`,
 `require-unicode-regexp`,
`typescript/consistent-type-exports`),
 which `denyWarnings` promotes to failures;
 clearing
those is separate work.
