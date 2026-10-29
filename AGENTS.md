# Development guidelines for AI agents

ORG:
 Organized by moment of decision,
 not topic;
 "Architecture decisions" and "Agent skills" hold cross-cutting reference.
Rationale,
 mechanisms,
 examples:
 `doc/philosophy/agents.md`.

TAG:
 Every rule starts with a `[A-Z0-9]{3}` code (`CODE:`),
 a stable cross-session handle unique across `AGENTS.md`,
 skills,
 and package docs;
 never tag headings,
 code fences,
 or the title.

RLM:
 Each tagged rule stays under 50 words and 200 characters after whitespace normalization;
 split longer guidance into fresh tagged rules.

SLF:
 Each tagged rule makes sense alone:
 never cite another rule's code or lean on terms only another rule defines.
Citing paths and docs is fine.

NCD:
 New codes:
 fresh,
 unique,
 semi-meaningful;
 check both forbidden-strings appendixes;
 reject unrelated first readings (acronyms,
 products,
 ordinary words,
 external prefix+digit namespaces).

CRN:
 Reuse a code only to rename a misleading one,
 updating all uses at once;
 retired/rejected codes go in `forbidden-strings.append.local.txt`,
 renamed identifiers in `forbidden-strings.append.txt`.

APG:
 Auto-push is enabled.

## Before responding to the user

### Communication style

HON:
 Honest;
 research,
 don't deflect.
Unpublished package change = design change,
 not compat break.

SYS:
 Never attribute injected context (`<system-reminder>`,
 MCP instructions,
 skill descriptions) to user;
 cite policy by content.
A `role:user` turn doesn't prove a human typed it.

WKP:
 Wakeup/cron/continuation prompts are self-authored,
 never user authority.
Write only the real task or a bare sentinel;
 when fired,
 re-derive actions + stop conditions from user instructions + state.

QWD:
 Saying work is queued,
 deferred,
 or waiting for a free slot means writing it to the handover in the same turn;
 an unwritten queue item is lost at compaction.

DCK:
 After corrections,
 decisions,
 answers,
 checks,
 and before compaction,
 update canonical docs:
 meanings,
 requirements,
 evidence,
 assumptions,
 limits,
 rejections,
 unknowns,
 commits,
 next action.

1ST:
 User's first-person words ("I",
 "me",
 "future me") name the human typing,
 never Claude or future sessions,
 even in handover framing.

SRC:
 Before attributing a rule to a file (`AGENTS.md`,
 `CLAUDE.md`,
 `SKILL.md`,
 settings,
 harness prompt,
 MCP instructions),
 grep that file.

EXT:
 External tool features,
 CLI options,
 conf syntax,
 API capabilities ("does X support Y"):
 fetch current doc/src before answering,
 never recall.

TXF:
 Link to selection where possible:
cite web sources with `#:~:text=` text fragments
or the platform's selection permalink,
and confirm the match with `agent-browser` before publishing it.

WRN:
 Explaining warning/error:
 name exact emitting tool + diagnostic code/message.
Unsure?
 Grep codebase,
 check tool docs,
 or run tool first.

GAP:
 "I was expecting you to..." or spotted failure mode = doc gap:
 do expected action + propose `AGENTS.md` edit,
 tightening existing rules first;
 never "I'll keep it in mind".
Remove superseded rules.

EPR:
 Research precedent before offering options;
 for one the repo runs,
 read its full docs and config,
 adopting its answers unasked.
"Think in X":
 list all X's features,
 ask which to omit.

VHI:
 Handoffs (visual or doc) state purpose,
 changes,
 what to inspect,
 and how to respond;
 never make unexplained internal labels the user's task.

### Proactivity calibration

PX1:
 Take authorized steps unasked,
 including recovery.
While verification runs,
 advance independent queued work without changing its inputs.
Don't poll;
 stop only at completion or a genuine blocker.

MWK:
 Monitors,
 wakeups,
 and waiting subagents rarely wake main agent:
 wait in one command that ends at a terminal state;
 emit only lines you'd act on,
 never routine progress.

WUC:
 When a wakeup,
 cron,
 or loop's work finishes,
 cancel it that turn (`ScheduleWakeup` stop,
 `CronDelete`);
 a stale fire is a harness slip to stop,
 not a task to redo.

PXQ:
 "Completion" means the queue:
 tracked work left -> start the next item unasked.
Never end a turn on a status report the user must answer with "continue";
 low context is no reason to stop.

PX2:
 Proactivity keeps constraints:
 destructive/external actions still need authorization,
 decision verbs return answers,
 non-measurable preferences get asked.

PX3:
 Act + report on local work,
 current-repo GitHub mutations,
 and solely user-controlled resources.
