# Standards-review gate shadow log

Running record of shadow runs of the gate in `doc/runbook/standards-review-gate.md`.
One entry per run,
appended as runs happen.

An empty log means no shadow run has happened yet,
and never means there was nothing to catch.
The research baseline these runs extend is the 6 of 8 recall measurement
under the heading `Recall measurement results`
in `doc/handover/superlinear-cost-dismissal-gate.md`.

## Entry format

- Date:
- Subject:
- Input size:
- Reviewer route:
- Wall-clock cost:
- Named superlinear cost:
  yes,
  no,
  partial (cost gestured as unverified,
  not named),
  or unjudged
  (the runner writes `unjudged`;
  a scorer reads the findings and settles it
  under the rule recorded in
  `doc/handover/superlinear-cost-dismissal-gate.md`,
  heading `Recall measurement results`)
- Findings kept:
- Skipped:
  12c25207857b docs(handover):
  record the finished tool-image migration and the pre-existing failures it surfaced (no code files)
- Skipped:
  17241cbad51b feat(mise):
  run the standards-review gate shadow from a post-commit hook (no code files)
- Skipped:
  bdc0b42ff309 docs(audit):
  the shadow log's first two hook entries (no code files)
- Date:
  2026-10-29T14:08:32.185Z
  Subject:
  8c4ad0e59df4 test(file-enforcer/core):
  list pacman in the ImageMagick yes tuple so the CachyOS cell can install it
  Input size:
  1 code files,
  23 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  45.2 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    No findings in the supplied diff.
    `````
  No unbounded-input processing or cross-call complexity violations are introduced.

- Skipped:
  7440e2db7b90 docs(troubleshooting):
  cargo-binstall compiles from source when the GitHub API quota stalls release resolution;
  runbook records 0.1.1 (no code files)
- Date:
  2026-10-29T14:09:19.675Z
  Subject:
  e5d55a8d94da fix(ide/app):
  name the RADV driver file of the CachyOS image,
  radeon_icd.json,
  in the GPU container arguments
  Input size:
  1 code files,
  20 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  28.0 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    No findings in the supplied diff.
    `````
  No unbounded-input complexity issues or applicable `AGENTS.md` violations identified.

- Skipped:
  de17d1f55759 fix(nested-wayland-session-cli):
  give container cargo runs an XDG_RUNTIME_DIR (no code files)
- Skipped:
  c1444df68b07 docs(test-fixture-file-enforcer-perf):
  drop the TODO to pin Node in the container (no code files)
