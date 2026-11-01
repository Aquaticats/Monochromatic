# AGENTS.md philosophy

## Purpose

`AGENTS.md` is for non-obvious,
 actionable guidance that neither an AI agent nor a human developer can infer from context or general knowledge alone.
It supplements,
 not replaces,
 common sense.

## Writing style

`AGENTS.md` is written caveman style:
 telegraphic,
 shortest expression for every rule.
It loads on every session,
 so each token there is paid again every time;
 brevity is a hard requirement,
 not a preference.
Drop articles ("the",
 "a",
 "an"),
 copulas,
 and connective filler wherever meaning survives;
 cut "in order to",
 "make sure to",
 "you should",
 and any clause that merely restates the rule.
Use the shortest accurate synonym (conf,
 src,
 docs,
 dir).
Fragments beat sentences.

This style binds `AGENTS.md` alone.
This philosophy doc and every other doc under `doc/` stay normal prose:
 reference loaded rarely,
 where clarity outranks token thrift.

Never compress away the load-bearing strings:
 tokens,
 paths,
 commands,
 identifiers;
 code blocks;
 section headings (external and internal cross-references resolve by name);
 hook-matched literals.
Caveman compresses prose,
 never the strings the rules act on.

## What does not belong

### Prose hook trigger inventories

Do not list the phrases caught by prose-quality hooks in `AGENTS.md`.
Naming exact triggers teaches agents a vocabulary game:
 avoid the literal string while keeping the same unsupported claim.
Keep `AGENTS.md` focused on the underlying discipline:
 measure facts,
 cite sources,
 and state unresolved uncertainty after investigation.
Put exact trigger strings in hook config and tests,
 where enforcement can change without becoming prompt material.

### Skill invocation pointers

The harness auto-loads every skill's description,
 which already states when to use it.
A section that only says "invoke skill X when doing Y" duplicates that description and earns nothing;
 drop it.
Keep only the residue a skill description does not carry:
 project-specific tokens,
 exemptions,
 paths,
 or a behavioral mandate the description omits.
The `css` section was removed whole for this reason;
 `choosing-technology` likewise;
 the `troubleshooting-doc` and `runbook` mentions were cut to the project-specific bits (exemptions,
 handover paths) their descriptions do not state.

### Repository identity and high-level structure

Do not describe what the project is,
 what its core features are,
 or sketch its architecture unless those facts carry constraints an agent would not otherwise know.
A capable agent reading the codebase will infer these faster and more accurately than a written description.

Removed:

- "Repository Information";
   obvious from package names and directory structure
- "Core Features";
   obvious from the packages directory
- "Architecture";
   was empty;
   real architecture constraints belong in specific technical sections

### Generic section titles

Avoid titles like "Important Reminders",
 "Notes",
 or "Miscellaneous".
Content with such titles belongs in a topically relevant section.
When no section fits,
 add one with a specific name.

### Runtime environment checks

Do not put detection logic or compatibility warnings in `AGENTS.md`
when a hook can enforce the same constraint automatically and silently.
`AGENTS.md` text is passive;
 an agent reads it once and may not apply it consistently.
A hook fires on every session start and injects a warning directly into context only
when the condition is actually violated.

### Removed: "Detecting the Current Shell"

The original section told agents how to detect whether the shell is bash-compatible (checking `$SHELL`,
 recognizing `pwsh`,
 etc.) and how to adjust syntax accordingly.
This was removed because:

1. AI agents already assume bash-compatible syntax by default and do not need to be told to.
2. The detection instructions were only useful if something was already wrong;
    a condition better caught by automation.
3. A `SessionStart` hook covers the failure case with no `AGENTS.md` noise for the common case.

### Hook setup

The replacement is a `check-shell.ts` SessionStart hook registered in your agent's global personal settings.
It runs on every session start,
 reads `SHELL`,
 and injects a warning into the session context when the shell is not in `{bash, zsh, sh, dash, ksh}`.
On a compatible shell it exits silently.

`check-shell.ts`:

