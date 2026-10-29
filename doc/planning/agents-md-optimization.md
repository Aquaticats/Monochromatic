# `AGENTS.md` optimization

Grilling session started 2026-09-25 from the request "optimize my `AGENTS.md`".

## Baseline

- `AGENTS.md`:
   1815 lines,
   5614 words,
   42264 chars,
   274 tagged rules
   (`wc --lines --words --chars AGENTS.md`;
   `rg --count '^[A-Z0-9]{3}:$' AGENTS.md`).
- `CLAUDE.md` embeds it verbatim via `file-enforcer.config.ts`,
   so every session pays the full cost.
- 143 commits touched `AGENTS.md` since 2026-06-25.
- Repo skills live in `.agents/skills/`,
   mirrored to `.claude/skills/` and `.factory/skills/`.

## Decisions

- Goal ranking:
   always-loaded token cost with full rule coverage,
   then adherence,
   then maintainability.
- Deletion authority:
   merge,
   move,
   and delete rules that fail the Purpose test in `doc/philosophy/agents.md`,
   including harness-duplicated rules,
   exact duplicates,
   and rules a linter already enforces.
   Extra write-lint-rewrite cycles are an accepted cost.
- Out of scope:
   the `CLAUDE.md` preamble in `file-enforcer.config.ts`;
   the user will handle it separately.
   Claude-Code-only rules therefore stay in `AGENTS.md` for now.
- Verification:
   no formal adherence check;
   rules evolve constantly,
   so a formal check costs more than it returns.

## Adopted without asking

- Misplaced rules move to the section matching their decision moment (ORG);
   M1T and PXF leave "Architecture decisions".
- Codes retired by merge or deletion get forbidden-strings entries (CRN).
- Rationale for removed or relocated rules goes to `doc/philosophy/agents.md`,
   following its existing "Removed" practice.

## Rule audit (2026-09-25)

Read-only subagent audit;
working files lived in the session scratchpad,
not committed.
Token counts via `mise run //package/llm/token-count:count` (Claude `count_tokens` endpoint).

- `AGENTS.md`:
   17431 tokens (`claude-opus-5-5`),
   12786 (`claude-sonnet-4-6`).
- Comma-level line breaks cost 1913 Opus tokens (11%);
   OpenAI o200k shows 17,
   so Codex barely pays.
- Estimated savings (Opus):
   situational rules to skills 4910 minus new skill descriptions;
   duplicate merges 1496 (46 rules become 18);
   harness duplicates 140 (RT1,
   RT3,
   RT4);
   reformat 1913.
   Built combined file measured 9795,
   a 43.8% cut.
- Lint-enforced (probed with repo oxlint conf):
   22 rules including IMM,
   LN4,
   PP1,
   PP3,
   PP5,
   PP9,
   TY1,
   TY3,
   TY6,
   ST9,
   TQ1,
   RG3,
   TSD,
   TD8.
- Not lint-enforced despite the rule:
   TLG,
   ST8,
   TD4,
   TQ2,
   LG2;
   these stay.
- No checker at all:
   WR2,
   TAG,
   RLM,
   SGD.
- Stale:
   RT3 names nonexistent `FetchUrl`;
   NCD and CRN point at the local forbidden-strings file,
   but shared codes live in `forbidden-strings.append.txt`;
   TY8 cites tsc 6;
   VA6 understates linter exemptions;
   M1T "MD1 tabs" collides with code MD1;
   VRB and CPN exceed RLM;
   DL1 and DL2 describe a finished migration;
   `doc/philosophy/agents.md` cites `handlers/` instead of `handler/`.
- 24 misplaced rules,
   beyond M1T,
   PXF,
   CXL.
- Side finding:
   `package/llm/token-count` claims all current Claude models share one tokenizer;
   Opus 5.5 and Sonnet 4.6 differ by 36% on the same file.

## Adopted without asking (from audit)

- Fix every stale item listed in "Rule audit".
- Delete the verified lint-enforced rules (per deletion authority);
   keep unenforced ones.

## Decisions (round 2)

- Situational rules:
   not clear cut;
   some only seem task-specific.
   Needs research before choosing destinations.
- Rewrite scope:
   compress every kept rule,
   talked through rule by rule with the user.
- Line-wrap format:
   keep clause-per-line breaks.
- JCH and OWB:
   move guidance into the linter diagnostics.
- TAG and RLM checker:
   tracked in issue #566.

## Rule walk

Each batch is measured with a scratch script:
RLM per rule plus `mise run //package/llm/token-count:count -- --model claude-opus-5-5`.

- Batch 1 (pre-response checklist and its absorbers):
   19 rules become 9,
   1164 to 703 tokens.
   Approved as shown.
   Retires PRE,
   CK1,
   CK2,
   CK3,
   CK5,
   CK6,
   CK7,
   CK8,
   CKA,
   BR2;
   RBK moves to "Doc placement".
- Batch 2 (communication style and proactivity):
   17 rules become 15,
   1300 to 1061 tokens;
   retires WK2 (into WKP) and HUP (into DCK).
   Approved,
   with user edit keeping "honest" in HON.
- Batch 3 (measure-vs-ask remainder):
   11 rules become 9,
   798 to 541 tokens;
   retires MA3 and FLG (into QGR,
   the more-cited code).
   Approved as shown.