- Date:
  2026-10-29T14:09:58.890Z
  Subject:
  e655dcad15c6 fix(*):
  module tests take fixture directories from scratchRoot,
  not tmpdir (Refs #1810)
  Input size:
  159 code files,
  4157 diff lines,
  capped at 2000
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  171.8 s
  Named superlinear cost:
  partial (cost gestured as unverified,
  not named)
  Findings kept:
    `````text
    - `tempCacheRoot` (module documentation,
      `package/dependency-catalog/cli-cube/src/cache.unit.test.ts`):
      **nit**,
      no specific tag.
      The updated comment names `os.scratchRoot()`,
      but the import identifies `scratchRoot` as a workspace test-harness utility,
      not a `node:os` API.
      This misdocuments the helper’s owner.
    
    No other confirmed findings in the supplied hunks.
    In particular,
    no confirmed **RG2** violation:
    `scratchRoot()` is not shown,
    so its transitive per-fixture cost remains unverified rather than proven linear.
    The final `watch-filter.unit.test.ts` hunk is also truncated,
    limiting review coverage.

    `````
- Skipped:
  2b2ee1af35a1 docs(handover):
  record the nested-wayland XDG_RUNTIME_DIR fix,
  issue 1813 and the dropped Node pin (no code files)
- Date:
  2026-10-29T14:13:30.737Z
  Subject:
  8ca06f2b17ad fix(*):
  fixture and verification helpers take their directories from scratchRoot,
  not tmpdir (Refs #1810)
  Input size:
  21 code files,
  483 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  92.4 s
  Named superlinear cost:
  partial (cost gestured as unverified,
  not named)
  Findings kept:
    `````text
    No substantiated findings in the supplied diff.
    
    No visible call-site complexity violations were found.
    However,
    `scratchRoot()` is not included,
    so its transitive cost,
    scratch-directory creation,
    and permissions remain unverified.
    Package manifests and export maps are also absent,
    preventing verification of the new imports’ dependency declarations and resolution.

    `````
- Date:
  2026-10-29T14:16:00.238Z
  Subject:
  038e3d9077d2 fix(test-harness-core-matrix):
  install with a full pacman sync and no hook sandbox
  Input size:
  3 code files,
  89 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  53.1 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    No findings against the supplied standards.
    `````
  No unbounded-input or cross-call complexity violations are introduced by the shown changes.

- Skipped:
  49139124a7c1 docs(handover):
  record the finished test-matrix and resolver work and the arm64 IDE release leg (no code files)
- Skipped:
  12f83b98f9b8 feat(mutation-test-cli):
  build the runtime image on CachyOS with pacman instead of Fedora with dnf (no code files)
- Skipped:
  7b1caae071e3 fix(music-player-desktop-app):
  install noto-fonts in the Windows GNU image so the Slint software renderer finds a font under Wine,
  and record the UCRT and wine runner changes (no code files)
- Date:
  2026-10-29T14:19:14.551Z
  Subject:
  af0ab873d999 fix(*):
  git-clone-size and deps-update keep their scratch under node_modules/.monochromatic (Refs #1810)
  Input size:
  3 code files,
  175 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  390.6 s
  Named superlinear cost:
  partial (cost gestured as unverified,
  not named)
  Findings kept:
    `````text
    - `scratchParent`:
      **major**,
      `XTD`.
      The no-project branch stores scratch under `XDG_CACHE_HOME` or `~/.cache`,
      rather than the permitted `${HOME}/temp/agent` or a build directory.
      An `XDG_CACHE_HOME` inside `/tmp` also defeats the stated guarantee that clone scratch never uses the system temporary directory.
    
    - `createScratchWorkspace`,
      `scratchParent`:
      **minor**,
      `ATH`.
      Both introduce scratch parents with recursive `mkdir` but never apply the required `0700` permissions.
      The private directories created by `mkdtemp` do not establish the required permissions on those parents.
    
    - `scratchParent`:
      **minor**.
      In the no-project branch,
      an empty or relative `XDG_CACHE_HOME` produces a relative scratch path because `??` only handles missing values.
      This contradicts the documented absolute-path contract and can place clone scratch beneath the working directory.
    
    - `livesUnderTheProjectScratchDirectory`:
      **major**,
      `HOM`,
      `THR`.
      The test uses the real working directory and environment instead of an isolated project fixture and disposable home.
      Its `makeTempDir` call can create persistent parents in shared project state,
      and its assertion depends on an ancestor `node_modules` existing.
    
    - `scratchParent`,
      `removesTheDirectoryOnDispose`:
      **minor**,
      `TCV`.
      The added tests do not exercise the no-project fallback or its configured-cache and default-home branches.
      The disposal test also claims exceptional-scope coverage but never throws.
    
    - `scratchParent`:
      **minor**,
      `LOG`,
      `LG1`.
      The new function performs filesystem operations and chooses between project and user-cache locations without function-tagged entry or decision logging.
      Logging the resulting directory in its caller does not record this function’s selection and fallback reasons.
    
    No confirmed complexity finding is established by the supplied diff:
    `findNodeModulesUp`’s implementation and the allocation callers are not shown,
    so their aggregate traversal cost remains unverified.

    `````
- Skipped:
  85a07ca37f50 docs(runbook):
  the crates.io bootstrap token for monochromatic-mpm-shadow is revoked and logged out (no code files)
- Date:
  2026-10-29T14:19:46.878Z
  Subject:
  2972f3c6a06f fix(*):
  production tools keep their working directories under node_modules/.monochromatic (Refs #1810)
  Input size:
  15 code files,
  631 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  240.0 s
  Named superlinear cost:
  yes
  Findings kept:
    `````text
    I’ll check the scratch-directory changes for isolation,
    `````
  permissions,
  and repeated traversal costs.
    - `scratchParent` (all six copies):
      **major**,
      `RG2`.
      Each invocation searches the ancestor chain through `findNodeModulesUp`,
      and callers repeat this per command,
      package,
      or shard,
      including twice per `runOneShard`.
      Across r operations and ancestry depth d,
      this introduces O(r × d) traversal work rather than a single linear traversal of the shared ancestry.
    
    - `scratchParent` (all six copies):
      **major**,
      `ATH`.
      Scratch is placed under `node_modules` or the cache directory instead of the prescribed `${HOME}/temp/agent`.
      Recursive `mkdir` also does not establish the required `0700` permissions,
      so the parent can retain group/other access bits rejected by trust checks.
    
    - `runConfiguredGhCommand`:
      **major**.
      When a checkout contains its `node_modules`,
      the new default working directory remains inside that checkout.
      This defeats the stated purpose of keeping `gh` outside ambient repository context and exposes command execution to repository discovery.
    
    - `scratchParent` (all six copies):
      **minor**.
      Outside a project,
      an empty or relative `XDG_CACHE_HOME` produces a relative result because `??` only handles nullish values.
      This contradicts the documented absolute-path contract and makes scratch placement depend on the current working directory.
    
    - `scratchParent` (all six copies):
      **minor**,
      `LOG`,
      `LG1`.
      These new filesystem boundaries have no tagged module/function logger and log neither the project/cache branch nor the asynchronous directory-creation lifecycle.
      Failures therefore lack the diagnostic context required by the logging standards.
    
    - `scratchParent` (all six copies):
      **nit**,
      `ST2`.
      The new modules contain no `//region`/`//endregion` markers explaining their logical sections.
    
    No syntax-boundary encoding or dependency-version findings in the supplied diff.

- Date:
  2026-10-29T14:21:08.802Z
  Subject:
  74c1091ed5a9 feat(mutation-test-cli):
  build the runtime image on the CachyOS image the host CPU runs
  Input size:
  1 code files,
  135 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  66.3 s
  Named superlinear cost:
  partial (cost gestured as unverified,
  not named)
  Findings kept:
    `````text
    - `ensureRuntimeImage` — **major**,
      `BXD`:
      Passing `alpineVariant: false` explicitly rejects non-x86-64 hosts,
      as the new `NoAlpineVariantError` documentation confirms.
      The standard requires Alpine on those hosts;
      this rejects the operation before either cached-image reuse or image building.
    
    No dependency-specifier or lockfile-consistency findings in the supplied diff.
    Transitive complexity remains unverified because the implementations of `resolveBaseImage` and the existing file-walking helpers are not supplied.

    `````
- Date:
  2026-10-29T14:21:31.560Z
  Subject:
  27e022ca3a5a fix(pi-plugin):
  bash-poke and morph-compact keep their scratch off /tmp (Refs #1810)
  Input size:
  5 code files,
  283 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  199.2 s
  Named superlinear cost:
  partial (cost gestured as unverified,
  not named)
  Findings kept:
    `````text
    - `createOneShotSocketServer`:
      major.
      Requiring `XDG_RUNTIME_DIR` removes the socket fallback in Unix environments where that variable is unset,
      including typical macOS shells.
      The function now throws before creating a server,
      even when the previous temporary-directory location would have worked.
    
    - `scratchParent` (both new files):
      minor.
      When no ancestor `node_modules` exists and `XDG_CACHE_HOME` is an empty string,
      `??` does not select the home-directory fallback.
      The resulting scratch path is relative to the working directory,
      contradicting the documented absolute-path contract and potentially failing in a read-only working directory.
    
    - `createOneShotSocketServer`:
      minor,
      `PP4`.
      The new missing-runtime-directory branch throws a built-in `Error`,
      whereas the standard requires custom error classes for failures.
    
    - `scratchParent` (both new files):
      minor,
      `LOG`,
      `LG1`.
      These new asynchronous filesystem boundaries have no tagged logger or entry,
      branch,
      and lifecycle logging.
      Scratch-location selection and directory-creation failures therefore lack the function-level diagnostic context required by these standards.
    
    - `scratchParent` (both new files):
      nit,
      `AD4`.
      The fallback expression nests `homedir()` inside two `join()` calls on one line,
      exceeding the allowance for nested calls without splitting them across lines.
      The TSDoc example likewise nests three calls on one line.
    
    - Complexity and stack safety:
      no violation is established from the supplied hunks.
      `findNodeModulesUp`’s implementation is not included,
      so its traversal cost,
      including repeated path-processing work across the helper boundary,
      remains unverified.

    `````
- Skipped:
  730c3ad83576 docs(handover):
  scope the thinking-text rule and record the ratio helper (no code files)
- Skipped:
  5808e73fb906 docs(handover):
  apply semantic line breaks the gate wanted (no code files)
- Date:
  2026-10-29T14:23:05.436Z
  Subject:
  ee9a4848c97e test(file-enforcer/core):
  assert plan and generator invariants instead of hard-coded tool counts
  Input size:
  2 code files,
  94 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  58.5 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    - `fn` in test `gives every tool exactly one request shape: a floating track, a pin or a channel`:
      **nit**,
      `WR5`.
      The new comment refers to “the next two tests,”
      violating the prohibition on
      relative-position references in comments.
      Its meaning depends on test ordering rather than named references.
    
    No correctness or complexity findings in the supplied diff.

    `````
- Date:
  2026-10-29T14:23:17.011Z
  Subject:
  3659c961a1f9 fix(file-enforcer/core):
  hand cargo binstall a GitHub token so an exhausted API quota cannot silently trigger a source build
  Input size:
  5 code files,
  216 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  227.2 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    No substantiated findings in the supplied diff for correctness,
    `````
  security,
  unbounded-input complexity (RG2),
  or coding-standard compliance.
    
    The updated tool-count and generated-header assertions cannot be validated without the unchanged production tool plan or test results.

- Date:
  2026-10-29T14:24:14.657Z
  Subject:
  64046360d2de refactor(*):
  pause pi-plugin/morph-compact by moving it to package-paused (Refs #1810)
  Input size:
  54 code files,
  233 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  57.4 s
  Named superlinear cost:
  partial (cost gestured as unverified,
  not named)
  Findings kept:
    `````text
    No substantiated findings in the supplied diff,
    `````
  including the shown documentation and configuration edits.
    
    No function-level or complexity findings:
    every TypeScript entry is an unchanged rename.
    The rename metadata does not expose function bodies or remaining consumers,
    so existing algorithmic complexity and completeness of the relocation remain unverified.

- Skipped:
  45085249f499 docs(handover):
  record the committed TypeScript layers,
  the user decisions and the morph-compact move (Refs #1810) (no code files)
- Date:
  2026-10-29T14:25:01.294Z
  Subject:
  877458c05423 fix(webapp-forge/server):
  paused tests take fixture directories from scratchRoot,
  not tmpdir (Refs #1810)
  Input size:
  3 code files,
  94 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  52.3 s
  Named superlinear cost:
  partial (cost gestured as unverified,
  not named)
  Findings kept:
    `````text
    No findings in the supplied diff.
    
    No complexity violations are evident at the changed call sites.
    The implementation of `scratchRoot()` is not provided,
    so its transitive cost and scratch-directory initialization cannot be verified.

    `````
- Skipped:
  a9584052a657 docs(handover):
  record the mutation-test work-tree copy failure and its fix in flight (no code files)
- Date:
  2026-10-29T14:27:03.012Z
  Subject:
  d4988f4c9b57 feat(test-harness-core-expect):
  two-size ratio linear-growth assertion
  Input size:
  3 code files,
  561 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  332.7 s
  Named superlinear cost:
  yes
  Findings kept:
    `````text
    ### `expectLinearTimeGrowth`: major, `RG2`
    
    The default bound accepts superlinear work such as `n log n` and `n^1.2` even with perfect timings.
    Measurements at two finite sizes cannot establish the advertised O(n) proof over unbounded input.
    
    ### `measureTwoSizeRatio`: major, `RG2`
    
    Each caller-controlled `repeats` iteration invokes `grow` and,
    through `timeOne`,
    `run` on size-dependent data.
    Even with linear callbacks,
    total cost is Θ(repeats × size),
    becoming quadratic when `repeats` scales with `size`;
    the helper calls cannot be counted as constant-time work.
    
    ### `measureTwoSizeRatio`: major, `RG2`
    
    Spreading the unbounded round arrays into `Math.min` and `Math.max` requires argument-stack space proportional to `repeats`.
    Sufficiently large round counts can exceed the engine’s argument limit,
    violating the O(1)-stack requirement.
    
    ### `expectLinearTimeGrowth`: major, `PP8`
    
    With `repeats: 0`,
    both minima are `Infinity`,
    producing a `NaN` ratio that passes because `NaN > maxRatio` is false.
    A `NaN` maximum similarly disables rejection,
    silently accepting an unusable verdict.
    
    ### `measureTwoSizeRatio`: major, `QIV`
    
    Clamping the denominator to `MIN_BASE_MS` suppresses apparent growth when the base measurement is below that floor.
    A genuine fourfold increase can consequently produce a passing ratio,
    treating insufficient timer resolution as evidence of linearity.
    
    ### `expectLinearTimeGrowth`: major, `QNB`
    
    Although measurements include best and worst timings,
    the assertion never considers their spread.
    Independently selected minima determine the verdict even when within-size variation exceeds the apparent growth difference,
    allowing noise to determine success or failure.
    
    ### `timeOne`: major, `QIV`
    
    The `run` signature accepts asynchronous callbacks,
    but their returned promises are ignored.
    Work after an `await` is excluded from timing,
    and rejection can escape independently of the assertion,
    so the accepted measurement need not cover the tested work.
    
    ### `measureTwoSizeRatio`, `expectLinearTimeGrowth`: minor, `LOG`, `LG1`
    
    Neither exported function creates a tagged logger or logs its entry and decision branches.
    The assertion-failure branch therefore also lacks the mandated diagnostic logging.
    
    ### `TwoSizeRatioError.constructor`: nit, `WR5`
    
    The comment “each named once for the message below” identifies its destination by relative position rather than by symbol or purpose.
    
    No blocker or security-boundary findings.

    `````
- Skipped:
  601557b4f743 docs(handover):
  record the passing IDE release dry run on the Ubuntu 24.04 release images (no code files)
- Skipped:
  01265da26bcc docs(troubleshooting):
  note the font package the CachyOS Windows GNU image needs under Wine (no code files)
- Skipped:
  faf93deb19ae docs(planning):
  correct the concurrency test diagnosis to load sensitivity and record the final lint,
  type and open-test state (no code files)
- Skipped:
  c89e17e51a79 docs(music-player-desktop-app):
  record that the CachyOS container build of the run task needs a newer glibc than a Fedora 44 host has (no code files)
- Date:
  2026-10-29T14:33:11.443Z
  Subject:
  d488e9e0de17 fix(test-harness-core-expect):
  fold min and max,
  drop the O(n) proof claim
  Input size:
  1 code files,
  155 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  181.6 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    - `smallest`,
      `largest` | **minor** | `LOG`,
      `LG1`:
      Neither new helper logs its entry or branches or establishes function-name-tagged diagnostic context,
      despite the standards requiring logging and tagging at function boundaries.
    
    No correctness or new complexity findings in the supplied changes.
    Each added helper performs an iterative linear scan with constant stack,
    and the fixed number of calls does not introduce superlinear work.
    The revised documentation appropriately distinguishes finite-size growth measurements from asymptotic proof.

    `````
- Date:
  2026-10-29T14:33:13.749Z
  Subject:
  cab3f212d7e8 fix(config-oxlint):
  point the jest-matcher exemption at the moved package
  Input size:
  1 code files,
  22 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  35.0 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    No findings.
    `````
  The provided diff shows no `AGENTS.md` violations,
  correctness issues,
  or complexity regressions;
  it only updates override globs and documents the package move.

- Date:
  2026-10-29T14:33:22.931Z
  Subject:
  166bd64f1a1e feat(oxlint-plugin/no-restricted-syntax):
  add no-system-tmpdir,
  which bans the system temporary directory (Refs #1810)
  Input size:
  4 code files,
  174 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  102.9 s
  Named superlinear cost:
  no
  Findings kept:
    `````text
    - `CallExpression`:
      **major**,
      no applicable tag.
      The visitor reports every noncomputed `.tmpdir()` call without checking its receiver.
      An unrelated object whose `tmpdir()` returns an approved scratch directory receives a diagnostic falsely asserting that it uses the system temporary directory.
    
    - `CallExpression`:
      **major**,
      no applicable tag.
      `import * as os from 'node:os'; os['tmpdir']()` evades both visitors:
      the import is not an `ImportSpecifier`,
      and the call is discarded because its member access is computed.
      This leaves an actual system-temporary-directory call unenforced.
    
    - `createOnce`:
      **minor**,
      `TCV`.
      The added fixture exercises only a named import from `node:os`;
      it does not exercise the `CallExpression` visitor or valid cases that should remain unreported.
      The namespace/default-import,
      computed-access,
      and alternate-module branches therefore lack coverage in the candidate diff.
    
    - `createOnce`,
      `ImportDeclaration`,
      `CallExpression`:
      **minor**,
      `LOG`,
      `LG1`.
      The new module and callback boundaries contain no tagged logging,
      and the visitors’ decisions are not logged.
      Calling `context.report` emits lint diagnostics but does not satisfy the required tagged decision logging.
    
    No complexity or stack-usage findings:
    import-specifier iterations cover disjoint lists,
    and each call visitor performs bounded work without invoking an unbounded helper.

    `````
- Skipped:
  b7a01a357e9c docs(AGENTS.md):
  add FDL,
  list find matches before deleting them because -delete implies -depth and disables -prune (no code files)
- Skipped:
  969474018ca3 chore(file-enforcer):
  regenerate mise.toml and the pnpr config for the mutation-test/cli package and the removed morph-compact plugin (no code files)
- Skipped:
  4eee423fda8f docs(planning):
  record the global lock regeneration,
  the FDL rule,
  and the mpm-shadow count fix by its owner (no code files)
- Date:
  2026-10-29T14:36:03.893Z
  Subject:
  d4988f4c9b57 feat(test-harness-core-expect):
  two-size ratio linear-growth assertion
  Input size:
  3 code files,
  561 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  158.1 s
  Named superlinear cost:
  unjudged
  Findings kept:
    - `expectLinearTimeGrowth`:
      **major**,
      `RG2`.
      The default 2.5 threshold admits superlinear growth such as `n log n`,
      even when the intended path is exercised.
      Passing measurements at two finite sizes cannot prove the advertised bound over unbounded input.
    
    - `timeOne`:
      **major**,
      `QIV`.
      The callback type permits asynchronous functions,
      but timing stops immediately after invocation without awaiting completion.
      Deferred work is excluded,
      allowing an asynchronous superlinear implementation to pass while its actual execution remains unmeasured.
    
    - `measureTwoSizeRatio` / `expectLinearTimeGrowth`:
      **major**,
      `QIV`.
      With `repeats: 0`,
      both round arrays are empty,
      producing an `Infinity / Infinity` ratio of `NaN`;
      because `NaN > maxRatio` is false,
      the assertion passes without any measured rounds.
      A `NaN` maximum ratio also bypasses rejection.
    
    - `measureTwoSizeRatio`:
      **major**,
      `RG2`.
      Spreading the round arrays into `Math.min` and `Math.max` creates argument lists proportional to caller-controlled `repeats`,
      violating the constant-stack requirement and potentially exceeding runtime argument limits.
      Iterative collection of the samples does not make these subsequent calls safe for unbounded counts.
    
    - `measureTwoSizeRatio` / `expectLinearTimeGrowth`:
      **major**,
      `QNB`,
      `QIV`.
      Although `worstMs` records variation,
      it never influences the verdict,
      so the assertion treats differences within run-to-run noise as meaningful.
      Flooring the denominator also turns below-resolution base measurements into ordinary ratios instead of identifying them as inconclusive.
    
    No additional findings on superlinear helper traversals,
    recursion,
    or accumulator rebuilding in the supplied implementation.

- Date:
  2026-10-29T14:36:04.058Z
  Subject:
  b7c1773fd104 test(mutation-test/cli):
  cover the source file list and rsync arguments,
  document the work-tree copy
  Input size:
  3 code files,
  634 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  195.1 s
  Named superlinear cost:
  unjudged
  Findings kept:
    - `git`:
      minor,
      `HOM`.
      The fixture commands inherit the real user’s home and Git configuration despite overriding hooks,
      signing,
      and identity.
      A global `core.excludesFile` can change which fixture files are ignored,
      making `listsSourceOnly` depend on user settings rather than an isolated environment.
    
    - `listingRejection`:
      minor,
      `LG2`.
      The catch returns the caught error without logging or rethrowing it.
      Returning it for an assertion uses the binding,
      but does not satisfy the rule’s explicit logging-or-rethrow requirement,
      including for expected failures.
    
    - `filesUnder`,
      `realCopy`,
      `listsSourceOnly`:
      minor,
      `AD4`.
      These functions introduce method chains such as `filter(...).map(...).toSorted()`,
      `map(...).join(...)`,
      and `split(...).slice(...)`.
      The standard requires nested function composition rather than chained transformations.
    
    - `retarget`,
      `filesUnder`,
      `git`,
      `put`,
      `listingRejection`:
      nit,
      `TD6`.
      Their new `@param` and `@returns` descriptions primarily restate argument contents or result shapes instead of explaining their purpose.
      `retarget` also includes the article “the” in its parameter description.
    
    No additional production-correctness,
    security,
    or unbounded-input complexity findings:
    `listSourceFiles` still splits its output once,
    and the new sorting operations are reached through controlled,
    fixed-size fixtures.

- Date:
  2026-10-29T14:36:24.229Z
  Subject:
  7da722af96f5 fix(*):
  standalone container and browser-test scripts keep scratch under node_modules/.monochromatic (Refs #1810)
  Input size:
  21 code files,
  662 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  157.7 s
  Named superlinear cost:
  unjudged
  Findings kept:
    - `makeScratchBase` (module initialization in all changed scripts) — **minor** — `ATH`:
      The new `scratchBase` constants place scratch under package-local `node_modules/.monochromatic`,
      whereas ATH requires `${HOME}/temp/agent`.
      Being gitignored does not satisfy that placement requirement.
    
    - `makeScratchBase` (module initialization in all changed scripts) — **minor** — `ATH`:
      `mkdirSync(..., { recursive: true })` does not enforce the required `0700` permissions,
      and existing directories retain their permissions.
      Private `mkdtemp` children do not establish the required permissions on the scratch base itself.
    
    No additional correctness or algorithmic-complexity findings are established by the supplied diff.

- Date:
  2026-10-29T14:36:40.675Z
  Subject:
  8b4374358e26 fix(mutation-test/cli):
  copy only git-listed source files into the shard work tree (unverified)
  Input size:
  6 code files,
  326 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  418.1 s
  Named superlinear cost:
  unjudged
  Findings kept:
    - `runOneShard` / `orchestrateRun`:
      **major**,
      `RG2`.
      Every shard launch UTF-8-encodes and writes the entire unbounded `sourceFileList`,
      including subsequent reshard launches.
      For S launches and B list bytes,
      this introduces O(S × B) list-staging work;
      computing the Git listing once does not make the overall processing linear.
    
    - `orchestrateRun`:
      **minor**,
      no directly applicable tag.
      The new comment claims that computing the file list once makes all shards see the same checkout state,
      but it freezes only path membership.
      Each shard still copies contents from the live source mount at launch,
      so concurrent edits can produce different source contents across shards.
    
    - `listSourceFiles` (module organization):
      **nit**,
      `ST2`.
      The new module contains no `//region` / `//endregion` markers for its logical sections,
      contrary to the required TypeScript section structure.
    
    - `listSourceFiles` (module header):
      **nit**,
      `TD2`.
      The opening `/** ... */` block documents the file and precedes imports rather than directly preceding a declaration.
      The standard reserves TSDoc blocks for declaration documentation.
    
    No security findings identified in the supplied diff.

- Date:
  2026-10-29T14:38:02.364Z
  Subject:
  7b077382d499 fix(*):
  fuzz,
  perf and harness fixtures take their directories from scratchRoot (Refs #1810)
  Input size:
  6 code files,
  136 diff lines
  Reviewer route:
  openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost:
  118.5 s
  Named superlinear cost:
  unjudged
  Findings kept:
    `````text
    No confirmed findings in the supplied diff.
    
    Cross-call complexity and scratch-directory safety remain unverified because `scratchRoot()` is not shown. Fixture-producer path consistency and the new imports’ dependency/export resolution also cannot be established from the supplied files; these are review limits, not confirmed violations.
    `````

- Date: 2026-10-29T14:40:19.291Z
  Subject: 8b773b22ac5e `fix(ide/app): inspection scripts and the runtime-publish test keep scratch under node_modules/.monochromatic (Refs #1810)`
  Input size: 3 code files, 70 diff lines
  Reviewer route: openai/gpt-6.1-sol via pi --print --no-session --no-tools --no-skills --no-themes --no-context-files
  Wall-clock cost: 88.4 s
  Named superlinear cost: unjudged
  Findings kept:
    `````text
    - `sweep`: **minor**, `PP4`. The new `XDG_RUNTIME_DIR` guard throws a plain `Error`, whereas the standard requires custom error classes and an `@throws` contract. The new failure condition is therefore neither distinctly typed nor documented as required.
    
    No new unbounded-input complexity findings in the shown diff.
    `````

- Skipped:
  2625b5785f10 (no code files):
  `feat(mise): shadow entries arrive in the gate's own markdown shape`