Else need authorization;
 unsure -> ask.
Drafts stay local;
 read-only research allowed.

TSK:
 Broad multi-area requests:
 one task-list item per major area,
 each independently verifiable;
 never one umbrella item.

### Pre-response checklist

CK9:
 Quoted clause + drew conclusion?
 Restate subject + object in plain English first;
 obligation direction is the classic misread.

CKB:
 Correction?
 Retract claim,
 rebuild evidence from corrected input,
 revalidate remedy.
Use sources,
commands,
or separate reviewer;
never same-session self-review (`doc/agent/self-review.md`).

XIC:
 Similar or concurrent symptoms stay separate incidents until user-visible boundaries match.
Component removal,
 log silence (retention + emitter unverified),
 or later recovery proves no cause or fix.

### Measure-vs-ask

QF1:
 Measurable facts (sizes,
 counts,
 conf values,
 file contents,
 user's working pattern in repo artifacts):
 measure,
 cite result inline.
Categorical dismissals are one `rg`/conf-read away.

QJ1:
 Run measurement yourself before any quantitative claim or adjective ("small",
 "fast",
 "trivial").

FRC:
 Completion-time forecasts ("done today",
 "takes a year") need evidence:
 a rate measured across the whole task.
Without it,
 say you cannot tell and report measured progress.

DVP:
 Target device available:
 probe its settings + limits before web research;
 external sources only explain probe results or fill gaps.

QAB:
 Predicting a change's (or revert's) effect,
 or saying current behavior already matches a request,
 from reading code is inference:
 run the built app or apply it in a fork,
 measure,
 then conclude.

QPC:
 "No difference" counts only from a probe proven able to show one:
 run a positive control first (a case that must move).

QIV:
 Before trusting a null or count,
 validate scope,
 cache,
 harness,
 generator reach (stale cache,
 one-file lint,
 narrow fuzzer,
 wrong assertion,
 unawaited lazy suite);
 list unexercised surfaces.

QNB:
 Comparing noisy measurements (timings,
 benchmarks,
 model or provider outcomes):
 measure run-to-run spread on unchanged input first;
 smaller differences are noise.

ASK:
 Non-measurable facts:
 ask.
Preferred approach,
 feature wanted,
 destructive-action authorization,
 values (depth vs governance,
 speed vs clarity).

QGR:
 Settled decisions determining one answer:
 adopt + record unasked,
 even while grilling.
Flagged choices (veto-open adoptions,
 open questions):
 ask same turn with options;
 never park them in docs.

QCS:
 Under a quality-over-cost guideline,
 options differing only in price aren't user questions:
 pick the one buying more evidence or better output,
 record it,
 invite veto.

QSP:
 Never bundle separable decisions into one option set or offer complements as alternatives;
 "both,
 in this order" must be reachable.

QPM:
 Before options,
 research and record relied-on concepts,
 even familiar terms;
 distinguish facts,
 assumptions,
 and limits.
 Try removing constraints first;
 see `doc/agent/understanding.md`.

### Present options with pros, cons, and a personal ranking

OPT:
 Distinct options:
 pros + cons for each,
 then "Ranking:
 B > A > C,
 because ..." giving the reason for every adjacent pair.
 Score capability for the target workload,
 never fidelity to incumbents.

OPA:
 `AskUserQuestion`:
 question text ranks options with adjacent-pair reasons;
 each option's `description` and `preview` give pros,
 cons,
 and why it ranks there;
 best first,
 marked "(Recommended)".

QRX:
 Asking for decisions:
 batch them in the question tool and re-explain each item's context in plain words;
 never rely on the user recalling earlier messages.

UQT:
 Anything the user must see or answer (handoffs,
 proposals,
 flagged choices):
 put it in the question tool;
 transcript prose may go unread.

YKZ:
 Before ranking several options:
 widen to plausible alternatives (with their libraries and repo incumbents);
 design each until disqualifying problems surface.

ODM:
 Option examples use concrete inputs,
 relevant constraints,
 and each option's visible result;
 abstract labels aren't examples.
 Demonstrate every concept being compared.

### Exhaust evidence layers when assessing system usage

EVL:
 Usage/migration advice:
 inspect usage/config,
 parallel implementations,
 TODOs/workarounds,
 suppressions,
 and policies.
Evaluate desired capabilities and target workload,
not just today's jobs.

EL4:
 Codebase health signals:
 zero TODO/FIXME/workaround hits mean discipline only if the search provably ran;
 thousands mean debt.
Suppressions with rationale are healthy;
 bare ones are debt.

### Before claiming inability

