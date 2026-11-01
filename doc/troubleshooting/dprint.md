# dprint workspace integration and formatting conflicts

This file documents four independent dprint operational issues that
share the same package surface.
 Each is treated as its own bug section
with symptom,
 root cause,
 verification,
 workarounds with tradeoffs,
what does not work,
 and a 5-constraint upstream audit.

---

## Bug 1: dprint installed as a workspace dev dependency causes 5+ second startup overhead

### Symptom

`dprint check` or `dprint fmt` invoked via `pnpm exec dprint` or
through a workspace task that depends on the npm-installed binary
spends multiple seconds on startup before producing any output,
 even
for a single small file.
 The same `dprint` binary invoked directly
(e.g. from a globally installed copy) returns in under 100ms.

### Root cause

The npm distribution of dprint is a Node wrapper that re-execs the
real native dprint binary located inside the package.
 Each invocation
pays the full Node startup cost (interpreter init,
 module resolution,
`package.json` discovery) before the native binary even starts.
 In a
pnpm-isolated workspace the resolution walk traverses additional
symlink layers,
 compounding the cost.

This is a packaging choice (Node wrapper around a native binary),
 not
a dprint-internal bug.
 The native binary itself is fast;
 the wrapper
is the slow part.

### Verification

Time the two invocation paths against a stable target:

```bash
# Node-wrapped path (slow):
time pnpm exec dprint --version

# Direct binary path (fast):
time "$(mise which dprint)" --version
```

On this workspace's hardware,
 the first form measures around 2.5s
cold-cache and 1.0s warm;
 the second measures sub-100ms regardless of
cache state.

### Verified workaround

Remove dprint from the root `package.json` `devDependencies` and let
mise (via `.prototools` / `mise.toml`) manage the version instead.
Workspace tasks invoke the mise-managed binary directly.
 The npm
package is no longer installed;
 the Node wrapper is no longer in the
critical path.

Tradeoff:
 every developer must have mise installed (already a
workspace requirement).
 The pnpm dependency tree no longer pins
dprint's version explicitly;
 mise pins it via `.tool-versions` /
`mise.toml`.
 Drift between developers is bounded by mise's lockfile
behaviour.

The `config-dprint` workspace package therefore does **not** declare
`dprint` as a peer dependency:
 doing so would re-introduce the npm
install path consumers would have to depend on.

### What does not work

- `corepack`-style auto-resolution:
   dprint is not a Node package
  manager and is not in corepack's supported list.
   The native binary
  cannot be invoked through corepack.
- Configuring pnpm to skip postinstall scripts for dprint:
   the slow
  part is the Node wrapper at *invocation* time,
   not at install time.
  Postinstall behaviour is irrelevant.

### Why we do not file this upstream

1. **Is it really upstream's fault?
   ** Partly.
    The Node wrapper exists
   so dprint can be installed via npm;
    it is by design.
    The slowness
   is a consequence of the design,
    not a defect.
2. **Can upstream fix it?
   ** Not without removing npm installation,
   which is a major distribution change.
3. **Are they supporting this use case?
   ** Yes;
    the wrapper is
   documented and works correctly,
    just slowly.
4. **Will they likely fix it?
   ** No movement in that direction in
   recent dprint releases.
    The shape is stable.
5. **Have we prototyped a minimal fix?
   ** Our fix is to bypass the
   wrapper entirely (use the native binary through mise).
    That is a
   downstream choice,
    not a fix dprint can ship.

Decision:
 no upstream report.
 The native binary path solves it locally.

---

## Bug 2: dprint VS Code extension cannot find dprint in WSL with pnpm-isolated installs

### Symptom

The dprint VS Code extension,
 connected to a WSL workspace,
 fails on
startup with:

```text
[Error] dprint client: couldn't create connection to server.
Launching server using command dprint failed. Error: spawn dprint ENOENT
```

Format-on-save through the extension stops working;
 CLI invocations
still succeed.

### Root cause