- Batch 4 (visual and device rules),
   revised after user correction:
   Android rules are not music-player-only,
   because more Android apps are planned.
   Always-loaded:
   CXD,
   HDM,
   SCF,
   PRV,
   VHI,
   ATS reworded general (426 to 407 tokens).
   Skill-bound:
   QVE,
   PFG,
   QVM,
   MXQ,
   HFM,
   RVC,
   PXF,
   ZDV,
   BZF,
   ANB,
   AVP,
   M1T (763 to 736 tokens,
   leaving `AGENTS.md`;
   748 after self-contained redraft).
   Approved.
- New rule SLF (self-contained rules),
   requested by user:
   approved;
   placed after RLM.
- Batch 6 (option presentation):
   6 rules become 5,
   352 to 293 tokens;
   retires OPI into OPT;
   OCG stays always-loaded (17 CLI packages,
   301 commits to `package/cli` since 2026-06-25) and moves to "Architecture decisions".
   Approved after user removed redundant OPT clause "not just the top pick".
- Batch 7 (evidence layers,
   inability,
   verification,
   research tools):
   18 rules become 10 always-loaded,
   1024 to 710 tokens;
   retires EL1,
   EL2,
   EL3,
   EL5,
   EL6,
   ELR,
   RT2,
   RT3.
   User kept RT1 (`rg` is misuse-prone) and RT4 (not every harness injects `gh` guidance).
   JEV moves to `doc/troubleshooting/rtw89-wifi-disconnects.md`,
   FLK to `package/pi-plugin/advisor/README.md`.
   Approved,
   including compressed RT1 (gains `--hidden`/`--no-ignore`) and RT4.
- Batch 8 (git cleanup):
   GCL,
   GC2,
   GCR stay always-loaded (282 to 234 tokens);
   GCI folds into GCR;
   GCW and WXG move to `package/git-policy/cli/README.md`;
   `doc/agent/regression-suite.md` Case 3 gets updated to match GCR.
   Approved.
- Batch 9 (command execution and long-form flags):
   13 rules become 12,
   1002 to 814 tokens;
   retires LF2 into LFF;
   CLH moves to "Command execution conventions".
   Approved after NXR fix:
   the first draft dropped "rerun via process tool or bounded execution" as covered by "never rerun it synchronously";
   user caught that it was neither covered nor compatible (bounded reruns can be synchronous).
- Batch 10 (hazardous and essential commands):
   11 rules become 10,
   887 to 758 tokens;
   retires CM4 into CM3;
   CM2 moves to "Cross-runtime and scripts",
   WC2 to "Before editing code".
   Approved,
   with BOX's vague "Authorization does not transfer" deleted at user request.
- Batch 11 (action scope,
   cross-runtime and scripts):
   12 rules stay 12,
   762 to 692 tokens.
   Approved except VR2.
- HON and VR2 redraft:
   user required that they cannot be read as conflicting;
   the reading-ambiguity clause moves wholly from HON into VR2.
   Approved.
- Batch 12 (simplification and linting):
   15 rules become 10 always-loaded,
   980 to 596 tokens;
   IMM shrinks to "Prefer immutable patterns" (user kept it);
   deletes lint-enforced
   LN4 (`no-for-in` message names the fix),
   LN8 (`denyWarnings: true`);
   merges LN5 into LN3;
   MXL widens to Rust;
   MXR and RDC move to `package/rust-linter-plugin/builtin/README.md` with codes kept (219 to 180 tokens).
   Approved.
- Batch 13 (logging,
   security,
   TSDoc):
   15 rules become 13,
   796 to 632 tokens;
   TSD shrinks to its unenforced `{@inheritDoc}` clause;
   TD3 merges into TD2;
   TD8 deleted (`tsdoc/require-example`).
   Approved.
- Batch 14 (TypeScript standards,
   type system,
   variables):
   21 rules become 17,
   968 to 667 tokens;
   deletes lint-enforced TQ1,
   TY1,
   TY4,
   VA6;
   trims lint-enforced clauses from ST5,
   TY2,
   TY3,
   TY5,
   TY6;
   TY7 absorbs TY6's type-guard clause.
   ST9 kept compressed despite `require-destructured-params`:
   about 10 files cite it as design rationale,
   and a late lint error forces reworking every caller (round 7 principle).
   Approved.
- Batch 15 (programming patterns,
   regex):
   14 rules become 8,
   800 to 379 tokens;
   deletes PP3,
   PP7,
   PP9,
   RG1,
   RG3 (lint messages name the fix),
   and OWB (prefer-readonly guidance and the ForeignBorrowed README carry it);
   PPX keeps only `#private`;
   PP1 kept as a compact principle (`prefer-await-to-then` misses `return p.then(...)`).
   Apply phase adds the prefer-index-scan clause to the `no-regex` `RegExp` constructor message.
   Approved.
- EDR (added concurrently after batch 9):
   compressed,
   69 to 67 tokens,
   kept because pi runs one response's tool calls in parallel by default.
   Approved.
- Batch 16 (libraries,
   dependencies,
   new packages):
   15 rules become 9 always-loaded,
   822 to 583 tokens;
   retires TP2 and TP3 into TP1 (drops vague "Note CLI command patterns across examples"),
   DM2 and AP3 into DM1;
   AP1 loses its fuzz-sidecar sentence (SBS covers it);
   RCI widens to any new owner;
   RCO moves to the `choosing-technology` skill (user chose move over always-loaded),
   AP5 to `package/config/rolldown/README.md`.
   Approved.