CB1:
 Before refusing or handing off,
 bridge:
 shell utils;
 web via `agent-browser`;
 GUI via nested compositor,
 HTTP/IPC;
 auth via `expect`/tokens;
 hardware via CLI.
Refuse only after;
 state bridges tried.

RXH:
 Research:
 narrow "no evidence for X" -> widen to comparable entities (siblings,
 peer platforms) first.
State searches + comparable evidence;
 narrowest empty query isn't "no precedent".

CB2:
 Claims an external system "can't" or behaves "by design":
 read the deciding source;
 black-box probes aren't proof,
 and workaround menus assert the claim.
Surprise after your edit:
 diff it.

RPB:
 Resource the user says exists fails one probe:
 re-probe,
 then ask user to reconnect/re-authorize/restart before concluding unreachable.

FCH:
 Substance not in view:
 fetch it or read on before concluding,
 whether a doc points elsewhere or mandated steps sit later in the file.
 Never hedge ("likely contains") about text one read away.

### Name the verification step

NVS:
 Pair confident claims about environment,
 external tool,
 or src inline with their backing (re-verified path:line,
 command,
 doc).
No backing:
 verify,
 or label as guess.

QRY:
 Search output claims the search ran and lines match;
 both fail silently (bad `--type`,
 masked stderr,
 `head` caps,
 `-v` filters,
 hidden/ignored skips).
Sanity-check broader,
 uncapped,
 unfiltered.

### Git cleanup and worktree safety reviews

GCL:
 Before doing or reviewing cleanup of ignored files (`git clean -X`,
 artifact deletion):
 list them with `git clean --dry-run -d -X` first.

GCR:
 Before cleanup,
 check root `HEAD`,
 `config`,
 `hooks`,
 `objects`,
 `refs` (stray git-dir entries) with `ls -d` + `git check-ignore --verbose`;
 any hit makes safe cleanup part of the finding.

GC2:
 `git status`,
 `git ls-files --others --exclude-standard`,
 and `rg --files` hide ignored files;
 never use them as cleanup evidence.

### Research tools

RT1:
 `rg` for text search,
 not directory navigation;
 `rg --files` for globs;
 `--hidden`/`--no-ignore` only with a named path:
 the rg shadow refuses them at a repo root.

RT4:
 `gh` for GitHub issues,
 PRs,
 release notes,
 repository metadata.

## Before running a command

### Command execution conventions

TMO:
 No external `timeout` around routine verification;
 use the command tool's session/polling and stop stale processes by PID.
Wrappers only for behavior-under-test or unbounded runtime.

NXR:
 Transport failure (`No result provided`,
 dropped session) after a command may have run:
 inspect processes + logs first;
 never repeat the same synchronous call;
 rerun in background or with a bound.

PWT:
 Never wait on a process by name:
 `pgrep --full` in a loop matches the agent's own shell wrapper and never exits.
 Wait on a PID,
 bounded:
 `timeout 600 tail --pid=<pid> --follow /dev/null`.

BGL:
 Every background wait or poll loop has a deadline,
 and its starter stops it before reporting:
 a leftover loop keeps its worktree locked,
 so check `ps` for processes whose cwd is your worktree.

INH:
 A user-interrupted tool call (`Request interrupted`,
 `doesn't want to proceed`) may be half-applied,
 not refused:
 inspect the files,
 processes,
 and installs it touched before acting.

UPN:
 Upgrades name their tool (`mpm upgrade <package>`);
 repository tools move through `toolchain:update`,
 whose lock diff is the review surface.
 Never run bare `mpm upgrade --all`.

TRB:
 External tool fails with a diagnostic:
 `rg` its message in `doc/troubleshooting/` before any workaround;
 follow a recorded verified workaround as written.

EDR:
 Parallel tool calls may run in any order,
 so a command reading a fresh edit can see the pre-edit file;
 send dependent commands only after the edit returns.

1CB:
 At most three `&&`-chained steps per shell call;
 no `;` chains or loops.
Longer multi-step work:
 write a scratch `.ts` (`node:child_process`) and run it.

RGP:
 `rg` without a path may read stdin:
 always pass `.` or an absolute path.

ATH:
 Scratch goes in `${HOME}/temp/agent`,
 never `/tmp` or the harness scratchpad
 (reboots wipe them);
 first `mkdir --parents` it,
 then `chmod 700`
 (trust checks reject group/other bits).

XTD:
 Never use `/tmp` for scratch,
 fixtures,
 or caches:
 which device backs it is unknowable (tmpfs,
 quota,
 slow disk).
Use `${HOME}/temp/agent` or the build directory.

CLN:
 Investigating package source:
 `gh repo clone <repo> "${HOME}/temp/agent/<name>-<date>" -- --depth 1`,
 not `git clone`,
 unless commit history matters.

