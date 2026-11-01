# Slint IDE 0.x scope interview

## Status

The user confirmed the consolidated contract.
The accepted scope is recorded in [the decision document][accepted-scope].
The user subsequently reviewed the approach and explicitly authorized implementation.
[The implementation plan][implementation] records the work queue;
[the handover][handover] records the current implementation and verification state.

## Explicit requirements

- Build an IDE with Slint.
- Refer to the paused editor at `package-paused/desktop-daemon/editord`.
- Cut the 0.x scope ruthlessly.
- Actual text editing is not needed.
- Follow-up clarification:
  "No changes to files via any way is needed in 0.x".
  Exclude file-mutation workflows whether performed directly or delegated to another process.
- Resolve the design through a grilling interview before implementing.
- Primary workflow:
  live source reading and navigation while changes happen elsewhere.
- Use editord as a familiar reference,
  documenting every deliberate behavioral difference.
- Support the current host only for 0.x.
- Support local filesystem projects only.
- Require go-to-definition,
  find references,
  hover types/documentation,
  inlay hints,
  and diagnostics.
- Require syntax highlighting.
- Bundle JetBrains Mono and Inter;
  include their licenses and avoid relying on host-installed copies.
- Follow the system's light/dark appearance,
  including changes while running.
- Keep combined file-path/content search and a browsable file tree.
- Explicit implementation follow-up:
  retain editord's Ctrl+0 through Ctrl+9 recency navigation,
  promotion,
  and tree reveal behavior.
  History is session-local because persistent session restore remains cut.
- Refresh automatically during external changes,
  including while text is selected.
  Best-effort content-relative caret/selection mapping
and approximate viewport anchoring should minimize visual location loss.
  Exact pixel preservation and universal mapping through rewritten text are not required.
  Selected ranges follow corresponding replaced regions,
  not a later occurrence of the old selected text.
- Do not include a go-to-line command.
- Determine language coverage from actual `tokei` output,
  bounded by Helix support.
  Do not add custom integrations for languages missing from Helix.
- Keep the paused editor's highlighting coverage where supported by Helix.
- Display diagnostics for the current file only;
  omit a project-wide problems view.
- Choose inlay placement by implementation evidence;
  the user delegates this choice rather than requiring either proposed presentation.
- Language support is capability-aware:
  implement all five feature paths,
  but do not fabricate capabilities unavailable for a particular language.
- Private application/tool cache and temporary writes outside the project are allowed.
  Project mutation remains excluded.
- Helix-owned components are preapproved candidates.
  The user explicitly requests no broad choosing-technology exercise for this task.
  Inspect only the integration APIs,
  relevant behavior,
  and license obligations needed for reuse.

Slint is settled as the UI toolkit.
Host language,
process architecture,
and dependencies are not selected.
No promise of editing in a later version is implied.

## Reference evidence

- [Paused editor README][editor-readme]:
  browser frontend plus daemon;
  search,
  filesystem operations,
  language services,
  recent files,
  and keyboard navigation.
- The README's `JetBrains parity` section calls every behavioral difference a bug.
  The user explicitly replaces that obligation with familiar-reference behavior and a complete difference record.
- [WebStorm comparison][comparison]:
  the recorded reason to retain editord shifted from scrolling to freshness.
  This is historical project rationale,
  not a newly verified claim about current WebStorm behavior.
- The comparison's `The condition: fuzzing earns the keep` section identifies unverified watcher edges
  and a self-save suppression window.
  Do not inherit its stronger correctness-by-construction claims as proof.
- [Current watcher source][watcher]:
  filesystem watching includes orphan-temp deletion and event suppression.
  Reusing a watcher does not automatically produce a read-only application.
- [Current keybindings][keybindings]:
  navigation and copying coexist with saving,
  formatting,
  rename,
  line edits,
  and launching an external terminal.
  Removing typing alone would not remove mutation or process-execution surfaces.
- [Implementation plan][editor-plan] records delivered phases;
  [TODO][editor-todo] records watcher concurrency,
  protocol liveness/backpressure,
  observability,
  and UI gaps.
  Neither is adopted wholesale as the new backlog.
