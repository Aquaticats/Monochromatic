# Make Monochromatic Terminal ready for daily use

## Recommendation and scope

The user asked on 2026-11-01:
"I've just discovered a bug in Rio.
And when I tried konsole before it had some visual artifacts.
Time to polish up our monochromatic terminal?
Your suggestion?"

Recommend a bounded reliability milestone for the existing terminal,
while retaining a working terminal for current sessions.
The benefit is direct control over behavior and regression tests for the user's workflow.
The cost is maintaining a terminal frontend with currently incomplete input and rendering behavior.
This is a proposal,
not an adopted implementation plan or a claim of current daily-use readiness.
No product code,
dependency,
configuration,
or default terminal was changed during this assessment.

The user waived the heavyweight technology-selection process for this session.
This recommendation uses repository source and the session's existing investigations;
it is not a completed technology vet or a measured comparative reliability result.

## What the evidence changes

The [Rio diagnosis](../troubleshooting/rio-variable-font-not-found.md)
identifies a named variable-font instance index passed to a collection-face parser.
It records an applied static-font-directory workaround verified in a fresh process.
That is a bounded defect with a working mitigation,
not evidence that rebuilding a terminal is the shortest repair.
The user's Konsole visual artifacts have not been reproduced or diagnosed in this assessment.

The [Contour investigation](../troubleshooting/contour-modifier-bindings-codex.md)
reproduced native scrollback loss for Codex's upper scrolling region.
The user reports that OdyTTY handles their Codex scrollback;
its source contains an explicit path preserving rows from that region.
Keeping the existing working OdyTTY session is an operational fallback,
not a new claim that it passed the user's community-policy requirements or a full reliability evaluation.

Monochromatic's [TODO file](../../package/terminal/app/TODO.md)
records severe typing stutter and a Ghostty resize panic:
`PageList integrity check failed: error.ViewportPinInsufficientRows`.
These are historical reports requiring fresh reproduction,
not failures re-established on the current build.

## Current implementation

- [The manifest](../../package/terminal/app/Cargo.toml)
  and [package README](../../package/terminal/app/README.md)
  describe a Slint frontend using `libghostty-vt` and `portable-pty`.
  Retain this existing architecture for the first diagnostic milestone.
  No new parser,
  renderer,
  or PTY dependency is proposed.
- [The UI](../../package/terminal/app/ui/app.slint)
  renders cells through Slint `Text` and uses a `Flickable` for pixel scrolling.
  Its `terminal-key` callback carries key text,
  Control,
  and Alt,
  but no Shift field.
  The [encoder](../../package/terminal/app/src/input.rs)
  has no terminal-mode parameter.
  Cursor drawing,
  selection,
  clipboard handling,
  bracketed-paste handling,
  and mouse reporting are absent from the inspected frontend paths.
- [The engine's `feed`](../../package/terminal/app/src/engine.rs)
  explicitly scrolls the viewport to the bottom after accepting output.
  [The UI output handler](../../package/terminal/app/src/main.rs)
  also sets the pixel scroll offset to the bottom.
  This is a source-level finding;
  the resulting user-visible behavior was not measured during this assessment.
- The output handler drains an unbounded channel on the UI thread,
  and `apply_snapshot` replaces the cell model.
  These are profiling targets for the recorded stutter,
  not an established cause.
- [Engine tests](../../package/terminal/app/src/engine_tests.rs)
  cover text/style extraction and a basic scroll-to-top case.
  [Input tests](../../package/terminal/app/src/input_tests.rs)
  cover basic printable,
  control,
  Alt,
  named-key,
  and empty input.
  They do not establish compatibility with the reported complete agent sessions.

The package's cell rendering uses Slint;
the [diagnosed Ghostty desktop crash](../troubleshooting/ghostty-fontconfig-language-type.md)
was in Ghostty's own font-discovery and rendering path.
That distinction makes a direct causal transfer unsupported.
It does not prove that Monochromatic is immune to font bugs or to defects in the shared VT core.

## Proposed milestones and acceptance checks

### Establish a usable baseline

Reproduce the recorded resize panic and typing stutter in a disposable session on the current deployment.
Measure keystroke-to-visible-output latency and its unchanged-build spread.
Separate input encoding,
PTY delivery,
VT processing,
and frame presentation before assigning cause.
Require repeatable resize/reflow behavior and responsive typing during actual terminal output.
If this exposes an unsuitable core integration,
revisit that dependency before extending the frontend.

### Complete everyday interaction

Preserve modifiers and negotiated terminal modes through the input interface.
Exercise Shift+Left,
Ctrl+Left,
Shift+Enter,
Escape,
and ordinary Ctrl+C in the real agent applications.
Add selection and copy/paste behavior,
including a Ctrl+Shift+C no-selection case that leaves the application running.
Verify bracketed paste and mouse reporting against applications that request them.
Render cursor visibility and style correctly.

### Preserve conversation history

Use the reproduced Contour scrolling-region fixture as a regression input.
Compare ordinary full-screen scrolling with the fixed-bottom-prompt layout,
and assert that the initial marker remains reachable in terminal history.
Keep the user's reading position stable while new output arrives,
including scrollback eviction and resize/reflow cases.
Exercise alternate-screen applications separately.

### Verify the rendered workflow

Run real Codex and Claude sessions in an isolated nested compositor with disposable state.
Test selection,
clipboard,
wheel input,
resizing,
and history retention at the user boundary.
Include installed static and variable JetBrains Mono faces,
bold/italic styles,
emoji,
CJK,
combining characters,
and the Ghostty crash-triggering `⎿` glyph.
Check configured display scaling and both color schemes.
Verify shell exit and window close leave no child processes behind.

These are proposed acceptance checks,
not tests run in this assessment.
The scope is a dependable single-window terminal;
the user already said tabs are unnecessary.
Broader terminal features can be considered after this workflow passes.

## Alternatives and ranking

### Develop Monochromatic while retaining a working terminal

Pros:
fixes and regression tests can target the user's exact agent workflow;
the existing VT and PTY implementation can be reused.
Cons:
the frontend has functional gaps,
and the recorded crash and stutter may require integration work before feature work pays off.

### Continue with existing terminals and repair individual defects

Pros:
Rio's documented workaround already addresses its font warning;
OdyTTY has a successful user observation for Codex scrollback;
upstream terminal projects carry much of the compatibility work.
Cons:
the remaining visual and workflow issues still need individual diagnosis,
and fixes depend partly on upstream release and packaging paths.

### Switch all work to Monochromatic immediately

Pros:
real use would expose relevant defects quickly.
Cons:
the source lacks essential interaction paths,
and the historical crash and stutter remain unverified.
This would put active work behind an unqualified frontend.

Ranking:
develop Monochromatic alongside a working terminal > repair existing terminals only > immediate full switch.
The first ranks ahead of the second for sustained control over the user's workflow,
at the cost of maintenance ownership.
The second ranks ahead of the third because it retains existing complete terminal behavior while addressing known defects.
For immediate relief alone,
the verified Rio workaround and working OdyTTY session are the more direct options.

No completion-time estimate is supported by this source assessment.
The next implementation step,
if accepted,
is the baseline milestone.