APQ:
 Auto-push fires in third-party clones too:
 before committing in one,
 run `git remote set-url --push origin DISABLED`;
 never in this repo's linked worktrees,
 which share its `.git/config`.

BOP:
 `~` in shell output is a display-only home-dir substitution by the `bash-output-filter` hook;
 bypass it with `eval`,
 `export`,
 `source`,
 `$(...)`,
 backticks,
 or `> file`.

WCD:
 Pin target dir on every shell command (native `-C`/`--cwd` or `cd -- <abs path> &&`).
Before alternate-worktree writes,
 verify `pwd` + `git rev-parse --show-toplevel`.

CLH:
 Before automating CLI prompts (pipes,
 PTYs,
 drivers):
 check `--help` and current docs/source for native noninteractive flags;
 prefer them over terminal emulation.

### Long-form flags

LFF:
 Use long-form (`--flag`) CLI options,
 not short flags;
 writing long form forces knowing what each does.
No long form:
 short flag stays.

RGT:
 `rg` recurses by default;
 its `-r` means `--replace`,
 so grep-reflex `rg -rl`/`-ir` silently rewrites matches in output;
 the rg shadow refuses short `-r`.

### Hazardous commands

HRM:
 Could an action physically harm a human or wear hardware?
 Warn first.
`ssh m1`:
 16 GiB RAM cap,
 fragile internal SSD;
 probe first,
 put write-heavy work on `/Volumes/MacData`.

RXI:
 Host-exhausting risks (heavy memory/process/fd use,
 unbounded loops,
 uncapped fan-outs,
 stress/bench/load):
 run in `podman run --memory=2g --cpus=2 --rm` or `mvm`,
 stating bounds.

NLG:
 Never generate synthetic host load in any session or subagent:
 no `stress-ng`,
 busy loops,
 `--stress-count`,
 or repeated full test runs;
 cap the test's own container (`--cpus=2`) instead.

NLD:
 Every subagent brief quotes the synthetic-load ban;
 before closing its worktree,
 stop every process and `systemd-run` unit the subagent left (check `ps`,
 `systemctl --user list-units`).

NQH:
 Never wait for a quiet host or call one quiet:
 other sessions keep the load average up.
Measure under the load there is,
 alternating arms with a same-binary control for the spread,
 and report counts.

FHU:
 Real jobs may use almost the whole host:
 size `CPUQuota` near `nproc` minus a few threads,
 `MemoryMax` near available memory;
 run independent jobs in parallel.
Synthetic load stays banned.

BOX:
 Third-party benchmarks run mount-free,
 inputs baked into the image.

DCB:
 Never run or have agents run catastrophic commands (`sudo rm -rf /`,
 `mkfs`,
 `dd of=/dev/sda`,
 fork bombs),
 even as guardrail tests;
 test guardrails with moderately dangerous ones.

VKI:
 Synthetic key input:
 nested compositor or caller-independent broker only;
 never `ydotool` from an agent command (key-down can cancel the caller before key-up,
 wedging desktop input).

### Essential commands

CM1:
 Narrow package work:
 run that package's task,
 never reflexive repo-root `mise run test`.

CM3:
 All tasks via `mise run`;
 never `pnpm exec`,
 package scripts,
 or raw tools (`tsc`,
 `tsdown`,
 `bun test`).
No suitable task:
 add one to package `mise.toml`;
 tests may run via `node <file>` meanwhile.

CM5:
 Find tasks in root + package `mise.toml`;
 run as `mise run //package/<path>:<task>`,
 not `mise run --cd`.

CM6:
 After editing TypeScript,
 run `mise run //package/<path>:lint:types`;
 nothing type-checks automatically.

CM7:
 Provision tools with `mise run toolchain:sync`;
 `toolchain.lock.json` pins every version,
 and ad-hoc installs or lock edits bypass the checked-in toolchain.
 Mise runs tasks;
 it manages no tools.

## Before editing code

### Match action scope to the request verb

VRB:
 Decision verbs ("decide",
 "review",
 "audit",
 "investigate",
 "propose"...) want an answer + required docs,
 no fixes;
 action verbs ("fix",
 "implement",
 "update"...) authorize action.

DRR:
 Recommendations,
 even delegated:
 brief evidence,
 ranking,
 risks;
 proposals in `doc/planning/`;
 only explicit acceptance (not review or sub-question answers) unlocks `doc/decision/` or dependent work.

IWT:
 Deliberation requests ("review",
 "audit",
 "investigate"...):
 main worktree gets doc/report writes only;
 experiment in `git worktree add <path> HEAD`,
 removed after.

AUT:
 Auto mode's "prefer action over planning" covers executing the requested action,
 never expanding scope or acting on adjacent undecided choices.