The dprint VS Code extension spawns `dprint` from the user's PATH at
extension activation.
 In a pnpm-isolated workspace,
 dprint is installed
under `node_modules/.bin/`,
 not in any directory that ends up on PATH
under WSL's typical shell rc files (the extension launches the server
before any login shell runs `.profile`).

The extension exposes a `dprint.path` setting precisely for this case.
Default-PATH resolution is the failure mode.

### Verification

```bash
# Inside the WSL shell where VS Code is connected:
which dprint
# Empty output: PATH does not include node_modules/.bin

ls node_modules/.bin/dprint
# Symlink exists; this is the path the extension needs to be told about.
```

### Verified workaround

Point the extension at the workspace-local install.

For `.code-workspace` files:

```json
{
  "settings": {
    "dprint.path": "./node_modules/.bin/dprint"
  }
}
```

For `.vscode/settings.json`:

```json
{
  "dprint.path": "./node_modules/.bin/dprint"
}
```

Reload the VS Code window after editing.
 The setting is workspace-scoped;
it does not leak into other projects.

Tradeoff:
 the path is hard-coded as relative.
 If the workspace root
moves (rare),
 the setting needs to follow.
 The setting is committed,
so every team member benefits without further configuration.

### Alternative workaround

Install dprint globally inside WSL so `which dprint` resolves:

```bash
npm install -g dprint
```

Tradeoff:
 maintains a separate global install per developer;
 version
drift between the global copy and the project-pinned copy can cause
formatting discrepancies.
 Prefer the workspace-relative setting unless
multiple projects on the same machine need different setups.

### What does not work

- Adding `node_modules/.bin` to PATH via `.bashrc` / `.zshrc`:
   the
  extension launches its server before login-shell rc files run,
   so
  the PATH it sees is the system default,
   not the user's interactive
  PATH.
- Symlinking `node_modules/.bin/dprint` to `/usr/local/bin/dprint`:
  works for that one project but breaks every other workspace that
  pins a different version.

### Why we do not file this upstream

1. **Is it really upstream's fault?
   ** Borderline.
    The extension has a
   `dprint.path` setting precisely because PATH-resolution can fail in
   workspace-managed setups;
    that is upstream acknowledgement that the
   failure mode is expected.
2. **Can upstream fix it?
   ** They could auto-detect
   `node_modules/.bin/dprint` when present.
    Probably worth a feature
   request,
    not a bug report.
3. **Are they supporting this use case?
   ** Documented escape hatch
   exists;
    the use case is supported.
4. **Will they likely fix it?
   ** Unknown;
    not a critical defect.
5. **Have we prototyped a minimal fix?
   ** No.

Decision:
 no upstream report.
 The setting works;
 documenting it here
is sufficient.

---

## Bug 3: dprint emits TypeScript baseUrl warnings on non-relative paths

### Symptom

Running dprint produces warnings such as:

```text
warn: Non-relative path "package/config/oxlint/src/index.ts" is not allowed when "baseUrl" is not set (did you forget a leading "./"?)
```

Formatting still completes;
 the warnings are noise but pollute CI logs
and obscure real diagnostics.

### Root cause

dprint's TypeScript plugin loads the workspace `tsconfig.json` to
resolve module paths.
 When `baseUrl` is unset,
 non-relative module
specifiers (paths that do not start with `.` or `..`) cannot be
resolved against a known root,
 and the plugin emits a warning per
file.