- Batch 17 (completeness,
   verification):
   17 rules become 12,
   964 to 796 tokens;
   retires TC2 into TCV,
   VB2 to VB4 into VB1,
   VB6 into VUB;
   CXL widens to any plugin or process cleanup (already applied to pi plugins) and stays under "Package completeness".
   Approved.
- Batch 18 (prose,
   Markdown,
   doc placement):
   21 rules become 18,
   1290 to 1056 tokens;
   retires MD7 into MD6,
   DL2 into DL1,
   deletes DL6;
   DPL drops lint-enforced `CONTEXT.md` clause;
   WRP drops its fixed-cause rationale;
   MD5 adds "(pipe or HTML)" against the `no-pipe-tables` autofix.
   Side finding:
   `doc/dependency-blocklist.md` and `doc/secret-management-caveman.html` sit outside any family.
   Approved.
- Batch 19 (commits,
   architecture,
   skills):
   14 rules stay 14;
   GCG rewritten to the user's actual convention:
   scope is package name without prefix,
   bare doc family for repo-level docs (`docs(planning)`),
   root file or tool name (`docs(AGENTS.md)`,
   `fix(mise)`),
   `*` only for multi-package.
   No breaking-change clause (unused in 3000 commits).
   No corrective comments on this session's 35 `docs(*)` commits (user choice over GCA's letter).
   Approved.
- Batch 20 (`AGENTS.md` meta rules,
   diagnostics,
   JCH,
   EPR):
   10 rules stay 10,
   694 tokens both ways;
   NCD and CRN keep the split (user choice after correction):
   reserved codes and renamed identifiers live in tracked `forbidden-strings.append.txt`,
   retired or rejected codes in untracked `forbidden-strings.append.local.txt` (ATR,
   VPH);
   NCD checks both.
   Retracted claim:
   Q47 said the local file "holds no codes";
   it holds both retired codes,
   misread from combined search output.
   TAG states round 4 uniqueness scope;
   CRN covers retired codes;
   JCH stays always-loaded until #423 (reopened round 2:
   `tsdoc/check-mutates` cannot detect absent effects);
   meta rules stay in `AGENTS.md` (`writing-for-agents` loads in 25% of pi sessions).
   Approved.
- Retired-code references:
   rewrite every reference in other docs to the successor code (user chose this over a retired-codes list).

## Situational-rule research (2026-09-25)

Read-only subagent research;
working files lived in the session scratchpad.

- Codex truncates project docs at `DEFAULT_PROJECT_DOC_MAX_BYTES = 32 * 1024`
   (`openai/codex` `codex-rs/config/src/config_toml.rs`);
   `~/.codex/config.toml` sets no `project_doc_max_bytes`.
   At 42264 bytes,
   Codex drops the last 63 rules,
   from RCI onward (new packages,
   package completeness,
   verification,
   doc placement,
   commit rules,
   agent skills).
   A 2026-09-07 Codex session shows instructions cut mid-word inside TCV.
- Skill loaded before first edit in its domain
   (sessions editing domain files;
   Bash edits invisible):
   `troubleshooting-doc` pi 97%,
   Claude main 58%;
   `testing-practices` pi 80%,
   Claude main 40%,
   Claude subagents 17%;
   `dum-dum-non-ts` pi 61%,
   Claude main 0 of 6,
   Claude subagents 1 of 26;
   `writing-for-agents` pi 25%.
- Many "situational" rules fired outside their assumed domain
   (GFP,
   CLN,
   HRM,
   RDC,
   MXR,
   RCI,
   PRV,
   TCV,
   XIC,
   QNB,
   GCL);
   user corrections repeat CXD,
   HDM,
   ATS principles in other domains.
   31 of 77 have no citations outside `AGENTS.md`-editing sessions;
   Claude transcripts only reach back to 2026-08-19.
- User precedent for narrow rules:
   package-local docs,
   not skills
   (commits `82c1006bf`,
   `5555179ec`,
   `a28f5d1c7`,
   `3c37169ca`,
   `fbbafc4e8`,
   `d45ec7b54`);
   on 2026-08-14 the user rejected naming the package file `AGENTS.md`.
- Conditional loading:
   Claude Code nested `CLAUDE.md` and `.claude/rules` `paths:` fire on file read,
   not create;
   Codex reads `AGENTS.md` only root-to-cwd at startup;
   pi loads context at startup only;
   Factory loads nested `AGENTS.md` on read.
   Lint and guard messages are the only mechanism firing at the violation in every harness.

## Decisions (round 3)

- Codex cutoff:
   set `project_doc_max_bytes = 65536` in `~/.codex/config.toml` (done 2026-09-25).
   Verified with `codex exec` (codex-cli 0.155.1) asking for the last rule code it sees:
   SK3 with the new limit;
   RCO with `--config project_doc_max_bytes=32768` as positive control.
   Shrinking `AGENTS.md` continues.
- Narrow-rule destinations:
   rules tied to one package or product go to that package's docs;
   narrow rules spanning packages go to skills,
   since the user can invoke skills manually when agents miss them;
   general rules stay always-loaded.

## Decisions (round 4)

- Rules moving to skills or package docs are compressed rule by rule too.
- Adopted without asking:
   moved rules keep their codes,
   since TAG makes codes stable cross-session handles;
   code uniqueness spans `AGENTS.md`,
   skills,
   and package docs.

## Decisions (round 5)