- `uname --kernel-name --machine` reports `Linux x86_64` for the current host.
  The user selected the current host as the sole 0.x acceptance target.
  Selected environment variables report Wayland and KDE for the desktop session.
  Probe actual display configuration before native UI verification.

The paused package remains untouched during the interview.
No deprecation,
replacement,
or code-reuse commitment is made.

## Settled interview decisions

### File mutations

The user excludes file changes through any mechanism for 0.x.
Cut editing,
formatting,
refactoring,
file creation,
rename,
move,
delete,
Git writes,
and delegated mutation through agents or commands.
A terminal or task launcher must not reintroduce these workflows.

This is a product scope decision,
not evidence of sandbox enforcement.
Clipboard copying is not project-file mutation.
Q14 explicitly allows private application/tool cache and temporary writes outside the project.
This does not add a persistent session-restore feature or authorize project-file mutations.

### Round 1 answers

- Q1:
  A,
  live source reading and navigation.
  Change-review and agent-observation workflows are not part of the selected starting scope.
- Q2:
  familiar reference,
  with every deliberate difference documented.
  Exact parity for retained features is rejected.
- Q3:
  current host only.
  Generic Linux support and other operating systems are not acceptance promises.
- Q4:
  local only.
  Do not add SSH,
  remote-daemon,
  or mounted-network-filesystem acceptance implicitly.

## Behavior-difference register

This register is required by the user.
Entries describe the intended scope,
not implemented or verified behavior.
Record every later departure with the reference behavior,
replacement behavior,
reason,
and acceptance check before implementing it.
Preserve a complete record when the plan moves into package documentation.

### UI platform

- Reference:
  editord serves a browser frontend;
  [its philosophy][editor-philosophy] relies on browser-provided find,
  zoom,
  selection,
  printing,
  and accessibility behavior.
- New scope:
  Slint UI on the current host.
  Browser-provided features are not automatically inherited;
  required replacements must be selected and tested explicitly.
- Reason:
  user-selected toolkit and host boundary.
- Acceptance:
  run the actual native artifact on the current desktop session.
  No browser facility counts as provided merely because editord had it.

### Mutation commands and hidden writes

- Reference:
  keybindings expose save,
  format,
  rename,
  and line edits;
  the filesystem service also mutates files.
  The watcher deletes orphaned atomic-write temporary files.
- New scope:
  no file-mutation workflow or delegated mutation.
  Do not reuse the watcher's cleanup side effect.
- Reason:
  explicit 0.x no-write requirement.
- Acceptance:
  disposable-fixture checks exercise retained UI commands and file-open/watch paths,
  verifying that application activity does not change fixture contents or directory entries.
  This alone does not prove an OS sandbox or absence of writes elsewhere.

### Compatibility contract

- Reference:
  README declares every JetBrains behavioral difference a bug.
- New scope:
  selected familiar behavior plus an explicit record of every difference.
  Feature-specific contracts are pending the next rounds.
- Reason:
  user accepted familiar-reference behavior rather than exhaustive parity.
- Acceptance:
  compare implemented commands,
  shortcuts,
  navigation,
  search,
  reload behavior,
  and omissions against the reference and this register.

### External updates and read-only caret

- Reference:
  `src/client/app/events.ts` in editord calls the file loader on an external modification;
  `src/client/app/file-loader.ts` replaces displayed text.
  These call sites do not establish the new caret-preservation contract.
- New scope:
  update automatically without a selection hold.
  Prefer corresponding positions in surviving text over the old numerical offset.
  The user explicitly accepts brute-force work to achieve this behavior.
  The later clarification makes mapping and viewport anchoring best effort:
  roughly the same viewport position is sufficient;
  minimize location loss for human eyes.
- Reason:
  explicit Q8 answer and subsequent caret example.
- Acceptance:
  display `I am a big cat.` with caret `I am a bi|g cat.`;
  externally replace `am` with `was`;
  after refresh the caret is `I was a bi|g cat.`.
  The marker is not document content.
  Also display `I am a big cat` with selection `I [am a] big cat`;
  externally replace the file with `I was a big cat, but now I am a human!`;
  after refresh the selection is `I [was a] big cat, but now I am a human!`.
  Brackets denote selection and are not document content.
  Do not jump to the later literal `am a` or clear the selection merely because the selected words changed.
  These supplied examples are concrete expected outcomes;
  best effort governs genuinely ambiguous rewrites and approximate viewport placement.
  Add repeated-text,
  replaced-context,
  selection,
  and viewport cases without asking the user to prescribe a fallback algorithm.
  No case may block refresh while selection exists.
  Native caret and viewport behavior still require a Slint consumer-boundary probe.

