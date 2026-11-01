# Issue 401 formatting investigation handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/dev-script/file-enforcer` is now `package/file-enforcer/core`;
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`;
> `package/module/toml-edit` is now `package/toml-edit/core`;
> `package/module/css-edit` is now `package/css/core-edit`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/wg-quicker-exempt` is now `package/wg/cli-quicker-exempt`.

## Status

Investigation and ranking are complete.
No implementation decision has been accepted.
GitHub issue [Aquaticat/Monochromatic#401](https://github.com/Aquaticat/Monochromatic/issues/401) is open.

The 2026-07-30 restart checkpoint is commit `81218de1e`.
Immediately after that commit,
`git status --short` showed concurrent unrelated modifications across `.agents/`,
`doc/`,
and several package directories,
plus the earlier `package/cli/wg-quicker-exempt/` work.
Do not alter,
stage,
or commit any unrelated dirty path.

## User requirements

- Investigate issue 401 more thoroughly than the earlier Claude analysis.
- Back suggestions with measurements,
  pinned upstream source,
  primary documentation,
  or executable probes.
- Update issue 401 incrementally as evidence develops.
- Reject prose breaks that split a tight grammatical unit.
  `I ate\na chicken.` is unacceptable.
  List-like breaks such as `I ate a chicken,\na pig,\nand a cow.` are acceptable.
  Exact authored prose line preservation is not required.
- Keep the learning CSS between five and twenty nonblank lines and add no presentation rules beyond the approved
  colors.
  CSS layout within that explicit line budget is not prescribed.
- For affected `oklch()` values,
  prefer lightness as `<number>`,
  chroma as `none` when zero or `<number>` otherwise,
  and hue as `none` when zero or `<number>deg` otherwise.
  Treat repository-wide scope for this syntax as unresolved unless the user broadens it.
- Consider repository-owned formatting and linting logic.
- Do not exclude the learning HTML from dprint.
  A formatter exemption would make formatter coverage untrustworthy and undermine the repository formatter over
  time.
- Treat replacing dprint as only one family of solutions.
- Treat replacing Stylelint as an intended direction because its dependency footprint is already considered
  excessive.
- Do not evaluate another existing CSS formatter or linter as the replacement.
  The current direction is repository-owned CSS tooling.
- Be comprehensive vertically,
  from immediate symptom mitigation through durable architecture.
- Be comprehensive horizontally,
  across materially different policy,
  configuration,
  workflow,
  assertion,
  generation,
  custom-tool,
  replacement,
  and upstream approaches.
- Maintain this handover after major evidence,
  corrections,
  decisions,
  issue updates,
  and before compaction or completion.
- Keep this policy package-local.
  Do not propose an `AGENTS.md` rule for it.

This is an investigation and proposal task.
Do not implement a selected product change until the user accepts or delegates the decision.
Documentation and disposable probes are authorized.

## 2026-07-30 restart checkpoint

A repository-owned CSS direction has now been validated in disposable probes,
but it is not production-ready and has not been selected for implementation.

### Combined dprint fixed point

The disposable prose-preserving `markup_fmt` Wasm plugin now delegates embedded CSS to a repository-owned
stdin/stdout formatter through `dprint-plugin-exec` 0.7.3.
The adapter had to avoid passing Malva-specific host override keys for synthetic CSS requests;
otherwise exec rejected the override as invalid configuration.
The patched adapter build completed successfully.

A dprint check using `dprint.issue401-owned-css-exec.json` passed without output for all five affected HTML files
and one standalone CSS file.
Earlier two-pass formatting of the five pages established:

- no new text-node line-break contexts;
- preservation of the existing text-node breaks;
- sixteen nonblank CSS lines in every page;
- no second-pass changes;
- HTML still fully owned by dprint,
  with no exclusions or ignore directives.

The exec path is a validated orchestration prototype,
not the only acceptable production adapter.
A repository-owned dprint process plugin remains an alternative if exec's process and configuration boundaries
become limiting.

### Editor formatting probe

An executable JSON-RPC probe launched `dprint lsp` 0.55.2 with exec 0.7.3 and unsaved editor text.
It returned four edits for standalone CSS and one edit for HTML containing embedded CSS,
with no stderr.
Both outputs included the repository-owned CSS formatting.
This directly disproves treating open
[`dprint-plugin-exec#34`](https://github.com/dprint/dprint-plugin-exec/issues/34) as a blocker for the tested
versions and configuration.
The issue remains relevant historical evidence,
but editor formatting must be judged by the successful local probe.

Dprint supplies formatting edits only.
A replacement semantic checker still needs a deliberate live-diagnostics integration.
The repository already provides a standard-stdio diagnostics server in `package/linter/rust/src/lsp.rs` and
JetBrains LSP4IJ settings machinery under `package/dev-script/file-enforcer/src/jetbrains/`.
Those are implementation precedents,
not current CSS clients.
The CSS checker should share one pure diagnostic interface across CLI and full-buffer LSP adapters,
with CSS and HTML host adapters mapping region offsets into host-file ranges.
The repository currently recommends `stylelint.vscode-stylelint` and the IntelliJ Stylelint plugin,
so Stylelint must remain during the shadow phase unless equivalent clients land.

A host-offset probe found one real `StyleText` region while ignoring fake style markup in an HTML comment and
attribute.
It mapped `width`,
`10px`,
and `rgb(` to exact host slices and one-based line and column positions;
all 14 active regions produced valid nonempty ranges.

The exec adapter remains suitable for a transition because dprint LSP formatted unsaved standalone and embedded
CSS.
A 5-run,
incremental-disabled benchmark of the 5 pages measured 344.2 ms mean for markup alone and 451.2 ms mean with
exec CSS delegation,
a 107.0 ms mean difference for that invocation.
A dedicated persistent dprint process adapter is the preferred final formatting adapter if the owner accepts the
owned tool,
because it can avoid a child-command boundary per format request and own cache invalidation directly.

### CSS formatter and checker probes

The dprint-discovered tracked corpus contains thirty-six standalone CSS files,
fifteen HTML files,
and forty-five actual CSS regions.
A structural formatter over `package/module/css-edit/` and `@lezer/html` reported:

- zero parse failures;
- zero second-pass differences;
- zero non-whitespace token-signature mismatches;
- sixteen-line output for each learning-Rust stylesheet.

A hardened second formatter probe now also preserves every comment token byte-for-byte,
keeps or establishes bounded blank-line groups,
normalizes selector combinator and declaration spacing,
preserves directive comments without interpreting them,
and rejects malformed CSS.
Across all 45 regions it again reported no parse failures,
second-pass differences,
semantic-token mismatches,
comment mismatches,
or host prefix and suffix changes.
Compact,
comment,
nesting,
and selector fixtures all reached a fixed point.
Malformed standalone and embedded CSS made dprint fail while SHA-256 hashes confirmed both files remained unchanged.

A repository-owned policy probe reproduced the current semantic diagnostic totals of sixteen disallowed
functions,
sixty-two disallowed properties,
and sixty-seven disallowed units.
It also rejects the currently missed `@media (height: 10em)` case and reports twenty preferred `oklch()` channel
changes on the five learning pages.

The corrected ledger assigns every one of the 82 active Stylelint rules exactly once:

- 27 formatter responsibilities;
- 16 repository-policy responsibilities;
- 38 checker responsibilities;
- 1 deliberate drop,
  `no-descending-specificity`;
- no omissions,
  unknown names,
  or duplicate assignments.

All 82 pinned upstream rule directories contain tests.
45 rule implementations import CSS reference data or dedicated grammar or selector parsers.
That is direct evidence that full Stylelint retirement requires maintained vocabulary and grammar ownership,
not just porting the eight rules that currently emit diagnostics.
The probe exercises the structural formatter and 6 policy responsibilities;
the remaining responsibility fixtures are a migration gate,
not completed parity.

Eighteen actual `stylelint-disable` directives were found,
with three in the active tree and fifteen under `package-paused/`;
no `stylelint-enable` directives exist.
The proposed migration does not invent a second inline directive language.
Move the three active exemptions into typed,
path- and subject-scoped policy data with mandatory reasons.
The dropped specificity rule needs no exemption.
Paused files remain excluded as they are today;
when a package resumes,
remove its stale Stylelint marker and adjudicate the newly visible diagnostics rather than silently carrying the
old exemption forward.

### Color semantics

[CSS Color 4 section 4.4](https://www.w3.org/TR/css-color-4/#missing) says a missing component written as `none`
behaves as zero outside interpolation,
but may borrow the corresponding component during interpolation.
Therefore the preferred direct colors `oklch(0.1 none none)` and `oklch(0.9 none none)` render like zero-channel
forms,
but `none` must not be described as universally interchangeable with zero.

A Chromium 149 executable probe reported support for the `none` forms and equal rendered RGBA bytes for direct
`oklch()` comparisons of missing versus zero lightness,
chroma,
and hue.
Mozilla bug 1813481 records `none` color components as fixed in Firefox 113,
which is older than the repository's Firefox ESR 140 baseline.
The browser's canvas-gradient probe did not expose the specification's interpolation distinction;
the primary specification remains the deciding evidence for that boundary.

### Dependency evidence

The measured Stylelint,
standard-config,
and HTML-custom-syntax closure contains one hundred twenty-six package identities,
2,507 files,
and 10,721,885 bytes.
Stylelint alone reaches one hundred fifteen identities,
2,256 files,
and 9.247 MiB in the installed tree.
The existing `css-edit` foundation uses one zero-dependency tokenizer package.

A lockfile reachability analysis now starts from all 148 pnpm importers except the Stylelint configuration package
and Stylelint-only root dependencies.
It traverses 604 reachable snapshots,
counts peer-context variants as shared,
and resolves every non-workspace root.
Of the 126 measured Stylelint-closure identities,
16 are shared and 110 are exclusive by this conservative identity rule.
The exclusive installed footprint is 2,024 files and 7,845,733 bytes.
This supersedes the earlier sixty-two-missing-root estimate.

## Canonical affected sources

The five pages are:

- `package/learning/rust/lessons/index.html`
- `package/learning/rust/lessons/0001-whats-different-about-this-book.html`
- `package/learning/rust/reference/index.html`
- `package/learning/rust/reference/reading-loop.html`
- `package/learning/rust/reference/aquascope-decoder.html`

`package/learning/rust/NOTES.md` defines the package-local CSS contract.
It shows a seven-nonblank-line canonical example and requires CSS near ten lines with no extra presentation rules.
The user defined the accepted interpretation as five to twenty nonblank lines.
Formatting within that range is unconstrained.
The shown `oklch(0.1 0 0)` and `oklch(0.9 0 0)` values do not match the later preferred zero-channel syntax;
the preferred forms are `oklch(0.1 none none)` and `oklch(0.9 none none)`.
They also clarified that prose may reflow,
but a formatter must not split tight grammatical units such as a verb from its object.
List-like breaks are acceptable.
All five pages contained byte-identical embedded stylesheet content when probed at
`HEAD` `68eb96063`.
`NOTES.md` also requires `<meta name="color-scheme" content="light dark">` on every page.
Only `reference/aquascope-decoder.html` currently has it;
the other four affected pages do not.
This is a separate package-content defect and prevents treating the current corpus as a valid golden source.

Primary repository artifacts:

- `doc/planning/learning-rust-formatting-boundary.md`
- `doc/troubleshooting/dprint.md`
- `doc/troubleshooting/stylelint.md`
- `package/learning/rust/NOTES.md`
- `package/config/dprint/index.json`
- `package/config/stylelint/index.mjs`
- `dprint.json`
- `stylelint.config.mjs`
- `mise.toml`

## Verified current-tool behavior

### Formatter conflict

At the measured revision,
dprint 0.55.2 with `markup_fmt` 0.23.1 and Malva 0.14.1 reflowed prose at `printWidth: 90`.
Formatting two representative prose pages produced 200 insertions and 153 deletions.
The shared dprint and Stylelint fixed point expanded the seven-nonblank-line example to twenty lines.
The clarified five-to-twenty-line budget accepts that result.
Line count is no longer a reason to reject it.

`<!-- dprint-ignore-file -->` protects an entire HTML file.
`<!-- dprint-ignore -->` immediately before `<style>` does not protect the embedded stylesheet.
A Malva `/* formatter-ignore */` directive at the start of the style content protects that CSS,
but repeats local mechanism in every page.

Excluding `package/learning/rust/**/*.html` in root `dprint.json` removes exactly the five pages while retaining
other inherited excludes and ten other HTML files.
Explicit checks of excluded paths fail with dprint exit 14 unless the caller passes `--allow-no-files`.
A disposable-worktree probe of all five paths with that flag exited 0.
The owner subsequently disqualified every exclusion because formatter exemptions undermine trust in the active
formatter.

### Pinned source trace

Exact upstream versions were cloned and inspected:

- dprint 0.55.2
- `markup_fmt` 0.23.1
- Malva 0.14.1
- Stylelint 17.14.1
- `stylelint-config-standard` 40.0.0

`markup_fmt/src/printer.rs:701-737` always sends non-empty `<style>` content to `format_style`.
`dprint_plugin/src/lib.rs:54-90` gives embedded content a synthetic `#.css` path and delegates it to the host
formatter.
The schema has path overrides,
indent controls,
and ignore directives,
but no path option that disables embedded-style delegation while markup formatting continues.
This rules out exact arbitrary line preservation,
but exact preservation is no longer a requirement.
Path-specific width and CSS layout settings remain live candidates.

Dprint's `crates/dprint/src/commands/formatting.rs:1413-1444` tests `--allow-no-files` for both `fmt` and `check`.
Upstream issue `dprint/dprint#772` requested that behavior and records its release in dprint 0.43.0.

Broader tracker searches found related `markup_fmt` layout requests,
including issues 16 and 242,
but no existing preserve-wrapping request.
`markup_fmt` 0.27.3 is the current release and still exposes no text or prose wrapping mode.
A preserve-wrapping option is now a verified feature direction rather than a defect report.
No upstream issue has been filed.

### Current-tool configuration reopened by the clarified contract

dprint 0.55.2 documents per-plugin `overrides` in `website/src/config.md:284-339`.
The embedded Malva request uses a synthetic `index.html#.css`-style path.
A verified override matching `package/learning/rust/**/*.html#.css` with
`singleLineBlockThreshold: 2` formatted each embedded style to seven nonblank CSS lines.
It retained the preferred `oklch(0.1 none none)` and `oklch(0.9 none none)` values and was idempotent.

Markup width probes counted newly introduced line-break contexts inside HTML text nodes:

- width 160 produced 94;
- width 240 produced 30;
- width 320 produced 11;
- width 500 produced 3;
- width 600 produced none.

Width 600 avoided new contexts in the current corpus by joining paragraphs.
At current commit `f08ae1bc9`,
it produced 1,491 insertions and 1,689 deletions across the five files.
The longest generated line measured 593 characters.
Width 1000 produced byte-identical output.
Adversarial fixtures at both widths still split `original` from `variable`,
`allocator` from `deallocates`,
and `you` from `reach`.
A high width is therefore current-corpus mitigation,
not durable grammatical wrapping.
`whitespaceSensitivity: 'strict'` produced 357,
100,
and 49 new contexts at widths 90,
160,
and 200;
it is not a preservation mode.

At width 160,
all new break contexts occurred in the two prose-heavy pages:
`lessons/0001-whats-different-about-this-book.html` and
`reference/aquascope-decoder.html`.
The other three pages produced none.
A selective two-file dprint exclusion plus width 160 and the scoped Malva override was technically feasible.
The owner disqualified it with every other formatter exclusion.

The valid node-ignore directive is `<!-- markup-fmt-ignore -->`,
not `<!-- dprint-ignore -->`.
The pinned schema uses `markup-fmt-ignore` for one node and `dprint-ignore-file` for a whole file.
Node-ignore probes before `<body>` and `<main>` preserved the prose,
but the raw subtree kept its old indentation while its parent was reformatted.
The mechanism works,
but it stops formatter ownership of the ignored subtree and is not an acceptable replacement for a formatter
solution.

### Prose-preserving `markup_fmt` prototype

A disposable patch added an opt-in `preserveTextWrapping` mode to `markup_fmt` 0.23.1 and its dprint adapter.
The default remained the existing width-based behavior.
The new branch:

- normalizes indentation and repeated intra-line ASCII whitespace;
- keeps each authored nonempty text line as a hard formatter boundary;
- continues formatting document structure,
  attributes,
  quotes,
  and delegated embedded CSS.

The modified library test suite passed with the option defaulted off.
The dprint Wasm release build succeeded.
An end-to-end dprint run over all five pages produced:

- no exclusions or node/file ignores;
- original and formatted intra-text break counts of 0,
  16,
  0,
  0,
  and 259 by file;
- zero introduced intra-text break contexts;
- sixteen nonblank CSS lines in each style;
- a maximum line length of 189;
- 2,305 insertions and 1,676 deletions for the initial structural normalization;
- no changes on a second dprint run.

A focused long-line fixture kept `I ate a chicken`,
`original variable`,
`allocator deallocates`,
and `you reach` intact at width 90 because those units were authored on one line.
The prototype is not production code or an upstream submission.
It establishes that a small opt-in extension of the in-use formatter can retain full HTML ownership while leaving
grammatical wrapping to authors.

### Stylelint surface

At the measured revision:

- Stylelint discovered 51 CSS or HTML paths;
- root ignores removed 32 paths;
- the active surface was 19 files containing 14 actual CSS regions;
- the merged config enabled 82 rules;
- 34 enabled rules advertised fixes and 48 did not;
- the repository-wide run reported 185 diagnostics;
- those diagnostics came from eight rule names;
- the five learning pages contributed 40 diagnostics.

The current Stylelint config requires percentage lightness and angle hue.
That conflicts with the newly stated affected-CSS preference for numeric lightness and `none` zero channels.

A path-scoped profile using `at-rule-empty-line-before: 'never'`,
`declaration-block-single-line-max-declarations: 2`,
`declaration-empty-line-before: 'never'`,
`lightness-notation: 'number'`,
`unit-allowed-list: ['deg']`,
and `unit-disallowed-list: null` produced zero diagnostics on all five formatted pages.
Stylelint fix mode and the next dprint run both left the files byte-identical.
The unit allow-list rejected percentage,
turn,
and pixel probes.

The generic rules still miss one desired condition.
`oklch(0.1 0 none)` passes,
and a bare zero hue is fixed toward `0deg` rather than `none`.
A package-local assertion,
one narrow repository-owned Stylelint rule,
or deliberate manual review is needed for exact zero-channel notation.

The eight triggered names do not establish replacement parity.
Every enabled rule needs an explicit port,
drop,
or temporary-retention decision before Stylelint can be retired deliberately.

### Live Stylelint configuration defect

`package/config/stylelint/index.mjs` intends
`media-feature-name-unit-allowed-list` to require `rem` for every media-feature name.
Its plain string key `'/[\w-]+/'` reaches Stylelint as `/[w-]+/` because JavaScript consumes the backslash.
The adjacent comment correctly says `String.raw` is necessary,
and neighboring patterns already use it.

A runtime probe through `mise run //:lint:stylelint -- --no-cache` accepted `height: 10em` and rejected
`width: 10em`.
`width` happens to contain `w`;
`height` does not.
This is a downstream config defect,
not Stylelint behavior.
Any parity suite must reject `height: 10em` and `width: 10em` and accept `height: 10rem`.

## Verified repository-owned tooling feasibility

`package/module/css-edit/` is a strict,
immutable,
byte-preserving CSS CST based on `@csstools/css-tokenizer`.
It follows the same repository edit-family precedent as `package/module/jsonc-edit/` and
`package/module/toml-edit/`.

Disposable probes established:

- the compact nested learning stylesheet parses;
- nested media rules and declarations remain distinct nodes;
- tokens retain source offsets;
- serialization is byte-identical;
- every one of the 14 active CSS regions parses and round-trips byte-identically.

The existing `@lezer/html` dependency exposes embedded style content as `StyleText` nodes with host offsets.
An adversarial probe found the real style region and ignored false `<style>` text in a comment and quoted
attribute.
This supports host-position diagnostics without serializing surrounding HTML.

Measured implementation surfaces:

- `module-css-edit` has ten production TypeScript files and 1,728 source lines;
- installed Stylelint production `lib/` has 357 JavaScript modules and 34,348 source lines;
- Stylelint 17.14.1 declares 35 direct runtime dependencies.

These are auditability measurements,
not evidence of semantic parity or proof that custom ownership is preferable.

A promising narrow custom seam is:

```ts
checkSources({ paths }): CheckReport;
fixSources({ paths }): FixReport;
```

Required invariants:

- check mode never writes;
- surrounding HTML is never reserialized;
- a CSS-only checker leaves surrounding HTML untouched;
- any markup formatter avoids breaks inside tight grammatical units;
- CSS fixes may change layout but keep the result around ten lines;
- diagnostics for embedded CSS use host HTML positions;
- fixes replace only narrowly approved token ranges;
- every fix is idempotent;
- semantic policy is separate from layout policy;
- package-specific line-budget and allowed-presentation checks stay outside generic CSS rules.

This is one candidate architecture,
not an accepted decision.
A full dprint replacement is not justified by issue 401 alone.
At one measured revision,
dprint resolved 775 paths across JSON,
TOML,
YAML,
CSS,
HTML,
XML,
and SVG,
with 724 paths in unrelated data or XML-family domains.

## Comprehensive solution space to finish

Do not collapse the investigation into a binary keep-or-replace question.
Evaluate each family at every relevant layer.

### Vertical layers

- **Symptom layer:**
  Leave source unchanged.
  Use scoped ignores.
  Or change individual conflicting rule values.
- **File layer:**
  Use near-ten-line and allowed-rule fixtures.
  Use formatter-ignore directives.
  Use file-specific lint profiles.
  Or use source-preserving region edits.
- **Package layer:**
  Declare the learning pages an authored-document boundary.
  Add package-owned check tasks.
  Generate shared style fragments.
  Or use package-local configuration.
- **Repository layer:**
  Use root path exclusions.
  Split check orchestration.
  Share semantic policy.
  Coordinate CI and editor behavior.
- **Architecture layer:**
  Use generic external tools.
  Own narrow repository rules.
  Generate sources.
  Or replace whole tools.

### Horizontal families

- Preserve the current source contract and exempt only these files from formatting.
- Revise the package contract to accept the common dprint and Stylelint fixed point.
- Keep dprint and Stylelint,
  but re-point only conflicting Stylelint rules for this path.
- Keep dprint markup formatting while protecting embedded CSS with supported Malva directives.
- Split markup and CSS ownership through orchestration or extracted regions.
- Replace formatter convergence with package-local line-budget and allowed-presentation checks.
- Make one stylesheet authoritative and generate or inject copies without turning prose into generated HTML.
- Move the style to a shared external CSS asset if offline and file-URL behavior remains correct.
- Use a package-local checker that validates the approximate line budget and semantic style constraints.
- Build a repository-owned semantic CSS checker over `css-edit`,
  initially in shadow mode.
- Build only the missing narrow lint rules while retaining Stylelint for generic syntax coverage.
- Retain dprint for unrelated formats while retiring only Malva,
  Stylelint,
  or both from the affected seam.
- Replace Stylelint or dprint entirely only if broader repository evidence justifies that ownership.
- Propose an upstream feature only if a pinned-source audit and tracker search establish an upstream-supported gap.
- Accept the current inconsistency temporarily if its operational cost is lower than all migrations.

For every serious option,
record evidence,
coverage retained and lost,
source fidelity,
maintenance owner,
editor and CI behavior,
migration reversibility,
and failure modes.
Give pros,
cons,
and a fully sorted ranking with reasons between adjacent entries.

## Current recommendation status

The previously posted recommendation is withdrawn.
Neither the all-five nor selective dprint exclusion is acceptable because both weaken trust in the repository's
active formatter.
The former first and second options are now disqualified rather than merely re-ranked.

The explicit CSS budget is five to twenty nonblank lines.
The seven-line Malva result remains valid,
and the previously described twenty-line fixed point is also within budget.
Line count no longer distinguishes those outputs.

For HTML ownership,
the leading direction is an opt-in prose-preserving mode in `markup_fmt`.
It ranks above high-width configuration because it passed both the current corpus and adversarial grammar fixture
without surrendering structural formatting.
High-width configuration remains a current-corpus bridge,
not the durable recommendation.

Stylelint retention is no longer the default.
Do not run an external technology-selection comparison.
The current direction is a repository-owned CSS tool whose formatting and linting responsibilities must be made
explicit against Malva and all eighty-two active Stylelint rules.
The owned design is now validated enough to rank,
but not enough for immediate repository-wide Stylelint removal.
The leading combined resolution is phased:
first land prose-preserving markup plus owned CSS formatting and package checks for the learning pages,
then run the remaining Stylelint responsibilities in shadow until their fixtures and editor clients are complete,
and remove Stylelint only after that gate.
No production source or configuration implementation has been made.

## GitHub updates already posted

- Initial evidence checkpoint:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139282431>
- Source-preserving in-house feasibility checkpoint:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139335872>
- Stylelint regex-string enforcement defect:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139346649>
- Package-local CSS contract correction:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139391488>
- Reopened current-tool configuration paths:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139475030>
- Superseded synthesis and rankings:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139562950>
- Correction withdrawing formatter exclusions and reopening Stylelint replacement:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139615382>
- Direction correction toward repository-owned CSS tooling:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139620214>
- Prose-preserving `markup_fmt` prototype:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5139733575>
- Combined owned-CSS evidence,
  complete responsibility ledger,
  and revised ranking:
  <https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5140530223>

The original issue comment is:
<https://github.com/Aquaticat/Monochromatic/issues/401#issuecomment-5124915718>

Post the next issue update only after the comprehensive option space has new verified evidence.
Do not repeatedly restate old checkpoints.

## Scratch artifacts

Prepared upstream clones:

- `/var/home/user/temp/agent/dprint-401-2026-07-30`
- `/var/home/user/temp/agent/markup-fmt-401-2026-07-30`
- `/var/home/user/temp/agent/malva-401-2026-07-30`
- `/var/home/user/temp/agent/stylelint-401-2026-07-30`
- `/var/home/user/temp/agent/stylelint-standard-401-2026-07-30`

Disposable probe sources include:

- `/var/home/user/temp/agent/issue401-css-edit-probe.ts`
- `/var/home/user/temp/agent/issue401-css-edit-corpus-probe.ts`
- `/var/home/user/temp/agent/issue401-lezer-html-probe.ts`
- `/var/home/user/temp/agent/issue401-lezer-html-cases.ts`
- `/var/home/user/temp/agent/issue401-style-region-hashes.ts`
- `/var/home/user/temp/agent/issue401-media-unit-probe.css`

Disposable worktrees are available at:

- `/var/home/user/temp/agent/issue401-worktree.LykdMVxQ`;
- `/var/home/user/temp/agent/issue401-current-worktree`;
- `/var/home/user/temp/agent/markup-fmt-preserve-401`.

Important current probe artifacts include:

- `/var/home/user/temp/agent/issue401-current-worktree/dprint.issue401-owned-css-exec.json`;
- `/var/home/user/temp/agent/issue401-owned-css-format-v2.ts`;
- `/var/home/user/temp/agent/issue401-owned-css-format-v2-report.json`;
- `/var/home/user/temp/agent/issue401-owned-css-check-v2.ts`;
- `/var/home/user/temp/agent/issue401-owned-css-check-v2-report.json`;
- `/var/home/user/temp/agent/issue401-learning-css-contract-formatted-report.json`;
- `/var/home/user/temp/agent/issue401-dprint-lsp-v2-report.json`;
- `/var/home/user/temp/agent/issue401-dprint-exec-benchmark.json`;
- `/var/home/user/temp/agent/issue401-rule-outcomes-v2.json`;
- `/var/home/user/temp/agent/issue401-stylelint-footprint.json`;
- `/var/home/user/temp/agent/issue401-stylelint-lockfile-exclusivity.json`.

## Documentation checkpoint

Commit `8d2bc2f2a9f4d85d22ed007c25d4bcf6d7c6efd3` records:

- `doc/planning/learning-rust-formatting-boundary.md`;
- `doc/troubleshooting/dprint.md`;
- this handover.

The planning document contains the corrected package contract,
formatter and linter measurements,
current-tool configuration probes,
repository-owned feasibility,
horizontal and vertical option families,
fully sorted rankings,
and the proposed resolution.

The troubleshooting document records the corrected directive names,
dprint override behavior,
width and strict-whitespace probes,
Stylelint fixed point and semantic gap,
content-meta defect,
workarounds,
and upstream filing decision.

Follow-up commit `56e1ae851` records the first checkpoint update to this handover.
Commit `001e49482` records the hardened formatter,
checker,
ledger,
dependency,
and editor evidence.
Commit `77b93df4b` rewrites the planning and dprint troubleshooting documents around the phased combined resolution.
The 3 issue documents passed scoped Markdown lint and `git diff --check`.
Unrelated `package/cli/wg-quicker-exempt/` changes remain in the worktree and were not staged.

## Exact next action

Wait for the owner to accept,
reject,
or modify the phased combined direction.
If accepted or delegated,
create production implementation tasks for:

1. opt-in prose-preserving markup;
2. owned CSS formatting and the dprint process adapter;
3. the learning package contract checker;
4. shadow Stylelint responsibility migration;
5. CSS and HTML live-diagnostic clients;
6. final Stylelint dependency removal.

Leave production configuration,
package source,
and page changes unstarted until that decision.