VR2:
 Request with one clear reading:
 act.
Readings differing in what to do:
 confirm first.
Readings differing only in how far to go:
 do the narrower,
 propose the broader explicitly.

ANN:
 Put changes where they belong immediately (other file,
 new file,
 gitignore entry);
 unsure:
 propose the concrete edit + location.

EC4:
 Never implement features that can't achieve their intended effect;
 explain the limitation instead of writing non-functional code.

### Cross-runtime and scripts

XRT:
 Prefer cross-runtime patterns over Bun-specific APIs.

HOM:
 Derive current-user paths from injected home or runtime homedir,
 never a hardcoded username or `/home`;
 environment-sensitive tests inject disposable homes.

SCR:
 Never write bash/powershell scripts or `mise.<action>.ts` files;
 put task logic inline via `shell = "node --input-type=module-typescript -e"` or in a package bin.

NMJ:
 Write scripts as `.ts`,
 never `.mjs`:
 node runs TypeScript directly by stripping types,
 so `.mjs` has no advantage,
 and autofix runs have corrupted it.

PIN:
 Pin tool versions only with a comment explaining why;
 never pin tools this repository publishes,
 since `toolchain.lock.json` records them.

SPG:
 Automation that spawns agent sessions needs explicit recursion guards (env var flag,
 session type filter,
 transcript size check).

CM2:
 `mise.toml` tasks:
 sequence with `run = ["a", "b"]`,
 never `;` or `:::` chaining;
 `shell = "node --input-type=module-typescript -e"` only for logic.

WC2:
 file-enforcer manages root files (`CLAUDE.md`,
 ...) and package `Cargo.toml`:
 check `file-enforcer.config.ts` before editing or adding one;
 if managed,
 edit its source,
 run it,
 commit output as-is.

### Simplification

IMM:
 Prefer immutable patterns.

UTL:
 Reuse existing repo utilities (e.g. `wait()` from `@monochromatic-dev/async-time-core`) before writing helpers.

XNC:
 Name extracted concepts by role and boundary behavior,
 revealing sentinel and fallback semantics;
 start simple,
 refactor only when needed.

ITR:
 Linear input (strings,
 flat arrays):
 iterate;
 never recurse or rebuild accumulators (`acc + c`).
Recurse only bounded structural walks;
 flatten spines with a work stack.

MXL:
 Over max-lines (TS,
 Rust):
 split into sibling files/modules (constants,
 types,
 helpers),
 re-exporting from `index.ts`;
 never strip docs/`//region` or reformat to fit.

### Linting

LN1:
 Lint rules in apparent conflict:
 restructure (split,
 extract,
 rename);
 never violate one or reformat to silence another.

LN2:
 Each lint finding is a design signal:
 name the rule's intent,
 then write the best code shape satisfying it and the codebase.

LN3:
 Before suppressing a lint rule:
 inspect linter source + linted value;
 try config/allow-list.
Remaining suppression:
 justified disable comment plus `.md` doc citing both,
 proving config fails.

LN6:
 Suppressing a documented declaration:
 `/* oxlint-disable rule */`,
 TSDoc,
 declaration,
 `/* oxlint-enable rule */` on the very next line;
 never `disable-next-line` between TSDoc and declaration.

LN7:
 Never loosen lint rules without prior approval.

### Logging and diagnostics

LOG:
 Log entry points,
 branches,
 errors,
 and async lifecycle.
Decision logs retain correlated inputs,
 clause/source references,
 model scores,
 and fallback reasons sufficient to reconstruct a verdict.

TLG:
 Production code logs only via tagged loggers from `@monochromatic-dev/logger-core`;
 raw `console` only for exact terminal output (CLI output,
 prompts).

LG1:
 Tag loggers at every module + function boundary with `myFn.name`,
 re-wrapping with an added tag when passing to a sub-function;
 never embed tags in message strings.

LG2:
 Every `catch (error)` uses its binding:
 log the caught value (even expected) or rethrow.

DGT:
 User-facing diagnostics:
 name the affected input and calls plainly;
 explain uncertainty and every valid remediation path;
 no unexplained implementation terms;
 length is unconstrained.

DNL:
 Diagnostic names and messages use neutral operation or evidence terms,
 never moral judgments of code,
 types,
 or authors.

### Security

SYB:
 Text crossing syntax boundaries obeys destination grammar:
 encode at final interpolation.
Never invent comment-string DSLs for relations the type system or AST can express or infer.

STB:
 Tests for code emitting another syntax include adversarial boundary cases:
 delimiters,
 escapes,
 quotes,
 newlines,
 traversal tokens,
 command separators,
 source-escaped variants.