### Go-to-line command

- Reference:
  the proposed new baseline included a go-to-line command.
  editord also navigates to positions supplied by search and language services.
- New scope:
  no user-invoked go-to-line command.
  Search-result,
  definition,
  and reference navigation still target source positions.
- Reason:
  explicit user cut.
- Acceptance:
  omit the command and shortcut without removing position-based navigation required by retained features.

## Round 2 answers

- Q5:
  all listed language-intelligence capabilities are required at minimum.
  A text-only navigator is not useful.
  The proposed no-language-service cut is rejected.
- Q6:
  syntax highlighting is indispensable.
  The proposed plain-text cut is rejected.
- Q7:
  B,
  both search and tree.
  The proposed search-only cut is rejected.
- Q8:
  automatic refresh,
  explicitly without holding while text is selected.
  Try to preserve current relative position and brute-force the work where necessary.
  The follow-up example establishes content-anchored caret mapping,
  not fixed offsets or fractional scrolling.
  The user then clarifies that preservation is best effort,
  roughly the same viewport position is sufficient,
  and minimizing visual location loss is the goal.
  No further preference question about deleted-context fallback is needed.
  A subsequent selection example requires mapping `am a` to the replacement `was a`,
  while ignoring a new literal `am a` later in the document.
  Preserve the selected region's correspondence through edits,
  not its original string value.
- Baseline correction:
  remove go-to-line.

The final confirmation also accepted the remaining floor:
one local root,
one window,
one file view,
line numbers,
selection/copy,
and in-file find.

The final confirmation also accepted the remaining cuts:
tabs,
split panes,
Git/status/diff UI,
agent/task dashboards,
media or rendered-document previews,
and persistent session restore.
The user did not request additions to these areas.

## Round 3 answers

- Q9:
  measure with `tokei` and support the languages the user actually uses.
  Do not ask the user to supply an inventory that can be measured.
- Q10:
  keep highlighting coverage.
- Q11:
  displayed-file diagnostics only.
  Server-internal project analysis is not restricted by this UI cut.
- Q12:
  choose whichever inlay placement is easier to implement.
  Do not assume the earlier above-line recommendation is accepted;
  determine the implementation path from Slint and reusable component evidence.
- Follow-up dependency direction:
  Helix components are preapproved;
  no broad dependency-selection exercise is requested.
  This is not authorization to embed every Helix subsystem or copy its editing/keybinding model.
- Follow-up language ceiling:
  any language missing from Helix is unsupported in 0.x.
  Do not develop custom integrations to fill that gap.

## Measured language scope

### Inventory method and limits

Ran `tokei 15.0.0` through a disposable `mise run inventory` task,
first with ordinary ignore behavior,
then including hidden files.
The hidden-file run also counted Git hooks;
repeat with `--hidden --exclude .git --output json` excluded Git internals,
verified by zero report paths containing `/.git/`.

The final pass respects ignore files and includes active,
paused,
and deprecated packages plus documentation.
It is not a tracked-files-only or handwritten-code-only census.
Generated TypeScript and JSON benchmark reports contribute to counts.
A `.js` ignore pattern may hide genuine tracked source;
do not interpret absent reports as proof a language is unused.

Standalone file languages are the base inventory.
Embedded code fences are reported separately:
Go,
Python,
Java,
and other documentation examples do not automatically add full semantic-language requirements.

Raw evidence and task definition are in the private scratch directory
`~/temp/agent/slint-ide-languages.mEr8K9`.
The durable findings are recorded here so the plan does not depend on scratch retention.

### Standalone counts from the corrected pass

- TypeScript:
  5,380 files and 608,892 code lines.
- Rust:
  427 files and 43,248 code lines.
- Kotlin:
  70 files and 5,892 code lines.