```ts
/**
 * SessionStart hook: warns the agent if the current shell is not bash-compatible.
 * AI tooling assumes bash syntax; non-compatible shells cause silent command failures.
 */
const shell = process.env.SHELL ?? '';
const shellName = shell.split('/',).at(-1,) ?? '';
const bashCompatibleShells = new Set(['bash', 'zsh', 'sh', 'dash', 'ksh',],);

if (shellName && !bashCompatibleShells.has(shellName,)) {
  console.log(
    JSON.stringify({
      hookSpecificOutput: {
        hookEventName: 'SessionStart',
        additionalContext:
          `Shell warning: current shell is ${shellName} (SHELL=${shell}), which is not bash-compatible. Use bash-compatible syntax for all shell commands, or prefix commands with \`bash -c\`.`,
      },
    },),
  );
}
```

Global personal settings (relevant excerpt):

```json
"SessionStart": [
  {
    "hooks": [
      {
        "type": "command",
        "command": "node <global-hooks-dir>/check-shell.ts"
      }
    ]
  }
]
```

### Tool usage examples and rationale

Do not include syntax examples for standard tools (e.g. `rg`,
 `gh`,
 `curl`) or explain why a well-known tool is useful.
Any competent agent or developer already knows how to use `ripgrep` or when to reach for official documentation
over source code.
Explaining these things wastes tokens on every context load and signals distrust of the reader.

Removed:

- `rg` examples (`-t ts`,
   `--type ts`,
   `-A 5 -B 5`):
   standard ripgrep flags,
   universally known
- "Reach for this first when working with a third-party library:
   the official docs are usually faster...";
   states the obvious

What belongs instead:
 the name of the tool,
 what it covers in this project's context,
 and any non-obvious constraint (e.g. "raw source is still useful when docs are incomplete").

### Negative prompts

Avoid "Never assume X" when the positive instruction already makes the intent clear.
The positive rule carries the action;
 the negative restatement is redundant noise.

### Code examples for rules that are self-explanatory

Code examples belong in `AGENTS.md` only when the rule itself is ambiguous without one.
When a rule is clear from its text (e.g. "prefer `const` over `let`"),
 the example adds tokens without adding understanding.

Examples that illustrate **what to flag during review** belong in the code-review skill,
 not in `AGENTS.md`.
This way the examples load only when a review is happening,
 not on every session.

### Explanatory rationale for standard practices

Drop rationale like "for better tree-shaking" or "to improve build performance" when the practice is widely understood.
Keep rationale only when the reasoning is project-specific or counterintuitive.

### Detailed sub-rules for generic workflows

Compress multi-bullet expansions of a single rule into one line when the sub-bullets are obvious consequences.

Example:
 "Never modify files in cloned third-party repositories;
 use configuration,
 env vars,
 or wrapper scripts" replaces five bullets explaining why modification is bad and what the alternatives are.

### Inline pointers to this philosophy doc

Do not add per-rule "(see doc/philosophy/`agents.md`)" / "(rationale in doc/philosophy/`agents.md`)" pointers to
`AGENTS.md`.
Normal operation needs the rule,
 its cue,
 and the tokens to act,
 not the rationale;
 an inline pointer implies the agent must read this doc to operate,
 which it must not.
Rationale,
 mechanism,
 and examples relocated here stay discoverable through the single global note at the top of `AGENTS.md` plus the
 section-name correspondence
 (each subsection under "Relocated rule rationale" is named for the `AGENTS.md` section it came from).
If a rule is genuinely crippled without relocated detail,
 move the detail back into `AGENTS.md` rather than pointing here.
A specific reference (exact section heading) is acceptable only when unavoidable;
 a vague "see PHILOSOPHY" never is.

### Implementation details behind a `mise run` task

How a task is wired internally is not an agent-facing rule.
Agents run tests with `mise run test` (or a package's `:test`),
 so they never need to know that Rust packages run on `cargo nextest run` instead of `cargo test`,
 that the fuzz crate scopes its run to `--lib`,
 or that Rust unit tests live in co-located `<name>_tests.rs` sidecars declared from the source file
 with `#[cfg(test)] #[path = "..."] mod tests;`.
The runner choice lives in the mise tasks;
 the sidecar layout lives in the source files themselves.
An agent editing a Rust file sees the existing `mod tests;` stub at the bottom and the sibling `*_tests.rs` beside it,
 and follows the pattern by example,
 the same way it infers any local convention from the code in front of it.
Restating that in `AGENTS.md` spends tokens every session to describe what the code already shows in context,
 and it invites the rule to drift from the implementation it documents.
Keep `AGENTS.md` to rules that change what an agent decides at a fork it would otherwise get wrong;
 leave runner choice and test-file mechanics to the tasks and the code.
This is the test-tooling instance of the general rule above
("Code examples for rules that are self-explanatory" and "Detailed sub-rules for generic workflows"):
 a convention an agent will copy from its surroundings does not need a written rule.

### Removed: 2026-09 rule-by-rule optimization

Every rule was reviewed with the user and measured;
 the record,
 including each rule's approved text and the evidence behind every removal,
 is `doc/planning/agents-md-optimization.md`.
Rules left `AGENTS.md` for three reasons:

1. A lint rule already enforces them and its diagnostic names the fix,
    so the always-loaded copy only repeated the error message.
    Compact principles standing in for many lint rules (IMM,
    PP1,
    ST9) stayed,
    because one clause prevents many lint round-trips.
2. They duplicated another rule,
    so they merged into the more-cited code.
