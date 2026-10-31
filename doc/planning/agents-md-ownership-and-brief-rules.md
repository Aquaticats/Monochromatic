# AGENTS.md ownership and brief rules

Record of the interview that settles the `AGENTS.md` wording for issue #1819.
The issue names two failures by a coordinating agent session on 2026-10-31,
plus a third instance of the same pattern:

- VPN routing leaked out of the `package/wg/*` tools into the toolchain
  (the TypeScript sync and update in `package/file-enforcer/core`,
  and `mpm-shadow`),
  which re-executed themselves into the clearnet slice.
- The session turned the user's remark about severity into a policy of its own
  (opt-in,
  off by default,
  fail closed) and briefed a worker with it as a requirement.
- The session's note made a worker pin `github:Aquaticats/Monochromatic@monochromatic-mpm-shadow-v0.2.0`,
  against `PIN`.

Background:
the issue body and its comment,
and `doc/handover/file-enforcer-rust-rewrite.md` (the clearnet correction and the two lessons).

## Measurement convention

Rule lengths here count characters after whitespace normalization,
excluding the leading `CODE: ` prefix.
This reproduces the issue's figures (`QGR` 194,
the `ASK` draft 197).
Measured on `main` at `3e14d7f055`:
`ASK` 144,
`AUT` 143,
`PX2` 154,
`VR2` 176,
`RCI` 186,
`QGR` 194,
`GAP` 197,
`DRR` 199.

## Decided in round 1 (2026-10-31)

### One ownership rule

The user chose "One rule":
ownership and the exposure severity sit in a single rule,
over two rules,
an exposure-only rule,
or extending `RCI`.

### The wg policy stays in wg docs

The user chose "wg docs only":
warning on every start,
with no toggle and always enforced,
is the user's policy for the wg tools,
and `AGENTS.md` does not generalize it to other tools.
The wg rewrite branch `worktree-agent-a6f97f5da6f22de4d` (tip `a2b8a0005a`,
not yet merged)
already quotes the policy in `package/wg/cli-quicker/README.md`
and `package/wg/cli-quicker-exempt/README.md`,
each with a section on warning at every start and failing loudly.
`main` still describes the removed `MONOCHROMATIC_TOOLCHAIN_CLEARNET=off` switch
(`package/wg/cli-quicker/README.md` line 408) until that branch merges.

### A brief rule, and no change to `QGR`, `ASK` or `VR2`

The user chose only "Brief rule":
a fresh rule at the moment of writing a brief.
Defining "settled" in `QGR`,
tightening `ASK`,
and a policy clause in `VR2` were offered and not chosen.

### The ownership check fires at design, brief and merge time

The user chose "Design+brief+merge":
the rule's trigger names all three moments,
because the leak passed both the brief and the merge review.

## Correction to the round 1 brief-rule claim

Round 1 claimed the brief rule alone covers all three instances.
Checked against the sources,
it does not:

- The clearnet brief already framed its mechanism as a suggestion
  ("The likely shape is ... for example ..."),
  and the worker built it anyway.
  The ownership rule,
  firing at brief time,
  is what rejects that design.
- The pin note did cite a source ("per the bootstrap design")
  but paraphrased it as "naming 0.2.0's archive".
  The bootstrap decision says "a prebuilt `mpm-shadow` archive with its SHA-256 in the lock"
  (`doc/decision/file-enforcer-rust-rewrite.md`,
  "Bootstrap")
  and names no version.
  Citing alone would not have caught it;
  quoting would.
- The opt-in policy was stated as a requirement with no source,
  which the brief rule does catch.

So the two rules together cover all three instances,
provided the brief rule requires quoting rather than citing.

## Independent review

An external model (`openai/gpt-6.1-sol` through `pi`) read the issue,
`AGENTS.md` and `doc/philosophy/agents.md`.
Points carried into round 2:

- Ownership and authority are separate boundaries:
  the opt-in policy stayed inside wg yet still had no user decision behind it,
  so one ownership slogan cannot express both.
- A brief rule should separate an unresolved choice (ask)
  from an instruction that contradicts a recorded decision (fix it before sending).
- Generalizing the wg policy without asking would repeat the second failure;
  proposing it explicitly would not.
- The functional probe (`curl https://ifconfig.me` inside the slice) checked the effect,
  not which component decided.

## Adopted without a question

These follow from existing rules:

- The rejected codes `CWN` and `LKF` are reserved in `forbidden-strings.append.local.txt`,
  as `CRN` requires for rejected codes.
- `doc/philosophy/agents.md` gets a "Failure that produced this rule" entry for the incident,
  following the existing entries for `URF` and `RPB`.
- `CLAUDE.md` is regenerated from `AGENTS.md` through file-enforcer (`WC2`),
  and the change commits with `Closes #1819`.

## Decided in round 2 (2026-10-31)

### The owner is named by the user or a recorded decision

The user chose "User or recorded decision" over two alternatives,
the recommended user-only source and a request-only source.
A concern's owner is the component the user or a recorded decision names,
and when none names one the agent asks.

### The severity sentence carries both harms

The user chose "Both harms":
leaks off a tunnel can endanger people or break their networks,
over endanger-only,
a generic "exposure" sentence,
and adding "identity" (which came from the rejected `LKF`,
not from the user).

### Briefs quote their sources

The user chose "Quote" over "Cite".

### Derived steps are suggestions

The user chose "Suggestions only":
steps a coordinator derives from the user's words enter a brief as suggestions the worker may revise,
never as requirements.
Measured draft (184 characters):