- JavaScript:
  57 files and 4,758 code lines.
- Slint:
  2 files and 1,848 code lines.
- QML:
  8 files and 484 code lines.
- HCL:
  1 file and 722 code lines.
- SQL:
  7 files and 521 code lines.
- Shell:
  5 files and 351 code lines.
- C:
  1 file and 208 code lines,
  a troubleshooting reproducer.
- C++:
  1 file and 17 code lines,
  the Qt logging bridge.
- Batch:
  2 files and 128 code lines,
  both Gradle wrappers.
- Other standalone families:
  JSON,
  TOML,
  YAML,
  HTML,
  CSS,
  Markdown,
  MDX,
  XML,
  SVG,
  Dockerfile,
  and plain text.

Do not rank the user's language needs by raw code-line totals:
generated data distorts those totals,
and the user requested coverage rather than a popularity threshold.

### Helix boundary evidence

Inspected upstream `languages.toml`,
`Cargo.toml`,
and `LICENSE` from `helix-editor/helix`.
Upstream master observed at `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`;
the corresponding language-registry blob is `a5403e68b1f7fd9fb7a9924d464d09c08c1be264`.
The fetched workspace manifest and license identify MPL-2.0;
preserve applicable source and license notices for reused components.

Registry mappings include these non-identical names:
Shell maps to `bash`,
C++ to `cpp`,
SVG to `xml`,
and `.mdx` to `markdown`.
A `.mdx` association is not proof of complete JSX-aware MDX semantics.

The inspected registry contains language entries for the standalone source/markup families in the inventory.
However,
its SQL,
XML/SVG,
and Batch entries do not specify a language server.
Grammar recognition and highlighting are not equivalent to support for all semantic features.
Configured servers for other entries are not proof that their executables are installed or every capability works.

## Round 4 answers

### Q13: Helix ceiling per capability

Accepted:
capability-aware language support.
Implement the five required feature paths in the app,
use capabilities available for each supported language,
and explicitly report unsupported capabilities.
Languages with Helix grammar support but no language server remain highlight/read/search-only.
Do not invent custom semantic backends to make every feature universal.

### Q14: application-private writes

Accepted:
private application/tool cache and temporary storage outside the project.
No project-file mutations,
no arbitrary task-launching feature,
and no persistent session-restore feature are added.
Inspect and exercise subprocess writes before implementation is accepted.

## Confirmed build contract

### Purpose and boundaries

Build a Slint desktop application for reading and navigating live local source code.
Target this host only.
Use editord as a familiar reference and document every deliberate difference.
Helix-owned components are preapproved;
this does not adopt Helix's modal interaction model or require embedding its full editor.

### Required surface

- One local project root,
  one window,
  and one source-file view.
- File tree and combined file-path/content search.
- Syntax highlighting,
  line numbers,
  read-only caret/selection,
  copying,
  and in-file find.
- Go-to-definition,
  references,
  hover types/documentation,
  inlay hints,
  and displayed-file diagnostics,
  subject to actual language capabilities.
- Actual-use language inventory bounded by Helix support.
  Unsupported types remain readable as plain text without fabricated language intelligence.
- Automatic external-change refresh with no selection hold.
  Reconcile caret and selection to corresponding text regions,
  including replacements;
  keep the reading location approximately stable in the viewport.
  The supplied caret and selection examples are acceptance cases.
- Private cache/temp writes outside the project when required by integration.

### Explicit cuts

- Editing,
  saving,
  formatting,
  refactoring,
  file creation,
  rename,
  move,
  deletion,
  and other direct or delegated project mutations.
- Go-to-line command.
  Position-targeted search and language navigation remain required.
- Tabs,
  split panes,
  and multiple project roots in the app.
- Git/status/diff UI,
  terminal/task control,
  and agent dashboards.
- Project-wide problems view.
- Media previews,
  rendered Markdown/document previews,
  and persistent session restore.
- Remote filesystem support,
  additional platform promises,
  and custom integrations for languages missing from Helix.

### Required verification

- Verify through the actual Slint application on the current desktop,
  not only unit tests or compilation.
- Exercise all required language-intelligence feature paths against actual servers.
  Distinguish unavailable capabilities from broken supported capabilities.