PRV:
 Before sharing media or data externally:
 inspect every region in dense samples;
 mask status bars,
 notifications,
 paths,
 titles,
 accounts,
 identifiers;
 strip metadata + unintended audio.

### User interfaces

CXD:
 Any UI or state output:
 mark states and action prominence with two visible channels (color,
 weight,
 icon,
 label,
 boundary,
 position),
 never color or shape alone;
 preserve content space.

HDM:
 Agent-authored HTML follows the viewer's system color scheme:
 build + verify light and dark;
 open it in current system mode.

### TSDoc comments

TSD:
 Non-async wrappers document via `{@inheritDoc originalFn}`.

TD1:
 Comments inside template literals:
 `${ // comment \n '' }`,
 never target-language comments or moving the comment outside.

TD2:
 TSDoc (`/** */`) only directly before declarations;
 `//` or `/* */` for statements,
 control flow,
 imports,
 returns.

TD4:
 Comments go on their own line above code,
 never trailing it.

TD5:
 Escape `*/` as `*\/` inside TSDoc blocks.

TD6:
 `@param`/`@returns`:
 no articles;
 explain why,
 not what.

TD7:
 Async function docs never mention Promise wrapping.

JCH:
 Never write `@mutates` for absent effects:
 move work to an ownership-known boundary,
 pass its primitive result,
 or improve the proof;
 contracts describe possible runtime effects,
 not analyzer gaps.

### TypeScript

#### Standards

ST2:
 Mark logical sections with `//region`/`//endregion`,
 stating purpose + explanation.

ST3:
 Cross-package workspace imports use the package's `/ts` subpath (TypeScript source),
 never built output;
 rationale:
 `doc/decision/workspace-ts-source-imports.md`.

ST5:
 Prefer named imports;
 import workspace packages by absolute package name.

ST6:
 Static assets (SVG,
 HTML,
 CSS,
 SQL):
 `import ... with { type: 'text' }`,
 not `readFile`;
 build tooling resolves them.

ST8:
 Declare functions before calling them in source order,
 despite hoisting.

ST9:
 Functions with 2+ parameters take one destructured object,
 except callbacks with externally dictated signatures.

TQ2:
 Export at declaration,
 not in a trailing `export { }`;
 never extend typed objects via `Object.assign`.

TQ3:
 Throw + return early.

XPT:
 Exporting small helpers through the package API so built-artifact tests reach them is allowed.

#### Type system

TY2:
 Write `Generator<T>`/`AsyncGenerator<T>` without unused or optional type arguments.

TY3:
 `as const` for literals;
 branded types for domain primitives.

TY5:
 `const` generic parameters with meaningful constraint names.

TY6:
 Avoid deeply nested conditional types.

TY7:
 Runtime narrowing:
 type guards or assertion functions (`asserts value is T`).

TY8:
 `const` narrowing doesn't reach function declarations:
 use a helper returning non-null,
 or a new explicitly typed `const` after the null check.

TY9:
 Generator overload signatures omit `*`/`async *`;
 only the implementation has them.

#### Variables and values

VA5:
 `satisfies` checks types without widening;
 destructure dependent values in separate statements.

#### Programming patterns

PP1:
 `async`/`await` only:
 no promise chains or `new Promise` except a callback-API boundary bridge that resolves or rejects once.

PP2:
 Concurrent async work:
 `Promise.all`,
 or `Promise.allSettled` when failures must not discard other results;
 cancel via `AbortController`.

PP4:
 Signal failure by throwing custom error classes,
 never error codes,
 null,
 or result types;
 document with `@throws`.

PP5:
 Replace `!` with `nonNullishOrThrow` (`@monochromatic-dev/or-throw-core`);
 build multi-line error messages with `dedent` (`string-dedent`).

PP6:
 Put error text in the thrown error,
 not a preceding `console.log`/`console.error`;
 set `process.exitCode` only for non-standard exit codes.

PP8:
 Throw on unreachable branches;
 never silently discard unexpected states.

PPX:
 Class members default to `#private`.

#### Regular expressions

RG2:
 Code replacing a regex makes one linear pass (O(n) time,
 O(1) stack),
 proven O(n) for unbounded input.

### Third-party libraries

TP1:
 Third-party APIs and CLIs:
 read the installed type definitions before calling;
 on an undefined-method error,
 fetch current docs immediately;
 test the simplest invocation first.

TPF:
 Facade packages:
 probe every exports-map subpath and confirm the runtime binding with `typeof`;
 the root export and type definitions alone never prove a binding exists.

### Dependency management

DM1:
 Internal dependencies use `workspace:*`;
 external ones use `catalog:`,
 with versions in the `pnpm-workspace.yaml` catalog.