3. They governed one product,
    one package,
    or one kind of task,
    so they moved beside that material:
    the `visual-design-review` skill,
    the `choosing-technology` skill,
    or the owning package's docs.
    Moved rules keep their codes.

Retired codes are reserved in the local forbidden-strings appendix so they are never reused.

## Relocated rule rationale

The explanatory "why/how" for each rule below was moved here so `AGENTS.md` keeps only the terse enforceable rule,
 cue,
 and tokens.
 Headings match the `AGENTS.md` section each rule came from;
 rules since moved to a skill or package doc keep their codes there.

### Rule details

#### Architecture decisions: why AD3 names concrete pattern only

AD3 is a narrow architecture rule:
 prefer calling async functions directly over building descriptor objects that another
interpreter executes.
 It is not a shortcut license.
 The earlier "apply YAGNI to architecture" clause compressed that
intent into a slogan,
 and agents misread the slogan as permission to skip verification,
 tests,
 documentation,
 or design
work.
 `AGENTS.md` should name concrete forks agents face,
 not broad slogans that can override other rules in practice.

#### Essential commands: why `bun test` is banned for this harness

`bun test` specifically:
 never substitute it for a missing mise task.
 The custom `@monochromatic-dev/test-harness-core` harness runs tests as a side effect of import,
 so `bun test <file>` prints `PASS` log lines (from the harness)
 and then reports `0 pass / 0 fail` (bun's runner finds no `bun:test` registrations).
 The misleading summary suggests the run was broken when in fact every test passed.
 Use `mise run //package/<path>:test:unit`;
 if no such task exists,
 run the file directly with `node <file>`,
 matching the Node-based test task template.
 A `PreToolUse` hook (`ccgr`,
 source at `package/claude-code-plugin/source/src/handler/guardrail.ts`) blocks the call when configured.

#### Type system: why `const` narrowing does not reach function declarations

TypeScript does not propagate `const` narrowing into function declarations (both tsc and tsgo);
 the compiler only extends flow analysis across `FunctionExpression`,
 `ArrowFunction`,
 and method/accessor closures,
 because declarations are hoisted and could be called before the narrowing guard.
 Fix:
 use a helper that returns non-null (`function requireElement<T>(sel): T { ... throw ... }`),
 or reassign to a new `const` with an explicit type annotation after the null check.

#### TypeScript: export-for-testability API seams

Built-artifact tests import package output.
 Sometimes the right seam is a small helper factored from production code,
not only the top-level behavior.
 Exposing that helper is acceptable because the package boundary is also the testing
boundary.
 Do not block the choice merely because it expands public surface.
 Once exported,
 treat it like normal supported
API:
 document it,
 type it,
 name it clearly,
 and keep behavior stable unless intentionally changed.

#### Git cleanup and worktree safety reviews: the cli-git tool-cache allowlist

When the review touches `cli-git`'s linked-worktree guard,
 account for the baked-in tool-cache allowlist
 (`default_allowed_worktree_dirs` in `package/git-policy/cli/src/native/effective_target.rs`,
 currently uv's git cache):
 repositories whose git-dir resolves under an allowed dir bypass that guard,
 so destructive git is not actually blocked there.

#### Command execution conventions: the bash-output path substitution

`~` in Bash tool output is a display substitution for `/var/home/user`
or `/home/user` applied by the `bash-output-filter` hook,
 plus stripping of the current cwd prefix.
 It applies only at the start of a line,
 so paths inside JSON or error messages are unaffected.
 Filesystem values are unchanged;
 this is display-only.
 To skip the filter for one command,
 include any blocklist trigger:
 `eval 'your command here'` is simplest;
 others are `export`,
 `source`,
 `$(...)`,
 backticks,
 `> file` redirect.

#### Command execution conventions: shell cwd does not persist (WCD)

A shell command's working directory is not reliably stable across calls,
 and the instability is broader than the Bash tool alone:
 any shell-running tool can have its cwd reset between commands.
 It was first observed inside linked git worktrees,
 where the cwd silently resets to the primary checkout between commands,
 most visibly after a background task completes or a notification arrives,
 and even after an `EnterWorktree` reports the session moved.
 An unprefixed `mise`,
 `pnpm`,
 `git`,
 or `bun` then runs against the wrong tree,
 so a build or lint quietly targets the wrong directory and reports success for code that was never touched.
 `Read`,
 `Write`,
 and `Edit` take absolute paths and are immune,
 so file edits land correctly while shell commands drift,
 which makes the mismatch hard to notice.
 The preservation behavior turned out to be more unstable than the worktree-only framing suggested,
 so the rule is now unconditional:
 pin the target dir explicitly on every shell command,
 by prepending `cd <abs-path> &&` or by passing absolute paths,
 rather than trusting an inherited cwd.
 In a worktree the pinned dir is the worktree absolute path.
 This is the one place where the usual "avoid `cd`" and "never prepend `cd` to a git command" guidance is overridden,
 because the working directory is unreliable rather than known.
 The cue is being about to run any shell command without explicitly pinning its target directory.

#### Hazardous commands: the resource-exhaustion example set

The "may exhaust the host" set is broader than the destructive-command set:
 anything that allocates much memory,
 spawns many processes,
 opens many file descriptors,
 runs unbounded loops,
 or consumes resources without a tight upper bound.
 Examples:
 stress harnesses and load generators (`mise run //:forge:stress`,
 `mise run //:test` with thousands of cases,
 k6/wrk runs);
 builds that fan out across many packages without concurrency caps;
 benchmarks that allocate large blobs or fork many workers (`bun bench`,
 `mitata` runs);
 scenarios that loop over `git.packObjects` / `git.indexPack` or other heavy isomorphic-git ops;
 subprocess fan-outs with no `--writers=` / `--concurrency=` ceiling;
 anything that imports a server runtime that opens libSQL,
 warms caches,
 or schedules timers in a tight loop.

#### Simplification: max-lines split pattern examples

Remediate a max-lines violation by splitting:
 re-export from `index.ts`;
 move helpers to siblings (e.g. `crc32.ts`,
 `headers.ts`),
 constants to `constants.ts`,
 types to `types.ts`.
 Pattern examples:
 `package/hyperscript/core/src/index.ts` (76 lines,
 pure re-exports),
 `package/llm/image-diff/src/index.ts` (75 lines,
 pure re-exports).

#### Name the verification step: the inline-citation examples

Examples of a confident claim paired inline with what backs it:
 "the bug is in `ci.py:851` (read the source)";
 "the codebase has 158k TS LOC across 1,903 files (`tokei` output above)";
 "express 4.
x is supported (verified by reading the package's README at the cloned repo)".

#### Communication style: cite-the-right-source detail

The harness system prompt's rule sources include the Git Safety Protocol,
 tool-use guidelines,
 and format instructions.
 A rule can sound like it lives in any source,
 and a quick recall feels like enough;
 it is not.
 Failure shape:
 writing "`AGENTS.md` says never amend" when "never amend" lives in the harness Git Safety Protocol;
 the user asks "which line?
" and the grep returns nothing.

#### Communication style: AGENTS.md growth discipline

Never substitute "I'll keep it in mind" or any promise to a future self:
 sessions have no memory;
 rules persist only in `AGENTS.md`,
 a skill,
 or a hook.
 The mechanism is monotonic by default (every unmet expectation adds rules),
 leading to unbounded growth.
 Counteract:
 `AGENTS.md` should grow only when no existing rule covers the failure mode.

The rejected `/grill-with-docs` follow-through rule is an `AGENTS.md` growth example:
 it is too narrow for every session,
 and the cost of not adding it is at most one follow-up prompt from the user.
 Record that rejection rationale here rather than loading a bespoke rule into every agent context.

#### Proactivity calibration: the git-commit guardrail example

The git-commit guardrail typifies the harness "too proactive" defaults this project overrides:
 "It is VERY IMPORTANT to only commit when explicitly asked,
 otherwise the user will feel that you are being too proactive.
" The same calibration applies to every similar default.

#### Measure-vs-ask: measurement recipes

- Codebase size:
   `tokei` or `find . -name '*.ts' | xargs wc -l`
- Build time:
   `time mise run build`
- Test count:
   count test files or run with reporter
- Dependency count:
   `pnpm ls --depth=0` or count entries in `package.json`
- Fix complexity:
   read the src code path that would change
- User commit cadence:
   `git log --format='%aI' --since=<date> | cut -dT -f1 | uniq -c` for commits-per-day distribution
- Daily commit span (proxy for working hours):
   per-day min and max hour from `git log --format='%aI'`
- Defect-recovery rate:
   count of `revert`/`regression`/`fix.*broken` commits over total commits in the window
- Concurrent-session evidence:
   compare `git log --since=<conversation-start>` timestamps against this conversation's UserPromptSubmit hook times

#### Name the verification step: search-result failure modes

Every search result carries two claims:
 (a) the search ran correctly,
 (b) the lines you're seeing are the matches.
 Both can fail silently.

Zero-match silent failures:
 invalid `--type` argument (e.g. `rg --type tsx`,
 where `tsx` is not a registered ripgrep type;
 the `ts` type already covers `*.tsx`);
 wrong path or glob excluding the intended files;
 `2>/dev/null` masking the actual error message;
 stale or empty target dir;
 stdin-reading mode triggered by missing path argument (see "Before running a command").

Non-zero-match silent failures (same shape,
 opposite direction):
 `head -N` truncating before later files have a chance,
 where one noisy file consumes the cap and buries everything alphabetically later (remove or raise the cap,
 or surface that the result is truncated);
 denylist filters (`rg -v 'a|b|c'`) hide whatever you forgot to keep
 and discard whatever you forgot to include (prefer allowlist patterns:
 a positive shape like literal `' cat '` with surrounding spaces for prose-form English usage of a word
 that is also a shell command);
 `-l` (filenames only) hides the context needed to tell real matches from noise (default to full lines,
 switch to `-l` only after confirming the noise cost is concrete);
 narrow `--type` on the first pass feels thorough but skips matches in unexpected file kinds (widen first).

#### Third-party libraries: investigation and replacement-audit detail

The source-clone investigation rule,
 cloned-repo modification boundary,
 troubleshooting write-up gate,
 and
`.out-of-scope/` upstream-filing check now live in the `troubleshooting-doc` skill.
 Replacement-audit depth now
lives in the `choosing-technology` skill.
 They fire only during external-tool diagnosis or technology selection,
not every session,
 so `AGENTS.md` retains only immediate API and CLI lookup rules.

#### Third-party libraries: the `.out-of-scope/` upstream-tracking exemptions

`.out-of-scope/` lists external-tool bug classes that still get a `doc/troubleshooting/<topic>.md` writeup but no
upstream GitHub issue,
 because filing one is wasted effort here.
 The `troubleshooting-doc` skill carries the active
check;
 examples here explain why entries exist.
 Examples include:

- Claude Code (`.out-of-scope/claude-code-upstream-bugs.md`):
   upstream very unresponsive,
   so tracking issues produce clutter without changing the outcome;
   encode the workaround as a rule in `AGENTS.md` instead.
- JSR (`.out-of-scope/jsr.md`):
   the workspace consumes no JSR-hosted packages (`doc/philosophy/tool-choices.md` covers tool selection),
   so install-path bugs there do not affect it.
- `bun install` (`.out-of-scope/bun-install.md`):
   the workspace uses pnpm as the package manager,
   not `bun install`,
   so those install-path bugs are never hit.

#### Choosing technology and vendors: the encoded checklist

The `choosing-technology` skill encodes context-fork questions,
 the six vendor vetting layers (layoffs,
 reviews,
outages,
 funding,
 signup friction,
 security),
 the open-source default,
 constraint-fit before stack-fit,
 replacement
parity audit,
 the alternative survey rule (name at least two with rejection reasons),
 and the
`doc/decision/<project>.md` maintenance rule.
 The available-skill description supplies the trigger;
 the enumerated
detail lives in the skill.

#### Before claiming inability: how each manual-action bridge works

GUI clicks:
 `agent-browser` drives most web UIs;
 the repository's nested compositor isolates native Wayland UI tests;
 application HTTP/IPC surfaces bridge authorized live UI;
 a caller-independent broker owns global virtual input through key release.
The host uses native Wayland,
 so X11-only `xdotool` cannot bridge its application input.
 A direct `ydotool` caller sends key-down and key-up separately.
If injected input cancels that caller,
 the persistent virtual device retains the pressed key until `ydotoold` restarts.
See [`doc/troubleshooting/ydotool-interrupted-key-release.md`](../troubleshooting/ydotool-interrupted-key-release.md).
 "Click" usually has a keyboard shortcut to synthesise,
 or a backing HTTP/IPC endpoint that bypasses the UI.
 Interactive auth:
 scripted with `expect`,
 or skipped via API tokens.
 Hardware activation:
 almost always a CLI.
 `AGENTS.md` keeps the one-line token index (the tools to reach for);
 this is how each one substitutes for the manual action.

#### Verify on a throwaway: why a guard test needs a throwaway

When the behavior under test is whether a guard blocks a destructive operation,
 running that operation against real state means a broken guard (the exact failure you are testing for) damages real
 state,
 while a passing guard tells you nothing a throwaway would not have.
 So build both the allowed case and the rejected case as fixtures.

#### Measure-vs-ask: categorical-dismissal examples (QF1)

Categorical dismissals that feel like recall but are one search away:
 "the project doesn't use X",
 "X doesn't apply here",
 "X is already handled by Y".
 These overlap the hedge-phrase list;
 QF1 keeps the rule (measure assumed facts,
 cite inline) while the examples live here and in the hedge section.
 `AGENTS.md` and tsconfig count as confs where X may be wired up.

#### runbook skill: the encoded sections and rules

The `runbook` skill encodes the required sections (Setup,
 Steps,
 What to check,
 Restore),
 the bold-every-UI-element rule,
 the expected-outcome-per-step rule,
 and the exact-strings-not-paraphrases rule.
 `AGENTS.md` keeps the "invoke it for any manual-action document" rule and the canonical example path.

#### Match action scope to the request verb: the security-alerts example

"Decide which security alerts we can fix immediately" is triage;
 the deliverable is the categorized list.
 Applying the fixes is a separate decision the user has not yet made;
 surface a concrete proposal and wait for green-light.

#### Package completeness: the sync/async test example

A test file covering sync matchers but skipping async matchers is the same as no async tests;
 the bug ships silently.

#### Enforcement mechanisms: why hook existence is not in AGENTS.md

`AGENTS.md` deliberately does not expose the existence of Claude Code hooks,
 not merely their names:
 the stop hook (`ccsr`) enforcing hedge-phrase and trailing-question rules
 and refusing stops that state says another turn could still advance,
 the PreToolUse guardrail (`ccgr`),
 the roadmap PostToolUse type-check.
 Flagging the machinery at all lets agents game the surface pattern
 (swapping one hedge for a novel one that passes but carries the same epistemic gap)
 and reframes an internalized rule as an external gate to dodge.
 The rule the agent must own stays (the hedge-phrase list);
 the fact that a hook backstops it is omitted.

`ccsr`'s forced-continuation detector was the one exception to that reasoning while it read none of the response text:
 there was no surface pattern to swap,
 so knowing it existed could not help an agent rewrite its way past it,
 and satisfying it meant doing the work.
 Since 2026-10-18 it asks Luna what situation the final message describes
 (announces a next step, waits on background work, waits on the user, pauses, or is closed),
 so it now conditions on wording too,
 and the reasoning for not naming it applies again.
 Two properties keep it from being a pure wording gate:
 the state-based releases (depth limit, no tool call since the last block) still read no text,
 and a background wait is blocked only when no background task is running,
 which is state the agent cannot rewrite.
 Every other detector conditions on wording,
 which is exactly why naming them would let an agent rewrite around them.

 It is not unbeatable,
 and the way it can be beaten is worth naming rather than leaving implicit.
 The depth guard that bounds the chain also bounds the enforcement:
 an agent that produces nothing useful for enough consecutive blocks
 exhausts the limit and is then allowed to stop.
 So the detector cannot be defeated by rewording,
 only by outlasting it,
 which costs the agent every one of those turns
 and leaves the whole run in the transcript.
 That tradeoff is deliberate:
 an enforcement with no exit is an unbounded loop,
 and the bound has to sit somewhere.
 The rule it backstops (`PXQ`) stayed in `AGENTS.md` and was measured failing there for a month,
 which is the evidence that an internalized rule needs a gate when the model will not own it:
 `doc/troubleshooting/claude-code-opus-5-premature-turn-end.md`.
 The trailing-question rule is dropped from `AGENTS.md` entirely rather than reworded:
 "ask via the question tool,
 never end on a bare question" restates standard harness behavior the `AskUserQuestion` tool
 and its description already carry,
 so it fails the Purpose test for non-obvious project knowledge.
 Exception:
 the `bash-output-filter` `~` substitution stays named,
 because its observable effect (line-start paths rewritten in tool output) must be understood precisely to debug paths,
 and it carries no gaming risk.
 Hook identities and implementation details otherwise live only in this doc.
 This follows the same principle as the "Runtime environment checks" rule in "What does not belong":
 passive text in `AGENTS.md` is weaker than an active hook.

#### Before claiming inability: the document-pointer worked example

A ToS that says "Services are governed by separate subscription agreements,
 not these Website Terms" points at a separate document where the substantive provisions live;
 fetch it before drawing conclusions about its contents.

#### Linting: why the oxlint-disable placement matters

The disable goes before the TSDoc so the TSDoc remains the immediately preceding comment.
 The enable goes on the very next line after the declaration;
 leaving a disable open longer than necessary silences unrelated violations.
 `// oxlint-disable-next-line` applies only to the literal next physical line,
 so placed between TSDoc and declaration it lands on the TSDoc and the suppression is lost;
 use the block-level disable + enable pair wrapping TSDoc and declaration tightly instead.

#### Simplification and Regular expressions: why no recursion or accumulator over linear input

JS guarantees no tail-call elimination (V8 has none),
 so recursion whose depth tracks input length is a stack-overflow bug at scale;
 accumulator recursion (`acc + c`,
 `[...acc, x]`) is additionally O(n^2) because each step rebuilds the whole string or array.
A loop,
 or a regex being removed,
 must therefore become a single linear pass (`map`/`filter`/`reduce`,
 `for...of`,
 or one counter loop in O(n) time and O(1) extra stack),
 never recursion over a string or flat array.
Recursion is safe only for bounded structural walks (AST,
 tree,
 grid,
 filesystem) whose depth tracks the data's nesting.
The trap is the degenerate spine:
 a member chain (`a.b.c`),
 call chain,
 or left-associative operator chain (`a + b + c`) nests once per operand,
 so its depth equals operand count and tracks length,
 not nesting;
 before recursing over any tree,
 ask whether it can degenerate into a spine on large or adversarial input,
 and if so flatten iteratively with an explicit work-stack.
A removed regex carries an extra hazard:
 a backtracking pattern can be superlinear (the ReDoS this ban guards against),
 so the replacement must prove O(n) on its own merit for attacker-controlled or unbounded input,
 not inherit linearity from the regex.
Post-mortem of this exact misapplication:
 `doc/audit/chain-flatten-skewed-tree.md`.

#### Simplification: the `.rs` max-lines budget

The Rust `max-lines` rule (`monochromatic-rust-linter`) counts only code lines:
 blanks and comments are excluded via the real lexer,
 so the dum-dum-non-ts comment style does not consume the budget.

#### Variables and values: why the no-function-root-let escapes are sanctioned, not dodges

`let` inside `ForStatement.init` is exempt,
 so a counter `for (let i = 0; ...)` loop is the sanctioned tool for an indexed or lookahead scan,
 not a rule to dodge.
The helper-shape allowlist suppresses the report when a function ends in `return <local-binding>`,
 which is why extracting such a helper is a clean remediation rather than a workaround.

#### Variables and values: why magic-number policy lives in the lint config

`no-magic-numbers` ships with `detectObjects: false` (`package/config/oxlint/src/rule/style.ts`),
 so a numeric literal used directly as an object-literal property value is never flagged by the tool.
 The former `AGENTS.md` rule restating this check was deleted in the 2026-09 optimization:
 `denyWarnings` makes every `no-magic-numbers` finding fail,
 and the config is the one source of truth for its exemptions.

Failure that shows why the config,
 not a plain-language rule,
 must decide:
 a redesign of `package/wc/app/src/styles-colors.ts` extracted five inline `l` channel values inside
 `cssOklch({...})` calls (e.g. `l: 0.16,`) into standalone named constants (`L_BLACK`,
 `L_NEAR_BLACK`,
 `L_MID`,
 `L_NEAR_WHITE`,
 `L_WHITE`),
 each referenced exactly once,
 on the assumption the linter required it.
 `detectObjects: false` had already exempted that exact pattern for months (since commit `3515cd5cb`),
 so the extraction served no lint-compliance purpose and added five single-use constants of pure ceremony.

#### Security: why source escapes are not portable across a syntax boundary

A Markdown `\<`,
 a shell quote,
 JSON escaping,
 URL encoding,
 or regex escaping makes text safe only in its own grammar;
 carried into another language it is either inert or actively wrong.
The destination grammar is the only authority,
 so normalize source semantics as needed
 and encode for the exact destination subcontext at the final interpolation boundary.
The rule reaches every text-emitting transformer:
 serializers,
 code generators,
 formatters,
 autofixes,
 docs generators,
 renderers,
 CLIs,
 and tests.
Happy-path formatting and idempotence tests are not enough;
 the adversarial boundary cases are where the boundary actually breaks.

#### Third-party libraries: the unforked-upstream-clone exception mechanics

The `troubleshooting-doc` skill's exception is a disposable prototype clone created solely to test or prepare an
upstream-fix patch diff,
 made only after origin verification and run without exposing credentials or this repo to
third-party scripts.
 That narrowness (disposable,
 origin-verified,
 credential-free) is what keeps it from eroding the
`troubleshooting-doc` no-modify rule.

#### Verify at the user boundary: why the resource-gated check runs first (URF)

A verification that depends on a resource the user just made available (a connected device,
 a running service,
 a live login,
 temporary access,
 physical hardware) is time-limited in a way ordinary work is not:
 the resource can vanish (unplugged,
 timed out,
 revoked) while you do something else,
 and then the check is no longer possible.
So it runs before unrelated work and before other parts of the same task.

The trap is the "these are all part of the same task" rationalization:
 a scope expansion ("do all of X too") or a long multi-item task list feels like a license to keep working through the
 list and leave the resource-gated check for the end.
It is not.
The resource's availability window,
 not the task-list order,
 decides what runs next;
 finish the resource-gated verification end to end before starting the next unit,
 even when that next unit is nominally in scope.
And "done" means the resource was actually exercised,
 not that the code that would use it was written.

Failure that produced this rule:
 given "my Android device is connected" plus a scope expansion to instrument every Rust crate with tracing,
 the on-device logcat verification was deferred behind the desktop,
 linter,
 terminal,
 forbidden-strings,
 and forbidden-regex crates and left for the final verification pass.
By then the user,
 assuming the check was already done,
 had disconnected the phone.

#### Documentation standards: why WR5 bans positional references

Positional pointers ("above",
 "below",
 "the previous section",
 "as noted earlier") go stale silently when content moves:
 lines get reordered,
 files split apart under MXL/MXR,
 headings get renumbered,
 or a doc gets promoted out of a package under DL1.
A name-based reference (rule tag,
 heading text,
 file path,
 symbol name,
 dependency name) survives every one of those moves,
 because it does not depend on where the referenced content currently sits.
The rule reaches TSDoc and code comments,
 not only markdown prose:
 this is the exact failure fixed in `package/terminal/app/Cargo.toml` and
`package/music-player/desktop-app/Cargo.toml`,
 where "the slint dependency above" became "the slint dependency in this file".

#### Before claiming inability: why one failed probe is not proof of absence (RPB)

Connect-gated and authorize-gated resources report empty until they are reconnected,
 authorized,
 or the server is restarted:
 an empty `adb devices`,
 a refused connection,
 or a 404 is consistent with both "absent" and "present but not yet established."
When the user has stated the resource is there,
 the probe disagreeing with them is weak evidence,
 not a verdict.
Re-probe,
 and ask the user to reconnect,
 re-authorize,
 or restart it,
 before concluding it is unreachable or moving on.

Failure that produced this rule:
 an empty `adb devices`,
 run once,
 was read as "the sandbox can't reach your device,"
 when the device was merely unplugged or unauthorized at that moment;
 a single reconnect by the user made it appear immediately,
 and the on-device verification then succeeded.

#### Architecture decisions: the clearnet leak behind OWK

OWK keeps a concern that spans components inside the owner the user named or approved,
 and its trigger names the three moments such a leak can pass:
 designing,
 briefing,
 and merging.

Failure that produced this rule (issue #1819,
 2026-10-31):
 the user asked "Make our wg auto make update traffics go through clearnet",
 which named the `package/wg/*` tools as the owner.
The coordinating session's brief proposed that the toolchain re-execute itself into the slice the wg exemption watches,
 framed as a suggestion ("The likely shape is ...").
The worker built it in both the TypeScript toolchain and `mpm-shadow`,
 and the merge review checked only that traffic from the slice left the tunnel,
 not which component decided.
The user's correction:
 "mpm-shadow should never determine itself goes through clearnet.
 Our wg set of pkgs should determine that!"

The severity sentence carries the two harms the user named:
 a tool routing around a VPN on its own can expose someone whose safety depends on the tunnel
 ("Imagine somewhere where if some people detect you're accessing crates.io you're paid a visit"),
 and the bypass route can itself fail on unusual networks.
The user rejected "Tunnel leaks" as too specific,
 so the sentence speaks of leaks in general.

"Named or approved" excludes an owner an agent adopted and recorded on its own,
 which `QGR`'s "adopt + record unasked" would otherwise let it quote back as authority.
Approval means the user accepted the ownership itself:
 approving a release that carries a design is not approving who owns the concern,
 and that release approval is how the self-placement nearly shipped in `mpm-shadow` 0.2.1.
"Cross-component" keeps the rule from asking who owns every helper inside one package.

How the wg tools present their bypasses (warned on every start,
 no toggle,
 always enforced) is the user's policy for those tools;
 it lives in their READMEs,
 not in `AGENTS.md`.
The interview that settled this wording is recorded in `doc/planning/agents-md-ownership-and-brief-rules.md`;
 the rejected drafts `CWN` and `LKF` are reserved codes.

#### Before briefing an agent: why BRQ quotes sources and demotes derivations

On 2026-10-31 (issue #1819) three instructions reached workers in the coordinating session's own wording:

- The clearnet mechanism,
   already framed as a suggestion ("The likely shape is ... for example ..."),
   was built as written.
   OWK,
   firing at brief time,
   rejects that design;
   BRQ makes such ideas revisable rather than binding.
- An opt-in,
   off-by-default,
   fail-closed policy for the wg bypasses,
   inferred from the user's remark about severity,
   went to a worker as a requirement.
   The user's actual policy was the opposite.
- A note told a worker to name "0.2.0's archive per the bootstrap design".
   The bootstrap decision says "a prebuilt `mpm-shadow` archive with its SHA-256 in the lock" and names no version;
   the worker wrote a tag-pinned request that `PIN` forbids.
   A citation sat beside a paraphrase that contradicted it,
   which a quote would have exposed.

So requirements are only what the user said or decided,
 quoted with their source,
 and "notes" covers mid-task corrections,
 since the third instance was a note rather than a brief.
Derivations are suggestions because the opt-in policy was presented as following from the user's words.
The policy clause closes the relabel path:
 without it,
 a coordinator could send "Suggest making every bypass opt-in" and let a worker build it.

Considered and not chosen:
 tightening `ASK` to "ask before acting on them or briefing an agent with them",
 defining "settled" in `QGR`,
 and a policy-versus-scope clause in `VR2`.