- Exercise the supplied caret and replaced-selection examples under external file changes.
- Verify refresh does not wait for selection release and stale asynchronous results do not overwrite newer state.
- Verify app and subprocess project-write boundaries against disposable fixtures.
- Check every deliberate reference deviation is in the behavior-difference register.

## Agent-owned implementation investigation

These are engineering responsibilities,
not questions to hand back to the user.

- Design full-file refresh with old/new text reconciliation.
  Investigate contextual old/new edit correspondence for caret,
  selection ranges,
  and a visible source-line anchor.
  Simple substring relocation is insufficient:
  replaced selections must expand or contract with their corresponding replacement,
  even when the old string reappears elsewhere.
  Use deterministic nearby fallbacks when correspondence is genuinely ambiguous.
  Brute-force allowance does not justify blocking the UI or unbounded work.
- Verify selectable highlighted text,
  annotations,
  hit testing,
  caret placement,
  and viewport anchoring through Slint before committing to a renderer design.
- Verify actual language-server capabilities and user-visible behavior for every selected language.
  Existing wrappers and completed-plan checkboxes do not establish current server support.
- Keep stale language-service results from being presented as current after external file updates.
  Caret mapping is application-owned;
  language-server document synchronization is a separate lifecycle to verify.
- Inspect writes from any selected language server or helper.
  A read-only UI does not prove subprocesses leave files unchanged.
- Establish search-result freshness and stale-target navigation behavior.
- Probe file-size and directory-size behavior using disposable fixtures and explicit resource bounds.

## Implementation decisions delegated to the agent

Remaining research determines code reuse,
host language,
process boundaries,
concrete rendering,
server integration,
and the implementation sequence.
These are not new user preference questions unless evidence reveals a conflict with the accepted contract.

Choose inlay placement using implementation evidence.
Retain familiar reference behavior where it fits the accepted scope;
document each departure rather than treating all JetBrains behavior as mandatory.

Do not split required features into unspecified future 0.x work:
the first completed deliverable must meet the confirmed contract.
No later editing roadmap is implied.

## Next action

The scope is confirmed and recorded in `doc/decision/slint-ide-0x-scope.md`.
Implementation is authorized and underway.
Resume from `doc/handover/slint-ide-0x.md`.
Do not ask for scope confirmation again.
If later integration probing exposes a genuine requirement conflict,
report the evidence and ask only the controlling decision.

Round 3 evidence:
`git ls-files` over `package`,
`package-paused`,
and `doc` yielded 9,904 tracked paths,
including 5,337 `.ts`,
427 `.rs`,
and 65 `.kt` paths.
These are extension counts,
not complete language counts or a claim about untracked/ignored files.
The result motivates asking about semantic language coverage rather than inferring it from repo contents.

Planning history:
`c5654c5e1` introduced the record;
`c37f58ca5` fixed semantic line breaks;
`3cf1e698c` applied the no-write requirement to candidate workflows;
`9f1894d94` recorded Round 1 answers and the difference contract;
`3609e6b4b` recorded required intelligence and caret preservation;
`aee461f00` recorded viewport/selection correspondence;
`4fb29aa6b` fixed the clarification's semantic breaks;
`383474dc5` recorded measured languages and the Helix ceiling.
Scoped Markdown lint and micromark rendered-output checks passed through the language-scope update.
Repeat both for the consolidated contract.

[handover]: ../handover/slint-ide-0x.md
[accepted-scope]: ../decision/slint-ide-0x-scope.md
[implementation]: slint-ide-implementation.md
[editor-readme]: ../../package-paused/desktop-daemon/editord/README.md
[editor-philosophy]: ../../package-paused/desktop-daemon/editord/PHILOSOPHY.md
[comparison]: ../../package-paused/desktop-daemon/editord/docs/decisions/webstorm-comparison.md
[watcher]: ../../package-paused/desktop-daemon/editord/src/server/operations/watch-filesystem.ts
[keybindings]: ../../package-paused/desktop-daemon/editord/src/client/app/keybindings.ts
[editor-plan]: ../../package-paused/desktop-daemon/editord/PLAN.md
[editor-todo]: ../../package-paused/desktop-daemon/editord/TODO.md