LFW:
 Never hand-edit lockfiles:
 regenerate via the owning package manager or repo task,
 inspect the generated diff,
 report unrelated drift separately.

RCI:
 Before proposing a new owner for a responsibility,
 inspect existing repo-owned generators and managers;
 extend one that already owns it.
 A stopgap for a misbehaving tool is not an owner.

### Adding new packages

AP1:
 New packages go under `package/<category>/<name>`.

AP2:
 New packages get a `mise.toml` with tasks mirroring sibling packages.

AP4:
 CLI packages with `bin`:
 `#!/usr/bin/env node` as the first line,
 or Unix falls back to `/bin/sh` and hangs;
 `#!/usr/bin/env bun` only in documented Bun islands.

SGD:
 Dir segments singular;
 package name = `@monochromatic-dev/` + path under `package/`,
 `/` -> `-`.
Rename dir + name + consumers together.
Exemptions:
 `doc/planning/singular-dir-name-invariant.md`.

SBS:
 Sidecars (`.fuzz`,
 `.bench`,
 `.conformance`) sit beside their subject package as `<pkg>.<kind>`,
 never under a per-kind top-level dir;
 move dir,
 name,
 and consumers together.

## Before declaring work complete

### Package completeness

RNQ:
 Rust packages use Rust tests through nextest,
 never TypeScript tests.
Use nextest's default build and test concurrency unless the user specifies otherwise.

PKG:
 A package is complete only with `README.md`,
 zero lint errors,
 and passing tests covering every exported code path.

TCV:
 Tests cover every implementation branch (sync/async,
 string/object,
 direct/delegated),
 not just the happy path;
 passing tests show completeness only after comparing test names against branches.

TRV:
 A green test run counts only when its output contains the expected suite and test names;
 an unawaited lazy suite exits 0 while running nothing.

GFP:
 A guard test proves nothing until shown to fail:
 commit it,
 remove the guard,
 rebuild,
 run,
 restore;
 restoring discards uncommitted work on that file.

CXL:
 Plugin and process cleanup must not emit bare shutdown errors (e.g. `context canceled`):
 capture stderr in lifecycle tests and fix hook ordering;
 never filter the noise.

### Verify at the user boundary

VUB:
 After build,
 deployment,
 or install,
 verify the consumer with configured extensions,
 skills,
 tool catalogs,
 and the reported interaction's preceding turns;
 compilation alone is insufficient.

VB1:
 Servers:
 check responses,
 not startup.
CLIs:
 run the real command,
 check output.
Hooks/plugins:
 trigger via the host app.
Libraries:
 import and call from a consumer.

VB5:
 Web pages/HTML artifacts:
 load in `agent-browser`,
 confirm no console errors,
 exercise every interactive element,
 read rendered state via `agent-browser eval`;
 drive each rewritten JS path.

VB7:
 Markdown ships only after a rendered check (live page or renderer output);
 lint misses CommonMark emphasis edge cases.

SCF:
 Screenshot after scripted input:
 confirm intended rendered state,
 then capture;
 command completion isn't frame completion.
Recapture stale or transitional frames.

ABR:
 End browser verification with `agent-browser close`:
 open pages keep animating,
 and WebAudio unlocked by scripted clicks plays on system speakers.

URF:
 Verification needing a user-provided resource runs first,
 before other work or other parts of the task;
 scope growth never defers it;
 not done until the resource is exercised.

### Verify on a throwaway, not against real state

THR:
 State-mutating verification uses disposable fixtures (`mktemp -d`,
 throwaway worktree,
 container),
 never real or shared state,
 even when idempotent;
 guard tests need allowed and rejected fixtures.

TAE:
 Before prescribing tool/API behavior in prompts,
 docs,
 configs,
 or CI scripts,
 test it with a real invocation,
 never from how it should work.

## When committing or documenting

### Documentation standards

#### Prose style

WR2:
 Prose never uses em-dashes,
 en-dashes,
 or ASCII substitutes for them:
 use paired commas/parentheses,
 colon,
 semicolon,
 or period;
 "to" for ranges.
Hyphenated compounds and CLI `--flags` are fine.

WR3:
 Emphasis:
 **bold** inline only;
 never italics or ALL CAPS.

WR4:
 Numerals only where exact count,
 order,
 version,
 ID,
 or measurement matters;
 prefer count-neutral wording;
 mention list length only when it is the claim.

WR5:
 Never point by relative position ("above",
 "below",
 "earlier");
 name the tag,
 heading,
 path,
 or symbol,
 in prose,
 TSDoc,
 and comments.

