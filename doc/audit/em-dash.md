# AUDIT.em-dash.md

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/forbidden-strings` is now `package/forbidden-strings/cli`;
> `package/cli/fy` is now `package/fy/cli`;
> `package/cli/mvm` is now `package/mvm/cli`;
> `package/cli/rgffplay` is now `package/rgffplay/cli`;
> `package/cli/terminal-exec` is now `package/terminal-exec/cli`;
> `package/cli/vmsync` is now `package/vmsync/cli`.

Date:
 2026-05-10 (original audit),
 2026-05-14 (post-sweep counts)

`AGENTS.md` rule:

> No em-dashes (`—`),
>  en-dashes (`–`),
>  or their ASCII substitutes (`-`,
>  `--`)
> when used in prose as em-dashes;
>  all such uses are informal.
> Use paired commas or parentheses for asides,
>  colon for elaboration or lists,
> semicolon for linked independent clauses,
>  period for abrupt breaks.
> Use "to" for ranges.
>  Hyphens remain fine in compound words ("user-facing"),
> and `--` remains fine in CLI flags (`--watch`);
>  the ban applies only to em-dash use.

## Summary

Original audit (2026-05-10):

<table>
<thead>
<tr>
<th>Category</th>
<th>Violations</th>
<th>Files</th>
</tr>
</thead>
<tbody>
<tr>
<td>Em-dash (`—`) prose asides</td>
<td>196</td>
<td>91</td>
</tr>
<tr>
<td>En-dash (`–`) violations</td>
<td>4</td>
<td>3</td>
</tr>
<tr>
<td>ASCII `--` em-dash substitute</td>
<td>1058</td>
<td>369</td>
</tr>
<tr>
<td>ASCII `-` em-dash substitute</td>
<td>625</td>
<td>58</td>
</tr>
</tbody>
</table>

Total original violations:
 1883.

Post-sweep counts (2026-05-14,
 issue #55):

<table>
<thead>
<tr>
<th>Category</th>
<th>Outside intentional</th>
<th>Inside intentional</th>
</tr>
</thead>
<tbody>
<tr>
<td>Em-dash (`—`) MD</td>
<td>0</td>
<td>199</td>
</tr>
<tr>
<td>Em-dash (`—`) TS</td>
<td>0</td>
<td>0</td>
</tr>
<tr>
<td>En-dash (`–`) MD</td>
<td>0</td>
<td>14</td>
</tr>
<tr>
<td>En-dash (`–`) TS</td>
<td>0</td>
<td>2</td>
</tr>
<tr>
<td>ASCII `--` MD</td>
<td>preserved CLI args</td>
<td>preserved CLI args</td>
</tr>
<tr>
<td>ASCII `--` TS</td>
<td>preserved CLI args</td>
<td>preserved CLI args</td>
</tr>
<tr>
<td>ASCII `-` MD</td>
<td>preserved bullets</td>
<td>preserved bullets</td>
</tr>
<tr>
<td>ASCII `-` TS</td>
<td>preserved TSDoc</td>
<td>preserved TSDoc</td>
</tr>
</tbody>
</table>

Intentional content (preserved by exclusion list):
 `AUDIT.em-dash.md` itself
(self-references),
 `PLANNING.forbidden-strings-em-dash.md`,
`package/cli/forbidden-strings/README.md`,
 `AGENTS.md` (rule statement in
backticks),
 `GLM_LIMITATIONS.md` (documents model violations),
`HANDOVER.em-dash-sweep-issue-55.md` (handover doc with em-dash examples
in backticks),
 and `package/module/hyperscript/src/css/index.unit.test.ts`
(en-dash as CSS counter-style symbol).
 The MD `--` sweep also excluded
`TODO.claude-code-words.md` and `TODO.forbidden-strings.md` because both
intentionally use `--` as definition markers.

Remaining ASCII `--` instances are CLI argument separators preserved by the
sweep heuristic:
 lines that look like CLI invocations (`mise`,
 `git`,
`npm`,
 etc. as first token),
 text inside fenced code blocks,
 inline
backtick spans,
 and `oxlint-disable`/`eslint-disable`/`biome-disable`
directives (which use `--` as a syntactic rule/reason separator).

Remaining ASCII `-` instances are list bullets at line start (`- item`),
markdown bullets within TSDoc comments,
 subtraction in math expressions,
identifiers and CLI flags,
 dashes in compound words ("user-facing"),
TSDoc `@param name - description` separators (per TSDoc convention),
 and
mid-prose dashes the conservative marker-after-close heuristic skipped
to avoid false positives.

The remaining `forbidden-strings` rule to prevent regressions is tracked
in a separate follow-up issue.

## A. Em-dash (`—`) prose asides

Replace with paired commas,
 parentheses,
 colon,
 semicolon,
 or period.

### AUDIT.md

- Line 120:
   Unbounded recursion in namespace normalization is a theoretical DoS vector on deeply nested XML — mitigate with a
   response size limit on fetch.
- Line 159:
   runtime dependencies.
   The only two flagged patterns — an eval('') for CSP feature detection and a new Function() for function
- Line 160:
   composition optimization — are well-known,
   legitimate techniques with safe fallbacks.
   The published tarball matches the expected
- Line 178:
   found no suspicious code — no network calls,
   telemetry,
   eval,
   environment variable access,
   or postinstall scripts.
   Security is
- Line 189:
   It has no install hooks,
   no telemetry,
   no network calls,
   and no obfuscated code — the supply chain risk is negligible.
   Security
- Line 193:
   deserialization,
   which requires attacker control of the trusted payload — low risk in typical use.
   This dependency is safe to

### GLM_LIMITATIONS.md

- Line 313:
   3.
   `rg -n "—\|–"` over the diff.
   `AGENTS.md` ban applies to all human-authored content including comments,
   docstrings,
   and string literals sent to other models.

### PHILOSOPHY.css.md

- Line 6:
   Same pattern as h-xml and h-html — call `$()` with named parameters,
   get a string back.
- Line 16:
   Editors see them as plain text — no property name autocomplete,
   no value validation,
   no type checking.
- Line 30:
   or using `adoptedStyleSheets` with constructable stylesheets — both add machinery
- Line 36:
   at runtime in the browser — the browser parses JavaScript,
   generates CSS strings,
   creates DOM elements,
- Line 47:
   h-css needs no build step — it's a pure function that runs wherever TypeScript runs.
- Line 95:
   No `var()` fallbacks — the token system guarantees every custom property is defined.

### PLANNING.extract-refactor-guardrail.md

- Line 121:
   dependents-extension is a design change,
   not just implementation — call this
- Line 126:
   exit codes) have moved across releases — use the `update-config` skill when

### TROUBLESHOOTING.cli-bin.md

- Line 24:
   regardless of shebang — the problem only manifests when running through the
- Line 45:
   The shebang is a Unix mechanism — Windows ignores it entirely.
- Line 53:
   This means adding a shebang never breaks Windows — it is purely additive.

### TROUBLESHOOTING.ghostty-cursor.md

- Line 21:
   `inverse` modifier — an inverse-video space character that always appears

### TROUBLESHOOTING.pi-compaction-empty-summary.md

- Line 141:
   - Setting `compressionRatio` to any value in Morph Compact does not help —

### TROUBLESHOOTING.pi-safeguard.md

- Line 217:
   Every flagged action shows "No judge model available — manual approval
- Line 385:
   4.
   Actual:
   "No judge model available — manual approval required.
  "

### TROUBLESHOOTING.typesafe-i18n-regex-redos.md

- Line 8:
   The hang is silent — no console error,
- Line 47:
   so on input with three or more nesting levels —
- Line 48:
   exactly what a JSON schema string contains —
- Line 84:
   Run under Chrome 145 / V8 — `lookup` never returns.
- Line 85:
   Run under Bun 1.
  x or SpiderMonkey — sub-millisecond.
- Line 93:
   and `askInstruction` —
- Line 120:
   This works because the affected keys hold static strings —
- Line 128:
   Mechanical but voluminous —
- Line 138:
   Triggers the same regex on the same input —
- Line 160:
   Confirmed by building with `compress: false` —
- Line 172:
   Only three translations have nested `{}` —
- Line 175:
   `askInstruction` —
- Line 193:
   Socket — all clean as of 2026-04-29).
- Line 204:
   - No bug-fix commits have landed at `codingcommons` since the revival —
- Line 227:
   Interpolation regex is `/{{(.+?)}}/g` —
- Line 251:
   `}` —

### doc/agent/domain.md

- Line 15:
   When naming a domain concept in output (issue title,
   refactor proposal,
   hypothesis,
   test name),
   infer the project's terms from the code you just read — not from a glossary that doe
- Line 19:
   Past decisions are recorded only in git history and code structure.
   When you need to understand why something was done a certain way,
   trace the code — don't assume a summary file w

### package/build-tool/css/src/import.ts

- Line 77:
   // Try relative first — CSS treats `@import 'foo.css'` as relative.

### package/build-tool/css/src/index.ts

- Line 149:
   // Write output — uses dynamic import so browser callers don't pull in node:
  fs

### package/build-tool/css/src/mixin-registry.ts

- Line 1:
   // Slightly over 100 lines — splitting the type guard or expandApplyInNodes into
- Line 140:
   `Mixin expansion exceeded ${MAX_PASSES} passes — likely caused by circular @apply references between mixins`,

### package/build-tool/css/src/mixin.ts

- Line 10:
   // Re-export from the registry so consumers import from `mixin.ts` only —
- Line 100:
   `@apply ${mixinName} is missing its source location — parsed nodes should always have one, so PostCSS may have receiv
   ed a programmatically constructed node instead of a parsed one`

### package/build-tool/css/src/package-resolver.ts

- Line 224:
   // subpath starts with '.
  /' — strip it for join

### package/claude-code-plugin/claude-spawn/TODO.md

- Line 212:
   screen to look at — need `grim` screenshot capture for remote debugging,

### package/cli/terminal-exec/src/exec.ts

- Line 41:
   throw new Error('execvp:
   unreachable — length checked above',
  );

### package/cli/terminal-exec/src/tokenize.ts

- Line 83:
   break;
   // unreachable — length checked above
- Line 119:
   break;
   // unreachable — length checked above

### package/cli/terminal-exec/src/validate.ts

- Line 177:
   throw new Error('unreachable — length checked above',
  );

### package/config/tsdown/src/index.node.ts

- Line 32:
   // Pi extension peer deps — provided by the pi runtime at load time.
- Line 37:
   // Pi AI providers — provided by the pi runtime at load time.

### package/desktop-daemon/editord/src/client/editor/editor-pane.ts

- Line 421:
   /** MutationObserver callback — dispatches `contentchange` on any editor DOM mutation.
   */

### package/desktop-daemon/editord/src/client/editor/line-ops.ts

- Line 40:
   /** Single line — clear it instead of removing.
   */

### package/desktop-daemon/editord/src/client/editor/text-resolve.ts

- Line 63:
   /** Offset past end — clamp to last text node's end.
   */

### package/desktop-daemon/editord/src/client/file-tree/load.ts

- Line 83:
   // Always verify with a fresh fetch — prefetch cache can be stale

### package/desktop-daemon/editord/src/client/highlight/collect.ts

- Line 93:
   // Skip empty lines — text node is '\n' placeholder with no visible text

### package/desktop-daemon/editord/src/client/selection/expand.ts

- Line 71:
   /** No selection or collapsed — apply the innermost range.
   */

### package/desktop-daemon/editord/src/client/selection/shrink.ts

- Line 104:
   /** No smaller range — collapse to cursor.
   */

### package/desktop-daemon/editord/src/client/ws/client.ts

- Line 239:
   // Push notifications — no request ID

### package/desktop-daemon/editord/src/protocol.ts

- Line 154:
   //region Re-exports — message types split to stay under max-lines

### package/desktop-daemon/editord/src/server/index-routes.ts

- Line 71:
   //region Static asset serving — built client bundles from dist/client/
- Line 119:
   //region Raw file serving — media files via HTTP for native browser rendering

### package/desktop-daemon/editord/src/server/index.ts

- Line 187:
   //region WebSocket — editor communication

### package/desktop-daemon/editord/src/server/lsp/json-rpc.ts

- Line 59:
   /** Consolidated buffer — rebuilt from chunks only when needed for parsing.
   */

### package/desktop-daemon/editord/src/server/lsp/lsp-features-rename.ts

- Line 90:
   /** Plain Range response — extract the symbol text from the range as placeholder.
   */

### package/desktop-daemon/editord/src/server/operations/apply-workspace-edit.ts

- Line 206:
   /** Skip disk write for the current file — the client applies those edits.
   */

### package/desktop-daemon/hall-monitor/README.md

- Line 8:
   All computation runs locally — no data leaves the machine.
- Line 12:
   1.
   **Capture** — takes a screenshot (Spectacle) and a webcam frame (ffmpeg/v4l2),
   downscales both via ffmpeg
- Line 13:
   2.
   **Buffer** — stores captures in a 10-minute rolling in-memory buffer so the LLM can compare across time
- Line 14:
   3.
   **Analyze** — starts a local `llama-server`,
   sends the buffered captures to LFM2.5-VL-1.6B,
   receives a PRODUCTIVE / UNPRODUCTIVE verdict
- Line 15:
   4.
   **Notify** — tracks the last 5 verdicts in a sliding window;
   when all 5 are UNPRODUCTIVE,
   fires a `notify-send` critical notification with the LLM's summary,
   then resets the win
- Line 16:
   5.
   **Repeat** — the llama-server is stopped between cycles to free VRAM,
   then the loop sleeps for 5 minutes
- Line 20:
   - [Bun] — runtime and bundler
- Line 24:
   - [llama.
  cpp] — local LLM inference (`llama-server`)
- Line 26:
   - [distrobox] — container to run llama-server (used here for AMD GPU overrides)

### package/desktop-daemon/hall-monitor/src/analyze.ts

- Line 15:
   2.
   DISTRACTION (webcam):
   Assume user is looking at the screen unless — user is holding/using a phone,
   head down sleeping,
   or chair is empty.
   Looking at the ceiling or the wall does

### package/desktop-daemon/hall-monitor/src/cycle.ts

- Line 77:
   // Skip the entire cycle when the session is locked — no point capturing
- Line 80:
   log.
  debug('[cycle] Screen is locked — skipping cycle',
  );

### package/desktop-daemon/hall-monitor/src/index.ts

- Line 79:
   '[hall-monitor] Starting — capturing every 5 minutes,
   retaining last 10 minutes',
- Line 85:
   // Not needed because we don't allow configuring interval:
   defense-in-depth — add a floor (e.g. Math.
  max(INTERVAL_MS,
   60_000))

### package/dev-script/inference-canary-viewer/src/chart/axis.ts

- Line 148:
   /** ISO format:
   YYYY-MM-DDTHH:
  MM:
  SS — time starts at index 11 */
- Line 182:
   // Same year:
   show MM-DD only — skip "YYYY-" prefix

### package/dev-script/inference-canary-viewer/src/chart/data-table.ts

- Line 90:
   * Missing fix scores distinguish between failed runs ("not run" — fix was

### package/dev-script/inference-canary-viewer/src/html/view-overview.ts

- Line 68:
   // Summary table — status is shown inline rather than in its own column

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/css-mixin-transpiler-initial-2026-03-06T23-36-23.000Z/canary.ts

- Line 143:
   // Skip the entire @mixin block — do not emit it.
- Line 151:
   // Malformed — emit '@' literally and keep scanning.
- Line 173:
   // Unknown at-rule — emit '@' and re-scan from the next character so the

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/css-mixin-transpiler-initial-2026-03-16T18-27-52.000Z/canary.ts

- Line 157:
   // CSS block comment — pass through unchanged
- Line 170:
   // Quoted string — pass through unchanged
- Line 179:
   // @mixin — skip the entire block,
   output nothing
- Line 200:
   // @apply — replace with mixin body

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/css-mixin-transpiler-initial-2026-03-19T16-04-48.000Z/canary.ts

- Line 29:
   * Return value from {@link parseNodes} — the collected nodes and the position after the stop character.
- Line 159:
   // Quoted strings — do not interpret { } @ inside them

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/csv-rfc4180-initial-2026-03-07T19-14-56.000Z/canary.ts

- Line 18:
   // End of input — emit the last (possibly empty) field and stop
- Line 80:
   // Unquoted field — read until comma,
   newline,
   or end

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/expr-eval-initial-2026-03-11T14-48-24.000Z/canary.ts

- Line 162:
   // Satisfy import/unambiguous requirement — this file is a module with side effects via stdin.

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/stak-interpreter-fix-2026-04-18T00-32-58.000Z/canary.ts

- Line 1:
   /** Stak interpreter — reads a program from stdin and writes output to stdout.
   */

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/sudoku-solver-fix-2026-05-06T23-31-32.000Z/canary.ts

- Line 1:
   /** Sudoku solver CLI — reads puzzles from stdin,
   solves via backtracking.
   */

### package/dev-script/inference-canary/src/canary-lint/Sonnet 4.6/sudoku-solver-initial-2026-04-04T07-17-24.000Z/canary.ts

- Line 1:
   /** Sudoku solver CLI — reads puzzles from stdin,
   writes solutions to stdout.
   */

### package/dev-script/inference-canary/src/stak/interpreter-ops.ts

- Line 33:
   throw new Error('stack underflow — unreachable',
  );

### package/dev-script/task-util/README.md

- Line 91:
   This is intentional — `composite` provides valuable constraints
- Line 159:
   Empty globs contribute no timestamps — the aggregation strategy returns `-Infinity`

### package/dev-script/task-util/src/append.unit.test.ts

- Line 176:
   // If we get here,
   permissions aren't enforced — skip assertion

### package/dev-script/task-util/src/tsgo-filter.ts

- Line 276:
   // tsgo #2666 — stale .
  tsbuildinfo causes false negatives;
   clean before each build

### package/figma/to-penpot/src/index.ts

- Line 770:
   // For decks,
   find all SLIDE nodes — each becomes a page
- Line 785:
   // For fig/jam,
   find all CANVAS nodes — each becomes a page
- Line 789:
   // Skip "Internal Only Canvas" — it's a Figma internal canvas

### package/module/es/TODO.testing.md

- Line 8:
   but `onLoadRedirectingTo` is not a direct named export — it is nested

### package/module/es/src/path/fallbacks.ts

- Line 185:
   /** Current working directory — falls back to `/` in browser */

### package/module/es/src/types/t object/t array/f/t iterable/map/r a/p p/index.ts

- Line 38:
   // `Promise.all` only collects already-running results — it does not "activate" them.

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.startsWithComment.unit.test.ts

- Line 727:
   // In JSONC,
   */ always terminates a block comment — quotes have no special meaning inside comments

### package/module/hyperscript/src/css/disallowed-properties.ts

- Line 33:
   // Shorthand properties — always use longhand
- Line 64:
   // Non-logical dimension properties — use inline-size / block-size equivalents
- Line 73:
   // Non-logical direction properties — use logical equivalents

### package/module/hyperscript/src/css/index.types.ts

- Line 40:
   /** At-rule name — narrows `decls` to the matching descriptor type */
- Line 46:
   /** Raw CSS string to inject inside the block (NOT escaped — caller responsible) */
- Line 66:
   /** At-rule name — standard names get autocomplete,
   arbitrary strings accepted via `(string & {})` */
- Line 72:
   /** Raw CSS string to inject inside the block (NOT escaped — caller responsible) */
- Line 101:
   /** CSS declarations — strict property names,
   strict values,
   custom properties */
- Line 103:
   /** Raw CSS string to inject inside the block (NOT escaped — caller responsible) */

### package/module/hyperscript/src/css/values.constructors.ts

- Line 102:
   * `ch` is the advance width of the `0` glyph in the element's font —

### package/module/hyperscript/src/xml/index.ts

- Line 102:
   /** Raw inner XML (NOT escaped — caller is responsible for well-formedness) */
- Line 151:
   // XML has no void elements — instead,
   childless elements self-close.

### package/module/image-diff/src/client.multi.compare.ts

- Line 85:
   throw new Error('unreachable — allResults is non-empty',
  );

### package/module/test/README.md

- Line 9:
   Vitest was evaluated and rejected — it requires substantial configuration
- Line 21:
   so tests run on any JavaScript runtime that supports ESM — including browsers.

### package/module/test/src/it.ts

- Line 184:
   // — the parent awaits each `it()` promise,
   so the sandbox
- Line 190:
   // global state — those must use `concurrency: 1` with thunks.
- Line 200:
   `PASS${runLabel} — threw as expected${failsReason} (${
- Line 208:
   `FAIL${runLabel} — expected to throw but passed${failsReason} (${
- Line 229:
   `FAIL${runLabel} — expected ${String(tracker.
  expected,
  )} assertions but ${
- Line 247:
   `FAIL${runLabel} — expected at least one assertion but none were called (${

### package/module/test/src/sinon.unit.test.ts

- Line 137:
   // Sequential execution required — tests stub the same

### package/pi-plugin/morph-compact/README.md

- Line 3:
   Morph Compact integration for pi — replaces default LLM summarization with line-deletion compression at 33K tok/s.
- Line 15:
   If the env var is not set,
   the extension reads the key from `~/.pi/agent/mcp.json` —
- Line 63:
   1.
   **Speed** — Morph Compact runs at 33K tok/s,
   so re-triggering compaction is
- Line 66:
   2.
   **Drift reduction** — every compaction cycle loses some information.
   Preserving

### package/pi-plugin/morph-compact/src/api-key.ts

- Line 78:
   // File doesn't exist,
   unreadable,
   or invalid JSON — not an error

### package/pi-plugin/morph-compact/src/compaction-handler.ts

- Line 104:
   'Morph Compact:
   nothing to compact — session too small',
- Line 156:
   `Morph Compact failed: ${message} — falling back to pi default`,

### package/pi-plugin/morph-compact/src/compress-branch.ts

- Line 164:
   'Nothing to compress — session has no messages and no previous compaction',
- Line 168:
   // No new messages since last compaction — return previous
- Line 203:
   'Morph Compact returned empty output — compression failed',

### package/pi-plugin/morph-compact/src/compress-branch.unit.test.ts

- Line 61:
   /** Minimal params without messages — nothing to compress.
   */
- Line 130:
   // Sequential execution required — tests stub MorphCompactClient.
  prototype.
  compact

### package/pi-plugin/morph-compact/src/index.ts

- Line 109:
   // Tier 1:
   argv — simplest path,
   zero cleanup

### package/pi-plugin/morph-compact/src/ipc-launch.ts

- Line 109:
   // Tier 4:
   TCP localhost — zero filesystem dependency
- Line 183:
   // Best-effort cleanup — temp files in /tmp are ephemeral

### package/pi-plugin/morph-compact/src/ipc-socket-tcp.ts

- Line 27:
   /** Loopback address — no remote access.
   */
- Line 115:
   // Close server after writing — one-shot

### package/pi-plugin/morph-compact/src/ipc-socket-tcp.unit.test.ts

- Line 111:
   // Second connection should fail — server is closed

### package/pi-plugin/morph-compact/src/ipc-socket-unix.ts

- Line 118:
   // Close server after writing — one-shot

### package/pi-plugin/morph-compact/src/ipc-socket-unix.unit.test.ts

- Line 114:
   // Second connection should fail — server is closed

### package/pi-plugin/terminal-title/README.md

- Line 3:
   Terminal tab title extension for pi — shows current tool,
   session state,
   and user prompt in the terminal window title.

### package/pi-plugin/terminal-title/src/index.ts

- Line 30:
   /** Minimal context shape needed by all event handlers — just `ui.setTitle()`.
   */

### package/ssg/aquati.cat/src/content/en/link-vs-button-quiz.mdx (54 occurrences, first 20 shown)

- Line 2:
   title:
   "Link vs Button — Tricky Questions"
- Line 10:
   `<a>` and `<button>` look interchangeable:
   both are clickable,
   both can run JavaScript,
   both can be styled to look like whichever the designer prefers.
   The actual choice between th
- Line 12:
   The scenarios below are the ones that trip people up.
   A few have a clean right answer,
   a few have more than one right answer,
   and some come down to how the feature actually behaves
- Line 19:
   explanation:
   <>If you write this as <code>{`<a href="/logout">`}</code>,
   any other website can sign your users out without them knowing.
   They just put <code>{`<img src="/logout">`}
- Line 24:
   explanation:
   <>Signing out changes something important — it ends the user's session.
   Actions like that should send a <code>POST</code> or <code>DELETE</code> request (the HTTP meth
- Line 28:
   explanation:
   <>You don't need two controls here.
   As long as one of them is a link pointing at <code>/logout</code>,
   the CSRF problem is still there — any other site can trigger it
- Line 46:
   explanation:
   <>A link is for sending the user to a new URL.
   Turning shuffle on doesn't change the URL and doesn't reload the page — it just flips an on/off switch inside the
   player
- Line 51:
   explanation:
   <>A <code>{`<button>`}</code> works for a toggle,
   but a plain button can't tell a screen reader whether shuffle is currently on or off.
   Add <code>{`aria-pressed="true"
- Line 55:
   explanation:
   <>Only one control toggles shuffle.
   Showing two would confuse users and assistive tech — they wouldn't know which one to click or how two controls are
   supposed to work
- Line 60:
   explanation:
   <>Another good fit is <code>{`<input type="checkbox">`}</code>.
   A checkbox is designed for exactly this kind of two-state on/off choice.
   You can style it to look like
- Line 74:
   explanation:
   <>A link sends the user somewhere — it changes the URL or jumps to a spot on the page.
   In this scenario the URL stays the same and the table just re-orders its existin
- Line 79:
   explanation:
   <>Clicking the header triggers an action (sorting) that rearranges what's already on the screen.
   A <code>{`<button>`}</code> fits that exactly.
   Put the <code>{`<button
- Line 105:
   explanation:
   <>Right if clicking opens a tooltip or popover (a small floating box of extra info) on the same page.
   Wrong if clicking takes the user to a separate pricing page — a b
- Line 113:
   explanation:
   <>Worth knowing about:
   the <code>{`<details>`}</code> element.
   Combined with a <code>{`<summary>`}</code> inside it,
   <code>{`<details>`}</code> creates a native expand
- Line 118:
   explanation:
   <>The right element depends entirely on what happens when the user clicks.
   Opens a popover on this page → button.
   Takes the user to a separate pricing page → link.
   "Le
- Line 128:
   explanation:
   <>A link can't send the user's form data to the server.
   If they right-clicked and chose "Open in new tab,
  " the browser would just visit the next URL without the addres
- Line 137:
   explanation:
   <>One submit button per form step.
   Two buttons that both submit the same form would let the user click both by accident — and a double submit on a
   checkout can mean be
- Line 159:
   explanation:
   <>A button alone can't navigate to <code>/product/42</code> — buttons have no <code>href</code>.
   The Save action does need a button,
   but we also need something to hand
- Line 168:
   explanation:
   <>No single HTML element can both navigate to a URL and trigger a non-navigating action at the same time.
   You genuinely need a link element and a button element — the
- Line 183:
   explanation:
   <><code>{`<a href="#top">`}</code> works without any JavaScript.
   When the user clicks it,
   the browser scrolls to the top of the page because <code>#top</code> is a spe
- ... and 34 more

### package/ssg/aquati.cat/src/pages/tag.ts

- Line 73:
   description:
   `${t.siteDescription()} — ${tag}`,

### package/ssg/aquati.cat/src/templates/head.ts

- Line 59:
   //region Capo.
  js priority 11 — pragma directives
- Line 72:
   //region Capo.
  js priority 10 — title
- Line 78:
   //region Capo.
  js priority 4 — sync CSS
- Line 87:
   //region Capo.
  js priority 3 — preload
- Line 129:
   //region Capo.
  js priority 2 — deferred scripts (type=module is implicitly deferred)
- Line 138:
   //region Capo.
  js priority 0 — remaining meta and links

### package/webapp-edu/paper2vn/README.md

- Line 58:
   The live-LLM tier inside the same file uses `test.skip(!env, reason)` so those tests skip silently here — no API key
   is threaded into the container.
- Line 80:
   The Bun smoke validates the prompt-to-parsed-chapters pipeline without the cost of spinning up a headless browser,
   which makes it a good pre-commit check while developing prompts.

### package/webapp-forge/server/TROUBLESHOOTING.isomorphic-git.md

- Line 48:
   \| Sideband multiplex (pack/progress/error onto channels 1/2/3) \| **missing — `GitSideBand.mux` is commented out**
   at `src/models/GitSideBand.js:82` \| must vendor

### package/webapp-forge/server/src/server/routes/git.unit.test.ts

- Line 177:
   // The triplet has zero/zero — that's a no-op delete-when-not-present.

### package/webapp-productivity/done-h-css-test/README.md

- Line 43:
   Adopting Drizzle would mean writing some queries with the query builder and others as raw `sql` template escape
   hatches — two query styles in one codebase.

### package/webapp-productivity/done/README.md

- Line 44:
   Adopting Drizzle would mean writing some queries with the query builder and others as raw `sql` template escape
   hatches — two query styles in one codebase.

## B. En-dash (`–`) violations

Replace with "to" for ranges,
 or paired commas/parentheses for asides.

### GLM_LIMITATIONS.md

- Line 313:
   3.
   `rg -n "—\|–"` over the diff.
   `AGENTS.md` ban applies to all human-authored content including comments,
   docstrings,
   and string literals sent to other models.

### package/module/hyperscript/src/css/index.unit.test.ts

- Line 235:
   decls:
   { system:
   'cyclic',
   symbols:
   '"–"',
   suffix:
   '" "',
   },
- Line 237:
   .
  toBe('@counter-style dash{system:
  cyclic;
  symbols:
  "–";
  suffix:
  " "}',
  );

### package/ssg/aquati.cat/src/components/question-radio.ts

- Line 11:
   * hardcoded A–E letter scheme.
   Labels and explanations accept full

## C. ASCII `--` em-dash substitute prose asides

Replace with proper punctuation or restructure sentence.

### AUDIT.consistency.md

- Line 123:
   **Config packages** -- mixed approaches:
- Line 131:
   **Module packages** -- mixed approaches:
- Line 137:
   **CLI packages** -- reasonably consistent (`main` + `bin`),
   except:
- Line 142:
   **Dev-script packages** -- inconsistent:
- Line 187:
   and hyphens within slugs (`TODO.ai-auto-commit.md`),
   consistent with TROUBLESHOOTING --

### AUDIT.dry.md

- Line 1062:
   for visual consistency across surfaces") -- covered by the `HIGHLIGHT_GROUPS`
- Line 1099:
   This conforms to the project rule "never `process.exit()` -- throw errors instead;
- Line 1219:
   the original definition,
   but the resolution is the same as the single-occurrence ones --
- Line 1220:
   extract once,
   import everywhere -- so they are grouped here rather than split across
- Line 1227:
   ("1.0 MiB",
   "123 KiB").
   Uses inline `KIB`/`MIB`/`GIB` constants -- the same constants
- Line 1292:
   The same loop -- precompute total length,
   allocate once,
   copy each chunk in -- appears
- Line 1395:
   exist yet -- create it as `package/module/dom-utils` or similar,
   or add it to
- Line 1415:
   - `package/dev-script/file-enforcer/src/io/write.ts:ensureDir(filePath: string)` --
- Line 1417:
   - `package/claude-code-plugin/source/src/handlers/session-start-housekeeping.ts:ensureDir(dirPath)` --
- Line 1435:
   to back the parsing -- the audit's existing function-level section already lists
- Line 1466:
   `padEnd`.
   The wrappers do not add behaviour -- they only re-shape the call site to
- Line 1474:
   utility-shape sweep -- the resolution is opposite (collapse into the stdlib instead
- Line 1511:
   to belong outside the S3 adapter -- the same operation works for any path-shaped
- Line 1520:
   escapes `\` and `'` for embedding into single-quoted TypeScript string literals --
- Line 1554:
   `spawn({command, args}): Promise<string>` -- 11-line diff between two 50-line files
- Line 1565:
   helper -- either as a new tiny package (e.g. `module/spawn-tagged`) or as an export
- Line 1574:
   - `package/ssg/aquati.cat/src/lib/content-group.ts:groupByLang(posts)` --
- Line 1576:
   - `package/ssg/aquati.cat/src/lib/content-group.ts:groupByName(posts)` --
- Line 1578:
   - `package/ssg/aquati.cat/src/lib/content-group.ts:groupByTag(posts)` --
- Line 1580:
   - `package/ssg/aquati.cat/src/lib/content-group.ts:groupByLangThenTag(posts)` --
- Line 1582:
   - `package/desktop-daemon/editord/src/client/inlay/group-by-line.ts:groupByLine<T>({items, keyFn})` --
- Line 1652:
   export * as positional from '.
  /p p/`index.ts`';
  ` -- 16 files

### AUDIT.fallow-tools.md

- Line 31:
   - The unique signal is genuinely out-of-scope for oxlint by design --
- Line 35:
   - Without CI or PR review in this repo,
   integration is purely local --
- Line 43:
   1.
   **Bare default** -- `bunx fallow` with no config
- Line 44:
   2.
   **`fallow init` default** -- ran `bunx fallow init`,
   used the auto-generated config
- Line 45:
   3.
   **Hand-tuned `.fallowrc.json`** -- a manual config that adds entry-point patterns
- Line 48:
   4.
   **file-enforcer-generated config** -- the prototype on branch `fallow-wrap-prototype`,
- Line 55:
   cloned at `/tmp/fallow-clone/fallow` and inspected for plugin behavior --
- Line 89:
   that the rule is inherently noisy) -- dropped 47 false positives
- Line 91:
   -- dropped 56 false positives in the deeply nested type-system barrel files
- Line 218:
   `package/*/*/mise.toml` for `bun <path>` invocations -- a future iteration.
- Line 284:
   - `bunx fallow --save-baseline foo.json` and `--baseline foo.json` --
- Line 329:
   These conflict with git's ref resolution -- `git rev-parse HEAD~1`
- Line 345:
   unused class members) is real and out-of-scope for oxlint by design --

### AUDIT.md

- Line 239:
   - ~~remark-lint-*~~ -- removed;
   replaced by markdownlint-cli2 + dprint-plugin-markdown
- Line 241:
   - ~~@shikijs/transformers~~ -- removed;
   replaced by CSS Custom Highlight API with Lezer parsers

### BUG-REPORT.oxlint-ignorePatterns-cwd.md

- Line 80:
   The **LSP implementation gets this right** --

### BUG-REPORT.tsdown-docs-bundling-default.md

- Line 4:
   **Severity:
  ** documentation contradiction -- misleads users about default behavior
- Line 210:
   The actual default behavior already externalizes production deps --
- Line 263:
   `DepsPlugin` (`src/features/deps.ts`) is registered whenever a `package.json` exists
   (`src/features/rolldown.ts:115-117`).
   On initialization,
   `getProductionDeps()` (`src/features/d
- Line 269:
   The FAQ suggests `deps.skipNodeModulesBundle: true` as the solution,
   which externalizes **all** `node_modules` imports (including `devDependencies`).
   The actual default already ext

### BUG-REPORT.vlt-build-metadata.md

- Line 127:
   **Option A -- strip in `#registryManifestRequest` (minimal,
   targeted):
  **
- Line 136:
   **Option B -- strip in Spec parser (comprehensive,
   prevents downstream issues):
  **
- Line 159:
   to an exact version with build metadata -- dependency specifier strings are

### CLAUDE-LIMITATIONS.md

- Line 14:
   - "pipeline operator stuck in pipeline" -- TC39 dig;
   reads as a natural English sentence
- Line 16:
   - "sloppiest sloppy slop" -- AI-generated content dig;
   word-cycling that sounds visceral
- Line 28:
   Claude's attempts sounded like a person trying to write a joke --
- Line 80:
   **OmniSVG (NeurIPS 2025)** -- a dedicated Image-to-SVG generation model
- Line 88:
   **VLM-based visual feedback loop** -- Claude instances did use their own vision
- Line 97:
   all LLMs degrade massively on complex SVG structures --

### GLM_LIMITATIONS.md

- Line 7:
   Every extracted file in this package -- 12 of them -- claims the same justification:
- Line 217:
   The README claims the new code reads tool arguments directly;
   in fact,
   it falls back to parsing free text exactly when the tool isn't called -- the precise antipattern the README s

### PHILOSOPHY.tool-choices.md

- Line 21:
   package manager has JSR scope routing configured -- which most don't by default.
- Line 165:
   Maintainer attention splits across HTTP routing,
   JSX reconciliation,
   and SSG --
- Line 187:
   The RC label reflects API finalization,
   not instability --

### PHILOSOPHY.vm-dev-environment.md

- Line 36:
   Also inherits Fedora's third-party repo fragility --
- Line 45:
   pushing everything to Flatpak -- which conflicts with installing
- Line 50:
   Disqualified by the lack of transactional package management --
- Line 58:
   **NixOS**:
   the most declaratively pure option --
- Line 99:
   `cargo:fd-find`,
   `cargo:fastmod`,
   `cargo:llmfit` --
- Line 125:
   The monorepo already has [file-enforcer](../../package/dev-script/file-enforcer) --
- Line 147:
   send/receive,
   and compression -- encrypted data stays encrypted in snapshots and
- Line 157:
   No TPM or auto-unlock mechanism -- avoids vTPM device requirements and keeps
- Line 179:
   Flatpak sandboxing blocks direct socket access by default --
- Line 194:
   Flatpak sandboxing breaks this path --
- Line 201:
   Terra is a third-party repo packaging other people's software (third-party of third-party) --
- Line 203:
   LibreWolf's repo is maintained by the LibreWolf project itself (first-party of third-party) --

### PIPE-BUG.md

- Line 3:
   Status:
   **root cause identified** -- [upstream issue filed](https://github.com/anthropics/claude-code/issues/31968)
- Line 246:
   `rg` reads nothing from stdin and exits with code 2 ("no files were searched") --
- Line 274:
   in evalstring.
  c.
   Irrelevant -- `eval` passes `SEVAL_NOOPTIMIZE`,
   and `can_optimize_connection`

### README.md

- Line 8:
   **Minimal MCP server** --
- Line 16:
   **Inference canary** --
- Line 26:
   **Security-audited dependency selection** --
- Line 37:
   **Custom Oxlint plugins** --
- Line 46:
   **Monorepo-aware CSS build tool** --
- Line 50:
   -- all without native binaries.
- Line 52:
   **OpenTofu firewall automation** --
- Line 59:
   **Custom typeface from SVG geometry** --
- Line 181:
   Development targets Linux (Fedora).
   Use WSL2 on Windows -- some tools

### TODO.edit-tool-staleness.md

- Line 15:
   3.
   `dprint fmt` runs -- scans workspace,
   reformats the test file (1 file changed),
   `stat()`s other `.ts` files
- Line 16:
   4.
   `bun test` runs -- imports `manager-defs.ts` at runtime (read-only)
- Line 43:
   1.
   Check if Claude Code's Edit tool source is accessible (it ships with the CLI -- look in the npm package or GitHub
   repo)

### TODO.figma-alternative.md

- Line 133:
   Since rendering is DOM,
   using the browser's own layout engine is the simplest path -- but Figma's auto-layout has behaviors that do not map
   1:1 to CSS flexbox.

### TODO.forbidden-strings.md

- Line 12:
   into any committed file -- source,
   docs,
   configs,
   commit messages.
- Line 50:
   or per-file constant overhead --
- Line 86:
   There is no other syntax -- the format is deliberately minimal.
- Line 112:
   which is what the perf budget requires --
- Line 156:
   - **GitHub native push protection / custom patterns** --
- Line 159:
   - **Native git hook + `grep` script** --
- Line 161:
   - **`pre-commit/pre-commit-hooks` (Python pre-commit framework)** --
- Line 245:
   even if not yet staged --
- Line 252:
   The only residual imprecision is the inverse case --
- Line 254:
   commit before the next scan --
- Line 268:
   the rule pattern,
   or any surrounding line context in failure messages --
- Line 345:
   Cleanup requires `git filter-repo` and a force-push -- a separate,
   destructive operation
- Line 419:
   If so,
   what's the policy -- block adding the rule until history is clean,

### TODO.md

- Line 21:
   Immediate action required -- blocking development or production.

### TODO.package-structure.md

- Line 54:
   - **`test-css-*` packages** -- use `test-` prefix instead of matching their directory category

### TODO.vm-dev-environment.md

- Line 63:
   not a separate package -- the provisioner is small enough to colocate
- Line 236:
   No image rebuild needed -- just re-run the provisioner in the existing VM.

### TROUBLESHOOTING.bash.md

- Line 14:
   suggesting concurrent execution -- but the commands were actually sequential.

### TROUBLESHOOTING.bun-fetch-streaming.md

- Line 107:
   \| undici fetch (bypasses Bun's native fetch) \| Unclear -- Bun polyfills Node APIs \| Untested \|
- Line 109:
   OpenRouter's Responses API (`/api/v1/responses`) supports `stream: true` but returns SSE over HTTP POST --

### TROUBLESHOOTING.bun-fs-glob-dotfiles.md

- Line 3:
   Status:
   **root cause identified** -- upstream issue filed:
   [oven-sh/bun#28021](https://github.com/oven-sh/bun/issues/28021)
- Line 96:
   Wildcard exclusion of dot files is handled by minimatch/glob pattern semantics,
   not by the `dot` flag --

### TROUBLESHOOTING.bundling.md

- Line 27:
   When a workspace package contains dynamic `import('node:...')` calls --
- Line 28:
   even inside functions guarded by try-catch --
- Line 38:
   so the application works correctly -- but the browser has already printed
- Line 93:
   - Do not rely on dynamic import failure as the sole browser detection mechanism --

### TROUBLESHOOTING.cLikeComments.md

- Line 29:
   This is not a bug in our parser -- it matches the behavior of every
- Line 41:
   but this heuristic is incomplete -- it can't distinguish between
- Line 49:
   - `customParsers.startsWithComment.unit.test.ts:396-410` --
- Line 51:
   - `customParsers.startsWithComment.unit.test.ts:413-428` --
- Line 56:
   - `customParsers.startsWithComment.unit.test.ts:529-538` --
- Line 62:
   The subset of comment patterns that hit this edge case --
- Line 64:
   a `/* */` block -- is vanishingly rare in real JSONC configuration files.

### TROUBLESHOOTING.css-hidden-attribute-specificity.md

- Line 41:
   Project conventions ban `!important`.
   Source-order ordering achieves the same outcome with the natural cascade and stays maintainable -- a future
   contributor reading the stylesheet

### TROUBLESHOOTING.css-tooling.md

- Line 352:
   This is correct behavior,
   but it means testing both strategies requires two separate imported fixture packages -- one with `exports` and one
   without.

### TROUBLESHOOTING.dependencies.md

- Line 20:
   Graph modifiers (`"modifiers"` in `vlt.json`) do not help either --
- Line 35:
   This is extremely rare -- npm stripping `+` from versions means the public registry
- Line 55:
   **Entry point -- dependency spec parsing:
  **
- Line 60:
   **Trigger -- single-version fast path:
  **
- Line 73:
   **Failure -- URL construction:
  **
- Line 83:
   In URL semantics,
   `+` is not a path-safe character --
- Line 87:
   **Second trigger -- extraction bypass of modifiers:
  **
- Line 103:
   to the graph-building phase -- the extraction code path at `extract-node.ts`
- Line 112:
   The vlt issue tracker (vltpkg/vltpkg) has no reports matching this pattern --
- Line 136:
   - **Use a different package manager** (npm,
   pnpm,
   yarn) for installs --

### TROUBLESHOOTING.dprint-exec.md

- Line 11:
   but never formats any file -- the `"Formatted file"` log entry lacks the `(Plugin N/M)`
- Line 33:
   the extension fallback picks whichever plugin was registered first -- determined by the
- Line 65:
   formatting for `.scss`,
   `.less`,
   and `.sass` files -- malva would refuse to handle them
- Line 150:
   **original on-disk content** -- it never sees the previous plugin's output.
- Line 186:
   but adding that complexity to the format pipeline is not worth the marginal gain --

### TROUBLESHOOTING.figma-browser-automation.md

- Line 6:
   The design canvas is rendered entirely in WebGL -- there is no DOM representation of frames,
   layers,
   text,
   or interactive elements inside the canvas.
- Line 59:
   Clunky,
   slow,
   and requires manual effort -- but actually works.
- Line 82:
   Figma chose WebGL for performance -- rendering thousands of vector objects in the DOM would be unusable.

### TROUBLESHOOTING.hetzner-firewall.md

- Line 17:
   This is intentional -- add an outbound ICMP rule in `hetzner.tf` if ping is needed for debugging.

### TROUBLESHOOTING.ios-safari-touch.md

- Line 26:
   The bug was filed in **2014** and resolved as "CONFIGURATION CHANGED" -- not "FIXED".
- Line 125:
   The failure is silent -- no console error,
   no warning,
   no indication that the CSS property

### TROUBLESHOOTING.mdx.md

- Line 157:
   The problem is not that `_components['callout-alert']` is missing --

### TROUBLESHOOTING.mise-watch.md

- Line 78:
   `Modify(Data(Any))` -- a real data write event -- because it cannot distinguish
- Line 85:
   **Layer 1:
   `--no-meta`** -- suppresses `Modify(Metadata(Any))` events at the
- Line 88:
   **Layer 2:
   `-j @content-changed.jaq`** -- a jaq filter program that compares
- Line 103:
   \| `touch` (mtime only) \| Suppressed \| -- \| No restart \|
- Line 104:
   \| `chmod` (perms only) \| Suppressed \| -- \| No restart \|
- Line 129:
   and cannot reconnect -- every reconnect attempt fails with "unauthorized".

### TROUBLESHOOTING.oxlint.md

- Line 10:
   The `lint:oxlint` task template in `mise.toml` handles this automatically --
- Line 24:
   `contains()` calls with bare,
   `typescript-eslint/`,
   and `@typescript-eslint/` prefixes --

### TROUBLESHOOTING.pi-safeguard.md

- Line 106:
   a home directory -- the flagger sends the action to the judge regardless,
- Line 129:
   -- every `read`,
   `write`,
   and `edit` tool call is blocked.
- Line 292:
   - `@earendil-works/pi-coding-agent` `core/model-registry.ts` --

### TROUBLESHOOTING.rg.md

- Line 33:
   The spaces in directory names had no effect --
- Line 55:
   The actual cause -- a content pattern that doesn't match -- is invisible

### TROUBLESHOOTING.rolldown.md

- Line 78:
   The OXC team has stated this is intentional --
- Line 141:
   - Adding every transitive dependency to `deps.alwaysBundle` --
- Line 177:
   But subpath specifiers like `node:path/posix` bypass that externalization --
- Line 218:
   - Adding `node:path/posix` to rolldown's `external` array --
- Line 220:
   - Setting `resolve.builtinModules` manually --

### TROUBLESHOOTING.testing.md

- Line 13:
   When two sibling describes share the same `name`,
   the error chain is ambiguous --

### TROUBLESHOOTING.toml.md

- Line 16:
   duplicate table headers with a parse error.
   This is **not** a footgun --
- Line 117:
   TOML has no equivalent -- every value is a standalone literal.

### TROUBLESHOOTING.tsdown.md

- Line 25:
   For `terminal-title` (8 DTS source files):
   55ms with DTS,
   22ms without -- ~33ms DTS overhead.
- Line 66:
   Rolldown discovers imports incrementally -- each layer of the dependency graph
- Line 74:
   the resolved options confirm `oxc: { stripInternal: false, sourcemap: false }` --

### TROUBLESHOOTING.typescript.md

- Line 292:
   - Combining multiple null checks into one `if` guard --
- Line 297:
   - Adding `as HTMLDivElement` --
- Line 315:
   The errors are all `\| undefined` narrowing failures --
- Line 359:
   This priority order is hardcoded -- no tsconfig option changes it.
- Line 397:
   - [microsoft/TypeScript#41883](https://github.com/microsoft/TypeScript/issues/41883) --
- Line 401:
   - [microsoft/TypeScript#44205](https://github.com/microsoft/TypeScript/issues/44205) --
- Line 404:
   - [microsoft/TypeScript#48779](https://github.com/microsoft/TypeScript/issues/48779) --
- Line 407:
   - [microsoft/TypeScript#40426](https://github.com/microsoft/TypeScript/issues/40426) --
- Line 460:
   and `skipLibCheck` would cover it.
   But you need a source for the `.d.ts` types --
- Line 631:
   The SVG enters the project through a path that bypasses this extension filter --
- Line 641:
   1.
   **Spawn trigger** -- when a non-source file is the first file opened
- Line 646:
   2.
   **Reuse + feature request** -- when tsgo is already running
- Line 654:
   tsgo does NOT crash from its own directory scanning --
- Line 660:
   **Include filter in `resolve()` gating ALL tsgo access** (`lsp-pool.ts`) --
- Line 662:
   before returning ANY tsgo client -- both reuse of existing clients and new spawns.
- Line 669:
   **Crash recovery with ScriptKind-aware retry** (`lsp-pool.ts`,
   `lsp-client.ts`) --
- Line 676:
   **Base tsconfig `exclude` for non-source extensions** (`tsconfig.options.json`) --
- Line 682:
   - **tsconfig `include`/`exclude` patterns in LSP mode** --
- Line 689:
   - **Shadow root / symlink directory** --
- Line 693:
   - **Restarting without any delay** --
- Line 701:
   1.
   **Filtering only in `resolveAll`** --
- Line 704:
   2.
   **Filtering only before spawning in `resolve()`** --
- Line 719:
   - [microsoft/typescript-go#2669](https://github.com/microsoft/typescript-go/issues/2669) --
- Line 721:
   - [denoland/deno#31423](https://github.com/denoland/deno/issues/31423) --
- Line 723:
   - [neovim/nvim-lspconfig#4018](https://github.com/neovim/nvim-lspconfig/issues/4018) --

### TROUBLESHOOTING.vlt-jsr.md

- Line 74:
   vlt already knows JSR registries are different -- it skips tarball URL guessing

### file-enforcer.config.ts

- Line 59:
   // write a literal into this committed config -- otherwise the

### oxlint.config.ts

- Line 7:
   * Uses spread instead of `extends` because `extends` only merges rules --

### package/audit/oph-common-look-and-feel/README.md

- Line 19:
   All screenshots are embedded as base64 AVIF data URIs -- no server required.

### package/build-tool/css/README.md

- Line 9:
   1.
   **Monorepo-aware `@import` resolution** -- PostCSS only resolves relative paths out of the box,
   not `node_modules` or `package.json` `exports`
- Line 10:
   2.
   **Custom `@mixin`/`@apply` processing** -- no standard PostCSS plugin provides the mixin semantics this monorepo
   needs
- Line 11:
   3.
   **Browser-compatible** -- the entire pipeline runs in both Node.
  js and browser environments (no native binary dependencies)
- Line 13:
   The package uses only **PostCSS** for all CSS processing -- AST walking for `@import` inlining,
   `@mixin` collection,
   and `@apply` expansion.
- Line 90:
   For consumers that already have CSS text in memory -- such as web components with Shadow DOM styles defined as
   JavaScript strings -- use `applyMixins()`:
- Line 117:
   1.
   **Resolve and bundle** -- a custom PostCSS plugin walks `@import` statements,
   resolves specifiers (relative paths,
   `package.json` `exports`,
   bare `node_modules`),
   and inlines the
- Line 118:
   2.
   **Collect mixins** -- PostCSS walks the bundled AST,
   extracts `@mixin` definitions into a registry,
   removes them from the tree
- Line 119:
   3.
   **Expand mixin bodies** -- nested `@apply` rules inside mixin definitions are resolved via fixed-point iteration
- Line 120:
   4.
   **Inline `@apply`** -- remaining `@apply` rules in the document are replaced with cloned mixin body nodes
- Line 121:
   5.
   **Write output** -- final CSS string written to disk

### package/build-tool/css/src/cli.ts

- Line 11:
   //region CLI -- parses args and runs the build

### package/build-tool/css/src/index.ts

- Line 31:
   //region Re-exports -- public API surface for consumers importing from `build.ts`

### package/build-tool/css/src/mixin-registry.ts

- Line 23:
   //region Mixin Registry -- stores mixin definitions and expands nested @apply references

### package/claude-code-plugin/README.md

- Line 97:
   1.
   session-start-housekeeping -- single event,
   single file,
   simplest.
   **Done.
  **
- Line 98:
   2.
   stop-reminders -- single event,
   multi-file logic.
   **Done.
  **
- Line 99:
   3.
   bash-output-filter -- single event,
   multi-file logic.
   **Done.
  **
- Line 100:
   4.
   terminal-title -- multi-event,
   multi-file logic.
   **Done.
  **
- Line 101:
   5.
   claude-spawn -- six events plus the user-facing `spawn-claude` CLI bin.
- Line 107:
   6.
   research-agent -- not a hook handler;
   ships only an agent definition.
   Its
- Line 184:
   The deepening that matters -- handler logic centralized in one source package,
- Line 185:
   runtime shared across plugins,
   tests colocated with handlers -- is achieved

### package/claude-code-plugin/bash-output-filter/README.md

- Line 40:
   1.
   **Git file mode lines** -- strips `create mode`,
   `delete mode`,
   `copy mode`,
- Line 42:
   2.
   **Git transport progress** -- strips `Enumerating objects`,
   `Counting objects`,
- Line 45:
   3.
   **Long line truncation** -- lines over 500 characters are truncated with
- Line 47:
   4.
   **Consecutive duplicate collapsing** -- 3+ identical consecutive lines become
- Line 49:
   5.
   **Trailing whitespace** -- spaces and tabs at end of lines are removed

### package/claude-code-plugin/bash-output-filter/TROUBLESHOOTING.md

- Line 31:
   `{` alone is not a valid command -- it must be parsed as part of `{ ...; }` compound syntax,
- Line 53:
   root cause -- `$PIPESTATUS` is still evaluated across a `;` boundary.
- Line 173:
   This is the same behavior as the original approach and all subsequent attempts --

### package/claude-code-plugin/guardrail/README.md

- Line 17:
   1.
   **General-purpose blocking** -- when `subagent_type` is missing or `"general-purpose"`,
- Line 21:
   2.
   **Resume blocking** -- when the call includes a `resume` parameter,
- Line 42:
   **`ccgr`** -- Claude Code GuardRail

### package/claude-code-plugin/prompt-time/README.md

- Line 39:
   The format intentionally omits seconds,
   date,
   and timezone -- Claude already has the date in the system prompt,
   and seconds add noise without value at human conversation cadence.

### package/claude-code-plugin/research-agent/README.md

- Line 21:
   It does **not** replace Explore for local codebase searches --

### package/claude-code-plugin/research-agent/TROUBLESHOOTING.md

- Line 16:
   4.
   The page returns navigation chrome and "Uh oh!
   There was an error while loading" --
- Line 31:
   This is a model-level behavior -- Haiku optimizes for speed and confidence over accuracy.

### package/claude-code-plugin/research-agent/agents/research.md

- Line 8:
   Do NOT use for simple local codebase searches -- use Grep/Glob directly for those.
- Line 75:
   fail to load discussion comments -- they return "Uh oh!
   There was an error while loading" placeholders.

### package/claude-code-plugin/source/src/handlers/bash-output-filter/filter.ts

- Line 80:
   /* stdin already consumed or unavailable -- nothing to pass through */

### package/claude-code-plugin/source/src/handlers/bash-output-filter/index.ts

- Line 32:
   * filter script.
   The filter script lives next to the bundled hook entry --

### package/claude-code-plugin/source/src/handlers/claude-spawn/hook-session-start.ts

- Line 94:
   /** Genuine child -- claim ownership by filling in session identity.
   */
- Line 100:
   /** File missing (stale env,
   already `.reported`) or unreadable -- skip.
   */
- Line 113:
   // Not on PATH -- attempt auto-setup.

### package/claude-code-plugin/source/src/handlers/claude-spawn/index.ts

- Line 80:
   /** File missing (already `.reported`) or unreadable -- skip.
   */

### package/claude-code-plugin/source/src/handlers/claude-spawn/inject.ts

- Line 59:
   * invocations.
   When `consume` is false,
   reads the state without renaming --

### package/claude-code-plugin/source/src/handlers/claude-spawn/session-finder.ts

- Line 52:
   // No coordination file for this PID -- walk up to its parent.
- Line 73:
   // Cannot read /proc -- platform limitation or process already exited.

### package/claude-code-plugin/source/src/handlers/guardrail.ts

- Line 70:
   'Specialized agent types (Explore,
   Plan,
   etc.) are allowed -- set subagent_type explicitly.
  ',
- Line 89:
   'Do not poll or resume running agents -- wait for the notification.
  ',

### package/claude-code-plugin/source/src/handlers/stop-reminders/index.ts

- Line 59:
   'This may be a false positive -- use your judgement.
  ',

### package/claude-code-plugin/source/src/handlers/terminal-title/index.ts

- Line 110:
   /* /dev/tty unavailable -- running inside sandbox or non-interactive context.
   */

### package/claude-code-plugin/statusline/README.md

- Line 120:
   When everything is comfortable,
   nothing extra is shown -- no news is good news.
- Line 165:
   No runtime-specific APIs -- works with Bun,
   Node,
   or Deno.
- Line 186:
   supports only the `agent` and `subagentStatusLine` keys --

### package/claude-code-plugin/statusline/statusline.ts

- Line 266:
   //region Activity word -- gerund extraction from transcript
- Line 423:
   * and finds the last gerund in it.
   No JSON parsing needed --

### package/claude-code-plugin/terminal-title/README.md

- Line 9:
   Tool titles reflect tense -- present during execution (PreToolUse),
   past after completion (PostToolUse).

### package/claude-code-plugin/verbose-tool-output/WONTFIX.md

- Line 82:
   The data is already present in the conversation -- only the renderer suppresses it.

### package/cli/fy/src/cli.unit.test.ts

- Line 75:
   //region Function calls -- calling exported functions with arguments
- Line 108:
   //region Non-function exports -- accessing values without calling
- Line 130:
   //region Default export -- accessing default export via "default" keyword
- Line 156:
   //region Error cases -- non-existent exports,
   type mismatches,
   bad specifiers
- Line 188:
   //region Help -- verifies --help output
- Line 202:
   //region Missing arguments -- verifies parser errors

### package/cli/fy/src/index.ts

- Line 21:
   //region Arg parsing -- positional:
   <specifier> <export> [args...]
- Line 67:
   //region Main execution -- resolve,
   import,
   call,
   print

### package/git-policy/cli/README.md

- Line 7:
   **Require root** -- rejects commands when the working directory is not the root
- Line 11:
   **Atomic push** -- injects `--atomic` into `git push` commands automatically,

### package/git-policy/cli/src/index.ts

- Line 14:
   //region Rule pipeline -- validate and transform args before forwarding to real git
- Line 48:
   //region Execution -- resolve real git,
   apply rules,
   spawn

### package/git-policy/cli/src/rules/require-root.ts

- Line 76:
   rl.
  debug('config with global/system/list flag -- exempt',
  );

### package/cli/mvm/README.md

- Line 5:
   VMs exist from creation until destruction -- no pause,
   stop,
   or snapshot lifecycle.
- Line 59:
   2.
   Install `qemu-guest-agent` inside the guest and enable it to start on boot --

### package/cli/mvm/TROUBLESHOOTING.virtiofs-windows.md

- Line 12:
   1.
   **viofs kernel driver** -- from the virtio-win ISO,
   installed via `pnputil`
- Line 17:
   4.
   **QEMU guest agent** -- from `qemu-ga-x86_64.msi` on the virtio-win ISO,
- Line 63:
   `msiexec` never launched.
   Not "launched and failed" -- never launched.

### package/cli/mvm/src/index-parsers-cmds.ts

- Line 39:
   //region Shared value parsers -- reusable metavar-labeled string parsers

### package/cli/mvm/src/index-parsers.ts

- Line 24:
   //region Result types -- discriminated union for subcommand dispatch

### package/cli/mvm/src/index.ts

- Line 24:
   //region Verbose flag -- stripped before parsing;
   logger detects it from raw process.
  argv at import time
- Line 64:
   //region Dispatch -- parse argv and route to the appropriate handler

### package/cli/mvm/src/virsh-wait.ts

- Line 127:
   // This is expected behavior -- the VM is shutting down.

### package/cli/rgffplay/src/index.ts

- Line 28:
   //region Glob pattern construction -- case-insensitive first letter per word
- Line 75:
   //region Music directory resolution -- XDG_MUSIC_DIR or xdg-user-dir fallback
- Line 201:
   //region Playback -- ffplay with matched files
- Line 211:
   //region Main execution -- parse args,
   find files,
   play

### package/cli/terminal-exec/src/launch.ts

- Line 101:
   /** Resolve on next tick -- if spawn failed,
   the error event fires synchronously.
   */

### package/cli/vmsync/HANDOVER.md

- Line 28:
   1.
   **mise in the Windows template** -- the current approach embeds `mise.exe` in the autounattend ISO,
- Line 32:
   2.
   **Simpler alternative:
   install mise at test runtime** -- since we're dropping winget,
- Line 48:
   3.
   **VirtioFsSvc drive mapping** -- the Windows test assumes virtiofs maps to `Z:\`.

### package/cli/vmsync/README.md

- Line 42:
   2.
   QEMU boots from the overlay -- all writes go to the overlay,
   reads fall through to the base

### package/cli/vmsync/src/boot.unit.test.ts

- Line 9:
   //region parseMemoryToBytes -- converts human-readable memory strings to byte counts

### package/cli/vmsync/src/config.unit.test.ts

- Line 15:
   //region validateName -- rejects unsafe VM names,
   accepts safe ones
- Line 141:
   //region stripJsoncComments -- removes comments while preserving string content
- Line 252:
   //region vmDir / vmConfigPath -- path construction from VM name
- Line 289:
   //region detectHypervisor -- platform-based hypervisor detection

### package/cli/vmsync/src/import.unit.test.ts

- Line 9:
   //region nameFromPath -- derives VM names from image file paths

### package/cli/vmsync/src/index-parsers.ts

- Line 21:
   //region Result types -- discriminated union for subcommand dispatch

### package/cli/vmsync/src/index.ts

- Line 22:
   //region Verbose flag -- stripped before parsing;
   logger detects it from raw process.
  argv at import time
- Line 61:
   //region Dispatch -- parse argv and route to the appropriate handler

### package/cli/vmsync/src/lifecycle.expensive.unit.test.ts

- Line 451:
   // only runs during boot,
   not status -- but the binary runs on Windows,

### package/cli/vmsync/src/sync.ts

- Line 86:
   /** Regions at depth 0 with actual data -- these were written during the boot session.
   */

### package/cli/vmsync/src/types.ts

- Line 17:
   //region VM configuration -- persisted as `vmsync.jsonc`

### package/oxlint-plugin/no-restricted-syntax/src/rule/no-variable-function-expression.ts

- Line 11:
   * Patterns like `const myFn = function myFn() {}` are redundant --

### package/oxlint-plugin/stylistic/README.md

- Line 38:
   **Union/intersection types excluded** -- a `union-per-line` rule was prototyped but dropped.
- Line 42:
   **Minimum 2 items** -- single-item constructs are never flagged.
- Line 45:
   **Shared implementation** -- all rules delegate to `checkItemsPerLine` in the utility layer.

### package/oxlint-plugin/stylistic/TODO.md

- Line 15:
   4.
   Verify `nano-spawn` subprocess stdout/stderr handling -- a full buffer with no reader on stderr could deadlock
- Line 18:
   7.
   Look at the `afterEach` cleanup -- `unlinkSync` on a non-existent file in the catch block should be fine,
   but verify

### package/oxlint-plugin/stylistic/src/index.ts

- Line 40:
   //region Per-line rules -- enforce one item per line in multi-element constructs

### package/oxlint-plugin/stylistic/src/oxlint-stylistic.unit.test.ts

- Line 85:
   // oxlint exits non-zero when violations are found -- capture stdout from the error
- Line 148:
   //region Valid fixtures -- expect zero stylistic violations
- Line 179:
   //region Invalid fixtures -- expect specific violations

### package/oxlint-plugin/tsdoc/src/oxlint-tsdoc.unit.test.ts

- Line 60:
   // oxlint exits non-zero when violations are found -- capture stdout from the error
- Line 103:
   //region Valid fixtures -- expect zero tsdoc violations
- Line 134:
   //region Invalid fixtures -- expect specific violations

### package/oxlint-plugin/tsdoc/src/rule/jsdoc-map.ts

- Line 23:
   'Remove @type -- TypeScript handles types.
  ',
- Line 27:
   'Remove @typedef -- use TypeScript type alias instead.
  ',
- Line 31:
   'Remove @callback -- use TypeScript type alias instead.
  ',
- Line 35:
   'Remove @property -- use TypeScript type members instead.
  ',
- Line 39:
   'Remove @prop -- use TypeScript type members instead.
  ',
- Line 43:
   'Remove @memberof -- not needed in TSDoc.
  ',
- Line 47:
   'Remove @augments -- use TypeScript extends instead.
  ',
- Line 51:
   'Remove @extends -- use TypeScript extends instead.
  ',
- Line 55:
   'Remove @class -- use TypeScript class syntax instead.
  ',
- Line 59:
   'Remove @constructor -- use TypeScript class syntax instead.
  ',
- Line 63:
   'Remove @function -- not needed in TSDoc.
  ',
- Line 67:
   'Remove @method -- not needed in TSDoc.
  ',
- Line 71:
   'Remove @namespace -- use TypeScript namespace instead.
  ',
- Line 75:
   'Remove @module -- use @packageDocumentation instead.
  ',
- Line 79:
   'Remove @member -- not needed in TSDoc.
  ',
- Line 83:
   'Remove @var -- not needed in TSDoc.
  ',
- Line 87:
   'Remove @global -- not needed in TSDoc.
  ',
- Line 91:
   'Remove @enum -- use TypeScript enum instead.
  ',
- Line 95:
   'Remove @lends -- not needed in TSDoc.
  ',
- Line 99:
   'Remove @fires -- not needed in TSDoc.
  ',
- Line 103:
   'Remove @listens -- not needed in TSDoc.
  ',
- Line 107:
   'Remove @mixes -- not needed in TSDoc.
  ',
- Line 111:
   'Remove @mixin -- not needed in TSDoc.
  ',
- Line 115:
   'Remove @interface -- use TypeScript interface instead.
  ',

### package/config/oxlint/README.md

- Line 18:
   Uses spread instead of `extends` because `extends` only merges rules --

### package/config/oxlint/src/overrides.ts

- Line 66:
   //region import -- Declaration files are ambient;
   module-system rules don't apply.
- Line 71:
   //region typescript -- Declaration files describe external shapes that violate source conventions.
- Line 86:
   //region eslint -- No runtime code exists in declaration files.
- Line 95:
   //region no-restricted-syntax -- External API signatures don't follow source conventions.
- Line 104:
   //region tsdoc -- Ambient declarations are often trivial stubs.

### package/config/oxlint/src/rule/correctness.ts

- Line 17:
   //region jest -- Suppress leaked jest rules from vitest plugin internals.

### package/config/oxlint/src/rule/restriction.ts

- Line 117:
   // No eval() or Function() constructor -- arbitrary code execution.
- Line 120:
   // No setTimeout/setInterval with string arguments -- implied eval.
- Line 123:
   // No new Boolean/String/Number wrapper objects -- use primitives.
- Line 126:
   // No alert/confirm/prompt -- use promise-based UI alternatives.
- Line 132:
   // No comma operator -- obscures evaluation order.
- Line 135:
   // No arguments.
  caller/arguments.
  callee -- deprecated and non-optimizable.
- Line 138:
   // No labels that shadow variable names -- confusing.
- Line 141:
   // No instanceof Array -- use Array.
  isArray() which works across realms.

### package/config/oxlint/src/rule/style.ts

- Line 151:
   //region stylistic -- one-item-per-line enforcement

### package/config/oxlint/src/rule/tsdoc.ts

- Line 44:
   // Only TSDoc-standard tags allowed.
   Unescaped @ in prose is flagged -- escape as \@.
- Line 53:
   // No JSDoc-style {Type} annotations in TSDoc -- TypeScript handles types.

### package/desktop-daemon/editord/PHILOSOPHY.md

- Line 100:
   and click-outside dismissal -- the browser handles all three natively.
- Line 109:
   not per mount path -- remounting the same disk at a different path

### package/desktop-daemon/editord/README.md

- Line 23:
   The browser's compositor thread owns scroll entirely -- no `preventDefault`,
   no JS scroll reimplementation.
- Line 60:
   editord is a file I/O and search service -- no OT,
   no CRDT,
   no sync protocol.

### package/desktop-daemon/editord/TODO.md

- Line 5:
   - ~~**scoping search to last focused directory**~~ --
- Line 8:
   - ~~**Escape requires two presses to close search overlay**~~ --

### package/desktop-daemon/editord/src/client/editor/auto-indent.ts

- Line 103:
   // into a single undo entry -- one Ctrl+Z undoes both.
   The grouping mechanism

### package/desktop-daemon/editord/src/client/editor/editor-pane-commands.ts

- Line 11:
   * on `EditorPane` that did nothing but forward `{ pane: this }` --

### package/desktop-daemon/editord/src/client/editor/query.ts

- Line 9:
   * live here because they are read-only queries over protocol types --

### package/desktop-daemon/editord/src/client/keybinding/keyboard-lock.ts

- Line 21:
   //region Keyboard Lock API type augmentation -- not yet in lib.
  dom.
  `d.ts`

### package/desktop-daemon/editord/src/server/lsp/tsconfig-includes.ts

- Line 8:
   * tsgo that fall outside the project's declared include scope --
- Line 106:
   /** Return empty array on failure -- caller should allow the file through as a safe fallback.
   */

### package/desktop-daemon/editord/src/server/operations/token-file.ts

- Line 8:
   * {@link FRESHNESS_THRESHOLD_MS} of now,
   the token is reused --
- Line 38:
   * to be considered "fresh" -- i.e. left by a process that was alive

### package/dev-script/catalog-tighten/src/version-parse.ts

- Line 125:
   // Same major.
  minor.
  patch -- compare prerelease
- Line 130:
   // Installed is a prerelease of the same triple -- not greater

### package/dev-script/catalog-tighten/src/version-resolve.ts

- Line 119:
   // package/ dir not found -- return empty

### package/dev-script/file-enforcer/README.md

- Line 4:
   Uses direct async function calls instead of a descriptor/engine pattern -- each call reads and writes immediately.

### package/dev-script/file-enforcer/TODO.md

- Line 21:
   No graceful shutdown on SIGINT/SIGTERM -- open file watchers and AbortControllers are never cleaned up.
- Line 36:
   The current direct execution model was chosen explicitly to avoid this complexity -- add only if there is a real use
   case.

### package/dev-script/file-enforcer/src/cli.ts

- Line 5:
   //region CLI entry point -- finds and imports file-enforcer.
  `config.ts`,
   optionally watches
- Line 29:
   // Importing the config executes it -- the config uses top-level await

### package/dev-script/file-enforcer/src/io/cache.unit.test.ts

- Line 55:
   // Modify file on disk -- cache should still return old content

### package/dev-script/file-enforcer/src/io/glob.ts

- Line 41:
   // No wildcards -- treat entire pattern as a literal path
- Line 80:
   * Returned paths preserve the prefix format of the input pattern --
- Line 168:
   // Walk the source path,
   peeling off fixed prefixes to isolate wildcard captures --

### package/dev-script/file-enforcer/src/io/write.unit.test.ts

- Line 109:
   /** Same content now -- should NOT record timestamp */

### package/dev-script/file-enforcer/src/package/ensure-package.unit.matrix.test.ts

- Line 38:
   /** Shape:
   string shorthand -- binary = effname = package name everywhere */

### package/dev-script/file-enforcer/src/package/mise.generate-index.ts

- Line 272:
   ' * Do not edit manually -- run the index generator to rebuild.
  ',

### package/dev-script/file-enforcer/src/watch/notify.ts

- Line 26:
   * Result is cached for the lifetime of the process --

### package/dev-script/file-enforcer/src/watch/watch-dir.ts

- Line 53:
   // for-await is the only way to consume an AsyncIterable from fs.
  watch --

### package/dev-script/file-enforcer/src/watch/watch-filter.unit.test.ts

- Line 164:
   /** Now modify the file -- its mtime will be "now",
   after our recorded timestamp */
- Line 189:
   // No trackWriteTime -- simulates content-based skip

### package/dev-script/file-enforcer/src/watch/watch.ts

- Line 58:
   // Debounce state -- `let` needed because the timer is replaced on each event
- Line 164:
   // Block forever -- watch mode runs until the process is killed.

### package/cli/forbidden-strings/PERF.md

- Line 4:
   Numbers below are not aspirational targets -- they are reproducible measurements
- Line 103:
   per-byte throughput on betterleaks-shape patterns.
   Mono -- the actual
- Line 104:
   CI workload -- is at 37 ms,
   ~14x under any reasonable budget.
- Line 224:
   1.
   **Case-sensitive AC** -- emits user-authored literal-rule hits AND queues
- Line 226:
   (e.g. `\b(p8e-(?i)[a-z0-9]{32})` -- the leading `p8e-` is case-sensitive).
- Line 227:
   2.
   **Case-insensitive AC** (`AhoCorasickBuilder::ascii_case_insensitive(true)`) --
- Line 230:
   here).
   Literal rules NEVER live in this bucket -- user literals are always
- Line 277:
   bytes-mode without unicode -- the parser treats them as the matching
- Line 278:
   byte sequence -- so they take the fast path.
   Lives in
- Line 461:
   horse race -- the tools serve different use cases and the numbers
- Line 502:
   Betterleaks `dir` walks the entire directory tree -- it does not respect
- Line 513:
   volume difference,
   not the engine -- but the data volume difference is
- Line 619:
   **Results** (do not overwrite -- regressions need history) when **any** of:
- Line 633:
   Do not re-derive that analysis on every session -- read the plan,
   then decide.

### package/cli/forbidden-strings/README.md

- Line 6:
   That breaks down when the forbidden literals would themselves leak if committed --
- Line 34:
   is probably a better fit -- larger ecosystem,
   SARIF output,
   GitHub-native code-scanning upload,
- Line 37:
   needs operators PCRE doesn't have" -- pick it when one of those two is the binding constraint.
- Line 101:
   Pipe via `printenv > file` rather than interpolating the secret into a `run:` block --
- Line 134:
   ~3.0e-4 respectively -- comfortably under 1 across realistic repo sizes and noise types.
- Line 152:
   Combined,
   these express "match X but not Y" without lookaround.
   Example -- ban any
- Line 167:
   smaller residual gate -- still correct,
   just slower per file.
- Line 195:
   and the opaque rule index appear in failure output -- otherwise a failing CI log

### package/cli/forbidden-strings/src/port-betterleaks-relaxations.ts

- Line 20:
   // Trailing `\b` arm in `(?:[X]\|...\|\b)` -- resharp rejects \b as a

### package/dev-script/inference-canary-viewer/README.md

- Line 15:
   Syntax highlighting uses the CSS Custom Highlight API via Lezer tokenization --

### package/dev-script/inference-canary-viewer/src/data/diff.ts

- Line 58:
   // Exit code 1 means files differ -- expected behavior

### package/dev-script/inference-canary/README.md

- Line 27:
   This is intentional -- a submission full of lint violations and type errors is not production-quality code,
- Line 52:
   geometric mean collapses scores toward zero -- a single hard probe tanks the entire result
- Line 60:
   which looks like a freebie -- but this is intentional.
- Line 92:
   Probes are intentionally hard -- a healthy model scores around 0.7-0.8,
   not 1.0.
- Line 132:
   Gitignored -- local to each machine.

### package/dev-script/inference-canary/RESEARCH.md

- Line 40:
   Without activation-level ground truth,
   asking a model "are you degraded?
  " falls into confabulation territory -- models can act introspective without being introspective.
- Line 47:
   - **January 2026** (GitHub #21046):
   community-reported "shadow downgrade" -- laziness,
   context loss,
   constraint violations.
- Line 65:
   If any of these fail,
   or if self-consistency drops (different answers on repeated runs),
   something is wrong -- either with the model weights being served,
   the quantization level,
   o

### package/dev-script/inference-canary/TODO.md

- Line 143:
   Status:
   stable spec,
   but the IPv4 parser sub-spec is genuinely obscure -- models know about

### package/dev-script/inference-canary/src/canary-lint/Haiku 4.5/css-mixin-transpiler-initial-2026-03-16T18-07-02.000Z/canary.ts

- Line 115:
   pos += 2;
   // Skip --

### package/dev-script/inference-canary/src/canary-lint/Kimi K2.5/css-mixin-transpiler-initial-2026-03-14T23-20-05.000Z/canary.ts

- Line 41:
   if (i === pos + 2) return null;
   // No name after --

### package/dev-script/inference-canary/src/canary-lint/Kimi K2.5/css-mixin-transpiler-initial-2026-03-16T18-27-51.000Z/canary.ts

- Line 267:
   // Name must start with --
- Line 327:
   // Name must start with --

### package/dev-script/inference-canary/src/canary-lint/MiniMax M2.5/css-mixin-transpiler-fix-2026-03-19T16-04-46.000Z/canary.ts

- Line 198:
   // Must start with --

### package/dev-script/inference-canary/src/canary-lint/MiniMax M2.5/css-mixin-transpiler-initial-2026-03-06T23-36-23.000Z/canary.ts

- Line 186:
   // Must start with --

### package/dev-script/inference-canary/src/canary-lint/MiniMax M2.5/css-mixin-transpiler-initial-2026-03-19T16-04-46.000Z/canary.ts

- Line 205:
   // Must start with --

### package/dev-script/inference-canary/src/canary-lint/Qwen 3.5 OSS/css-mixin-transpiler-fix-2026-03-16T18-07-02.000Z/canary.ts

- Line 19:
   /** The mixin name (including -- prefix).
   */

### package/dev-script/inference-canary/src/codegen/css-mixin-test-css.ts

- Line 37:
   // Top-level @apply -- mixin body expands directly into the stylesheet

### package/dev-script/inference-canary/src/codegen/css-mixin-verify.ts

- Line 90:
   // Later property overrides mixin property -- either both present in order or only winner kept

### package/dev-script/inference-canary/src/codegen/css-mixin.ts

- Line 97:
   '5.
   A rule block may contain multiple `@apply` rules -- expand each in order',
- Line 108:
   'Example 1 -- basic:
  ',
- Line 121:
   'Example 2 -- top-level apply:
  ',

### package/dev-script/inference-canary/src/codegen/perf-test-data/mise.generate-expr-perf.ts

- Line 100:
   // Might or might not be zero -- compute to find out

### package/dev-script/inference-canary/src/codegen/probe-factory-additional-diagnostics.ts

- Line 92:
   // Main run was fine but additional runs failed -- build standalone prompt

### package/dev-script/inference-canary/src/codegen/probe-factory-build-fix.ts

- Line 97:
   // No lint/runtime issues but perf is slow -- create a standalone perf fix prompt

### package/dev-script/inference-canary/src/codegen/probe-factory-run-score.ts

- Line 194:
   // Combine main and additional run correctness via Math.
  min --

### package/dev-script/inference-canary/src/codegen/sudoku-puzzles.ts

- Line 12:
   //region Normal mode puzzles -- solvable,
   two unsolvable variants,
   multi-solution
- Line 112:
   //region --all mode puzzles -- exact enumeration,
   many-solution,
   unsolvable
- Line 196:
   //region Assembled inputs -- stdin strings combining puzzles for each mode

### package/dev-script/inference-canary/src/codegen/sudoku-solver-verify.ts

- Line 34:
   //region Normal mode verification -- 4 checks:
   solvable,
   2x unsolvable,
   multi-solution single exit
- Line 107:
   //region --all mode verification -- 3 checks:
   2-solution exact,
   many-solution bounded,
   unsolvable

### package/dev-script/inference-canary/src/codegen/sudoku-solver.ts

- Line 6:
   * enumerate every valid solution.
   Tests both correct solving and unsolvable detection --
- Line 21:
   //region Prompt -- instructs the model to build a backtracking solver with --all support

### package/dev-script/inference-canary/src/container-base.ts

- Line 10:
   /** Result of a spawned command -- never throws,
   callers inspect exitCode */

### package/dev-script/inference-canary/src/container-runtime.ts

- Line 15:
   //region Configuration -- timeout,
   image tag,
   and buffer size shared by `container-exec.ts`
- Line 29:
   //region Runtime detection -- uses `which` to locate executables on PATH

### package/dev-script/inference-canary/src/container.ts

- Line 30:
   //region Staging directory -- uses LINT_DIR instead of os.
  tmpdir() so all container I/O stays under one well-known tree
- Line 67:
   //region Public API -- runInContainer is the sole entry point for executing generated code

### package/dev-script/inference-canary/src/index-cli.ts

- Line 16:
   //region Parser definition -- defines all recognized CLI flags and their value parsers
- Line 39:
   //region Parsed arguments -- module-level exports consumed by `index.ts`

### package/dev-script/inference-canary/src/index.ts

- Line 44:
   //region API key resolution -- validates INFERENCE_VALIDATION_OPENROUTER_API_KEY before any network calls
- Line 53:
   //region Model selection -- resolves the set of models to test and which probes to skip from recent artifacts
- Line 70:
   //region Execution -- selects probe tier (simple/fast/slow),
   runs canary,
   throws on degradation

### package/dev-script/inference-canary/src/linter-artifacts-recent.ts

- Line 77:
   //region Failure artifact detection -- whole-model failures like 429 or auth errors
- Line 104:
   // Missing or malformed `meta.json` -- skip
- Line 111:
   //region Per-probe artifact detection -- individual probe results
- Line 154:
   // Missing or malformed `meta.json` -- skip

### package/dev-script/inference-canary/src/linter-artifacts.ts

- Line 31:
   //region Artifact writing -- writes generated source and `meta.json` sidecar for oxlint/tsgo to consume

### package/dev-script/inference-canary/src/linter-oxlint.ts

- Line 20:
   //region Types -- oxlint JSON output shape and the parsed result type returned to callers
- Line 158:
   //region Runner -- spawns oxlint,
   handles non-zero exits (oxlint exits 1 on violations),
   returns OxlintResult

### package/dev-script/inference-canary/src/linter.ts

- Line 19:
   //region Types -- severity counts,
   lint result,
   and related types consumed by codegen probes
- Line 59:
   //region Public API -- lintSource is the sole entry point;
   writes artifact,
   runs oxlint + tsgo,
   returns combined result

### package/dev-script/inference-canary/src/mise.migrate-labels.ts

- Line 110:
   // Missing or malformed -- skip

### package/dev-script/inference-canary/src/models.ts

- Line 11:
   //region Model config type -- per-model overrides for verbosity and display label
- Line 31:
   //region Model registry -- the canonical list of models tested by default;
   add/remove models here
- Line 35:
   // Claude 4.6 models use adaptive effort -- even at "low",
   the model decides how much to
- Line 107:
   // contract includes similar red lines,
   but the opportunistic timing -- stepping in as a
- Line 108:
   // replacement the same day a competitor was punished for holding firm -- does not inspire

### package/dev-script/inference-canary/src/probes.ts

- Line 21:
   //region Simple probes -- cheap text-only checks,
   disabled by default
- Line 51:
   /** Checks JSON output compliance -- degraded models often break structure */

### package/dev-script/inference-canary/src/runner-stream-helpers.ts

- Line 23:
   //region PartialCompletionError -- thrown on stream abort,
   carries whatever data was collected before cancellation

### package/dev-script/inference-canary/src/runner-stream.ts

- Line 131:
   // OpenRouter surfaces reasoning via `reasoning_details` on the delta -- an array of

### package/dev-script/inference-canary/src/runner-types.ts

- Line 8:
   //region Shared branded types -- template-literal types used across runner,
   models,
   server-time,
   and artifacts
- Line 31:
   //region Message and timing types -- chat message shape and streaming timing breakdown used by `runner-stream.ts`
- Line 96:
   //region Probe and report result types -- ProbeResult (per-probe) and CanaryReport (per-model) returned by runCanary

### package/dev-script/inference-canary/src/server-time.ts

- Line 23:
   /** OpenRouter models endpoint -- lightweight,
   public,
   no auth required */

### package/dev-script/page-weight/README.md

- Line 43:
   The script accepts any dist directory -- use it against any SSG output,

### package/dev-script/page-weight/src/html.ts

- Line 59:
   * Returns `null` for missing attributes,
   non-string values,
   or the empty string --

### package/dev-script/task-util/README.md

- Line 135:
   Mise does offer `sources`/`outputs` for staleness checking,
   but these only work with files --

### package/dev-script/task-util/TROUBLESHOOTING.mise-sources.md

- Line 50:
   hash always matches,
   and the auto-output touch file's mtime is always newer --
- Line 71:
   comparison (check 2) should still catch this -- but only if the glob patterns

### package/dev-script/task-util/src/append.ts

- Line 108:
   //region Parser definition -- required --to option and variadic positional text lines

### package/dev-script/task-util/src/append.unit.test.ts

- Line 20:
   //region Fixture Setup -- Per-test fixtures

### package/dev-script/task-util/src/command.ts

- Line 54:
   //region Parser definition -- defines CLI flags and rest arguments after --
- Line 91:
   No command specified after --

### package/dev-script/task-util/src/command.unit.test.ts

- Line 34:
   //region Fixture Setup -- Per-test fixtures
- Line 218:
   name:
   'fails when only -- is provided without command',

### package/dev-script/task-util/src/depends-exec.ts

- Line 49:
   //region Execution -- run command with collapsed output

### package/dev-script/task-util/src/depends-resolve.ts

- Line 23:
   //region Item resolution -- resolve individual items to timestamps

### package/dev-script/task-util/src/depends.ts

- Line 22:
   * task-depends -s "src/*.
  ts" -o "dist/*.
  js" -- mise run build
- Line 25:
   * task-depends -o "sh:
  podman image exists img && echo Infinity \|\| echo -Infinity" -- podman build .
- Line 28:
   * task-depends -s "sh:
  git log -1 --format=%ct" -o "dist/*.
  js" -- mise run build
- Line 31:
   * task-depends --output-time-strategy oldest -s "src/**" -o "dist/**" -o "sh:
  ..." -- mise run build
- Line 194:
   No command specified after --
- Line 195:
   Usage:
   task-depends -s "src/*" -o "dist/*" -- command args...
- Line 196:
   Usage:
   task-depends -o "sh:
  podman image exists img" -- command args...

### package/dev-script/task-util/src/depends.unit.test.ts

- Line 19:
   //region Fixture Setup -- temp directory with controllable source and output files
- Line 103:
   `bun ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
- Line 123:
   `bun ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
- Line 143:
   `bun ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
- Line 162:
   `bun ${cliPath} -v -s "${srcDir}/**" -o "${outDir}/**" -- ${
- Line 183:
   `bun ${cliPath} -v -s "${srcDir}/**" -o "${outDir}/**" -- ${
- Line 205:
   `bun ${cliPath} -s "${srcDir}/**" -s "${libDir}/**" -o "${outDir}/**" -- ${
- Line 228:
   `bun ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -o "${dist2Dir}/**" -- ${
- Line 247:
   `bun ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -- ${
- Line 268:
   `bun ${cliPath} -s "${srcDir}/**/*.
  xyz" -o "${outDir}/**" -- ${
- Line 290:
   `bun ${cliPath} -s "sh:
  echo Infinity" -o "sh:
  echo -Infinity" -- ${
- Line 307:
   `bun ${cliPath} -s "sh:
  echo -Infinity" -o "sh:
  echo Infinity" -- ${
- Line 325:
   `bun ${cliPath} -s "sh:
  echo -Infinity" -o "sh:
  echo Infinity" -o "sh:
  echo -Infinity" -- ${
- Line 343:
   `bun ${cliPath} -s "sh:
  echo 0" --output-time-strategy oldest -o "sh:
  echo Infinity" -o "sh:
  echo -Infinity" -- ${
- Line 360:
   `bun ${cliPath} -s "sh:
  echo -Infinity" -o "sh:
  echo Infinity" -o "sh:
  echo Infinity" -- ${
- Line 377:
   `bun ${cliPath} -v -s "sh:
  echo Infinity" -o "sh:
  echo -Infinity" -- ${
- Line 402:
   `bun ${cliPath} -s "sh:
  echo Infinity" -o "${outDir}/**" -- ${
- Line 421:
   `bun ${cliPath} -s "sh:
  echo -Infinity" -o "${outDir}/**" -- ${
- Line 443:
   // Output files are from now -- source is newer → stale
- Line 447:
   `bun ${cliPath} -s "sh:
  echo 2208988800" -o "${outDir}/**" -- ${
- Line 467:
   `bun ${cliPath} -s "sh:
  echo 2208988800000" -o "${outDir}/**" -- ${
- Line 487:
   `bun ${cliPath} -s "sh:
  echo 2040-01-01T00:00:00Z" -o "${outDir}/**" -- ${
- Line 507:
   `bun ${cliPath} -s "sh:
  echo 946684800" -o "${outDir}/**" -- ${
- Line 527:
   `bun ${cliPath} -s "${srcDir}/**" -o "sh:
  echo 946684800" -- ${
- Line 550:
   `bun ${cliPath} -s "sh:
  exit 1" -o "${outDir}/**" -- ${
- Line 611:
   `bun ${cliPath} --source-time-strategy oldest -s "sh:
  echo 946684800" -s "sh:
  echo 2208988800" -o "${outDir}/**" -- ${
- Line 631:
   `bun ${cliPath} --source-time-strategy newest -s "sh:
  echo 946684800" -s "sh:
  echo 2208988800" -o "${outDir}/**" -- ${
- Line 653:
   `bun ${cliPath} --output-time-strategy oldest -s "${srcDir}/**" -o "${outDir}/**" -o "sh:
  echo -Infinity" -- ${
- Line 674:
   `bun ${cliPath} --output-time-strategy newest -s "${srcDir}/**" -o "${outDir}/**" -o "sh:
  echo -Infinity" -- ${
- Line 713:
   `bun ${cliPath} --source-time-strategy "sh:
  sort -n \| head -1" -s "sh:
  echo 946684800" -s "sh:
  echo 2208988800" -o "${outDir}/**" -- ${
- Line 734:
   `bun ${cliPath} --source-time-strategy "sh:
  sort -rn \| head -1" -s "sh:
  echo 946684800" -s "sh:
  echo 2208988800" -o "${outDir}/**" -- ${
- Line 754:
   `bun ${cliPath} --output-time-strategy "sh:
  sort -n \| head -1" -s "${srcDir}/**" -o "sh:
  echo Infinity" -o "sh:
  echo -Infinity" -- ${
- Line 809:
   `bun ${cliPath} -s "sh:
  echo Infinity" -o "sh:
  echo -Infinity" -- ${
- Line 835:
   `bun ${cliPath} -s "${srcDir}/**" -o "sh:
  echo Infinity" -- ${
- Line 854:
   `bun ${cliPath} -s "${srcDir}/**" -o "sh:
  echo 946684800" -- ${
- Line 874:
   `bun ${cliPath} -s "${srcDir}/**" -o "${outDir}/**" -o "sh:
  echo Infinity" -- ${

### package/dev-script/task-util/src/oxlint-augment.ts

- Line 54:
   //region Rule guidance -- enhanced messages for specific lint rules

### package/dev-script/task-util/src/oxlint-augment.unit.test.ts

- Line 326:
   // Only one note injected -- for no-misused-promises

### package/dev-script/task-util/src/pnpm-output-filter.ts

- Line 6:
   * pnpm lacks a per-package allowlist for cycle warnings --

### package/dev-script/task-util/src/tsgo-filter.unit.test.ts

- Line 359:
   // Should not reach here -- tsgo should fail on the type error

### package/dev-script/vm-builder/README.md

- Line 26:
   1.
   **Build** -- `podman build` produces `localhost/monochromatic-dev:latest`
- Line 28:
   2.
   **Convert** -- `bootc-image-builder` converts the image to
- Line 30:
   3.
   **Fix ownership** -- restores `output/` from root:
  root to current user

### package/dev-script/vm-builder/src/build-and-import.ts

- Line 100:
   /** libvirt session URI -- connects to the user's QEMU/KVM daemon (no sudo needed).
   */
- Line 264:
   // Domain not defined -- nothing to remove.
- Line 352:
   // virt-manager is not a Flatpak -- no override needed.

### package/dev-script/vm-builder/src/import.ts

- Line 74:
   /** libvirt session URI -- no sudo needed.
   */

### package/figma/kiwi/RESEARCH.md

- Line 27:
   There is NO custom header -- the first bytes `AF 04` are simply the
- Line 69:
   difference -- type is unused for enum fields anyway.
   The rest matches exactly.

### package/figma/kiwi/src/index.ts

- Line 592:
   // Unknown tag -- cannot skip without knowing the type.

### package/figma/to-penpot/TROUBLESHOOTING.import.md

- Line 220:
   5.
   This approach was partially implemented -- hex loading works,

### package/mcp/mvm/src/index.ts

- Line 30:
   //region Server setup -- create and serve the MCP server

### package/mcp/mvm/src/response.ts

- Line 6:
   //region Types -- response shape definitions
- Line 25:
   //region Response builders -- construct MCP-compliant response objects

### package/mcp/mvm/src/tools-exec.ts

- Line 18:
   //region Execution tools -- run commands inside VMs

### package/mcp/mvm/src/tools-lifecycle-mutate.ts

- Line 21:
   //region Mutation tools -- VM creation and destruction

### package/mcp/mvm/src/tools-lifecycle.ts

- Line 17:
   //region Lifecycle tools -- VM listing and template updates

### package/mcp/mvm/src/tools-transfer.ts

- Line 19:
   //region Transfer tools -- move files between host and guest VMs

### package/mcp/nvim/README.md

- Line 16:
   **get_diagnostics** --
- Line 20:
   **get_all_diagnostics** --

### package/mcp/nvim/src/dedup.ts

- Line 3:
   //region Dedup key -- builds a string key for comparing diagnostics across sources
- Line 28:
   //region Public API -- merge and deduplicate diagnostics

### package/mcp/nvim/src/dedup.unit.test.ts

- Line 14:
   //region helpers -- factory for test diagnostics
- Line 39:
   //region dedupDiagnostics -- merges editor and lint diagnostics with deduplication
- Line 206:
   //region uniqueDiagnostics -- removes duplicates within a single array

### package/mcp/nvim/src/format.unit.test.ts

- Line 12:
   //region formatDiagnostic -- formats a diagnostic into a human-readable line

### package/mcp/nvim/src/lint-runner.ts

- Line 16:
   //region Types -- lint result shape
- Line 38:
   //region Runner -- orchestrate oxlint across file groups
- Line 87:
   //region Group files by tsconfig ancestor -- each group runs in its own cwd
- Line 170:
   //region Utilities -- map merging

### package/mcp/nvim/src/lint-runner.unit.test.ts

- Line 10:
   //region parseOxlintOutput -- converts oxlint JSON to grouped Diagnostic maps

### package/mcp/nvim/src/nvim-client.ts

- Line 36:
   //region Public API -- query diagnostics and file info across all Neovim instances

### package/mcp/nvim/src/nvim-client.unit.test.ts

- Line 12:
   //region SEVERITY_MAP -- maps vim.
  diagnostic.
  severity codes to human-readable labels
- Line 62:
   //region normalizeMessage -- reformats embedded help text from LSP diagnostics

### package/mcp/nvim/src/nvim-connection.ts

- Line 17:
   //region Connection management -- discover and cache connections to all Neovim instances

### package/mcp/nvim/src/nvim-lua.ts

- Line 16:
   //region Raw diagnostic mapping -- converts Lua msgpack output to typed Diagnostics
- Line 56:
   //region Lua snippets -- shared Lua code executed via nvim_exec_lua

### package/mcp/nvim/src/nvim-types.ts

- Line 10:
   //region Severity mapping -- vim.
  diagnostic.
  severity codes to human-readable labels
- Line 22:
   //region Types -- diagnostic and file metadata shapes

### package/mcp/nvim/src/oxlint-parse.ts

- Line 22:
   //region Directory walking -- find config files by walking up the filesystem
- Line 61:
   //region Parsing -- convert oxlint JSON diagnostics to our Diagnostic type

### package/mcp/nvim/src/oxlint-spawn.ts

- Line 23:
   //region Process spawning -- low-level oxlint invocation

### package/mcp/nvim/src/oxlint-types.ts

- Line 10:
   //region Types -- oxlint JSON output shape
- Line 103:
   //region Severity mapping -- oxlint lowercase to our uppercase format

### package/mcp/nvim/src/tool-helpers.ts

- Line 12:
   //region Helper functions -- build response text for tool handlers

### package/mcp/stdio/src/json-rpc.ts

- Line 6:
   //region JSON-RPC 2.0 base types -- foundation for all MCP message exchange
- Line 111:
   //region Standard JSON-RPC error codes -- used for protocol-level failures
- Line 127:
   //region Message validation -- type guard for untrusted JSON parsed from stdin

### package/mcp/stdio/src/json-rpc.unit.test.ts

- Line 15:
   //region isJsonRpcMessage -- validates minimum JSON-RPC 2.0 shape
- Line 112:
   //region error code constants -- verify expected values

### package/mcp/stdio/src/line-reader.unit.test.ts

- Line 9:
   //region helpers -- create ReadableStream from string content
- Line 62:
   //region readLines -- async generator yielding newline-delimited lines

### package/mcp/stdio/src/protocol.ts

- Line 49:
   //region Tool definitions and handlers -- describes tools exposed to MCP clients

### package/mcp/stdio/src/server-define-tool.ts

- Line 5:
   //region defineTool -- convenience for declaring tool entries

### package/mcp/stdio/src/server-types.ts

- Line 14:
   //region Tool entry -- pairs a name with its options for immutable registration
- Line 38:
   //region Registered tool -- internal representation after normalization
- Line 59:
   //region Server configuration -- identity passed during initialization
- Line 76:
   //region Server handle -- returned by createMcpServer

### package/mcp/stdio/src/server.ts

- Line 29:
   //region createMcpServer -- builds an immutable server from config and tool entries
- Line 82:
   //region Protocol payloads -- initialization and tool listing
- Line 115:
   //region Request dispatch -- routes JSON-RPC methods to handlers
- Line 168:
   //region Public handle -- single dispatch function exposed to the transport

### package/mcp/stdio/src/server.unit.test.ts

- Line 31:
   //region defineTool -- bundles name with tool entry options
- Line 78:
   //region createMcpServer -- builds immutable server and dispatches messages
- Line 83:
   //region initialize -- returns server identity and capabilities
- Line 122:
   //region ping -- responds with empty object
- Line 142:
   //region tools/list -- returns registered tool definitions
- Line 230:
   //region tools/call -- dispatches to registered tool handlers
- Line 419:
   //region unknown method -- returns method not found error
- Line 442:
   //region notifications -- returns undefined for notifications

### package/mcp/stdio/src/transport.ts

- Line 12:
   //region Output writer abstraction -- supports both Bun FileSink and standard WritableStream
- Line 53:
   //region Stdio message loop -- reads stdin lines,
   validates,
   dispatches,
   writes responses
- Line 155:
   //region Message serialization -- writes JSON-RPC responses to stdout

### package/mcp/stdio/src/transport.unit.test.ts

- Line 18:
   //region helpers -- test doubles for stdin/stdout and server handle

### package/module/dom/src/prompt.ts

- Line 1:
   // Prompt Dialog Polyfill -- Drop-in replacement for window.
  prompt using dialog element

### package/module/es/src/path/fallbacks.ts

- Line 6:
   //region normalize -- resolve `.` and `..`,
   collapse slashes
- Line 67:
   //region dirnameFallback -- browser fallback for dirname
- Line 116:
   //region joinFallback -- browser fallback for join
- Line 148:
   //region resolveFallback -- browser fallback for resolve

### package/module/es/src/path/find-monorepo-root.ts

- Line 65:
   'using OPFS for monorepo root discovery -- `mise.toml` must exist in OPFS to be found',
- Line 83:
   l.
  warn('no filesystem available for monorepo root discovery -- search will fail',
  );
- Line 116:
   /* happy-opfs import failed -- fall through to empty stub */

### package/module/es/src/path/index.ts

- Line 37:
   //region Node delegation -- use real node:
  path/posix when the runtime has it

### package/module/es/src/types/t boolean/t is/t p string/f/t unknown/r s/p p/index.ts

- Line 3:
   //region General String Types -- General utility string types

### package/module/es/src/types/t function/f/t function/memoize/r s/p p/index.unit.test.ts

- Line 179:
   // Now add 4 -- should evict 2 (oldest after refresh),
   not 1

### package/module/es/src/types/t function/t is/t/r s/p p/behaviorTest/genericsSchema.behaviorTest.ts

- Line 582:
   // @ts-expect-error -- StringToNumberSchema is not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 596:
   // @ts-expect-error -- StringToNumberSchema is not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 606:
   // @ts-expect-error -- StringToNumberSchema is not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 650:
   // @ts-expect-error -- WeightedStringSchema is not assignable to RealSchema (demonstrates compile-time safety)
- Line 662:
   // @ts-expect-error -- WeightedStringSchema is not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 674:
   // @ts-expect-error -- WeightedStringSchema is not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 714:
   // @ts-expect-error -- NamedUserSchema is not assignable to RealSchema (demonstrates compile-time safety)
- Line 729:
   // @ts-expect-error -- NamedUserSchema is not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 744:
   // @ts-expect-error -- NamedUserSchema is not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 781:
   // @ts-expect-error -- AsyncUserSchema is not assignable to RealSchemaAsync (demonstrates compile-time safety)
- Line 813:
   // @ts-expect-error -- UserTransformSchema is not assignable to RealMaybeAsyncSchema (demonstrates compile-time
   safety)
- Line 855:
   // @ts-expect-error -- Unknown loses additional properties from unknown input
- Line 860:
   // @ts-expect-error -- Generic pattern with unknown creates never type
- Line 862:
   // @ts-expect-error -- Never type means no properties accessible
- Line 871:
   // @ts-expect-error -- Unknown pattern loses additional properties even from any
- Line 939:
   // @ts-expect-error -- Generic pattern creates never for invalid input
- Line 961:
   // @ts-expect-error -- RealSchema<string,
   Promise<number>> not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 971:
   // @ts-expect-error -- RealSchema<string,
   Promise<number>> not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 995:
   // @ts-expect-error -- RealSchema<string,
   Promisable<number>> not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 1013:
   // @ts-expect-error -- RealSchema<string,
   Promisable<number>> not assignable to RealSchema<unknown,
   unknown> (demonstrates compile-time safety)
- Line 1061:
   // @ts-expect-error -- ValidatedTransformSchema is not assignable to RealSchema (demonstrates compile-time safety)
- Line 1079:
   // @ts-expect-error -- ValidatedTransformSchema is not assignable to RealSchema (demonstrates compile-time safety)

### package/module/es/src/types/t function/t is/t/r s/p p/behaviorTest/simplifiedSchema.behaviorTest.ts

- Line 317:
   // @ts-expect-error -- unknown input narrowed to Schema,
   weight property lost
- Line 323:
   // @ts-expect-error -- unknown creates never type in generic pattern
- Line 325:
   // @ts-expect-error -- unknown creates never type,
   no weight property
- Line 330:
   // @ts-expect-error -- unknown is not Schema
- Line 334:
   // @ts-expect-error -- unknown can't extend Schema
- Line 345:
   // @ts-expect-error -- any input gets narrowed to Schema,
   losing weight
- Line 358:
   // @ts-expect-error -- any input gets narrowed to Schema,
   losing weight
- Line 383:
   // @ts-expect-error -- union type is not assignable to Schema
- Line 387:
   // @ts-expect-error -- union type can't extend Schema
- Line 395:
   // @ts-expect-error -- union narrowing loses weight property
- Line 401:
   // @ts-expect-error -- union narrowing loses weight property
- Line 445:
   // @ts-expect-error -- notASchema creates never type in generic
- Line 464:
   objWithParse.
  extraStuff;
   // Preserved -- Unknown pattern now retains original properties

### package/module/es/src/types/t object/t array/t p string/t typeof/f/t unknown/r s/p p/index.ts

- Line 219:
   // @ts-expect-error -- Might be Async Iterable
- Line 227:
   // @ts-expect-error -- Might be Iterable

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.array.unit.test.ts

- Line 19:
   //region Empty arrays -- basic and with comments
- Line 42:
   //region Primitives and separators -- single/multiple numbers and trailing comma
- Line 71:
   //region Strings and escapes -- ensure quoted parsing cooperates
- Line 83:
   //region Nested arrays -- delegate inner arrays
- Line 98:
   //region Errors -- malformed separators
- Line 116:
   //region Array-level vs first-item comments -- semantics for outside/inside comments

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.arrayCore.ts

- Line 21:
   //region Array elements -- Recursive,
   immutable element parsing for arrays (MUTUALLY RECURSIVE)
- Line 112:
   //region Entry and comment skip -- Drop the opening '[' then consume leading comments/space
- Line 125:
   //region Empty array fast-exit -- Handle immediate closing bracket
- Line 146:
   //region Element recursion -- Delegate to exported pure helper

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.arrayHelpers.ts

- Line 10:
   //region Array header -- Consume '[' then leading comments to capture array-level comment
- Line 41:
   //region Array separators -- Determine end of array or next element start

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.parseValue.ts

- Line 22:
   //region Value dispatcher -- Single entry to parse one value from the start (MUTUALLY RECURSIVE)

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.record.unit.test.ts

- Line 19:
   //region Empty objects -- basic and with comments
- Line 44:
   //region Pairs and separators -- single/multiple and trailing comma
- Line 75:
   //region Nesting -- arrays and objects
- Line 94:
   //region Comments semantics -- outside vs inside
- Line 132:
   //region Errors -- malformed structures

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.recordCore.ts

- Line 24:
   //region Record value parsing -- Parse value with leading comment after colon (MUTUALLY RECURSIVE)
- Line 66:
   //region One record member -- Compose key + colon + value for a single member (MUTUALLY RECURSIVE)
- Line 110:
   //region Record members -- Recursive,
   immutable member parsing for records (MUTUALLY RECURSIVE)

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.recordHelpers.ts

- Line 12:
   //region Record header -- Consume '{' then extract record-level comment from context
- Line 42:
   //region Record separators -- Determine end of record or next member start
- Line 112:
   //region Record key parsing -- Parse a single key with its leading comment
- Line 160:
   //region Colon expectation -- Verify ':
  ' after key

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.tokenizers.ts

- Line 9:
   //region Value tokenizers -- Pure helpers for literals and numbers with explicit contracts

### package/module/es/src/types/t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/index.ts

- Line 8:
   //region Imports and aliases -- External types/helpers and local aliases used by the parser
- Line 125:
   //region Pre-scan for comments -- Strip/record leading comments to decide how to dispatch
- Line 129:
   //region Top-level dispatch and heuristics -- Select array/object path;
   attempt simple trailing-comma fix,
   else fallback
- Line 154:
   //region Error handling -- Only arrays or objects are valid after trimming leading comments
- Line 166:
   //region Re-exports -- Surface helpers for testing and external use

### package/module/es/src/types/t object/t record/f/t record/omit/p n/index.unit.test.ts

- Line 74:
   // @ts-expect-error -- intentionally passing non-existent key to test runtime error

### package/module/es/src/types/t object/t record/f/t record/pick/p n/index.unit.test.ts

- Line 53:
   // @ts-expect-error -- intentionally passing non-existent key to test runtime error

### package/module/es/src/types/t object/t regexp/t global/t/index.ts

- Line 11:
   // @ts-expect-error -- Isn't global

### package/module/es/src/types/t object/t store/consensus.ts

- Line 2:
   // BackendResult with string values and numeric priority tiers -- store

### package/module/es/src/types/t string/t char/t/index.ts

- Line 28:
   // @ts-expect-error -- Type 'string' is not assignable to type '$'.
   Type 'string' is not assignable to type '{ length:
   1;
   }'.
  ts(2322)
- Line 32:
   // @ts-expect-error -- Type 'string' is not assignable to type '$'.
   Type 'string' is not assignable to type '{ length:
   1;
   }'.
  ts(2322)

### package/module/es/src/types/t string/t nonEmpty/t/index.ts

- Line 25:
   // @ts-expect-error -- Type '""' is not assignable to type '`${any}${string}`'.
  ts(2322)

### package/module/image-diff/src/describe.ts

- Line 132:
   // Prefer the native Gemini API -- avoids the OpenRouter proxy overhead

### package/module/logger/README.md

- Line 4:
   Works immediately at import -- auto-discovers available backends for the current runtime
- Line 52:
   The caller owns serialization -- template literals cover the common case
- Line 75:
   is marked unavailable rather than creating one at cwd -- this prevents
- Line 85:
   Async sinks are fire-and-forget -- the log call never blocks the caller.

### package/module/logger/src/logger.ts

- Line 91:
   /** Eager initialization promise -- throws at module load if no backends available.
   */

### package/module/logger/src/sinks/file.ts

- Line 88:
   /* ENOENT or similar -- keep walking up */
- Line 117:
   // Dynamic import for Node.
  js modules -- cache appendFile for use in fileSink.
  write

### package/module/matrix/README.md

- Line 100:
   but does not affect execution -- the process runs as the current user.

### package/module/test/README.md

- Line 29:
   `Promise.allSettled` is the most portable concurrency primitive available --
- Line 49:
   Use empty-name describe as the top-level wrapper --
- Line 70:
   so the limit only gates thunks that have not yet been invoked --
- Line 127:
   Custom `SinonSandboxConfig` is not supported --
- Line 319:
   chai's `.throw()` which expects a **function** to call -- a semantic mismatch.
- Line 387:
   Children must be thunks for the limit to take effect --
- Line 519:
   the right number of assertions ran -- prevents silently passing async tests.
- Line 562:
   This keeps control flow visible -- no implicit lifecycle runs behind the scenes.
- Line 629:
   the stub affects all code running in the process -- including concurrent tests.
- Line 662:
   Stubbing **local** objects (created within the test) is safe at any concurrency --
- Line 762:
   - **`test.only` / `describe.only`** --
- Line 765:
   - **`describe.shuffle`** --
- Line 767:
   Concurrent-by-default execution already surfaces those immediately --
- Line 769:
   - **`test.extend` / fixtures** --
- Line 787:
   - **`onTestFinished` / `onTestFailed`** --
- Line 808:
   - **`toBeNullable`** --
- Line 811:
   - **`toBeOneOf`** --
- Line 814:
   `toThrowErrorMatchingSnapshot`,
   `toThrowErrorMatchingInlineSnapshot`) --
- Line 838:
   - **`toHaveBeenCalledBefore` / `toHaveBeenCalledAfter`** --
- Line 842:
   `toHaveLastResolvedWith`,
   `toHaveNthResolvedWith`) --
- Line 859:
   - **`expect.closeTo`** --
- Line 861:
   inside `toHaveBeenCalledWith` but not inside `toEqual` --
- Line 864:
   - **`expect.not.*`** (negated asymmetric matchers) --
- Line 867:
   - **`expect.schemaMatching`** --
- Line 869:
   - **`expect.toBeOneOf`** (asymmetric) --
- Line 881:
   - **`expect.unreachable(message?)`** --
- Line 883:
   - **`expect.soft`** --
- Line 886:
   via `AggregateError`,
   the benefit is narrow --
- Line 888:
   - **`expect.poll`** --
- Line 891:
   - **`expect.extend`** --
- Line 937:
   - **`vi.mock` / `vi.doMock` / `vi.unmock`** (module mocking) --
- Line 970:
   This package does not re-export `assert` --
- Line 986:
   All concurrent suites in a file share that one limiter --
- Line 995:
   This harness runs children **concurrently** by default --
- Line 1009:
   This harness has no file-level parallelism --

### package/module/test/src/describe.ts

- Line 70:
   * Set to empty string to make this level invisible in output --

### package/module/test/src/expect.ts

Moved on 2026-10-04 to `package/module/test-expect/src/expect.ts`.
The recorded lines retain their historical positions.

- Line 215:
   /** Negated matchers -- every method asserts the opposite.
   */

### package/module/token-count/README.md

- Line 11:
   Content is never sent to a model for generation --

### package/module/token-count/src/cli.ts

- Line 16:
   //region CLI -- parses args and counts tokens in files

### package/module/token-count/src/types.ts

- Line 13:
   /** Claude model for tokenization;
   defaults to `claude-sonnet-4-6`.
   Only selects the tokenizer -- no inference is performed.
   */

### package/module/zip-writer/src/index.ts

- Line 11:
   * only stored files.
   The write-only design has minimal attack surface --

### package/pi-plugin/auto-mode/src/command-parser.unit.test.ts

- Line 88:
   name:
   "handles -- end-of-options separator",
- Line 90:
   const result = analyzeBashCommand("rm -- -f");
- Line 93:
   // -f after -- should be a positional arg,
   not a flag

### package/pi-plugin/auto-mode/src/signals.unit.test.ts

- Line 193:
   name:
   "respects -- end-of-options separator",
- Line 195:
   // -f after -- should NOT be treated as a flag
- Line 201:
   name:
   "detects flag before -- separator",
- Line 306:
   //region rm -- -f
- Line 309:
   name:
   "does not flag -f after -- as rm -f",
- Line 311:
   // rm -- -f:
   -f is a positional argument (filename),
   not a flag
- Line 312:
   const analysis = analyzeBashCommand("rm -- -f");
- Line 313:
   // This should NOT trigger the rm -f signal since -f is after --
- Line 315:
   // rm -- -f by itself should not flag (no -rf,
   no -f flag before --)

### package/rolldown-plugin/import-attributes/README.md

- Line 51:
   1.
   **`transform` hook** -- rewrites `with { type: '...' }` clauses into query parameters
- Line 55:
   2.
   **`resolveId` hook** -- for dynamic imports,
   rolldown's Rust scanner discovers

### package/runtime-error/bun/src/infinite-loop.ts

- Line 10:
   // intentional busy loop -- no yield,
   no sleep,
   no exit

### package/runtime-error/bun/src/oom.ts

- Line 9:
   /** Number of mebibytes per allocation chunk -- large enough to exhaust memory in a few iterations.
   */

### package/test-fixture/data-sequence/src/array.0to999.ts

- Line 1:
   /* v8 ignore file -- @preserve */

### package/test-fixture/data-sequence/src/generator.0to999.ts

- Line 1:
   /* v8 ignore file -- @preserve */
- Line 5:
   //region Generator Test Fixtures -- Provides generator functions that yield numbers from 0 to 999 for testing
   synchronous generators,
   asynchronous generators,
   error handling,
   and p

### package/test-fixture/data-sequence/src/promises.0to999.ts

- Line 1:
   /* v8 ignore file -- @preserve */
- Line 5:
   //region Promise Test Fixtures -- Provides an array of 1000 promises with progressive timing delays for testing
   asynchronous operations,
   concurrent processing,
   and performance char

### package/test-fixture/data-sequence/src/script.ts

- Line 3:
   //region Fixture Data Generation Utility -- Provides manual generation logic for creating fixture data arrays and
   promises used in testing scenarios

### package/test-fixture/file-enforcer-perf/src/bench-in-container.ts

- Line 136:
   // Warm runs -- content unchanged,
   all writes skipped.
- Line 152:
   // 1 source changed -- modify one source file,
   invalidate its cache entry,
   re-run.
- Line 170:
   // 1 dest changed -- modify one dest file externally,
   invalidate its cache entry,
   re-run.

### package/test-fixture/file-enforcer-perf/src/perf.bench.test.ts

- Line 154:
   //region I/O benchmarks -- glob expansion,
   file reading,
   file writing
- Line 187:
   //region Pure computation benchmarks -- string/JSON operations

### package/test-fixture/file-enforcer-perf/src/perf.config.ts

- Line 51:
   //region All rules -- single Promise.
  all to minimize CFS yield points
- Line 160:
   // Concat rules -- combine `readme.md` from groups of 4 packages
- Line 163:
   // Glob mirror rules -- mirror lib and type files across packages
- Line 168:
   // GetProperty extractions -- parse JSON and extract nested values
- Line 172:
   // Dedup -- combine all 20 readmes and remove duplicate lines
- Line 175:
   // Deep glob -- mirror 6-level nested files

### package/test-fixture/file-enforcer-perf/src/run-constrained-utils.ts

- Line 79:
   // Walk backwards to find the JSON line -- it starts with '{'

### package/test-fixture/file-enforcer-perf/src/validate-resources.ts

- Line 64:
   //region Serial CPU benchmark -- SHA-256 hashing,
   single-threaded
- Line 76:
   //region Parallel CPU benchmark -- 8 workers each computing hashes
- Line 99:
   //region Memory benchmark -- allocate and fill 256 MB
- Line 111:
   //region IO benchmark -- write + read small files to detect throttling

### package/typeface/aquaticat/src/build-font-paths.ts

- Line 47:
   // -- let needed because we reduce across multiple paths and their points

### package/typeface/aquaticat/src/build-font-resolve-points.ts

- Line 34:
   // -- let needed because M/L/H/V each update different axes of the cursor

### package/typeface/aquaticat/src/parse-svg.ts

- Line 6:
   //region Types -- data extracted from the master SVG
- Line 168:
   // -- let needed because the regex loop reassigns on each command letter encountered

### package/webapp-content/messages-demo/README.md

- Line 68:
   cooldown that drives this in production is not exercised here -- only the underlying

### package/webapp-content/messages-demo/src/client/composer.worker.ts

- Line 310:
   // Retry loop with exponential backoff -- each attempt depends on the

### package/webapp-content/messages-demo/src/client/composer/send.ts

- Line 56:
   'empty -- nothing to send',

### package/webapp-content/messages-demo/src/client/editor/index.ts

- Line 333:
   // the browser's selection Range -- without restoring,
   every

### package/webapp-content/messages-demo/src/client/outbox.ts

- Line 185:
   /** Online/visibility listener -- kicks the drain when conditions improve.
   */

### package/webapp-content/messages-demo/src/client/outbox.unit.test.ts

- Line 5:
   * 2.
   "IDB requested but absent" -- `idbAvailable: true` in a Bun env

### package/webapp-content/messages-demo/src/lib/db/migrations.ts

- Line 109:
   // for concurrent re-execution with different params -- doing so silently

### package/webapp-content/messages-demo/src/lib/http.ts

- Line 9:
   /** 200 OK -- request succeeded.
   */
- Line 12:
   /** 201 Created -- new resource created.
   */
- Line 15:
   /** 204 No Content -- request succeeded,
   no body.
   */
- Line 18:
   /** 302 Found -- temporary redirect that may change method on follow.
   */
- Line 21:
   /** 303 See Other -- response is at the URL in `Location`.
   */
- Line 24:
   /** 304 Not Modified -- conditional GET hit cache.
   */
- Line 27:
   /** 400 Bad Request -- malformed input.
   */
- Line 30:
   /** 403 Forbidden -- identity check failed.
   */
- Line 33:
   /** 404 Not Found -- resource never existed.
   */
- Line 36:
   /** 409 Conflict -- request collides with current state (revision cap,
   missing chunks).
   */
- Line 39:
   /** 410 Gone -- resource existed and was deleted.
   */
- Line 42:
   /** 413 Payload Too Large -- single chunk exceeded the hard cap.
   */
- Line 45:
   /** 500 Internal Server Error -- handler crashed.
   */

### package/webapp-content/messages-demo/src/server.ts

- Line 112:
   'dist/client/index.
  js missing -- run `mise run build:js:client` to enable the composer',

### package/webapp-content/messages-demo/src/server/api/drafts.ts

- Line 302:
   //region Local validation helpers -- thin wrappers,
   no library

### package/ssg/aquati.cat/README.md

- Line 60:
   - Authoring a post without committing is undefined behavior --
- Line 85:
   The `data-hl-*` attributes add ~1.1 KB compressed across all pages --
- Line 106:
   `yuv444p` balloons file size with no perceivable benefit for photographic content --
- Line 161:
   requested are retained -- the subsetted icon font is a few KB regardless
- Line 178:
   1.
   **Leaf assets** -- images,
   fonts,
   JS,
   PDFs,
   favicons (no outgoing references to other hashable assets)
- Line 179:
   2.
   **CSS** -- rewrite font `url()` references with hashed names from phase 1,
   then hash the CSS itself
- Line 180:
   3.
   **Reference rewriting** -- replace original basenames with hashed basenames in all HTML files and
   `manifest.webmanifest`

### package/ssg/aquati.cat/TODO.generalize-ssg.md

- Line 189:
   The risk is low -- no architectural changes,
   just plumbing a config object through existing code.

### package/ssg/aquati.cat/src/build.ts

- Line 48:
   // File justification:
   120 lines -- linear pipeline script;
   splitting the
- Line 74:
   //region Build orchestration -- loads content,
   processes MDX,
   generates pages and assets

### package/ssg/aquati.cat/src/build/favicon.ts

- Line 169:
   //region Standalone execution -- allows running via `mise run generate:favicons`

### package/ssg/aquati.cat/src/build/postprocess.ts

- Line 151:
   //region Phase 1 -- fingerprint leaf assets
- Line 199:
   //region Phase 2 -- fingerprint CSS
- Line 290:
   //region Phase 3 -- rewrite references
- Line 479:
   * -- neither branch touches HTML,
   so they race on disjoint files.

### package/ssg/aquati.cat/src/client/search.ts

- Line 118:
   // resolved at bundle time -- it must be a runtime import.
- Line 122:
   // @ts-expect-error -- Pagefind bundle is generated at build time by `pagefind --site dist`;
   no type declarations exist

### package/ssg/aquati.cat/src/components/site-footer.ts

- Line 29:
   // Cookie Clicker -- the game's newsticker has this news
- Line 31:
   // AI slop -- "slop" as slang for low-effort AI-generated content
- Line 35:
   // SpongeBob -- the Krusty Krab training video:
   "the finest dining establishment ever established for dining"
- Line 37:
   // Recursive error message -- an error handler that errors while handling errors
- Line 39:
   // TC39 pipeline operator proposal -- stuck in the standardization pipeline since 2017
- Line 41:
   // Mafumafu -- "すーぱーぬこになれんかった" (Super Nuko ni Narenkatta,
   2019)
- Line 43:
   // Francis Bacon -- "Of Studies" (1597),
   with the literal food reading swapped in
- Line 45:
   // Viral video -- Exotic Black TV cleaning a Himalayan marmot with a paint roller (2025)
- Line 47:
   // Phil Karlton -- There are only two hard things in Computer Science:
   cache invalidation and naming things.

### package/ssg/aquati.cat/src/content/en/about.mdx

- Line 11:
   Aquaticat -- UI/UX Designer and Developer.
- Line 20:
   My work spans the full product lifecycle --
- Line 37:
   **Interactive Media Design -- Ontario College Advanced Diploma**
- Line 38:
   Algonquin College,
   2023 -- 2026

### package/ssg/aquati.cat/src/content/en/portfolio-done.mdx

- Line 2:
   title:
   "DONE -- To-Do List App"
- Line 14:
   within the Monochromatic monorepo (2025 -- present).
- Line 17:
   DONE aims to eliminate the overhead that plagues most task management tools --
- Line 43:
   Developed visual moodboards establishing the geometric,
   minimal aesthetic --

### package/ssg/aquati.cat/src/content/en/portfolio-monochromatic.mdx

- Line 2:
   title:
   "Monochromatic -- Open-Source Design System and Monorepo"

### package/ssg/aquati.cat/src/content/en/portfolio-morph-apply.mdx

- Line 2:
   title:
   "Morph Apply -- Job Application Tracker Redesign"
- Line 30:
   1.
   **Research** -- Competitive analysis of existing AI tools,
- Line 32:
   2.
   **Branding** -- Developed a cohesive visual identity that conveys
- Line 34:
   3.
   **Wireframing** -- Iterated from low-fidelity sketches to high-fidelity layouts
- Line 35:
   4.
   **Prototyping** -- Interactive Figma prototypes for key user flows
- Line 36:
   5.
   **Presentation** -- Pitched the design to stakeholders with supporting research

### package/ssg/aquati.cat/src/content/en/portfolio-website.mdx

- Line 62:
   -- for example,
   `backdrop-filter: blur(...)` --

### package/ssg/aquati.cat/src/images/convert.ts

- Line 31:
   * Returns `false` on **any** access error,
   not only missing files --

### package/ssg/aquati.cat/src/images/format.ts

- Line 53:
   //region Top-level conversion pipeline -- scans directories and converts raster images to AVIF

### package/ssg/aquati.cat/src/lib/cache.ts

- Line 27:
   // File justification:
   164 lines -- schema definitions,
   I/O,
   and lookup form a

### package/ssg/aquati.cat/src/lib/content-group.ts

- Line 7:
   // File justification:
   104 lines -- grouping functions share the same type

### package/ssg/aquati.cat/src/lib/jsx-to-html.ts

- Line 20:
   * components) are called with their props and return `SafeHtml` directly --
- Line 146:
   //region Public API -- JSX runtime exports
- Line 217:
   //endregion Public API -- JSX runtime exports

### package/ssg/aquati.cat/src/styles/icons.ts

- Line 17:
   * -- is what makes tight subsetting possible:
   harfbuzz only retains

### package/webapp-edu/paper2vn/README.md

- Line 20:
   Single self-contained HTML file -- the build assembles CSS,
   the client JavaScript bundle,
   the placeholder sprite pack,
   and i18n strings into one file you can drop on any static hos
- Line 123:
   The bundled sprite pack uses its own license -- see `src/assets/sprites/manifest.json`.

### package/webapp-edu/paper2vn/TODO.lint-cleanup.md

- Line 61:
   Mostly auto-fixable formatting -- `dprint` should handle this once configured for the package.
- Line 72:
   2.
   Lift magic numbers to named constants -- knocks out `no-magic-numbers` and most `numeric-separators-style`.
- Line 73:
   3.
   Add missing TSDoc -- volume work,
   but mechanical.

### package/webapp-edu/paper2vn/src/client/dialogue/generator.ts

- Line 127:
   }\n\n[TRUNCATED -- ${paperText.
  length} total chars]`

### package/webapp-edu/paper2vn/src/client/i18n/ru/index.ts

- Line 52:
   baseUrlHint:
   'По умолчанию -- стандартный URL провайдера.
  ',
- Line 62:
   'Вы -- Рука,
   мягко говорящая наставница,
   которая читает академическую статью своему единственному ученику ("Master").
   Точны,
   сдержанно увлечены,
   верны источнику и не выдумываете ре
- Line 64:
   'Разбейте статью на 3-8 глав,
   следуя её логической структуре,
   и отвечайте на русском.
   Возвращайте JSON:
   `{ "title":
   string,
   "chapters":
   [{ "title":
   string,
   "summary":
   string,
   "dial

### package/webapp-edu/paper2vn/src/client/llm/anthropic.ts

- Line 107:
   `anthropic:
   HTTP ${res.
  status} ${res.
  statusText} -- ${

### package/webapp-edu/paper2vn/src/client/llm/ollama.ts

- Line 69:
   `ollama:
   HTTP ${res.
  status} ${res.
  statusText} -- ${

### package/webapp-edu/paper2vn/src/client/llm/openai-compatible.ts

- Line 86:
   `openai-compatible:
   HTTP ${res.
  status} ${res.
  statusText} -- ${

### package/webapp-edu/paper2vn/src/client/parse/pdf.ts

- Line 34:
   // unconfigured -- pdfjs falls back to a fake worker that runs on

### package/webapp-edu/paper2vn/src/client/screens/lecture.ts

- Line 53:
   /** Beat-runtime state -- kept on the screen instance.
   */
- Line 363:
   // End of paper -- park on the last beat.

### package/webapp-edu/paper2vn/src/client/storage-keys.ts

- Line 15:
   /** Save slots index -- list of `{ id, label, paperTitle, updatedAt }`.
   */

### package/webapp-edu/paper2vn/src/client/types.ts

- Line 22:
   /** A generated chapter -- one logical section of the paper.
   */

### package/webapp-edu/paper2vn/src/paper2vn.e2e.test.ts

- Line 53:
   /** Provider config (full or partial -- merged into defaults inside the page).
   */
- Line 165:
   //region Tier 1 -- pure UI tests
- Line 534:
   //region Tier 2 -- live LLM round-trip

### package/webapp-edu/paper2vn/src/styles/tokens.ts

- Line 61:
   /** Quarter rem -- smallest spacing unit */
- Line 79:
   /** Border radius -- subtle */
- Line 82:
   /** Border radius -- pill / rounded */
- Line 88:
   /** Eighth rem -- shadow offset */

### package/webapp-forge/PHASE-2-DEFERRED.md

- Line 115:
   mechanical now that `iso-server.ts` exists -- the renderers read git
- Line 121:
   The plan lists "Better Auth:
   email/password,
   sessions,
   magic links" --

### package/webapp-forge/server/src/data/queries-phase2-resources.unit.test.ts

- Line 298:
   // Guard against unused import warning -- exercise insertLabel here too

### package/webapp-forge/server/src/server/routes/git.cli.unit.test.ts

- Line 472:
   // like a clone would,
   but without writing the pack -- it's the

### package/webapp-forge/server/src/storage/adapter-s3.ts

- Line 14:
   * Path style:
   `${endpoint}/${bucket}/${key}`.
   Keys may contain `/` --

### package/webapp-forge/stress/src/scenarios/force-push.ts

- Line 18:
   * - "only affected blob/diff fragments rebuild" -- the dependency graph

### package/webapp-productivity/done-h-css-test/FRAMEWORK_EVALUATION.md

- Line 37:
   2.
   **No meta-framework ready.
  ** Nuxt 4 (stable July 2025,
   currently 4.3.1) does not integrate Vapor yet -- the roadmap checkbox is unchecked.
   `vue-i18n` has a known Vapor incompati
- Line 42:
   With SSR off the table,
   Vapor's beta status becomes less concerning -- client-only SPA rendering is more stable than SSR/hydration.
   But this raised the question:
   if the app is a cl
- Line 98:
   Done is not a simple app.
   It has a multi-tenant orchestrator with auth and process spawning,
   AI integration with structured output parsing,
   a task blocking dependency graph with ci
- Line 106:
   **Spreadsheets:
  ** Genuine reactive dependency graph (cell A1 depends on B2 depends on C3).
   But the rendering is a grid -- typically a `<canvas>` or virtualized table,
   not a compone
- Line 110:
   **Chat with presence:
  ** Typing indicators,
   read receipts,
   message arrival,
   presence dots -- each is one or two DOM mutations on specific,
   known elements.
   New message = `container.
  a
- Line 114:
   **Offline-first apps (Notion,
   Linear):
  ** The hard part is sync and conflict resolution,
   not UI state.
   Debatable whether offline-first is the right direction at all -- it trades ser
- Line 124:
   Done is strong evidence for this:
   an app with AI,
   multi-tenant orchestration,
   real-time timers,
   dependency graphs,
   full-text search,
   external sync,
   and email notifications -- and t
- Line 128:
   The above is coherent reasoning but still speculation.
   After the competition,
   we'll rewrite Done's UI layer in multiple frameworks and compare empirically.
   The codebase is structur

### package/webapp-productivity/done-h-css-test/PLAN.md

- Line 182:
   The schema below is the complete initial migration -- run it once on first startup.
- Line 709:
   The `sessions` table is the source of truth -- no crypto to get wrong.
- Line 778:
   Minimal styling -- these are functional forms,
   not the app itself.
- Line 1075:
   It handles auth,
   reverse proxy,
   and process management -- no Caddy or AuthCrunch needed.
- Line 1104:
   Coolify's reverse proxy handles HTTPS termination -- the orchestrator only listens on HTTP.
- Line 1114:
   Each child process runs `Bun.build()` at startup to bundle client assets -- no separate build step in the Dockerfile.
- Line 1253:
   These patterns are confirmed working -- no surprises expected during implementation.
- Line 1267:
   \| Static imports for DB + route handlers \| **validated** \| Dynamic imports unnecessary -- top-level await ensures
   build completes first \|
- Line 1279:
   \| Orchestrator multi-process spawning \| high \| Most complex untested piece -- budget extra time on day 5 \|
- Line 1342:
   **Core -- done:
  ** 1.1,
   1.2,
   2.1,
   2.2,
   2.3,
   2.4,
   4.1,
   4.2
- Line 1343:
   **Core -- partial:
  ** 1.3 (tasks + settings done;
   attachments/reminders not started)
- Line 1344:
   **Core -- not started:
  ** 4.3,
   4.4,
   5.1,
   5.2,
   5.3,
   7.1a,
   7.1b,
   7.1d,
   7.4,
   7.5

### package/webapp-productivity/done-h-css-test/README.md

- Line 20:
   1.
   **CSS** -- `@monochromatic-dev/build-css` resolves `@import` and expands `@mixin`/`@apply` into plain CSS
- Line 21:
   2.
   **Client JS** -- tsdown bundles one entry per page (inbox,
   in-progress,
   task-details,
   search,
   settings) via `mise run build:js:client`
- Line 22:
   3.
   **Server** -- `Bun.serve()` with declarative `routes` for pages and REST API;
   fallback handler serves static assets from `dist/client/`
- Line 23:
   4.
   **Database** -- SQLite (@tursodatabase/database) with FTS5 full-text search,
   initialized via side-effect import at startup
- Line 24:
   5.
   **Client** -- Vanilla TypeScript with custom elements;
   reads server-embedded JSON from `<script id="page-data">`,
   builds DOM imperatively

### package/webapp-productivity/done-h-css-test/SPEC.md

- Line 14:
   The orchestrator handles everything:
   auth,
   reverse proxy,
   and process management -- no Caddy or AuthCrunch needed.
- Line 16:
   Path-based routing (`done.app/u/<user-id>/`) instead of subdomains -- avoids DNS API calls on registration,
   offensive subdomain risk,
   wildcard cert complexity,
   and registrar rate l
- Line 22:
   5.
   On every request to `/u/<user-id>/*`,
   orchestrator validates the session cookie and checks that the session's user-id matches the path -- then reverse-prox
   ies to `localhost:
  $POR
- Line 27:
   User IDs are opaque (ULIDs),
   not user-chosen names -- no abuse vector for offensive URLs.
- Line 39:
   It does not just organize tasks -- it actively surfaces the right task at the right time and place.
- Line 69:
   All active timers increment in real time on the client using `setInterval(1s)` math against the server-provided
   `timerStartedAt` timestamp -- no polling or SSE needed for display.
- Line 70:
   Tasks blocked by other tasks are **not shown in Suggestions or All** -- they appear only nested/indented under the
   tasks blocking them (in any view where the blocker is visible).
   T
- Line 107:
   System-level notification display (not implemented in app -- defers to OS/browser push notifications via PWA).
- Line 112:
   If the SMTP provider has issues,
   that's the provider's problem -- the app fires and forgets.
- Line 117:
   This is the primary notification mechanism -- more reliable than browser push since the app may be suspended.
- Line 123:
   The raw `.db` file is also excluded for the same reason -- only the JSON export (tasks,
   settings,
   attachment metadata) is sent.
- Line 124:
   This is a caution-first data safety measure -- if the instance dies,
   the user has a recent export.
- Line 207:
   Outbound writes (modifying source files) are deferred post-competition -- too risky to auto-edit a user's codebase
   in week 1.
- Line 246:
   No separate API layer,
   no CORS,
   no client-side fetch() for basic CRUD -- forms submit natively,
   SvelteKit enhances with client-side navigation.
- Line 255:
   Coolify's own reverse proxy handles HTTPS termination -- the orchestrator only listens on HTTP.
- Line 258:
   1.
   **orchestrator** -- Bun image with the built SvelteKit app and orchestrator code.
   Listens on port 3000 (HTTP).
   Handles registration,
   login,
   session validation,
   path ACL enforcem
- Line 259:
   2.
   **llama-cpp** -- CPU-only `ghcr.io/ggml-org/llama.cpp:server` image.
   Shared AI inference for all users.
- Line 276:
   FTS5 search queries JOIN on `tasks.rowid`,
   not `tasks.id`.
   This is correct but easy to confuse -- be careful in implementation.

### package/webapp-productivity/done-h-css-test/TODO.figma-audit.md

- Line 85:
   1.
   "System calendar" -- a connect/disconnect button-style row with a sync icon and descriptive text
- Line 86:
   2.
   "Data privacy" -- a toggle switch beside the title,
   with description text below
- Line 87:
   3.
   "Dark theme" -- a toggle switch beside the title,
   with description text below
- Line 131:
   - In create mode,
   the header says "New task" and the save button says "Create" -- this matches Figma's "Task details" heading but the
   Figma design always shows "Task details" regar
- Line 155:
   1.
   **Missing icons throughout** -- nav links,
   metadata chips,
   action buttons all lack the icons shown in Figma.
   This is the single biggest visual difference.
- Line 156:
   2.
   **Drawer profile area** -- missing avatar icon and dropdown caret
- Line 157:
   3.
   **Suggested section layout on tablet** -- "My location" and "My focus" should sit side by side,
   not stack
- Line 158:
   4.
   **Section heading toggle** -- Unicode triangles instead of SVG chevrons
- Line 159:
   5.
   **Search results heading** -- no heading showing the current query term above results
- Line 160:
   6.
   **"Attach file" / "Take photo" buttons** -- missing paperclip and camera icons

### package/webapp-productivity/done-h-css-test/src/client/components/search-bar.ts

- Line 53:
   // HTMLElement creation -- SVG elements require the SVG namespace.

### package/webapp-productivity/done-h-css-test/src/client/components/side-drawer.ts

- Line 117:
   //region Inline sidebar -- visible in sidebar mode
- Line 132:
   //region Popover panel -- visible via hamburger in stacked mode

### package/webapp-productivity/done-h-css-test/src/client/components/task-detail-autofill.ts

- Line 17:
   /** Mutable metadata state -- autofill writes directly into this object.
   */
- Line 42:
   /** Clears all pending state -- call on reconfigure.
   */

### package/webapp-productivity/done-h-css-test/src/client/components/task-detail-render.ts

- Line 59:
   // elements -- SVG requires the SVG namespace.

### package/webapp-productivity/done-h-css-test/src/client/components/task-detail-types.ts

- Line 53:
   /** Component mode -- `"create"` for new tasks,
   `"edit"` (default) for existing.
   */

### package/webapp-productivity/done-h-css-test/src/client/in-progress.ts

- Line 87:
   // Live timer updates -- correlate each card with its task by DOM order

### package/webapp-productivity/done-h-css-test/src/client/inbox.ts

- Line 125:
   //region New-task dialog -- FAB opens a modal <dialog> with task-detail in create mode

### package/webapp-productivity/done-h-css-test/src/client/styles-tokens-dark.ts

- Line 9:
   /** Dark mode overrides -- swaps foreground/background primitives.
   */

### package/webapp-productivity/done-h-css-test/src/client/styles-tokens.ts

- Line 13:
   //region Primitive color tokens -- Raw color values that never change between modes.
- Line 47:
   //region Semantic color tokens -- Aliases that flip between light and dark modes.
- Line 87:
   //region Dark mode -- re-exported from `styles-tokens-dark.ts`

### package/webapp-productivity/done-h-css-test/src/lib/ai/client.ts

- Line 19:
   //region Rate limiter -- sliding-window counter
- Line 134:
   error:
   'Rate limit exceeded -- try again in a moment',

### package/webapp-productivity/done-h-css-test/src/lib/ai/prompts.ts

- Line 11:
   //region Autofill -- infer metadata from a task title
- Line 74:
   //region Suggestion ranking -- rank tasks by relevance to user context

### package/webapp-productivity/done-h-css-test/src/lib/db-migrations.ts

- Line 8:
   //region Migration SQL -- separated for readability;
   executed once at startup

### package/webapp-productivity/done-h-css-test/src/lib/db/tasks-helpers.ts

- Line 64:
   /** Outcome of a `completeTask()` call -- carries blockers when completion is refused.
   */

### package/webapp-productivity/done-h-css-test/src/lib/types.ts

- Line 81:
   /** Payload accepted by `createTask()` -- only `title` is required;
   all others default.
   */
- Line 94:
   /** Partial update payload accepted by `updateTask()` -- omitted fields stay unchanged.
   */

### package/webapp-productivity/done-h-css-test/src/server.ts

- Line 72:
   //region Page routes -- return full HTML documents (via renderPage / inline HTML)
- Line 115:
   //region API routes -- return JSON
- Line 121:
   //region Static asset serving -- bundled JS from dist/client/

### package/webapp-productivity/done/FRAMEWORK_EVALUATION.md

- Line 37:
   2.
   **No meta-framework ready.
  ** Nuxt 4 (stable July 2025,
   currently 4.3.1) does not integrate Vapor yet -- the roadmap checkbox is unchecked.
   `vue-i18n` has a known Vapor incompati
- Line 42:
   With SSR off the table,
   Vapor's beta status becomes less concerning -- client-only SPA rendering is more stable than SSR/hydration.
   But this raised the question:
   if the app is a cl
- Line 98:
   Done is not a simple app.
   It has a multi-tenant orchestrator with auth and process spawning,
   AI integration with structured output parsing,
   a task blocking dependency graph with ci
- Line 106:
   **Spreadsheets:
  ** Genuine reactive dependency graph (cell A1 depends on B2 depends on C3).
   But the rendering is a grid -- typically a `<canvas>` or virtualized table,
   not a compone
- Line 110:
   **Chat with presence:
  ** Typing indicators,
   read receipts,
   message arrival,
   presence dots -- each is one or two DOM mutations on specific,
   known elements.
   New message = `container.
  a
- Line 114:
   **Offline-first apps (Notion,
   Linear):
  ** The hard part is sync and conflict resolution,
   not UI state.
   Debatable whether offline-first is the right direction at all -- it trades ser
- Line 124:
   Done is strong evidence for this:
   an app with AI,
   multi-tenant orchestration,
   real-time timers,
   dependency graphs,
   full-text search,
   external sync,
   and email notifications -- and t
- Line 128:
   The above is coherent reasoning but still speculation.
   After the competition,
   we'll rewrite Done's UI layer in multiple frameworks and compare empirically.
   The codebase is structur

### package/webapp-productivity/done/PLAN.md

- Line 182:
   The schema below is the complete initial migration -- run it once on first startup.
- Line 709:
   The `sessions` table is the source of truth -- no crypto to get wrong.
- Line 778:
   Minimal styling -- these are functional forms,
   not the app itself.
- Line 1075:
   It handles auth,
   reverse proxy,
   and process management -- no Caddy or AuthCrunch needed.
- Line 1104:
   Coolify's reverse proxy handles HTTPS termination -- the orchestrator only listens on HTTP.
- Line 1114:
   Each child process runs `Bun.build()` at startup to bundle client assets -- no separate build step in the Dockerfile.
- Line 1253:
   These patterns are confirmed working -- no surprises expected during implementation.
- Line 1267:
   \| Static imports for DB + route handlers \| **validated** \| Dynamic imports unnecessary -- top-level await ensures
   build completes first \|
- Line 1279:
   \| Orchestrator multi-process spawning \| high \| Most complex untested piece -- budget extra time on day 5 \|
- Line 1342:
   **Core -- done:
  ** 1.1,
   1.2,
   2.1,
   2.2,
   2.3,
   2.4,
   4.1,
   4.2
- Line 1343:
   **Core -- partial:
  ** 1.3 (tasks + settings done;
   attachments/reminders not started)
- Line 1344:
   **Core -- not started:
  ** 4.3,
   4.4,
   5.1,
   5.2,
   5.3,
   7.1a,
   7.1b,
   7.1d,
   7.4,
   7.5

### package/webapp-productivity/done/README.md

- Line 21:
   1.
   **CSS** -- `@monochromatic-dev/build-css` resolves `@import` and expands `@mixin`/`@apply` into plain CSS
- Line 22:
   2.
   **Client JS** -- tsdown bundles one entry per page (inbox,
   in-progress,
   task-details,
   search,
   settings) via `mise run build:js:client`
- Line 23:
   3.
   **Server** -- `Bun.serve()` with declarative `routes` for pages and REST API;
   fallback handler serves static assets from `dist/client/`
- Line 24:
   4.
   **Database** -- SQLite (@tursodatabase/database) with FTS5 full-text search,
   initialized via side-effect import at startup
- Line 25:
   5.
   **Client** -- Vanilla TypeScript with custom elements;
   reads server-embedded JSON from `<script id="page-data">`,
   builds DOM imperatively

### package/webapp-productivity/done/SPEC.md

- Line 14:
   The orchestrator handles everything:
   auth,
   reverse proxy,
   and process management -- no Caddy or AuthCrunch needed.
- Line 16:
   Path-based routing (`done.app/u/<user-id>/`) instead of subdomains -- avoids DNS API calls on registration,
   offensive subdomain risk,
   wildcard cert complexity,
   and registrar rate l
- Line 22:
   5.
   On every request to `/u/<user-id>/*`,
   orchestrator validates the session cookie and checks that the session's user-id matches the path -- then reverse-prox
   ies to `localhost:
  $POR
- Line 27:
   User IDs are opaque (ULIDs),
   not user-chosen names -- no abuse vector for offensive URLs.
- Line 39:
   It does not just organize tasks -- it actively surfaces the right task at the right time and place.
- Line 69:
   All active timers increment in real time on the client using `setInterval(1s)` math against the server-provided
   `timerStartedAt` timestamp -- no polling or SSE needed for display.
- Line 70:
   Tasks blocked by other tasks are **not shown in Suggestions or All** -- they appear only nested/indented under the
   tasks blocking them (in any view where the blocker is visible).
   T
- Line 107:
   System-level notification display (not implemented in app -- defers to OS/browser push notifications via PWA).
- Line 112:
   If the SMTP provider has issues,
   that's the provider's problem -- the app fires and forgets.
- Line 117:
   This is the primary notification mechanism -- more reliable than browser push since the app may be suspended.
- Line 123:
   The raw `.db` file is also excluded for the same reason -- only the JSON export (tasks,
   settings,
   attachment metadata) is sent.
- Line 124:
   This is a caution-first data safety measure -- if the instance dies,
   the user has a recent export.
- Line 207:
   Outbound writes (modifying source files) are deferred post-competition -- too risky to auto-edit a user's codebase
   in week 1.
- Line 246:
   No separate API layer,
   no CORS,
   no client-side fetch() for basic CRUD -- forms submit natively,
   SvelteKit enhances with client-side navigation.
- Line 255:
   Coolify's own reverse proxy handles HTTPS termination -- the orchestrator only listens on HTTP.
- Line 258:
   1.
   **orchestrator** -- Bun image with the built SvelteKit app and orchestrator code.
   Listens on port 3000 (HTTP).
   Handles registration,
   login,
   session validation,
   path ACL enforcem
- Line 259:
   2.
   **llama-cpp** -- CPU-only `ghcr.io/ggml-org/llama.cpp:server` image.
   Shared AI inference for all users.
- Line 276:
   FTS5 search queries JOIN on `tasks.rowid`,
   not `tasks.id`.
   This is correct but easy to confuse -- be careful in implementation.

### package/webapp-productivity/done/TODO.figma-audit.md

- Line 85:
   1.
   "System calendar" -- a connect/disconnect button-style row with a sync icon and descriptive text
- Line 86:
   2.
   "Data privacy" -- a toggle switch beside the title,
   with description text below
- Line 87:
   3.
   "Dark theme" -- a toggle switch beside the title,
   with description text below
- Line 131:
   - In create mode,
   the header says "New task" and the save button says "Create" -- this matches Figma's "Task details" heading but the
   Figma design always shows "Task details" regar
- Line 155:
   1.
   **Missing icons throughout** -- nav links,
   metadata chips,
   action buttons all lack the icons shown in Figma.
   This is the single biggest visual difference.
- Line 156:
   2.
   **Drawer profile area** -- missing avatar icon and dropdown caret
- Line 157:
   3.
   **Suggested section layout on tablet** -- "My location" and "My focus" should sit side by side,
   not stack
- Line 158:
   4.
   **Section heading toggle** -- Unicode triangles instead of SVG chevrons
- Line 159:
   5.
   **Search results heading** -- no heading showing the current query term above results
- Line 160:
   6.
   **"Attach file" / "Take photo" buttons** -- missing paperclip and camera icons

### package/webapp-productivity/done/src/client/components/search-bar.ts

- Line 92:
   // HTMLElement creation -- SVG elements require the SVG namespace.

### package/webapp-productivity/done/src/client/components/side-drawer.ts

- Line 126:
   //region Inline sidebar -- visible in sidebar mode
- Line 141:
   //region Popover panel -- visible via hamburger in stacked mode

### package/webapp-productivity/done/src/client/components/task-detail-render.ts

- Line 59:
   // elements -- SVG requires the SVG namespace.

### package/webapp-productivity/done/src/client/in-progress.ts

- Line 88:
   // Live timer updates -- correlate each card with its task by DOM order

### package/webapp-productivity/done/src/lib/ai/client.ts

- Line 19:
   //region Rate limiter -- sliding-window counter
- Line 135:
   error:
   'Rate limit exceeded -- try again in a moment',

### package/webapp-productivity/done/src/lib/ai/prompts.ts

- Line 11:
   //region Autofill -- infer metadata from a task title
- Line 74:
   //region Suggestion ranking -- rank tasks by relevance to user context

### package/webapp-productivity/done/src/lib/db/task-timer.ts

- Line 29:
   /** Outcome of a `completeTask()` call -- carries blockers when completion is refused.
   */

### package/webapp-productivity/done/src/lib/types.ts

- Line 81:
   /** Payload accepted by `createTask()` -- only `title` is required;
   all others default.
   */
- Line 94:
   /** Partial update payload accepted by `updateTask()` -- omitted fields stay unchanged.
   */

### package/webapp-productivity/rss/src/client.ts

- Line 7:
   //region Scroll event observer -- Tracks element visibility and dispatches custom scroll lifecycle events
- Line 97:
   //region Feed element binding -- Connects scroll events to the ignore API for auto-dismissal

### package/webapp-productivity/rss/src/feed.ts

- Line 30:
   //region Feed fetching and sorting -- Retrieves feeds from URLs,
   parses them,
   and sorts by date

### package/webapp-productivity/rss/src/handler.ts

- Line 21:
   //region HTTP handlers -- Serve rendered HTML and persist ignored items

### package/webapp-productivity/rss/src/ignore.ts

- Line 18:
   //region Ignore content loading -- Reads raw JSONL content for salt derivation and link filtering

### package/webapp-productivity/rss/src/index.ts

- Line 30:
   //region Memoized pipeline -- Pull-based feed processing with content-derived cache invalidation
- Line 100:
   //region h3 application -- Maps HTTP method + path to handler functions

### package/webapp-productivity/rss/src/item.ts

- Line 24:
   //region Item extraction and normalization -- Converts feed entries to a uniform dated format

### package/webapp-productivity/rss/src/opml-text.ts

- Line 25:
   //region OPML text fetching -- Retrieves raw OPML content from HTTP and file URLs

### package/webapp-productivity/rss/src/outline.ts

- Line 25:
   //region OPML parsing and outline extraction -- Converts raw XML into validated feed outline structures

### package/webapp-search/ai-tree/src/index.ts

- Line 56:
   // @ts-expect-error -- mcp_servers not yet in SDK types

### package/webapp-search/exa-search/src/asset.ts

- Line 37:
   //region HTML structure -- Declarative page composition via h-html

## D. ASCII `-` em-dash substitute prose asides

Replace with proper punctuation or restructure sentence.

### AUDIT.dry.md

- Line 623:
   (4 files in the family) - each addresses a distinct symptom;
   no content overlap.
- Line 624:
   - `pi-*` (2:
   compaction-empty-summary,
   safeguard) - distinct.
- Line 625:
   - `css-*` (2:
   hidden-attribute-specificity,
   css-tooling) - distinct.
- Line 626:
   - `dprint*` (2:
   dprint,
   dprint-exec) - distinct.
- Line 627:
   - `performance.*` (3:
   performance,
   performance.
  build,
   performance.
  logging) - distinct.

### AUDIT.md

- Line 99:
   - **zod** - Impossible to avoid.
- Line 106:
   - **`@logtape/*`** - To be replaced by our custom logger

### PHILOSOPHY.browser-support.md

- Line 7:
   1.
   **CSS Container Queries** - `@container` and container units

### PHILOSOPHY.md

- Line 9:
   - **[Portability and Core Principles](../philosophy/portability.md)** - Foundational principles around portability,
   interoperability,
   and detachable solutions
- Line 10:
   - **[Build and Execution](../philosophy/build-execution.md)** - Technical decisions about build systems and script
   - execution
- Line 11:
   - **[Tool Choices](../philosophy/tool-choices.md)** - Rationale for HTTP framework,
   editor,
   linting,
   testing,
   bundler,
   and AI SDK selections
- Line 12:
   - **[Browser Support](../philosophy/browser-support.md)** - Future considerations for browser feature adoption
- Line 13:
   - **[CSS](../philosophy/css.md)** - h-css hyperscript pattern,
   Shadow DOM style injection,
   and why alternatives don't fit

### README.md

- Line 23:
   Statistical threshold detection (mean - 2*stddev) flags model degradation

### TODO.automation.md

- Line 7:
   - [**CI/CD Pipeline**](#cicd-pipeline) - Continuous integration and deployment
- Line 8:
   - [**Development Automation**](#development-automation) - Developer workflow automation
- Line 9:
   - [**Release Automation**](#release-automation) - Automated versioning and publishing
- Line 10:
   - [**Infrastructure Automation**](#infrastructure-automation) - Infrastructure as code
- Line 11:
   - [**Testing Automation**](#testing-automation) - Automated testing strategies
- Line 12:
   - [**Monitoring Automation**](#monitoring-automation) - Automated monitoring and alerting
- Line 22:
   **Status**:
   High Priority - Development workflow
- Line 33:
   **Status**:
   High Priority - Build reliability
- Line 44:
   **Status**:
   High Priority - Production reliability
- Line 81:
   **Status**:
   High Priority - Developer productivity
- Line 92:
   **Status**:
   High Priority - Code standards
- Line 103:
   **Status**:
   High Priority - Developer experience
- Line 140:
   **Status**:
   High Priority - Release management
- Line 151:
   **Status**:
   High Priority - Distribution
- Line 162:
   **Status**:
   High Priority - Release quality
- Line 199:
   **Status**:
   High Priority - Infrastructure management
- Line 210:
   **Status**:
   High Priority - Operations
- Line 221:
   **Status**:
   High Priority - Modern deployment
- Line 258:
   **Status**:
   High Priority - Quality assurance
- Line 269:
   **Status**:
   High Priority - Compatibility
- Line 280:
   **Status**:
   High Priority - System validation
- Line 317:
   **Status**:
   High Priority - Operational visibility
- Line 328:
   **Status**:
   High Priority - System reliability
- Line 339:
   **Status**:
   High Priority - Operational efficiency
- Line 374:
   1.
   **CI/CD Pipeline Enhancement** - Improve build and deployment reliability
- Line 375:
   2.
   **Code Quality Automation** - Enforce standards automatically
- Line 376:
   3.
   **Release Automation** - Streamline package publishing
- Line 377:
   4.
   **Basic Monitoring Automation** - Essential operational visibility
- Line 381:
   1.
   **Development Environment Automation** - Improve developer experience
- Line 382:
   2.
   **Testing Automation** - Comprehensive quality assurance
- Line 383:
   3.
   **Infrastructure Automation** - Reliable and scalable operations
- Line 384:
   4.
   **Advanced Monitoring** - Proactive issue detection and resolution
- Line 388:
   1.
   **Advanced Automation** - Sophisticated workflow automation
- Line 389:
   2.
   **Predictive Analytics** - Proactive maintenance and optimization
- Line 390:
   3.
   **Full Automation Integration** - End-to-end automated workflows
- Line 391:
   4.
   **Continuous Improvement** - Self-optimizing automation systems

### TODO.build-system.md

- Line 5:
   - [**Code Quality Issues**](../todo/code-quality.md#current-linting-issues) - Related linting and TypeScript fixes
- Line 6:
   - [**Performance Optimization**](../todo/performance.md#build-performance) - Build system performance improvements
- Line 7:
   - [**Automation**](../todo/automation.md#cicd-pipeline) - CI/CD pipeline integration
- Line 8:
   - [**Package Development**](../todo/packages.md#module-library-packages-modulees) - Module library build requirements
- Line 14:
   **Status**:
   High Priority - Blocking new developers
- Line 51:
   **Status**:
   Critical - TypeScript compilation errors
- Line 101:
   **Status**:
   High Priority - Build reliability
- Line 123:
   **Status**:
   Medium Priority - Build efficiency
- Line 155:
   **Status**:
   Normal Priority - Development reliability
- Line 188:
   **Status**:
   Normal Priority - Build optimization
- Line 232:
   **Status**:
   High Priority - Production deployment
- Line 244:
   **Status**:
   Normal Priority - Developer experience
- Line 267:
   - **IMPORTANT**:
   Never run direct package scripts - always use `mise run` commands

### TODO.cli-tools.md

- Line 5:
   - [**Build System Integration**](../todo/build-system.md#package-management-improvements) - Package management
   - tooling integration
- Line 6:
   - [**Automation Tools**](../todo/automation.md#development-automation) - Development workflow automation
- Line 7:
   - [**Package Development**](../todo/packages.md#build-utilities) - Build utility enhancement
- Line 8:
   - [**Performance Tools**](../todo/performance.md#build-performance) - Performance monitoring and optimization tools
- Line 9:
   - [**Security Tools**](../todo/security.md#development-security) - Security-focused development tools
- Line 136:
   **Status**:
   Normal Priority - Developer productivity
- Line 149:
   **Status**:
   Normal Priority - Developer experience
- Line 173:
   - NEVER use process.
  exit() - throw errors instead
- Line 185:
   **Status**:
   High Priority - Tool security
- Line 217:
   **Status**:
   Low Priority - Experimental
- Line 259:
   **Status**:
   Normal Priority - Environment management
- Line 272:
   **Status**:
   Normal Priority - Performance optimization

### TODO.code-quality.md

- Line 5:
   - [**Build System Issues**](../todo/build-system.md#missing-export-issues) - TypeScript compilation and export fixes
- Line 6:
   - [**Security Practices**](../todo/security.md#development-security) - Secure coding guidelines and practices
- Line 7:
   - [**Performance Patterns**](../todo/performance.md#runtime-performance) - Performance-focused coding patterns
- Line 8:
   - [**Package Standards**](../todo/packages.md#cross-package-improvements) - Cross-package coding standards
- Line 9:
   - [**Automation Tools**](../todo/automation.md#code-quality-automation) - Automated code quality improvements
- Line 19:
   - **NEVER use single-letter variables like `i`,
   `j`,
   `k`** - they provide no semantic meaning
- Line 24:
   **Status**:
   Critical - Blocks builds
- Line 125:
   1.
   **Question every construct** - Each programming construct adds complexity
- Line 126:
   2.
   **Prefer immutability** - Mutable variables should be eliminated when possible
- Line 127:
   3.
   **Prefer declarative over imperative** - Loops can often be replaced with higher-order functions
- Line 128:
   4.
   **Extract and name concepts** - Helper functions like `isTaskPending` improve readability
- Line 129:
   5.
   **Think functionally first** - There's often a functional solution that's cleaner
- Line 130:
   6.
   **Simplify progressively** - Don't stop at the first working solution
- Line 189:
   - **NEVER use single-letter variables** - they provide no semantic meaning
- Line 196:
   - **NEVER use process.
  exit()** - throw errors instead
- Line 214:
   1.
   **Code Quality Issues** - Always fix in code,
   not config
- Line 215:
   2.
   **Style Preferences** - Can be configured for team consistency
- Line 216:
   3.
   **Framework-specific** - May need configuration for specific use cases
- Line 263:
   **Status**:
   High Priority - Security integration

### TODO.completed.md

- Line 53:
   - `moon run precommit` - manually run all pre-commit checks
- Line 54:
   - `moon run validate` - run format + build + test for thorough local validation
- Line 110:
   - [x] Disable `jsdoc/tag-lines` - formatting concern,
   not linting
- Line 124:
   - `fixture.promises.0to999.ts` - changed to `promiseIndex`,
   `batchStart`,
   `index`
- Line 125:
   - `fixture.generator.0to999.ts` - changed to `value`,
   `delayMilliseconds`,
   `iteration`,
   `milliseconds`,
   `valueIndex`
- Line 126:
   - `iterable.chunks.ts` - changed to `chunkStart`,
   `value`
- Line 127:
   - `iterable.entries.ts` - changed to `value`,
   `index`
- Line 128:
   - `iterables.intersection.ts` - changed to `value`
- Line 129:
   - `moon.index-claude-user-messages.ts` - changed to `batchStart`
- Line 130:
   - `logtape.shared.ts` - changed to `messageIndex`
- Line 131:
   - `any.echo.unit.test.ts` - changed to `iteration`
- Line 132:
   - `function.memoize.ts` - changed to `argIndex`
- Line 133:
   - `iterable.take.unit.test.ts` - changed to `value`
- Line 134:
   - `promises.some.bench.ts` - changed to `index` in Array.
  from callbacks
- Line 135:
   - `fixture.index.ts` - changed to `index` in Array.
  from callbacks
- Line 136:
   - `iterable.partition.ts` - changed to `item` for iterator values
- Line 140:
   - `moon.index-claude-mcp-logs.ts` - changed to `error` (3 occurrences)
- Line 141:
   - `deprecated.testing.ts` - changed to `error`
- Line 142:
   - `fs.fs.default.ts` - changed to `error`
- Line 146:
   - `error.assert.equal.unit.test.ts` - added braces to arrow functions returning void
- Line 147:
   - `any.constant.unit.test.ts` - stored undefined result before testing
- Line 148:
   - `any.identity.unit.test.ts` - stored undefined result before testing
- Line 149:
   - `any.test.ts` - stored undefined result before testing
- Line 153:
   - `package/figma-plugin/css-variables/src/iframe/index.ts` - replaced window.
  getComputedStyle and window.
  parent.
  postMessage
- Line 154:
   - `package/figma-plugin/css-variables/src/frontend/index.ts` - replaced window.
  parent.
  postMessage and window.
  addEventListener
- Line 155:
   - `logtape.default.ts` - replaced window.
  sessionStorage with globalThis.
  sessionStorage
- Line 246:
   1.
   **Question every construct** - Each programming construct adds complexity
- Line 247:
   2.
   **Prefer immutability** - Mutable variables should be eliminated when possible
- Line 248:
   3.
   **Prefer declarative over imperative** - Loops can often be replaced with higher-order functions
- Line 249:
   4.
   **Extract and name concepts** - Helper functions like `isTaskPending` improve readability
- Line 250:
   5.
   **Think functionally first** - There's often a functional solution that's cleaner
- Line 251:
   6.
   **Simplify progressively** - Don't stop at the first working solution

### TODO.development.md

- Line 9:
   - `main` - Protected production-ready branch
- Line 13:
   - `dev` - Active development branch

### TODO.documentation.md

- Line 5:
   - [**Package Documentation**](../todo/packages.md#cross-package-improvements) - Package-specific documentation needs
- Line 6:
   - [**Security Documentation**](../todo/security.md#security-documentation) - Security guidelines and procedures
- Line 7:
   - [**Performance Documentation**](../todo/performance.md#monitoring--metrics) - Performance guidelines and metrics
- Line 8:
   - [**Automation Documentation**](../todo/automation.md#documentation-automation) - Automated documentation generation
- Line 9:
   - [**Build System Documentation**](../todo/build-system.md#mise-configuration-enhancements) - Build system and
   - tooling documentation
- Line 17:
   **Status**:
   Normal Priority - Content creation enhancement
- Line 32:
   **Status**:
   High Priority - Developer experience
- Line 45:
   **Status**:
   Normal Priority - Performance improvement
- Line 68:
   **Status**:
   Medium Priority - User experience
- Line 84:
   **Status**:
   Medium Priority - Content management
- Line 99:
   **Status**:
   Low Priority - Developer experience
- Line 118:
   **Status**:
   Normal Priority - User experience
- Line 133:
   **Status**:
   Low Priority - Future enhancement
- Line 152:
   **Status**:
   On Hold - Potentially annoying
- Line 155:
   Currently on hold - could be annoying for users.
- Line 172:
   **Status**:
   Medium Priority - Decentralized web
- Line 178:
   **Status**:
   Medium Priority - GitHub integration
- Line 184:
   **Status**:
   Medium Priority - Flexibility

### TODO.md

- Line 5:
   - [**Build System & Package Management**](../todo/build-system.md) - mise,
   TypeScript,
   dependencies
- Line 6:
   - [**CLI Tools & Utilities**](../todo/cli-tools.md) - Custom tools and automation scripts
- Line 7:
   - [**Documentation & UI/UX**](../todo/documentation.md) - Content,
   design,
   and user experience
- Line 8:
   - [**Development Environment**](../todo/development.md) - Tooling,
   setup
- Line 9:
   - [**Code Quality & Patterns**](../todo/code-quality.md) - Linting,
   testing,
   best practices
- Line 10:
   - [**Package-Specific Improvements**](../todo/packages.md) - Module library,
   config packages,
   style framework
- Line 11:
   - [**Security & Infrastructure**](../todo/security.md) - Application security,
   deployment hardening
- Line 12:
   - [**Performance & Optimization**](../todo/performance.md) - Build performance,
   runtime optimization
- Line 13:
   - [**Automation & DevOps**](../todo/automation.md) - CI/CD,
   development automation,
   release management
- Line 14:
   - [**VM Dev Environment**](../todo/vm-dev-environment.md) - Portable immutable VM image
   - ([rationale](../philosophy/vm-dev-environment.md))
- Line 15:
   - **Completed Tasks** - Reference for finished work

### TODO.packages.md

- Line 9:
   - [**Module Library**](#module-library-packages-modulees) - Functional programming utilities
- Line 10:
   - [**Configuration Packages**](#configuration-packages) - Shareable tool configurations
- Line 11:
   - [**Style Packages**](#style-packages) - CSS framework and design system
- Line 12:
   - [**Site Packages**](#site-packages) - Applications and documentation
- Line 13:
   - [**Figma Plugins**](#figma-plugins) - Design tool integrations
- Line 14:
   - [**Build Utilities**](#build-utilities) - Build-time tools and scripts
- Line 24:
   **Status**:
   Normal Priority - Core library expansion
- Line 37:
   **Status**:
   High Priority - Missing critical functionality
- Line 49:
   **Status**:
   Normal Priority - Common operations
- Line 61:
   **Status**:
   Normal Priority - Common need
- Line 73:
   **Status**:
   High Priority - Type safety
- Line 76:
   - [ ] Add `validate.url()` - URL validation with protocol options
- Line 77:
   - [ ] Add `validate.json()` - JSON validation with schema support

### TODO.performance.build.md

- Line 9:
   **Status**:
   High Priority - Developer experience
- Line 20:
   **Status**:
   High Priority - Daily development impact
- Line 31:
   **Status**:
   Normal Priority - Build tools

### TODO.performance.bundle.md

- Line 9:
   **Status**:
   Normal Priority - Application performance
- Line 20:
   **Status**:
   Normal Priority - Bundle size
- Line 31:
   **Status**:
   Normal Priority - Load performance

### TODO.performance.caching.md

- Line 9:
   **Status**:
   High Priority - Performance multiplier
- Line 20:
   **Status**:
   High Priority - Network performance
- Line 31:
   **Status**:
   High Priority - Development experience

### TODO.performance.infrastructure.md

- Line 9:
   **Status**:
   High Priority - Application performance
- Line 20:
   **Status**:
   High Priority - Latency reduction
- Line 31:
   **Status**:
   Normal Priority - Deployment efficiency

### TODO.performance.md

- Line 74:
   1.
   **Build Performance Monitoring** - Establish baseline measurements
- Line 75:
   2.
   **Critical Path Optimization** - Focus on most impactful improvements
- Line 76:
   3.
   **Memory Leak Detection** - Identify and fix resource issues
- Line 77:
   4.
   **Basic Caching** - Implement fundamental caching strategies
- Line 81:
   1.
   **Bundle Optimization** - Reduce load times and resource usage
- Line 82:
   2.
   **Runtime Performance** - Optimize hot paths and algorithms
- Line 83:
   3.
   **Advanced Caching** - Implement sophisticated caching strategies
- Line 84:
   4.
   **Performance Testing** - Establish automated performance validation
- Line 88:
   1.
   **Infrastructure Optimization** - Optimize deployment and hosting
- Line 89:
   2.
   **Advanced Monitoring** - Implement comprehensive performance tracking
- Line 90:
   3.
   **Performance Culture** - Establish performance-first development practices
- Line 91:
   4.
   **Continuous Optimization** - Create ongoing performance improvement processes

### TODO.performance.monitoring.md

- Line 9:
   **Status**:
   High Priority - Visibility and optimization
- Line 20:
   **Status**:
   High Priority - User experience
- Line 31:
   **Status**:
   High Priority - Optimization guidance

### TODO.performance.runtime.md

- Line 9:
   **Status**:
   High Priority - User experience
- Line 20:
   **Status**:
   High Priority - Function tracing performance
- Line 31:
   **Status**:
   High Priority - Resource efficiency
- Line 42:
   **Status**:
   High Priority - Responsiveness

### TODO.security.md

- Line 7:
   - [**Application Security**](#application-security) - Code security and vulnerability management
- Line 8:
   - [**Infrastructure Security**](#infrastructure-security) - Deployment and hosting security
- Line 9:
   - [**Development Security**](#development-security) - Secure development practices
- Line 10:
   - [**Dependency Security**](#dependency-security) - Supply chain security
- Line 11:
   - [**Monitoring & Incident Response**](#monitoring--incident-response) - Security monitoring
- Line 21:
   **Status**:
   High Priority - Security fundamentals
- Line 32:
   **Status**:
   Medium Priority - Multi-user scenarios
- Line 43:
   **Status**:
   High Priority - Privacy and compliance
- Line 80:
   **Status**:
   High Priority - Deployment security
- Line 91:
   **Status**:
   High Priority - Docker deployment
- Line 102:
   **Status**:
   High Priority - Web server security
- Line 139:
   **Status**:
   High Priority - Developer education
- Line 150:
   **Status**:
   High Priority - Credential security
- Line 161:
   **Status**:
   High Priority - Vulnerability detection
- Line 198:
   **Status**:
   High Priority - Supply chain security
- Line 209:
   **Status**:
   High Priority - Third-party risk
- Line 237:
   **Status**:
   High Priority - Threat detection
- Line 248:
   **Status**:
   High Priority - Security incidents
- Line 283:
   1.
   **Dependency Vulnerability Scanning** - Immediate risk reduction
- Line 284:
   2.
   **Secrets Audit and Management** - Prevent credential exposure
- Line 285:
   3.
   **Input Validation** - Basic security hardening
- Line 286:
   4.
   **Security Logging** - Visibility into security events
- Line 290:
   1.
   **Container Security** - Secure deployment practices
- Line 291:
   2.
   **API Security** - Protect application interfaces
- Line 292:
   3.
   **Infrastructure Hardening** - Secure server configurations
- Line 293:
   4.
   **Security Testing** - Automated vulnerability detection
- Line 297:
   1.
   **Threat Detection** - Advanced monitoring and alerting
- Line 298:
   2.
   **Incident Response** - Comprehensive response capabilities
- Line 299:
   3.
   **Compliance** - Meet security standards and regulations
- Line 300:
   4.
   **Security Culture** - Training and awareness programs

### TROUBLESHOOTING.css-tooling.md

- Line 110:
   - `hoist: false` - Don't hoist dependencies (good for strictness)
- Line 111:
   - `nodeLinker: isolated` - Use isolated node_modules (good for correctness)
- Line 112:
   - `enableGlobalVirtualStore: true` - Use global store (good for disk space)
- Line 113:
   - `hoistWorkspacePackages: false` - Don't hoist workspace packages
- Line 174:
   Created a PostCSS plugin that handles `@mixin` and `@apply`.
   Tested it standalone - works perfectly.
- Line 176:
   Put it in Vite's `css.postcss.plugins` - works for most files.
- Line 206:
   - `postcss` (default) - uses PostCSS
- Line 207:
   - `lightningcss` - uses LightningCSS
- Line 319:
   1.
   **Every abstraction has leaky edges** - LightningCSS is fast until you need custom at-rules with CSS variables
- Line 320:
   2.
   **Package managers are not interchangeable** - pnpm's strictness creates real-world problems that npm/yarn don't have
- Line 321:
   3.
   **Frameworks own their pipelines** - Astro's CSS handling is Astro's business,
   not yours
- Line 322:
   4.
   **"Just works" never does** - Every tool that promises simplicity hides complexity
- Line 323:
   5.
   **The JavaScript ecosystem is held together by duct tape** - Native ESM,
   TypeScript,
   bundlers,
   package managers,
   frameworks - they all make assumptions that conflict with each o

### TROUBLESHOOTING.editors.md

- Line 138:
   - Helix doesn't require a plugin system - language servers provide IDE features

### TROUBLESHOOTING.typeguards.md

- Line 152:
   **PARTIALLY FALSE** - It depends on the input type.
- Line 170:
   **FALSE** - `any` still gets narrowed by typeguards.

### package/dev-script/inference-canary-viewer/README.md

- Line 47:
   4.
   Calculates per-model degradation thresholds (mean - 2 * stddev,
   floored at 0.3,
   minimum 3 samples)

### package/dev-script/inference-canary/README.md

- Line 131:
   Used for statistical threshold computation (mean - 2 * stddev).

### package/dev-script/inference-canary/src/canary-lint/Kimi K2.5/stak-interpreter-initial-2026-03-14T23-20-05.000Z/response.txt

- Line 132:
   const value = stack[stack.
  length - 1];
- Line 181:
   stack.
  push(a - b);
- Line 195:
   stack.
  push(a - b * Math.
  floor(a / b));

### package/dev-script/inference-canary/src/canary-lint/MiniMax M2.5/css-mixin-transpiler-initial-2026-03-14T22-39-24.000Z/response.txt

- Line 781:
   Still has some issues - let me reconsider and rewrite with a cleaner approach:
- Line 1186:
   1.
   Nested `@apply` in mixins - handled via recursive `expandApplyInContent`
- Line 1187:
   2.
   Multiple `@apply` in one rule - loop processes each one
- Line 1188:
   3.
   Same mixin referenced multiple times - each reference gets fresh expansion
- Line 1189:
   4.
   `@apply` inside nested rules with `&` - processed since we track brace depth
- Line 1190:
   5.
   Top-level `@apply` - handled with braceDepth check

### package/dev-script/inference-canary/src/canary-lint/Nvidia Nemotron 3 Super/css-mixin-transpiler-initial-2026-03-19T17-45-20.000Z/response.txt

- Line 289:
   // - Take the mixin body string for applyName,
   and then tokenize it and expand any `@apply` inside it?
- Line 290:
   // - But we are in the middle of the input token stream.
   We cannot easily go back to re-tokenize the stored mixin body string.

### package/figma/to-penpot/PENPOT_FORMAT.md

- Line 75:
   - `data` (optional - the actual design data,
   not stored in file JSON)
- Line 277:
   1.
   `objects/{id}.json` - metadata:
   `{id, size, contentType, bucket, hash}`
- Line 278:
   2.
   `objects/{id}.{ext}` - the actual binary data (png,
   jpg,
   svg,
   woff,
   woff2,
   ttf,
   otf)
- Line 338:
   - Export:
   `backend/src/app/binfile/v3.clj` - `write-entry!`,
   `export-file`,
   `export-files`
- Line 339:
   - Import:
   `backend/src/app/binfile/v3.clj` - `read-entry`,
   `import-file`,
   `import-storage-objects`
- Line 345:
   - Cleaner:
   `backend/src/app/binfile/cleaner.clj` - pre/post decode fixes
- Line 346:
   - Common:
   `backend/src/app/binfile/common.clj` - `file-attrs`,
   shared utilities

### package/module/es/README.md (52 occurrences, first 20 shown)

- Line 20:
   - **Boolean utilities** - Equality,
   logical operations,
   type predicates
- Line 21:
   - **Error utilities** - Comprehensive error handling and assertion functions
- Line 22:
   - **Function utilities** - Composition,
   memoization,
   currying,
   and functional patterns
- Line 23:
   - **Numeric utilities** - Addition,
   type guards,
   range validation,
   BigInt support
- Line 24:
   - **String utilities** - Validation,
   transformation,
   hashing,
   and formatting
- Line 25:
   - **Basic array utilities** - Type guards,
   range generation,
   basic operations
- Line 29:
   - **Iterable utilities** - Good sync support,
   missing many async variants
- Line 30:
   - **Array utilities** - Basic operations exist,
   missing advanced algorithms
- Line 31:
   - **Promise utilities** - Basic support,
   missing advanced async patterns
- Line 32:
   - **Type utilities** - Some type-level programming,
   needs expansion
- Line 36:
   - **Object utilities** - Pick,
   omit,
   merge,
   transform,
   deep operations
- Line 37:
   - **Date/time utilities** - Parsing,
   formatting,
   arithmetic,
   timezone handling
- Line 38:
   - **Math utilities** - Statistics,
   interpolation,
   geometric operations
- Line 39:
   - **Validation utilities** - Schema validation,
   input sanitization
- Line 40:
   - **Collection utilities** - Set operations,
   Map transformations
- Line 41:
   - **Stream utilities** - Async stream processing and transformation
- Line 42:
   - **Parser utilities** - Text parsing,
   tokenization,
   grammar handling
- Line 43:
   - **Crypto utilities** - Hashing,
   encoding,
   secure random generation
- Line 44:
   - **Network utilities** - URL manipulation,
   query string handling
- Line 45:
   - **Geometry utilities** - Point,
   vector,
   shape operations
- ... and 32 more

### package/module/es/TODO.2-week-presentability.md

- Line 5:
   **Timeline**:
   August 16 - August 30,
   2025
- Line 13:
   **Status**:
   Critical - Blocks all development
- Line 29:
   **Status**:
   High Priority - Common utility functions expected by users
- Line 58:
   **Status**:
   High Priority - Quality assurance
- Line 74:
   **Status**:
   High Priority - Professional presentation
- Line 89:
   **Status**:
   Medium Priority - Marketing and usability
- Line 105:
   **Status**:
   Medium Priority - Distribution readiness
- Line 167:
   - [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - 350+ additional functions
- Line 177:
   - Focus is on quality over quantity - 10 excellent functions vs 50 mediocre ones

### package/module/es/TODO.3-month-aggressive-plan.md

- Line 5:
   **Timeline**:
   August 16 - November 16,
   2025 (12 weeks)
- Line 46:
   **Status**:
   Foundation phase - must complete before parallel work
- Line 111:
   **Focus**:
   Maximum parallel function implementation - 250+ functions
- Line 151:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Object Utilities Section
- Line 171:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Async Utilities Section
- Line 191:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Math Utilities Section
- Line 209:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - String Utilities Section
- Line 225:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Date Utilities Section
- Line 243:
   **Focus**:
   Advanced functionality and specialized utilities - 150+ functions
- Line 249:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Network Utilities Section
- Line 265:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Crypto Utilities Section
- Line 281:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Color Utilities Section
- Line 295:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Binary Data Section
- Line 310:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Parser Utilities Section
- Line 326:
   **Reference**:
   [`TODO.missing-implementations.md`](TODO.missing-implementations.md) - Geometry Utilities Section
- Line 346:
   **Status**:
   Unification phase - critical for launch readiness

### package/module/es/TODO.api-refactors.md

- Line 15:
   **Status**:
   Critical Priority - Affects ALL functions
- Line 141:
   **Status**:
   Critical Priority - Affects many function signatures
- Line 254:
   **Status**:
   Critical Priority - Quality assurance for ALL exports
- Line 281:
   - [ ] **Value constants** - Test literal type inference and immutability
- Line 288:
   - [ ] **[`array.type.fixedLength.ts`](src/array.type.fixedLength.ts:1)** - Has good type testing ✓
- Line 289:
   - [ ] **[`array.type.mapTo.ts`](src/array.type.mapTo.ts:16)** - Create comprehensive type tests
- Line 290:
   - [ ] **[`array.type.tuple.ts`](src/array.type.tuple.ts:17)** - Create type tests
- Line 291:
   - [ ] **[`array.type.withoutFirst.ts`](src/array.type.withoutFirst.ts:16)** - Create type tests
- Line 292:
   - [ ] **[`iterable.type.maybe.ts`](src/iterable.type.maybe.ts:1)** - Create type tests
- Line 293:
   - [ ] **[`promise.type.ts`](src/promise.type.ts:1)** - Create type tests
- Line 299:
   - [ ] **Configuration object types** - Test all option object types
- Line 491:
   - [**Missing Implementations Todo**](TODO.missing-implementations.md) - All new functions must follow these patterns
- Line 492:
   - [**Testing Todo**](TODO.testing.md) - Testing updates required for new signatures
- Line 493:
   - [**TSDoc Todo**](TODO.tsdoc-improvements.md) - Documentation updates for new API patterns
- Line 494:
   - [**Main Build System**](../../TODO.build-system.md) - Build system implications of major refactors

### package/module/es/TODO.code-review.md

- Line 13:
   - `t object/t logger/f/t never/r s/p p/index.ts:100` - Uses `.catch()` callback pattern which is banned by project
   - rules ("No `.then()`,
   `.catch()`,
   `.finally()`").
- Line 30:
   - `t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/customParsers.ts` (331 lines) - has a comment about
   - mutual recursion but not an explicit justification for exceeding th
- Line 31:
   - `path/index.ts` (241 lines) - no justification comment
- Line 32:
   - `t object/t jsonc/f/t string hasQuotedSyntax jsonc/r s/p n/fastPath.ts` (239 lines) - no justification comment
- Line 50:
   - `t never/f/t never/onLoadRedirectingTo/r s/p p/index.ts` - Silently does nothing if no anchor element is found.
   Per project rules:
   "Never silently discard unexpected states.
  "
- Line 54:
   - `t object/t promise/f/t number/wait/r a/p p/index.ts:43` - TSDoc `@example` block uses `.then()` pattern which
   - contradicts project rules.
- Line 66:
   - `t object/t logger/t sink/t file/p p/index.ts` - Writes log files to `node_modules/.monochromatic/` which is
   - unconventional and will be wiped on `npm install`.
- Line 69:
   resolved in `package/module/throws`;
   the old module-es expression-position helper TODO was removed.
- Line 73:
   - `t object/t logger/t sink/t sessionStorage/r s/p p/index.ts:43` - `lineCounter++` uses postfix increment which is
   - a mutation without justification.
- Line 76:
   - `customParsers.ts:101` - `['-', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9'].some(...)` could be simplified.
- Line 79:
   - `path/index.ts` - The `dirnameFallback` function uses a `for` loop with `let charIndex` and mutable `lastSlash`.

### package/module/es/TODO.development-workflow.md (159 occurrences, first 20 shown)

- Line 9:
   **Status**:
   High Priority - Systematic development process
- Line 13:
   - [ ] **Function specification** - Define function signature,
   behavior,
   and type safety requirements
- Line 14:
   - [ ] **Algorithm research** - Research optimal algorithms and implementations
- Line 15:
   - [ ] **Type design** - Design comprehensive TypeScript types and constraints
- Line 16:
   - [ ] **Implementation** - Implement with logger parameter and named parameter patterns
- Line 17:
   - [ ] **Runtime testing** - Create comprehensive unit tests with edge cases
- Line 18:
   - [ ] **Type testing** - Create type tests for all type behavior
- Line 19:
   - [ ] **Performance testing** - Benchmark performance and optimize if needed
- Line 20:
   - [ ] **Security review** - Security audit for functions processing user input
- Line 21:
   - [ ] **Documentation** - Create comprehensive TSDoc with examples
- Line 22:
   - [ ] **Integration testing** - Test function composition with other utilities
- Line 23:
   - [ ] **Cross-platform validation** - Test on Node.
  js,
   browsers,
   and other runtimes
- Line 27:
   - [ ] **Functional correctness** - Verify function behavior matches specification
- Line 28:
   - [ ] **Type safety validation** - Ensure excellent TypeScript integration
- Line 29:
   - [ ] **Performance validation** - Verify performance meets standards
- Line 30:
   - [ ] **Security validation** - Security review for all user-facing functions
- Line 31:
   - [ ] **Documentation review** - Ensure comprehensive documentation with examples
- Line 32:
   - [ ] **API consistency review** - Ensure consistency with existing function patterns
- Line 33:
   - [ ] **Test coverage validation** - Verify comprehensive test coverage
- Line 34:
   - [ ] **Integration validation** - Verify function works well with others
- ... and 139 more

### package/module/es/TODO.ecosystem-integration.md (154 occurrences, first 20 shown)

- Line 9:
   **Status**:
   Normal Priority - Framework ecosystem support
- Line 13:
   - [ ] **React hooks utilities** - Custom hooks using functional utilities
- Line 14:
   - [ ] **React component helpers** - Utilities for React component patterns
- Line 15:
   - [ ] **State management integration** - Integration with Redux,
   Zustand,
   etc.
- Line 16:
   - [ ] **React performance optimization** - Utilities optimized for React rendering
- Line 17:
   - [ ] **React TypeScript patterns** - Enhanced TypeScript patterns for React
- Line 21:
   - [ ] **Vue composition utilities** - Utilities for Vue 3 Composition API
- Line 22:
   - [ ] **Vue reactive integration** - Integration with Vue's reactivity system
- Line 23:
   - [ ] **Vue component helpers** - Utilities for Vue component patterns
- Line 24:
   - [ ] **Pinia integration** - State management integration
- Line 28:
   - [ ] **Angular service utilities** - Utilities for Angular dependency injection
- Line 29:
   - [ ] **RxJS interoperability** - Convert between our async iterables and RxJS
- Line 30:
   - [ ] **Angular component helpers** - Utilities for Angular component patterns
- Line 34:
   - [ ] **Express utilities** - Middleware and route handling helpers
- Line 35:
   - [ ] **Fastify utilities** - Plugin and route utilities for Fastify
- Line 36:
   - [ ] **NestJS integration** - Decorators and service utilities
- Line 40:
   **Status**:
   High Priority - Universal JavaScript runtime support
- Line 44:
   - [ ] **Deno compatibility** - Ensure all utilities work in Deno environment
- Line 45:
   - [ ] **Bun optimization** - Optimize for Bun's performance characteristics
- Line 46:
   - [ ] **Node.
  js LTS support** - Support all current Node.
  js LTS versions
- ... and 134 more

### package/module/es/TODO.exports-fixes.md

- Line 7:
   **Status**:
   Critical - Blocking TypeScript compilation
- Line 11:
   - [ ] **`isEmptyArray`** - Referenced in:
- Line 15:
   - [ ] **`isAsyncGenerator`** - Referenced in:
- Line 18:
   - [ ] **`isGenerator`** - Referenced in:
- Line 21:
   - [ ] **`isMap`** - Referenced in:
- Line 24:
   - [ ] **`isArray`** - Referenced in:
- Line 27:
   - [ ] **`isArrayEmpty`** - Referenced in:
- Line 30:
   - [ ] **`isArrayOfLength1`** - Referenced in:
- Line 33:
   - [ ] **`arrayIsNonEmpty`** - Referenced in:
- Line 38:
   **Status**:
   Critical - Type naming inconsistency
- Line 40:
   - [ ] **`isPositiveInt` vs `PositiveInt`** - Referenced in:
- Line 46:
   **Status**:
   Critical - Undefined functions
- Line 48:
   - [ ] **`getRandomId`** - Referenced in:
- Line 55:
   **Status**:
   High Priority - Module resolution
- Line 97:
   - [**Build System Todo**](../../TODO.build-system.md#missing-export-issues) - Related TypeScript compilation fixes
- Line 98:
   - [**Code Quality Todo**](../../TODO.code-quality.md#typescript-compilation-errors) - TypeScript error resolution

### package/module/es/TODO.governance-strategy.md (175 occurrences, first 20 shown)

- Line 9:
   **Status**:
   High Priority - Sustainability for 500+ function library
- Line 13:
   - [ ] **Function lifecycle management** - Process for adding,
   maintaining,
   and retiring functions
- Line 14:
   - [ ] **Quality gate enforcement** - Automated quality checks for all 500+ functions
- Line 15:
   - [ ] **Performance monitoring** - Continuous performance monitoring across all utilities
- Line 16:
   - [ ] **Security auditing schedule** - Regular security audits for comprehensive utility coverage
- Line 17:
   - [ ] **Dependency management** - Strategy for managing dependencies at scale
- Line 18:
   - [ ] **Technical debt tracking** - Monitor and manage technical debt across large codebase
- Line 22:
   - [ ] **Review process for new functions** - Comprehensive review process for 350+ new functions
- Line 23:
   - [ ] **Quality metrics tracking** - Track quality metrics across all function categories
- Line 24:
   - [ ] **Automated quality validation** - Automated checks for code quality,
   performance,
   security
- Line 25:
   - [ ] **Expertise assignment** - Assign domain experts to specific function categories
- Line 26:
   - [ ] **Cross-category consistency** - Ensure consistency across 25+ function categories
- Line 30:
   **Status**:
   Normal Priority - Community building for comprehensive library
- Line 34:
   - [ ] **Contributor onboarding** - Process for onboarding contributors to large library
- Line 35:
   - [ ] **Function request process** - How users request new utility functions
- Line 36:
   - [ ] **Implementation assignment** - Assign function implementations to contributors
- Line 37:
   - [ ] **Domain expertise development** - Develop expertise in specialized domains
- Line 38:
   - [ ] **Contribution recognition** - Recognition system for significant contributions
- Line 42:
   - [ ] **Coding standards enforcement** - Ensure all contributions meet high standards
- Line 43:
   - [ ] **Documentation standards** - Comprehensive documentation requirements
- ... and 155 more

### package/module/es/TODO.improvements.md

- Line 9:
   - [ ] **[`boolean.equal.ts`](src/boolean.equal.ts:174)** - [`equal()`](src/boolean.equal.ts:174) function
- Line 15:
   - [ ] **[`any.toExport.ts`](src/any.toExport.ts:45)** - [`toExport()`](src/any.toExport.ts:45) function
- Line 20:
   - [ ] **[`string.limitedGetComputedCss.ts`](src/string.limitedGetComputedCss.ts:533)** - CSS parsing functions
- Line 27:
   - [ ] **[`any.hasCycle.ts`](src/any.hasCycle.ts:56)** - [`hasCycle()`](src/any.hasCycle.ts:56)
- Line 32:
   - [ ] **[`function.memoize.ts`](src/function.memoize.ts:47)** - Memoization functions
- Line 41:
   - [ ] **[`error.throw.ts`](src/error.throw.ts:34)** - All `not*OrThrow()` functions
- Line 46:
   - [ ] **[`array.range.ts`](src/array.range.ts:73)** - [`arrayRange()`](src/array.range.ts:73)
- Line 51:
   - [ ] **[`numeric.type.int.ts`](src/numeric.type.int.ts:312)** - Type validation functions
- Line 58:
   - [ ] **[`fs.emptyPath.ts`](src/fs.emptyPath.ts:20)** - File system functions
- Line 67:
   - [ ] **[`array.is.ts`](src/array.is.ts:117)** - [`isArrayNonEmpty()`](src/array.is.ts:117)
- Line 72:
   - [ ] **[`iterable.is.ts`](src/iterable.is.ts:31)** - Type guard functions
- Line 79:
   - [ ] **Function composition utilities** - [`function.pipe.ts`](src/function.pipe.ts:280)
- Line 89:
   - `isArrayEmpty()` vs `isEmptyArray()` - choose consistent pattern
- Line 90:
   - `someIterable()` vs `iterableSome()` - standardize naming convention
- Line 110:
   - [`function.pipe.ts`](src/function.pipe.ts:19) - 1500+ lines,
   consider splitting by arity
- Line 112:
   - [`string.limitedGetComputedCss.ts`](src/string.limitedGetComputedCss.ts:533) - CSS parsing could be separate module
- Line 130:
   - [ ] **[`iterables.intersection.ts`](src/iterables.intersection.ts:118)** - Intersection algorithm
- Line 135:
   - [ ] **[`any.hasCycle.ts`](src/any.hasCycle.ts:56)** - Cycle detection
- Line 141:
   - [ ] **[`numeric.add.ts`](src/numeric.add.ts:34)** - Numeric addition functions
- Line 150:
   - [ ] **[`any.toExport.ts`](src/any.toExport.ts:45)** - Code generation security
- Line 155:
   - [ ] **[`dom.setCssFromParam.ts`](src/dom.setCssFromParam.ts:30)** - CSS injection
- Line 162:
   - [ ] **File system functions** - Path validation
- Line 171:
   - [ ] **[`dom.prompt.ts`](src/dom.prompt.ts:22)** - Dialog implementation
- Line 176:
   - [ ] **[`string.hash.ts`](src/string.hash.ts:15)** - Crypto API usage
- Line 202:
   1.
   **Boolean equality optimization** - Major performance impact
- Line 203:
   2.
   **Function naming consistency** - Breaking changes need early implementation
- Line 204:
   3.
   **Security fixes** - Input validation and sanitization
- Line 226:
   - [**Performance Todo**](../../TODO.performance.md#javascript-performance) - Performance optimization strategies
- Line 227:
   - [**Security Todo**](../../TODO.security.md#secure-coding-practices) - Security considerations for improvements
- Line 228:
   - [**Code Quality Todo**](../../TODO.code-quality.md#typescript-standards-and-patterns) - Code quality standards

### package/module/es/TODO.md (53 occurrences, first 20 shown)

- Line 15:
   - [**Export Fixes**](TODO.exports-fixes.md) - Critical TypeScript compilation errors
- Line 16:
   - [**API Refactors**](TODO.api-refactors.md) - Major breaking changes (logger params,
   named params,
   type testing)
- Line 17:
   - [**Missing Implementations**](TODO.missing-implementations.md) - Comprehensive utility function roadmap
- Line 18:
   - [**Testing Coverage**](TODO.testing.md) - Test files and coverage improvements
- Line 19:
   - [**TSDoc Documentation**](TODO.tsdoc-improvements.md) - Documentation and example improvements
- Line 20:
   - [**Function Improvements**](TODO.improvements.md) - Performance,
   security,
   and API improvements
- Line 21:
   - [**Package Infrastructure**](TODO.package-infrastructure.md) - Publishing,
   distribution,
   CLI tools,
   logging
- Line 22:
   - [**Ecosystem Integration**](TODO.ecosystem-integration.md) - Framework integration,
   migration,
   compatibility
- Line 23:
   - [**Governance Strategy**](TODO.governance-strategy.md) - Long-term strategy and risk management
- Line 24:
   - [**Development Workflow**](TODO.development-workflow.md) - Development process and team coordination
- Line 512:
   - [ ] **Industry-leading type safety** - Best-in-class TypeScript experience
- Line 513:
   - [ ] **Performance excellence** - Optimized algorithms throughout
- Line 514:
   - [ ] **Developer experience leadership** - Intuitive APIs and comprehensive docs
- Line 515:
   - [ ] **Functional programming completeness** - Every pattern and utility available
- Line 516:
   - [ ] **Security by default** - All user input properly validated and sanitized
- Line 517:
   - [ ] **Accessibility excellence** - All utilities support accessibility needs
- Line 521:
   - [ ] **Eliminate external utility dependencies** - One library for all needs
- Line 522:
   - [ ] **Establish TypeScript FP standards** - Define best practices for ecosystem
- Line 523:
   - [ ] **Enable advanced development patterns** - Support cutting-edge development
- Line 524:
   - [ ] **Performance benchmark leadership** - Fastest and most efficient implementations
- ... and 33 more

### package/module/es/TODO.missing-implementations.md (404 occurrences, first 20 shown)

- Line 13:
   **Status**:
   Critical Priority - Fundamental data manipulation
- Line 17:
   - [ ] **`object.pick(keys, obj)`** - Select specific properties with type safety
- Line 18:
   - [ ] **`object.omit(keys, obj)`** - Remove specific properties with type safety
- Line 19:
   - [ ] **`object.merge(obj1, obj2, ...objs)`** - Deep merge objects with conflict resolution
- Line 20:
   - [ ] **`object.assign(target, ...sources)`** - Shallow merge with proper typing
- Line 21:
   - [ ] **`object.clone(obj)`** - Deep clone objects immutably
- Line 22:
   - [ ] **`object.freeze(obj)`** - Deep freeze objects recursively
- Line 23:
   - [ ] **`object.seal(obj)`** - Deep seal objects recursively
- Line 27:
   - [ ] **`object.map(fn, obj)`** - Transform object values with mapping function
- Line 28:
   - [ ] **`object.mapKeys(fn, obj)`** - Transform object keys with mapping function
- Line 29:
   - [ ] **`object.filter(predicate, obj)`** - Filter object properties by predicate
- Line 30:
   - [ ] **`object.filterKeys(predicate, obj)`** - Filter object properties by key predicate
- Line 31:
   - [ ] **`object.reduce(fn, initial, obj)`** - Reduce object to single value
- Line 32:
   - [ ] **`object.transform(transformer, obj)`** - Transform keys and values simultaneously
- Line 36:
   - [ ] **`object.flatten(obj, separator?)`** - Flatten nested objects to dot notation
- Line 37:
   - [ ] **`object.unflatten(obj, separator?)`** - Convert dot notation back to nested
- Line 38:
   - [ ] **`object.invert(obj)`** - Swap keys and values with proper typing
- Line 39:
   - [ ] **`object.groupBy(fn, obj)`** - Group object properties by grouping function
- Line 40:
   - [ ] **`object.partition(predicate, obj)`** - Split object into two based on predicate
- Line 44:
   - [ ] **`object.isEmpty(obj)`** - Type-safe empty object checking
- ... and 384 more

### package/module/es/TODO.package-infrastructure.md (108 occurrences, first 20 shown)

- Line 9:
   **Status**:
   Normal Priority - Modern JavaScript registry
- Line 11:
   - [ ] **JSR configuration optimization** - Review [`jsr.json`](jsr.json:1) configuration
- Line 12:
   - [ ] **JSR-specific exports** - Ensure all exports work correctly on JSR
- Line 13:
   - [ ] **JSR documentation integration** - Leverage JSR's documentation features
- Line 14:
   - [ ] **JSR performance optimization** - Optimize for JSR's bundling and caching
- Line 15:
   - [ ] **JSR compatibility testing** - Validate all functions work on JSR platform
- Line 19:
   **Status**:
   Normal Priority - Package distribution
- Line 21:
   - [ ] **Export map optimization** - Review [`package.json` exports](package.json:6) for completeness
- Line 22:
   - [ ] **Platform-specific builds** - Optimize Node.
  js vs browser build differentiation
- Line 23:
   - [ ] **Tree-shaking optimization** - Ensure optimal bundle size for consumers
- Line 24:
   - [ ] **Package metadata enhancement** - Keywords,
   description,
   repository configuration
- Line 25:
   - [ ] **Publishing automation** - Automated semantic versioning and publishing
- Line 29:
   **Status**:
   High Priority - Platform compatibility
- Line 31:
   - [ ] **Node.
  js build optimization** - Optimize Node.
  js-specific implementations
- Line 32:
   - [ ] **Browser build optimization** - Optimize browser compatibility and bundle size
- Line 33:
   - [ ] **Build artifact validation** - Ensure all platform builds work correctly
- Line 34:
   - [ ] **Platform detection** - Runtime platform detection utilities
- Line 35:
   - [ ] **Polyfill strategy** - Handle platform-specific API differences
- Line 41:
   **Status**:
   Normal Priority - Development tooling
- Line 57:
   - [ ] **Enhanced error handling** - Better error messages and recovery
- ... and 88 more

### package/module/es/TODO.testing.md (128 occurrences, first 20 shown)

- Line 18:
   **Status**:
   Critical Priority - ALL exports need type testing
- Line 24:
   - **150+ functions** - Parameter and return type validation
- Line 25:
   - **50+ constants** - Type constant and value constant validation
- Line 26:
   - **25+ type utilities** - Type-level computation validation
- Line 27:
   - **All interfaces** - Object shape and property validation
- Line 31:
   **Any Utilities** - Type tests needed:
- Line 42:
   **Array Utilities** - Type tests needed:
- Line 50:
   **Boolean Utilities** - Type tests needed:
- Line 57:
   **Error Utilities** - Type tests needed:
- Line 64:
   **Function Utilities** - Type tests needed:
- Line 75:
   **Iterable Utilities** - Type tests needed:
- Line 82:
   **String Utilities** - Type tests needed:
- Line 88:
   **Numeric Utilities** - Type tests needed:
- Line 98:
   - [ ] **Numeric types** - [`Int`](src/numeric.type.int.ts:1),
   [`PositiveInt`](src/numeric.type.int.ts:1),
   [`NegativeInt`](src/numeric.type.int.ts:1),
   etc.
- Line 99:
   - [ ] **Array types** - [`Tuple`](src/array.type.tuple.ts:17),
   [`ArrayFixedLength`](src/array.type.fixedLength.ts:1),
   etc.
- Line 100:
   - [ ] **String types** - [`DigitString`](src/string.digits.ts:1),
   [`LangString`](src/string.language.ts:1),
   etc.
- Line 104:
   - [ ] **Function constants** - [`alwaysTrue()`](src/function.always.ts:1),
   [`emptyFunction()`](src/function.is.ts:101)
- Line 105:
   - [ ] **Utility constants** - Any exported constant values
- Line 183:
   - [ ] **`any.type.test.ts`** - Type tests for all any utilities
- Line 184:
   - [ ] **`array.type.test.ts`** - Type tests for all array functions and types
- ... and 108 more

### package/module/es/TODO.tsdoc-improvements.md (53 occurrences, first 20 shown)

- Line 9:
   - [ ] **[`any.echo.ts`](src/any.echo.ts:26)** - [`echo()`](src/any.echo.ts:26)
- Line 14:
   - [ ] **[`any.hasCycle.ts`](src/any.hasCycle.ts:56)** - [`hasCycle()`](src/any.hasCycle.ts:56)
- Line 19:
   - [ ] **[`function.always.ts`](src/function.always.ts:1)** - [`alwaysTrue()`](src/function.always.ts:1)
- Line 24:
   - [ ] **[`object.is.ts`](src/object.is.ts:30)** - [`isObject()`](src/object.is.ts:30)
- Line 29:
   - [ ] **[`map.is.ts`](src/map.is.ts:28)** - [`isMap()`](src/map.is.ts:28) and [`isWeakMap()`](src/map.is.ts:56)
- Line 36:
   - [ ] **[`array.is.ts`](src/array.is.ts:51)** - Multiple functions need enhancement:
- Line 42:
   - [ ] **[`array.range.ts`](src/array.range.ts:73)** - Functions need better docs:
- Line 48:
   - [ ] **[`string.hash.ts`](src/string.hash.ts:15)** - [`hashString()`](src/string.hash.ts:15)
- Line 53:
   - [ ] **[`string.trim.ts`](src/string.trim.ts:20)** - Multiple functions:
- Line 57:
   - [ ] **[`string.capitalize.ts`](src/string.capitalize.ts:87)** - [`capitalizeString()`](src/string.capitalize.ts:87)
- Line 63:
   - [ ] **[`function.deConcurrency.ts`](src/function.deConcurrency.ts:1)** -
   - [`deConcurrency()`](src/function.deConcurrency.ts:1)
- Line 68:
   - [ ] **[`function.ignoreExtraArgs.ts`](src/function.ignoreExtraArgs.ts:40)** -
   - [`ignoreExtraArgs()`](src/function.ignoreExtraArgs.ts:40)
- Line 74:
   - [ ] **[`fs.emptyPath.ts`](src/fs.emptyPath.ts:20)** - Multiple functions need docs:
- Line 80:
   - [ ] **[`fs.pathJoin.shared.ts`](src/fs.pathJoin.shared.ts:1)** - Utility functions:
- Line 88:
   - [ ] **[`boolean.equal.ts`](src/boolean.equal.ts:174)** - [`equal()`](src/boolean.equal.ts:174) and
   - [`equalAsync()`](src/boolean.equal.ts:543)
- Line 94:
   - [ ] **[`any.toExport.ts`](src/any.toExport.ts:45)** - [`toExport()`](src/any.toExport.ts:45)
- Line 99:
   - [ ] **[`string.limitedGetComputedCss.ts`](src/string.limitedGetComputedCss.ts:533)** - Multiple functions:
- Line 106:
   - [ ] **[`any.observable.ts`](src/any.observable.ts:3)** - Observable functions:
- Line 110:
   - [ ] **[`any.when.ts`](src/any.when.ts:24)** - Conditional functions:
- Line 116:
   - [ ] **[`error.throw.ts`](src/error.throw.ts:34)** - Multiple functions need enhancement:
- ... and 33 more

### package/module/es/TODO.typeguard-refactor.md

- Line 256:
   - [ ] `string.is.ts` - `isString`
- Line 257:
   - [ ] `numeric.bigint.ts` - `isBigint`,
   `isNumeric`
- Line 258:
   - [ ] `numeric.int.ts` - `isPositiveInt`,
   `isNegativeInt`,
   `isNonNegativeInt`
- Line 259:
   - [ ] `numeric.date.ts` - `isObjectDate`
- Line 260:
   - [ ] `object.is.ts` - `isObject`
- Line 261:
   - [ ] `array.empty.ts` - `isEmptyArray`
- Line 262:
   - [ ] `error.is.ts` - `isError`
- Line 266:
   - [ ] `map.is.ts` - `isMap`
- Line 267:
   - [ ] `set.is.ts` - `isSet`,
   `isWeakSet`
- Line 268:
   - [ ] `promise.is.ts` - `isPromise`
- Line 269:
   - [ ] `iterable.is.ts` - `isIterable`
- Line 270:
   - [ ] `generator.is.ts` - `isGenerator`,
   `isAsyncGenerator`
- Line 274:
   - [ ] `string.digits.ts` - `isDigitString`,
   `isNo0DigitString`,
   `isDigitsString`
- Line 275:
   - [ ] `string.letters.ts` - Letter validation functions
- Line 276:
   - [ ] `string.numbers.ts` - All numeric string validators
- Line 280:
   - [ ] `jsonl.basic.ts` - `isJsonl`
- Line 281:
   - [ ] `function.is.ts` - `isAsyncFunction`,
   `isSyncFunction`
- Line 282:
   - [ ] `schema.basic.ts` - Refactor to new pattern

### package/module/es/src/types/README.md

- Line 73:
   - `type string/` - Functions returning `string` or `Promise<string>`
- Line 74:
   - `type boolean/` - Functions returning `boolean` (including type guards)
- Line 75:
   - `type object/` - Functions returning object types
- Line 76:
   - `type function/` - Functions returning function types
- Line 77:
   - `type number/` - Functions returning numeric types
- Line 81:
   - `type object/type array/` - Functions returning array objects
- Line 82:
   - `type object/type iterable/` - Functions returning iterable objects
- Line 83:
   - `type function/type generator/` - Functions returning generator functions
- Line 84:
   - `type object/type array/type param string/` - `string[]`
- Line 88:
   - `from/` - Indicates transformation from input type to return type
- Line 92:
   - `type iterable/` - Takes iterables as input
- Line 93:
   - `type array/` - Takes arrays as input
- Line 94:
   - `type unknown/` - Takes unknown values as input
- Line 95:
   - `type string/` - Takes strings as input
- Line 96:
   - `type number/` - Takes numbers as input
- Line 100:
   - `restriction sync/` - Synchronous operations only
- Line 101:
   - `restriction async/` - Asynchronous operations only
- Line 105:
   - `params positional/` - Uses positional parameters
- Line 106:
   - `params named/` - Uses named/object parameters
- Line 114:
   - `p p/` - Async version with positional parameters,
   handles both sync and async predicates/iterables
- Line 115:
   - `p n/` - Async version with named parameters,
   handles both sync and async predicates/iterables
- Line 119:
   - `r s/p p/` - Sync-only version with positional parameters,
   performance optimized for purely synchronous code
- Line 120:
   - `r s/p n/` - Sync-only version with named parameters,
   performance optimized for purely synchronous code
- Line 1248:
   - ✅ **Core structure** - Return-type-first organization established
- Line 1249:
   - ✅ **Type categories** - Major return types (string,
   boolean,
   object,
   function) structured
- Line 1250:
   - ✅ **Sub-type hierarchy** - Complex types (array,
   iterable,
   generator) organized
- Line 1251:
   - ✅ **Constraint system** - Sync/async restrictions and parameter styles implemented
- Line 1252:
   - ✅ **Concrete examples** - Real utilities documented with usage patterns
- Line 1253:
   - ✅ **Navigation system** - Search patterns and cross-references established
- Line 1254:
   - ✅ **Developer guidelines** - Comprehensive instructions for adding utilities
- Line 1255:
   - 🔄 **Migration ongoing** - Functions being moved from legacy [`src/type/`](../../../../../bak/20251014/type)
   - structure
- Line 1256:
   - ⏳ **Full coverage** - Complete migration of all 500+ utilities planned

### package/module/es/src/types/t string/t hasQuotedSyntax/README.md

- Line 18:
   - **`hasQuotedSyntax`** - A branded string type indicating the presence of quoted syntax
- Line 22:
   - **`singleQuote`** - Strings containing single quotes (`'`)
- Line 23:
   - **`doubleQuote`** - Strings containing double quotes (`"`)
- Line 24:
   - **`backtick`** - Strings containing backticks/template literals (`` ` ``)

### package/ssg/aquati.cat/src/content/en/magicbread.mdx

- Line 135:
   need to consider the market - the potential customers.

### package/ssg/aquati.cat/src/content/en/mdx.mdx

- Line 36:
   This HTML output is stolen from MDN,
   Adding Captions and Subtitles to Video

### package/ssg/aquati.cat/src/content/en/web-resources.mdx

- Line 10:
   [Web Typography - A handbook for designing beautiful and effective responsive typography](https://book.webtypography.
   net/)
- Line 36:
   [Every Layout - Relearn CSS layout by example](https://every-layout.dev/)
- Line 56:
   [adactio - Jeremy Keith](https://adactio.com)

### package/ssg/aquati.cat/src/content/en/wolf-icons.mdx

- Line 3:
   description:
   "You thought this was just an unassuming wolf - but wait!
   It's scary!
   What?
   It's not?
  "

### package/ssg/aquati.cat/src/content/zh/mdx.mdx

- Line 36:
   This HTML output is stolen from MDN,
   Adding Captions and Subtitles to Video

### package/webapp-productivity/rss/TODO.code-organization.md

- Line 30:
   - [ ] 2.1.1.3 Create `outline.types.ts` - OPML/Outline types
- Line 31:
   - [ ] 2.1.1.4 Create `api.types.ts` - API request/response types

### package/webapp-productivity/rss/TODO.configuration.md

- Line 137:
   - [ ] 3.4.6 `opmls.ts` - OPML path

### package/webapp-productivity/rss/TODO.index.md

- Line 45:
   1.
   3.1.1 **Testing Infrastructure** - Establish test framework and basic coverage
- Line 46:
   2.
   3.1.2 **Documentation** - Document existing code and APIs
- Line 47:
   3.
   3.1.3 **Code Organization** - Restructure for maintainability
- Line 51:
   1.
   3.2.1 **Performance Profiling** - Identify bottlenecks
- Line 52:
   2.
   3.2.2 **Caching Strategy** - Implement Caddy proxy caching
- Line 53:
   3.
   3.2.3 **Performance Optimizations** - Apply improvements
- Line 57:
   1.
   3.3.1 **Configuration Management** - Externalize settings (optional)
- Line 58:
   2.
   3.3.2 **Final Documentation** - Complete all documentation
- Line 59:
   3.
   3.3.3 **Test Coverage** - Achieve 80% coverage target
- Line 69:
   - 5.1 **Simplicity over features** - Keep solutions simple and maintainable
- Line 70:
   - 5.2 **Infrastructure-first** - Leverage Caddy for cross-cutting concerns
- Line 71:
   - 5.3 **Manual control** - Users explicitly manage their feeds
- Line 72:
   - 5.4 **No magic** - Transparent,
   predictable behavior
- Line 73:
   - 5.5 **Fork-friendly** - Simple enough for customization

### package/webapp-search/exa-search/README.md

- Line 49:
   - `index.html` - Main HTML structure (at package root)
- Line 50:
   - `src/index.ts` - TypeScript logic for API integration and UI updates
- Line 51:
   - `src/index.css` - Styling with dark/light mode support