The cross-reference doc
([`TROUBLESHOOTING.typescript.md`](typescript.md#typescript-path-warnings-with-dprint))
covers the canonical fix:
 set `baseUrl: "./"` in the root
`tsconfig.json`.

### Verified workaround

See
[`TROUBLESHOOTING.typescript.md`](typescript.md#typescript-path-warnings-with-dprint).

Tradeoff:
 setting `baseUrl` changes how TypeScript resolves modules in
the rest of the workspace.
 The setting is opt-in for that reason;
 the
workspace has accepted it because every package's `tsconfig.json`
inherits the same base and behaves consistently.

### What does not work

- Adding `"./"` prefix to every import in the codebase:
   relative
  prefixes cure the warning but lose the monorepo's package-name
  imports that consumers rely on.
- Ignoring the warnings via dprint config:
   there is no per-rule
  suppression for this diagnostic;
   it surfaces as an unconditional
  log entry.

### Why we do not file this upstream

The warning correctly identifies a TypeScript configuration ambiguity.
Suppressing it would silently mask real misconfigurations;
 dprint's
diagnostic is appropriate.

1. **Is it really upstream's fault?
   ** No.
2. **Can upstream fix it?
   ** Nothing to fix.
3. **Are they supporting this use case?
   ** Yes;
    the warning explicitly
   names the missing setting.
4. **Will they likely fix it?
   ** N/A.
5. **Have we prototyped a minimal fix?
   ** N/A.

Decision:
 no upstream report.
 Fix is on our side via the tsconfig
`baseUrl` setting.

---

## Bug 4: Learning Rust HTML conflicts with repository formatting policy

### Symptom

The learning Rust workspace contains 5 standalone HTML pages with repeated inline CSS.
`package/learning/rust/NOTES.md` requires plain `file:///` pages,
repeated inline styles,
and no presentation beyond approved foreground and background colors.
The owner clarified these formatting requirements:

- every HTML file remains owned by dprint;
- CSS contains 5 to 20 nonblank lines;
- prose may reflow but may not split a tight grammatical unit;
- direct affected colors use numeric lightness,
  `none` for zero chroma,
  and `none` for zero hue;
- nonzero hue uses `deg`.

The desired colors are:

```css
oklch(0.1 none none)
oklch(0.9 none none)
```

The affected paths are:

```text
package/learning/rust/lessons/index.html
package/learning/rust/lessons/0001-whats-different-about-this-book.html
package/learning/rust/reference/index.html
package/learning/rust/reference/reading-loop.html
package/learning/rust/reference/aquascope-decoder.html
```

The incumbent configuration reports all 5 files as differently formatted and produces 40 Stylelint diagnostics
on them.

### Source-level root cause

`markup_fmt` 0.23.1 sends every nonempty `<style>` element to `format_style` in
`markup_fmt/src/printer.rs:701-737` at
[commit `2d902a5a1`](https://github.com/g-plane/markup_fmt/commit/2d902a5a1af8d7f07e06f189ae1e3132ea344e74).

Its dprint adapter synthesizes an embedded path such as `index.html#.css` in
`dprint_plugin/src/lib.rs:54-90` at the same commit.
Dprint then routes the CSS request to a registered CSS plugin.

That delegation is useful.
It is the seam where a repository-owned formatter can replace Malva without replacing markup formatting or
dprint's unrelated format coverage.

The prose conflict is separate.
At root width 90,
`markup_fmt` wraps text by available width rather than grammatical structure.

### Width and ignore probes

New text-node break contexts measured by width were:

- width 160:
  94;
- width 240:
  30;
- width 320:
  11;
- width 500:
  3;
- width 600 on the current corpus:
  0.

Width 600 produced lines up to 593 characters.
Adversarial fixtures still split `original variable`,
`allocator deallocates`,
and `you reach` at width 600 and 1000.

`whitespaceSensitivity: 'strict'` was not a preservation mode.
It produced 357,
100,
and 49 new contexts at widths 90,
160,
and 200.

The configured directives are:

- node:
  `<!-- markup-fmt-ignore -->`;
- file:
  `<!-- dprint-ignore-file -->`.

Node ignores preserved prose but also preserved inconsistent subtree indentation.
File ignores and root exclusions work technically,
but the owner disqualified every formatter exemption because it makes future formatter coverage untrustworthy.

### Prose-preserving formatter probe

A disposable `markup_fmt` patch added opt-in `preserveTextWrapping` behavior.
It keeps each authored nonempty text line as a formatter boundary,
normalizes repeated intra-line ASCII whitespace,
and continues formatting structure,
attributes,
quotes,
and embedded languages.
The default remains the incumbent width-based behavior.

The default-off library tests passed and a local dprint Wasm build succeeded.
Across all 5 pages the combined run produced:

- no exclusion or ignore directive;
- no introduced text-node break context;
- original and formatted break counts of 0,
  16,
  0,
  0,
  and 259;
- 16 nonblank CSS lines per page;
- no second-pass change.

A grammar fixture kept `I ate a chicken`,
`original variable`,
`allocator deallocates`,
and `you reach` intact at width 90 when each phrase was authored on one line.

This establishes a feature direction,
not an upstream defect.

### Repository-owned CSS host probe

A strict structural formatter over `package/css/core-edit/` was connected to the markup host seam through
`dprint-plugin-exec` 0.7.3.
The markup adapter had to stop passing Malva-specific override keys to exec CSS requests;
exec correctly rejected unknown configuration.

The hardened formatter probe covers:

- compact and expanded blocks;
- comments and directive comments;
- blank-line groups;
- strings containing CSS punctuation;
- selector lists and combinators;
- nested rules;
- declarations;
- statement and block at-rules;
- malformed CSS.

The dprint-discovered corpus contained 36 standalone CSS files and 9 embedded regions,
for 45 regions total.
The probe reported no parse failure,
second-pass difference,
semantic-token mismatch,
comment change,
or surrounding-host change.

A dprint JSON-RPC probe formatted unsaved standalone CSS and unsaved HTML with embedded CSS through `dprint lsp`.
It returned edits for both and emitted no stderr.
Open
[`dprint-plugin-exec#34`](https://github.com/dprint/dprint-plugin-exec/issues/34)
does not reproduce under dprint 0.55.2,
exec 0.7.3,
and the tested associations.

A 5-run `--incremental=false` benchmark on the 5 pages measured:

- prose-preserving markup alone:
  344.2 ms mean;
- markup plus exec CSS delegation:
  451.2 ms mean;
- mean difference:
  107.0 ms per invocation.

Exec is suitable for transition validation.
A dedicated persistent dprint process adapter is the preferred final host adapter because it can avoid a child
command per request and own cache invalidation directly.

### Failure behavior

Malformed standalone and embedded CSS made dprint fail with the strict tokenizer diagnostic.
SHA-256 hashes before and after were identical for both files.
No partial format was written.

The production adapter should turn parser exceptions into concise path and range diagnostics,
while retaining this fail-closed behavior.

### Stylelint transition and owned checks

The active Stylelint surface contains 82 rules,
not only the 8 currently producing diagnostics.
The revised responsibility ledger assigns:

- 27 to the formatter;
- 16 to repository policy;
- 38 to correctness checks;
- 1 deliberate drop,
  `no-descending-specificity`.

All 82 pinned rule directories contain tests.
45 implementations import CSS reference data or dedicated grammar or selector parsers.
Removing Stylelint before those responsibilities have owned fixtures would discard coverage.

The owned policy probe reproduced the incumbent semantic counts:

- 16 disallowed functions;
- 62 disallowed properties;
- 67 disallowed units.

It also rejected the missed `height: 10em` case and reported 20 preferred channel changes on the current learning
pages.
An HTML fixture proved that diagnostic offsets map to the host file and false `<style>` text in a comment or
attribute is ignored.

A package profile independently checks:

- exactly one real style region;
- the approved stylesheet structure and values;
- the 5-to-20 nonblank-line budget.

After preferred channels were applied in the disposable worktree,
all 5 pages passed the package profile with 16 nonblank lines.
A transitional Stylelint override also passed,
Stylelint fix made no change,
and the next owned dprint check passed.

### Color semantics

[CSS Color 4 section 4.4](https://www.w3.org/TR/css-color-4/#missing)
says `none` behaves as zero outside interpolation,
but may borrow the other color's corresponding component during interpolation.
Do not describe `none` and zero as universally interchangeable.

A Chromium 149 probe rendered direct missing and zero component comparisons to equal RGBA bytes.
[Mozilla bug 1813481](https://bugzilla.mozilla.org/show_bug.cgi?id=1813481)
records `none` color components as fixed in Firefox 113,
which predates the repository's Firefox ESR 140 baseline.

### Separate package-content defect

`package/learning/rust/NOTES.md` requires this element in every page:

```html
<meta name="color-scheme" content="light dark">
```

Only `reference/aquascope-decoder.html` currently contains it.
The other 4 pages need correction regardless of the formatting decision.

### What does not work

- Dprint exclusions,
  file ignores,
  and node ignores violate the owner's formatter-trust requirement.
- Width 600 avoids current-corpus breaks by producing long lines and still fails adversarial grammar.
- `whitespaceSensitivity: 'strict'` does not preserve prose wrapping.
- A Malva ignore protects CSS but does not solve prose.
- Removing Stylelint layout rules does not stop Malva from formatting embedded CSS.
- The incumbent generic color rules cannot require `none` for zero channels.
- Replacing dprint across unrelated formats expands far beyond the measured issue.
- Removing Stylelint now would lose unimplemented correctness and editor diagnostics.
- Extracting a shared stylesheet violates the package contract.

### Recommended resolution

Do not exempt any affected page from dprint.
After owner acceptance:

1. Land opt-in prose-preserving markup behavior with the default disabled.
2. Route CSS host requests to the repository-owned formatter.
3. Add the package-local exact stylesheet and line-budget checks.
4. Change the affected colors and `NOTES.md` to preferred `none` channels.
5. Add the missing color-scheme meta element to the other 4 pages.
6. Use a transitional Stylelint override that converges with the owned output.
7. Run the remaining Stylelint responsibilities in shadow while owned fixtures and editor clients land.
8. Remove Stylelint only after responsibility,
   suppression,
   CLI,
   and editor parity gates pass.

The full option analysis,
82-rule ledger,
and ranking are in
`doc/planning/learning-rust-formatting-boundary.md`.

### Scoped-formatting constraint

The root `format` task is not a safe scoped substitute during concurrent work.
A single HTML path reaches dprint,
but nested Stylelint,
Oxlint,
and Markdown formatter tasks still run globally.
Use an explicit scoped task or disposable worktree,
then copy only reviewed files.

### Verification requirements

A completed implementation must verify:

- all 5 pages remain in dprint discovery;
- dprint CLI and LSP both invoke owned CSS formatting;
- no grammatical break context is introduced;
- a second formatting pass changes nothing;
- malformed embedded CSS leaves its host file unchanged;
- each style has 5 to 20 nonblank lines;
- package policy rejects changed colors and extra presentation;
- CSS diagnostics map to exact host ranges;
- Stylelint and owned shadow results are reconciled before removal;
- all 5 pages contain the color-scheme meta element;
- light and dark browser modes compute the approved colors;
- pages and relative links still work through `file:///`;
- no unrelated file changes through a global formatter.

### Upstream filing decision

Tracker searches found related `markup_fmt` layout requests,
including issues
[g-plane/markup_fmt#16](https://github.com/g-plane/markup_fmt/issues/16)
and
[g-plane/markup_fmt#242](https://github.com/g-plane/markup_fmt/issues/242),
but no existing preserve-authored-text-lines request.

1. **Is it really upstream's fault?**
   No established defect exists.
   The formatter performs width-based wrapping as designed.
2. **Can upstream fix it?**
   Upstream could accept an opt-in preservation feature,
   but cannot choose this package's policy.
3. **Are they supporting this use case?**
   They support canonical formatting and ignore controls,
   not grammatical prose wrapping.
4. **Would the repository welcome our contribution?**
   Unknown until a proposal is discussed.
5. **Will they likely fix it?**
   Unknown;
   no issue has been filed.
6. **Have we prototyped a compatible minimal fix?**
   Yes.
   The default-off patch passed the library tests and dprint integration probe.

Decision:
do not file a defect.
If the owner accepts the direction,
prepare an upstream feature proposal or maintain the local adapter deliberately.