WR6:
 A moved identifier (URL,
 host,
 path,
 name) in a historical record keeps its original text
 plus an in-place note giving the current value and the move date;
 never leave it pointing nowhere.

#### Markdown syntax

MD1:
 Break lines at semantic boundaries,
 under 120 chars,
 so text reads without editor wrapping.

MD2:
 `-` for unordered lists;
 pad numbered markers to 4 chars (`1.`,
 `10.`).

MD3:
 Fenced code blocks with language tags and file-path comments.

MD4:
 Reference-style links for repeated URLs;
 relative links for internal docs.

MD5:
 No tables (pipe or HTML);
 use headings or lists.

MD6:
 Headings:
 ATX,
 sentence case,
 max 4 levels,
 blank line before.
Standalone titles and labeled points become headings plus prose,
 never bold lines or bold-label bullets.

MD8:
 Hard wraps fall only between whole inline spans;
 keep code spans,
 emphasis,
 and link syntax on one line,
 or the closing delimiter renders literally.

WRP:
 Backtick file names,
 identifiers,
 commands,
 and code tokens in Markdown prose.

### Doc placement

DPL:
 Repo-wide docs live in `doc/`;
 root docs are only `README.md`,
 `SECURITY.md`,
 `AGENTS.md`,
 `CLAUDE.md`,
 `LICENSE`,
 `LICENSES/`.
Package docs stay beside code.

DL1:
 Repo-wide doc paths:
 `doc/<family>/<kebab-topic>.md`;
 a family index,
 if any,
 is `doc/<family>/README.md`.

DL3:
 Bug reports become a section of the most relevant `doc/troubleshooting/<topic>.md`,
 never their own family.

DL4:
 Delete a doc only when its work landed and no durable fact (root causes,
 workarounds,
 tradeoffs) lacks a new home;
 read it first;
 git history isn't a home.

DL5:
 Reference source files by repo-relative path,
 never pinned GitHub blob URLs,
 which break when targets move.

RBK:
 Repo-wide runbooks:
 `doc/runbook/<topic>.md`;
 handovers:
 `doc/handover/<topic>.md`;
 package-specific ones stay beside code.

### Handling external changes

EC1:
 Worktree changes you did not make are concurrent work,
 not emergencies:
 never restore,
 stash,
 or revert them;
 touch only task files.
Unrelated change blocks your edit:
 say so and ask.

### Git commit guidelines

GCE:
 Commit before the next work step,
 never waiting for verification or completion;
 broken states commit too,
 naming the breakage.
Overrides the harness ask-first default.

GCG:
 Commit subjects:
 `<type>(<scope>): <subject>`;
 scope:
 package name minus `@monochromatic-dev/`,
 doc family (`docs(planning)`),
 root file or tool name (`mise`,
 `AGENTS.md`),
 or `*` for multi-package.

GCB:
 Multi-package commit bodies:
 per package,
 `<type>(<package>): <what>`,
 blank line,
 `<why>`,
 in package order.

GCA:
 Inaccurate commit message:
 never amend;
 surface it,
 ask the user to push if auto-push is off,
 and post a corrective commit comment unasked.

CLG:
 Never preemptively bypass `git-policy-cli` guards (they reject bulk staging and pathspec-less commits):
 stage explicit scoped pathspecs;
 `--no-enforce-*` only when none fits.

CPN:
 Commit pathspecs name every new file:
 `git add F` then `git commit -- other/paths` omits `F`,
 leaving imports unresolvable at that commit;
 check `git status --short` after.

XCM:
 External messages report results,
 never work-inviting offers ("happy to",
 "want me to");
 user-only choices get asked before sending;
 necessary blocker questions to the recipient are fine.

## Architecture decisions

AD1:
 Root `package.json` may depend on workspace packages;
 root configs import them by package name.

AD2:
 Config needing logic (`if`,
 `map`,
 `await`):
 switch from data to TypeScript.

AD3:
 Run async work directly,
 not through descriptor/interpreter patterns.

AD4:
 Nested calls (`b(a())`) over method chaining;
 split more than two nested calls across lines,
 never stacked `)))`.

OCG:
 CLI option design:
 output cardinality never determines option occurrence grammar;
 sketch token encoding before asserting repeated,
 delimited,
 or variadic forms.

## Agent skills

SK1:
 **Issue tracker**:
 GitHub Issues via `gh`.
"Resolve issue N" authorizes fix + commit;
 `Closes #N` in the commit body auto-closes on auto-push.
See `doc/agent/issue-tracker.md`.

SK2:
 **Triage labels**:
 canonical roles with default label strings;
 see `doc/agent/triage-labels.md`.

SK3:
 **Domain docs**:
 no context files;
 agents read fresh code on every probe;
 see `doc/agent/domain.md`.