- All 12 skill-bound visual and device rules go to one `visual-design-review` skill.
- Rules must be relatively self-contained:
   no rule depends on another rule's code to make sense.
   Approved PX2 and PXQ cited PX1 and PX3,
   so they were redrafted and need re-approval.

## Decisions (round 6)

- Vague rule text gets deleted rather than guessed at:
   `AGENTS.md` is living,
   so the user re-adds precise wording when an agent misbehaves.

- Proposal code blocks mark sections with Markdown headings,
   not HTML comments.

## Decisions (round 7)

- Keep compact principles that stand in for many lint rules (for example IMM's "Prefer immutable patterns"):
   one clause prevents many lint round-trips,
   which spelling out each lint rule would not.
   Deletion of lint-enforced rules still applies to rules that restate one lint check.

## Open questions


## Approved text

### Batch 1

```md
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
Unbuilt-fix difficulty or duration:
 drop the estimate,
 never label it.

ASK:
 Non-measurable facts:
 ask.
Preferred approach,
 feature wanted,
 destructive-action authorization,
 values (depth vs governance,
 speed vs clarity).

NVS:
 Pair confident claims about environment,
 external tool,
 or src inline with their backing (re-verified path:line,
 command,
 doc).
No backing:
 verify,
 or label as guess.

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

RBK:
 Repo-wide runbooks:
 `doc/runbook/<topic>.md`;
 handovers:
 `doc/handover/<topic>.md`;
 package-specific ones stay beside code.
```

### Batch 2

```md
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

DCK:
 Long sessions:
 after each correction,
 decision,
 answer,
 verification,
 and pre-compaction,
 update canonical docs:
 requirements,
 evidence,
 rejected ideas,
 open questions,
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

PX1:
 Take authorized steps unasked;
 skip "should I...".
Notifications,
 recoverable failures,
 background runs:
 keep working,
 don't poll.
Stop only at completion or genuine blocker.

MWK:
 Monitors and wakeups rarely wake main agent:
 emit only terminal states and lines you'd act on,
 never routine progress;
 prefer one completion notification.

PXQ:
 "Completion" means the queue,
 not the task:
 finished item with tracked work left -> start the next unasked.
Never end a turn on a status report the user must answer with "continue".

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
```

### Batch 3

```md
DVP:
 Target device available:
 probe its settings + limits before web research;
 external sources only explain probe results or fill gaps.

QAB:
 Predicting a change's (or revert's) effect from unchanged code is inference:
 apply it in a fork,
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
 wrong assertion);
 list unexercised surfaces.

QNB:
 Comparing noisy measurements (timings,
 benchmarks,
 model or provider outcomes):
 measure run-to-run spread on unchanged input first;
 smaller differences are noise.

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
 Before asking which mechanism,
 try dissolving the constraint demanding one;
 offer the menu only if it survives.
```

### Batch 4, always-loaded

```md
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

SCF:
 Screenshot after scripted input:
 confirm intended rendered state,
 then capture;
 command completion isn't frame completion.
Recapture stale or transitional frames.

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

VHI:
 Handoffs (visual or doc) state purpose,
 changes,
 what to inspect,
 and how to respond;
 never make unexplained internal labels the user's task.

ATS:
 Custom interactive elements (web,
 Android):
 explicit min 48px/dp layout width + height;
 never rely on touch area expanding past bounds where neighbors can overlap.
```

### Batch 4, `visual-design-review` skill

```md
QVE:
 Start visual review from the accepted design;
 name consequential concerns the user didn't raise and explore them as built variants,
 never a vague approval question.

PFG:
 Minimum padding/spacing is a hard floor:
 test fit there;
 when it fails,
 reflow or truncate permitted content,
 never a below-minimum compact fallback.

QVM:
 Design matrix:
 per-variant pros,
 cons,
 analysis,
 full ranking,
 recommended additions;
 end with separable questions whose answers select among visible new variants.

MXQ:
 Size design matrices by consequential independent dimensions and meaningful variants,
 never by a number the user gave only as an example.

HFM:
 Ask visual-design questions via one self-contained,
 verified HTML form:
 built options,
 pros/cons,
 ranking,
 final free-text field.

RVC:
 After a decision,
 review shows only the active design;
 rejected candidates stay in docs,
 compared only on explicit request.

PXF:
 Screenshot-driven UI:
 measure reference geometry,
 colors,
 spacing,
 states;
 before completion,
 render result side-by-side at matching scale.
Memory isn't evidence.

ZDV:
 Device mockups:
 capture at cited physical px,
 display at 100% of cited dp,
 show px,
 dp,
 current scale + reset;
 never upscale a dp-sized bitmap.

BZF:
 Device frames:
 measured opaque chassis,
 bezels,
 hinge + corners;
 screenshot sits inside the screen opening,
 never clipped to reveal page.

ANB:
 Android mocks:
 show status + navigation bars at current target geometry (screen dimensions include them);
 keep app controls out of cutouts and insets.

AVP:
 Android screen comparisons:
 build nonfunctional Compose prototype,
 install on target emulator,
 capture each candidate at panel px,
 then present in HTML.

M1T:
 Multi-row Material Design 1 tabs:
 each label one content-width line plus horizontal padding;
 wrap whole tabs across rows,
 never text inside a tab.
```

### New rule SLF

```md
SLF:
 Each tagged rule makes sense alone:
 never cite another rule's code or lean on terms only another rule defines.
Citing paths and docs is fine.
```

### Batch 6

```md
OPT:
 Distinct options:
 pros + cons for each,
 then "Ranking:
 B > A > C,
 because ..." giving the reason for every adjacent pair.

OPA:
 `AskUserQuestion`:
 pros + cons in each `description`,
 best first,
 "(Recommended)" on top label;
 full ranking with adjacent-pair reasons in surrounding prose.

YKZ:
 Before ranking several options:
 widen to plausible alternatives (with their libraries and repo incumbents);
 design each until disqualifying problems surface.

ODM:
 Option examples must demonstrate every concept the question asks the user to compare.

OCG:
 CLI option design:
 output cardinality never determines option occurrence grammar;
 sketch token encoding before asserting repeated,
 delimited,
 or variadic forms.
```

### Batch 7

```md
EVL:
 "Should we use X better?":
 before recommending,
 report X's usage + conf,
 parallel systems meeting the same need (with content),
 TODO/workaround comments,
 suppressions,
 and stated policies.

EL4:
 Codebase health signals:
 zero TODO/FIXME/workaround hits mean discipline only if the search provably ran;
 thousands mean debt.
Suppressions with rationale are healthy;
 bare ones are debt.

XIC:
 Similar or concurrent symptoms stay separate incidents until user-visible boundaries match.
Component removal,
 log silence (retention + emitter unverified),
 or later recovery proves no cause or fix.

VKI:
 Synthetic key input:
 nested compositor or caller-independent broker only;
 never `ydotool` from an agent command (key-down can cancel the caller before key-up,
 wedging desktop input).

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
 Doc points elsewhere for substance:
 fetch that before concluding;
 never hedge ("likely contains") about a document one tool call away.

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

RT1:
 `rg` for text search,
 not directory navigation;
 `rg --files` for globs;
 add `--hidden`/`--no-ignore` when dot-dirs or ignored files may hold matches.

RT4:
 `gh` for GitHub issues,
 PRs,
 release notes,
 repository metadata.
```

### Batch 7, moved to single-product docs

```md
JEV:
 Wi-Fi drop debugging:
 reproduce with live link,
 supplicant,
 kernel,
 and reachability capture;
 journal silence or later recovery isn't cause.

FLK:
 One provider "Context limit exceeded" isn't a stable limit:
 measure repeated same-input outcomes before lowering global context budgets;
 prefer model-aware budgeting.
```

### Batch 8

```md
GCL:
 Before doing or reviewing cleanup of ignored files (`git clean -X`,
 artifact deletion):
 list them with `git clean --dry-run -d -X` first.

GC2:
 `git status`,
 `git ls-files --others --exclude-standard`,
 and `rg --files` hide ignored files;
 never use them as cleanup evidence.

GCR:
 Before cleanup,
 check root `HEAD`,
 `config`,
 `hooks`,
 `objects`,
 `refs` (stray git-dir entries) with `ls -d` + `git check-ignore --verbose`;
 any hit makes safe cleanup part of the finding.
```

### Batch 8, moved to `package/git-policy/cli/README.md`

```md
GCW:
 Worktree-guard reviews:
 `DEFAULT_ALLOWED_WORKTREE_DIRS` (`src/allowed-worktree-dirs.ts`) lets git-dirs under allowed dirs bypass the guard.

WXG:
 Worktree-copy incidents:
 first verify main worktrees bypass admin observation,
 recovery,
 settlement,
 and copying;
 then classify the effective source before lock analysis.
```

### Batch 9

```md
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

1CB:
 At most three `&&`-chained steps per shell call;
 no `;` chains or loops.
Longer multi-step work:
 write a scratch `.ts` (`node:child_process`) and run it.

RGP:
 `rg` without a path may read stdin:
 always pass `.` or an absolute path.

ATH:
 Before using `${HOME}/temp/agent` scratch:
 `mkdir --parents` it,
 then `chmod 700`;
 trust checks reject group/other permission bits.

CLN:
 Investigating package source:
 `gh repo clone <repo> "${HOME}/temp/agent/<name>-<date>" -- --depth 1`,
 not `git clone`,
 unless commit history matters.

APQ:
 Auto-push fires in third-party clones too:
 before committing in one,
 run `git remote set-url --push origin DISABLED`.

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

LFF:
 Use long-form (`--flag`) CLI options,
 not short flags;
 writing long form forces knowing what each does.
No long form:
 short flag stays.

RGT:
 `rg` recurses by default;
 its `-r` means `--replace`,
 so grep-reflex `rg -rl`/`-ir` silently rewrites matches in output.
```

### Batch 10

```md
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

CM1:
 Narrow package work:
 run that package's task,
 never reflexive repo-root `mise run test`.

CM2:
 `mise.toml` tasks:
 sequence with `run = ["a", "b"]`,
 never `;` or `:::` chaining;
 `shell = "node --input-type=module-typescript -e"` only for logic.

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

WC2:
 file-enforcer generates root files (`CLAUDE.md`,
 `mise.toml`,
 ...):
 check `file-enforcer.config.ts` before editing root config;
 if managed,
 edit its source,
 run file-enforcer,
 commit output as-is.
```

### Batch 11

```md
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

XRT:
 Prefer cross-runtime patterns over Bun-specific APIs.

HOM:
 Derive current-user paths from injected home or runtime homedir,
 never a hardcoded username or `/home`;
 environment-sensitive tests inject disposable homes.

SCR:
 Never write bash/powershell scripts or `mise.<action>.ts` files;
 put task logic inline via `shell = "node --input-type=module-typescript -e"` or in a package bin.

PIN:
 Pin tool versions only with a comment explaining why.

SPG:
 Automation that spawns agent sessions needs explicit recursion guards (env var flag,
 session type filter,
 transcript size check).
```
### Batch 12

```md
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
```

### Batch 12, moved to `package/rust-linter-plugin/builtin/README.md`

```md
MXR:
 `.rs` files:
 300 code lines max;
 split into sibling modules.
`tests/`,
 `*_tests.rs`,
 `fuzz/`,
 `build.rs` exempt;
 never disable.

RDC:
 Rustdoc (`///`/`//!`;
 plain `//` doesn't count) on every documentable `.rs` item,
 public + private.
cxx-qt files exempt `use` + trait impls;
 tests/fuzz exempt;
 never disable.
```
### Batch 13

```md
LOG:
 Log extensively:
 entry points,
 branch decisions,
 error paths,
 async lifecycle;
 never remove logging to "clean up".

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
```

### Batch 14

```md
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

VA5:
 `satisfies` checks types without widening;
 destructure dependent values in separate statements.
```

### Batch 15

```md
PP1:
 `async`/`await` only:
 no promise chains or `new Promise`.

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

RG2:
 Code replacing a regex makes one linear pass (O(n) time,
 O(1) stack),
 proven O(n) for unbounded input.
```

### EDR (late addition, placed after NXR)

```md
EDR:
 Parallel tool calls may run in any order,
 so a command reading a fresh edit can see the pre-edit file;
 send dependent commands only after the edit returns.
```

### Batch 16

```md
TP1:
 Third-party APIs and CLIs:
 read the installed type definitions before calling;
 on an undefined-method error,
 fetch current docs immediately;
 test the simplest invocation first.

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
```

### Batch 16, moved

RCO goes to the `choosing-technology` skill's "Replacement parity overlay";
AP5 goes to `package/config/rolldown/README.md`.

```md
RCO:
 Removing an incumbent:
 ledger every consumed responsibility with its owner,
 selection status,
 parity test,
 and retired behavior;
 recommend removal only when every entry has a viable owner.

AP5:
 Client-side bundling packages:
 add `rolldown.client.config.ts` extending `@monochromatic-dev/config-rolldown/.client.ts`,
 a `build:js:client` task,
 and this package as a devDependency.
```

### Batch 17

```md
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

VUB:
 After building,
 deploying,
 or installing,
 verify the artifact the way its consumer uses it;
 compiling or installing alone isn't verification.

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

ABR:
 End browser verification with `agent-browser close`:
 open pages keep animating,
 and WebAudio unlocked by scripted clicks plays on system speakers.

VB7:
 Markdown ships only after a rendered check (live page or renderer output);
 lint misses CommonMark emphasis edge cases.

URF:
 Verification needing a user-provided resource runs first,
 before other work or other parts of the task;
 scope growth never defers it;
 not done until the resource is exercised.

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
```

### Batch 18

```md
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

EC1:
 Worktree changes you did not make are concurrent work,
 not emergencies:
 never restore,
 stash,
 or revert them;
 touch only task files.
Unrelated change blocks your edit:
 say so and ask.
```

### Batch 19

```md
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
```

#### Commit message measurements (last 3000 non-merge commits, 2026-09-29)

- Every subject matches `<type>(<scope>): <subject>`;
   one lacks a scope;
   none uses `!` or `BREAKING CHANGE`;
   none is capitalized or ends with a period.
- Types:
   `docs` 2016,
   `test` 364,
   `fix` 280,
   `feat` 122,
   others under 60.
- 1115 subjects use scope `*`:
   1034 touch only repo-level paths (`doc/planning`,
   `doc/troubleshooting`,
   `doc/handover`,
   `doc/audit`,
   `AGENTS.md`),
   50 one package plus repo-level paths,
   31 several packages;
   720 have no body.
- 49 multi-package commits use a non-`*` scope.
- 48 bodies carry per-package typed lines (GCB format).
- No git-policy check validates commit messages;
   `staged-changes-ignored` covers only pathless `--amend`/`--allow-empty`,
   so CPN is unenforced.

#### SK1 to SK3 consumers (checked 2026-09-29)

- The synced `setup-matt-pocock-skills` skill finds the block
  by its `## Agent skills` heading
  and writes `### Issue tracker`-style subheadings;
   no skill matches the bold labels,
   so they are cosmetic.
- Consumer skills (`code-review`,
   `to-spec`,
   `wayfinder`) expect the tracker info in context;
   the synced `code-review` skill checks `docs/agents/issue-tracker.md`,
   which does not exist here (moved on 2026-07-15 to `doc/agent/issue-tracker.md`),
   so SK1's path pointer is what redirects agents.

### Batch 20

Placement:
DGT and DNL move to "Before editing code" under a "Logging and diagnostics" heading;
JCH moves to "TSDoc comments".

```md
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

JCH:
 Never write `@mutates` for absent effects:
 move work to an ownership-known boundary,
 pass its primitive result,
 or improve the proof;
 contracts describe possible runtime effects,
 not analyzer gaps.

EPR:
 Research ecosystem precedent before offering naming or technology options it could inform.
"Think in X":
 list X's features,
 ask which to omit.
```

## Next action

Applied 2026-09-29 after user confirmation;
only the #423-gated JCH follow-up remains (issue comment posted on #423).

### Apply log

- d03ec673e:
   `AGENTS.md` rewritten from approved text,
   `CLAUDE.md` regenerated (12930 tokens,
   31214 bytes).
- 6deb6cc12:
   `visual-design-review` skill (12 rules);
   Claude Code lists it.
- 37117e363:
   JEV,
   FLK,
   GCW,
   WXG,
   MXR,
   RDC,
   AP5 into package docs;
   RCO into `choosing-technology`.
- 718e8663e:
   retired-code references point at successors or enforcing lint rules;
   living Rust comments cite MXL,
   MXR,
   RDC without a file.
   Dated research snapshots citing old `AGENTS.md` line numbers stay as historical records.
- Untracked `forbidden-strings.append.local.txt` entry `local-039` reserves the 53 retired codes;
   verified by scanning a `PP9:` probe (flagged) and a `PP8:` probe (passed).
- 5dadad6c7:
   philosophy doc removal record,
   magic-number section,
   `handler/` path;
   regression-suite Case 3 matches GCR and GCL.
- 21dfa9bec:
   `no-regex` constructor message prefers index scans,
   parsers,
   string APIs;
   unit tests,
   types,
   and lint pass;
   root oxlint shows the new message.
- Verified after apply:
   `AGENTS.md` lints clean and renders 204 rule paragraphs;
   `codex exec --config project_doc_max_bytes=32768` reports SK3 as the last rule code (RCO before the rewrite).
- Not touched:
   `package/music-player/design/HANDOFF.md` still says `AGENTS.md` rule `PFG` (another session's active file).

### Walk totals

Rule text only (no headings),
Opus 5.5 tokens via `mise run //package/llm/token-count:count`:
276 rules and 17192 tokens become 204 always-loaded rules and 12488 tokens.
73 codes leave `AGENTS.md` (merged,
deleted,
or moved);
SLF is the only new code;
no duplicate codes.

### Whole-file draft (assembled in scratch)

Assembled from current headings plus approved text with every recorded move:
`AGENTS.md` 17618 tokens and 42677 bytes
(grown from the 17431-token baseline by concurrent additions)
become 12927 tokens and 31216 bytes,
a 27% cut;
31216 bytes fits Codex's default 32768-byte `project_doc_max_bytes` with 1552 to spare.
The audit's 43.8% estimate assumed dropping clause-per-line breaks (1913 tokens,
kept in round 2) and moving most situational rules to skills (research kept most as general).

Placements chosen during assembly (no earlier record):
WC2 to "Cross-runtime and scripts",
VKI to "Hazardous commands",
VHI to "Communication style",
ATS with CXD and HDM under "User interfaces"
(ATS moved on 2026-10-05 to the "Target size" section of `.agents/skills/visual-design-review/SKILL.md`,
by user decision),
which sits before "TSDoc comments".

```text
# Development guidelines for AI agents
  ORG TAG RLM SLF NCD CRN APG
## Before responding to the user
### Communication style
  HON SYS WKP DCK 1ST SRC EXT WRN GAP EPR VHI
### Proactivity calibration
  PX1 MWK PXQ PX2 PX3 TSK
### Pre-response checklist
  CK9 CKB XIC
### Measure-vs-ask
  QF1 QJ1 DVP QAB QPC QIV QNB ASK QGR QCS QSP QPM
### Present options with pros, cons, and a personal ranking
  OPT OPA YKZ ODM
### Exhaust evidence layers when assessing system usage
  EVL EL4
### Before claiming inability
  CB1 RXH CB2 RPB FCH
### Name the verification step
  NVS QRY
### Git cleanup and worktree safety reviews
  GCL GCR GC2
### Research tools
  RT1 RT4
## Before running a command
### Command execution conventions
  TMO NXR EDR 1CB RGP ATH CLN APQ BOP WCD CLH
### Long-form flags
  LFF RGT
### Hazardous commands
  HRM RXI BOX DCB VKI
### Essential commands
  CM1 CM3 CM5 CM6
## Before editing code
### Match action scope to the request verb
  VRB DRR IWT AUT VR2 ANN EC4
### Cross-runtime and scripts
  XRT HOM SCR PIN SPG CM2 WC2
### Simplification
  IMM UTL XNC ITR MXL
### Linting
  LN1 LN2 LN3 LN6 LN7
### Logging and diagnostics
  LOG TLG LG1 LG2 DGT DNL
### Security
  SYB STB PRV
### User interfaces
  CXD HDM ATS
### TSDoc comments
  TSD TD1 TD2 TD4 TD5 TD6 TD7 JCH
### TypeScript
#### Standards
  ST2 ST3 ST5 ST6 ST8 ST9 TQ2 TQ3 XPT
#### Type system
  TY2 TY3 TY5 TY6 TY7 TY8 TY9
#### Variables and values
  VA5
#### Programming patterns
  PP1 PP2 PP4 PP5 PP6 PP8 PPX
#### Regular expressions
  RG2
### Third-party libraries
  TP1
### Dependency management
  DM1 LFW RCI
### Adding new packages
  AP1 AP2 AP4 SGD SBS
## Before declaring work complete
### Package completeness
  PKG TCV GFP CXL
### Verify at the user boundary
  VUB VB1 VB5 VB7 SCF ABR URF
### Verify on a throwaway, not against real state
  THR TAE
## When committing or documenting
### Documentation standards
#### Prose style
  WR2 WR3 WR4 WR5
#### Markdown syntax
  MD1 MD2 MD3 MD4 MD5 MD6 MD8 WRP
### Doc placement
  DPL DL1 DL3 DL4 DL5 RBK
### Handling external changes
  EC1
### Git commit guidelines
  GCE GCG GCB GCA CLG CPN XCM
## Architecture decisions
  AD1 AD2 AD3 AD4 OCG
## Agent skills
  SK1 SK2 SK3
```

### Concurrent `AGENTS.md` changes

Other sessions edited `AGENTS.md` during the walk (`git diff d38e8e6ca HEAD -- AGENTS.md`,
checked 2026-09-29):

- 0879faf1f added EDR to "Command execution conventions" after batch 9 was approved;
   approved with batch 15.
- b9c0e6e48 added a fuzz-sidecar sentence to AP1.
- 654507bf9 added SBS under "Adding new packages";
   it joins batch 16.

### Apply phase (after the walk)

- Before rewriting,
   rerun `git diff d38e8e6ca HEAD -- AGENTS.md`;
   review every concurrent change not yet covered by approved text.
- Rewrite `AGENTS.md` from approved text,
   including section moves and new "User interfaces" heading;
   regenerate `CLAUDE.md` via file-enforcer.
- Create `.agents/skills/visual-design-review/SKILL.md` with an imperative description.
- Add moved rules to their homes:
   JEV to `doc/troubleshooting/rtw89-wifi-disconnects.md`,
   FLK to `package/pi-plugin/advisor/README.md`,
   GCW and WXG to `package/git-policy/cli/README.md`,
   MXR and RDC to `package/rust-linter-plugin/builtin/README.md`,
   RCO to the `choosing-technology` skill's "Replacement parity overlay",
   AP5 to `package/config/rolldown/README.md`.
- Rewrite references to retired codes;
   add forbidden-strings entries for retired codes.
- Update `doc/philosophy/agents.md` and `doc/agent/regression-suite.md` Case 3.
- Follow-up after #423 enables `no-invalid-parameter-effect-contracts`:
   move JCH guidance into its `staleMutatesTag` message and delete JCH
   (`tsdoc/check-mutates` is syntax-only and cannot carry it).
- Add "Prefer an index scan,
   parser,
   or string API" to the `no-regex` `regexpConstructor` message
   (`package/oxlint-plugin/no-restricted-syntax/src/rule/no-regex.ts`),
   approved with batch 15.

### Resume notes

Walk scratch files live in the session scratchpad `walk/` directory:
`bN-after.md` holds proposed text;
`measure.ts <batch> <CODES>` extracts originals to `bN-before.md`,
checks RLM,
and counts Opus 5.5 tokens.
Scratch is not durable;
this doc's approved-text sections are canonical.
Proposal code blocks mark sections with Markdown headings.

## Voyage sorting fit assessment (2026-10-01)

The user asked whether to use Voyage `rerank-3` to sort `AGENTS.md`.
This is a capability lookup and provisional proposal,
not adoption or a completed vendor selection.
No instruction file or integration changed;
no Voyage request was made for this assessment.

### Evidence and existing boundaries

- `AGENTS.md:3` defines organization by moment of decision.
  The adopted section-move policy in "Adopted without asking" preserves that organization.
- `file-enforcer.config.ts:2211` to `:2245` embeds `AGENTS.md` into generated `CLAUDE.md`.
  Any eventual canonical reorganization affects both consumers.
- The [reranker guide](https://docs.voyageai.com/docs/reranker) defines query-relative document relevance.
  The API model identifier is `rerank-3`.
- The [release announcement](https://blog.voyageai.com/2026/09/30/rerank-3/)
  confirms instruction-following support,
  so a supplied sorting criterion is possible.
  Its retrieval evaluations do not measure adherence to this repository's instructions.
- Existing Voyage use includes multimodal image embeddings in `package/llm/image-diff/src/voyage.ts`
  and separate guard-model experiments documented in `pi-auto-mode-voyage-fit.md`.
  Neither establishes whether sorting these rules improves adherence.
  The image provider's type-assertion suppression concerns API response types,
  not instruction ordering.
- "Decisions" prioritizes token cost with full rule coverage,
  then adherence and maintainability,
  and declines a standing formal adherence check.
  Reordering alone is not evidence of reduced token cost.

### Provisional proposal and limits

Keep the current lifecycle layout as the baseline.
A human-reviewed canonical reorder and a task-specific runtime reorder are distinct experiments.
Voyage could produce suggestions under an explicit criterion;
its relevance score is not instruction authority,
importance,
or precedence.
No comparative ordering or adherence experiment was run here.

Any later experiment would preserve complete rule blocks,
codes,
wording,
headings,
and lifecycle context,
without filtering or silent truncation.
Ordering and highlighting would be evaluated separately.
Runtime ordering also introduces task-context disclosure and provider-failure questions.
A one-off comparison could inspect representative task outcomes and rule violations;
it would not silently introduce the standing regression suite already declined.

The open question before concrete adoption is the intended sorting criterion:
canonical lifecycle organization or relevance to the current task.
Until that is settled and the proposed behavior is exercised,
there is no recommendation to deploy Voyage as the instruction sorter.

### Verification

The assessment section passed the repository's scoped Markdown stdin check
and rendered with Sätteri's installed `markdownToHtml` API.
Full-file lint identified an unchanged semantic-line-breaks finding in "Concurrent `AGENTS.md` changes";
its prose line was wrapped without changing content.