```text
Briefs carry as requirements only the user's words and recorded decisions,
 quoted with source;
 derivations and own ideas are suggestions;
 ask before an inference becomes a requirement.
```

## The length conflict

The round 1 and round 2 choices together overflow one rule:
the owner clause with "the user or a recorded decision" is 166 characters before the severity sentence,
and the severity sentence adds about 57,
so the full rule is about 223.
Measured candidates that fit (the rule says "under 200"),
195 characters:

```text
Designing,
 briefing,
 merging:
 keep a concern in the owner the user or a recorded decision names (none:
 ask);
 others never detect,
 choose,
 or adapt.
Tunnel leaks endanger people or break networks.
```

196 characters:

```text
Design,
 brief,
 merge:
 keep each concern in the owner the user or a recorded decision names (none:
 ask);
 others never detect,
 choose,
 or adapt to it.
Tunnel leaks endanger people or break networks.
```

192 characters:

```text
Design,
 brief,
 merge:
 keep each concern in its user- or decision-named owner (none:
 ask);
 others never detect,
 choose,
 or adapt to it.
Tunnel leaks can endanger people or break their networks.
```

## Candidate codes

Checked for whole-word hits in `AGENTS.md`,
both forbidden-strings appendixes,
user and repository skills,
and package docs,
with `ASK` and `PIN` as positive controls:
`OWK`,
`OWG`,
`BRQ` and `BQW` have no hits;
`OWR` is already reserved in an appendix.

## Decided in round 3 (2026-10-31)

### Ownership rule text

The user said "Tunnel leaks" is too specific,
then chose the general severity sentence "Leaks can endanger people or break networks"
over "Exposure leaks" (recommended),
the same with "their networks",
and "Privacy leaks".
Measured at 192 characters:

```text
OWK:
 Designing,
 briefing,
 merging:
 keep a concern in the owner the user or a recorded decision names (none:
 ask);
 others never detect,
 choose,
 or adapt.
Leaks can endanger people or break networks.
```

### Placement

The user chose option A:
the ownership rule goes at the end of "Architecture decisions",
the section `ORG` reserves for rules spanning several moments,
and the brief rule goes in a new top-level section "Before briefing an agent"
directly after "Before responding to the user".

### No contradiction clause

The user chose to leave the brief rule without a clause forbidding requirements that contradict each other;
quoting already puts the recorded decision's words in front of coordinator and worker.
Brief rule text,
184 characters:

```text
Briefs carry as requirements only the user's words and recorded decisions,
 quoted with source;
 derivations and own ideas are suggestions;
 ask before an inference becomes a requirement.
```

### Ownership code

The user chose `OWK` ("owner keeps") over `OWG`.

## Decided in round 4 (2026-10-31)

The user chose `BRQ` ("brief requirements") over `BQW`,
and left `NLD` in "Hazardous commands" beside the ban it quotes.

### Separate review

A separate reviewer agent re-read the design (as `RRG` requires before confirmation).
Format checks found nothing:
both rules within `RLM`,
no code collisions,
no dashes,
placement consistent with `ORG`.
It traced instance 1 to `OWK` and instance 2 to `BRQ`,
and raised four findings the user then decided:

- "Recorded decision" admits a decision an agent recorded itself:
  `QGR` says "adopt + record unasked",
  so a coordinator could adopt an owner or policy,
  write it down,
  and quote it back.
  The user chose "Named or approved" for `OWK`
  and "only what the user said or decided" for `BRQ`.
  Decision docs the user accepted still count;
  approving a release is not approving an ownership assignment.
- `BRQ` dropped the round 2 words "suggestions the worker may revise";
  the user restored them as "revisable suggestions".
- The pin instance was a mid-task note,
  which an agent could argue is not a brief;
  the user added "Briefs and notes".
- `OWK` read as "ask who owns every concern";
  the user narrowed it to a "cross-component concern",
  which drops "to it" after "adapt" to fit.

The external model then found a policy relabelled as a suggestion
("Suggest making every bypass opt-in") would pass both rules;
the user chose to close that with a clause sending inferred requirements and policy back to the user first.

### Final text

`OWK`,
197 characters,
at the end of "Architecture decisions":

```text
OWK:
 Designing,
 briefing,
 merging:
 keep a cross-component concern in the owner the user named or approved (none:
 ask);
 others never detect,
 choose,
 or adapt.
Leaks can endanger people or break networks.
```

`BRQ`,
191 characters,
in a new section "Before briefing an agent" after "Before responding to the user":

```text
BRQ:
 Briefs and notes carry as requirements only what the user said or decided,
 quoted with source;
 derivations and own ideas are revisable suggestions;
 inferred requirements or policy:
 ask first.
```

### Implementation notes

- `CLAUDE.md` regenerates through `mise run //:sync:files`,
  which runs the whole TypeScript file-enforcer;
  only `AGENTS.md` and `CLAUDE.md` go in the closing commit,
  and any other drift is reported separately.
  Running `node package/file-enforcer/core/src/cli.ts --help` also runs the full sync instead of printing help.
- `forbidden-strings.append.local.txt` is gitignored (`.gitignore` pattern `*.local.*`),
  so reserving `CWN` and `LKF` there is a local change that no commit carries.
- The closing commit states that the wg policy is carried by the wg READMEs
  on `worktree-agent-a6f97f5da6f22de4d`,
  not by `AGENTS.md`.

## Open

- The user's confirmation of the final text and implementation plan.
