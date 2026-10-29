---
name: troubleshooting-doc
description: >-
  Use when investigating an external tool's behavior, bug, quirk, capability gap, or fix difficulty;
  proactively write the troubleshooting doc the moment you finish diagnosing or working around one, even
  when the user did not ask; also use when writing or updating a doc/troubleshooting/<topic>.md file.
  The write-up is a required completion step, not an offer.
---

# Writing a troubleshooting file

Fires automatically,
 no user request needed,
 when you investigate an
external tool's behavior,
 bug,
 quirk,
 capability gap,
 or fix difficulty.
The source-clone and clone-boundary sections apply during investigation.
It also fires the moment you finish diagnosing or land a workaround for
an external tool's bug,
 quirk,
 surprising behavior,
 or capability gap.
Writing the doc is a required completion step,
 not something to offer or
defer:
 do not declare the work done,
 and do not say "I could document
this" or "want me to write it up?
",
 until doc/troubleshooting/<topic>.
md
exists.
 Walk this skill end-to-end whenever you write or update one.

Other surface phrases that should trigger the skill:
"document this",
 "write it up",
 "add a troubleshooting entry".
But the primary trigger is self-initiated:
 you just fixed or explained an
external tool behaving unexpectedly,
 so write it up now without being asked.

The skill encodes the required sections,
 the source-trace rule,
source-clone investigation rule,
 third-party clone boundary,
out-of-scope upstream-filing check,
and the 6-constraint upstream-filing check that gates the draft GitHub
issue at the end.

A troubleshooting file is the durable artifact of investigating an
external tool.
 Future sessions and external readers must be able to
reproduce,
 verify,
 and act on every claim.
 The canonical worked example
is [doc/troubleshooting/resharp.md](../../../doc/troubleshooting/resharp.md);
match its shape unless the topic genuinely lacks a section.

## File naming

`doc/troubleshooting/<topic>.md`.
 `<topic>` is kebab-case,
specific enough to distinguish from sibling docs (`bun-fetch-streaming`,
not `bun`).
 One bug or one cluster of related bugs per file.

## External source investigation

When investigating an external tool's behavior,
 bug,
 quirk,
capability gap,
 or fix difficulty,
 clone the tool's source and read the
relevant code path.
 "No public diagnosis exists" is not a stopping
point when the source is open.
 Cite file path,
 line number,
 and code
excerpt whenever you present a source finding.

This applies whether you reproduced the bug yourself,
 are summarizing an
undiagnosed tracker report,
 or are estimating whether a fix is possible.

## Third-party clone boundaries

Never modify files in cloned repositories we do not own for local
workarounds.
 "Third-party" is determined by repository ownership,
 not
origin URL:
 a fork under our git user's GitHub namespace is our code and
may be edited,
 including to prepare a pull request.

For repos we do not own,
 use configuration,
 environment variables,
wrapper scripts,
 or consumer-side code at our boundary.
 The only
unforked-clone edit path this skill permits is the disposable prototype
clone in "Auto-prototype when constraints 1-5 hold or sorta-hold".

## Required sections

1. **Title (`#`)**:
    one-line problem statement naming the tool,
    the
   version,
    the surface trigger,
    and the resulting failure mode.
    See
   the resharp doc's title for the shape.
2. **Symptom**:
    what the user sees.
    Quote error strings verbatim,
    list
   the surface-syntax patterns that trigger them,
    and note which error
   variant each pattern produces when more than one exists.
3. **Root cause**:
    the call chain that produces the failure,
    walked
   step by step.
    Every claim about source code cites
   `path/to/file.ext:LINE` and quotes the relevant code excerpt in a
   fenced block.
    Asserting "the parser rewrites X" without showing the
   rewrite is not allowed;
    this is the doc-writing specialization of
   `AGENTS.md`'s broader source-citation rule.
    Link every citation of a source this repository does not own to the
   selection itself:
    a web page gets a `#:~:text=` text fragment,
    and a file in a hosted repository gets the platform's selection
   permalink (on GitHub, a commit-pinned URL with a line range, for
   example `blob/<commit>/path/file.ext#L10-L20`).
    Load each link in `agent-browser` and confirm it lands on the quoted
   text before it goes into the doc or into an issue draft.
    Files in this repository keep repo-relative paths.
    If a previous reading or
   hypothesis was wrong,
    name it explicitly and quote the evidence
   that disproved it,
    so the next investigator does not re-derive the
   bad cause (the resharp doc's "earlier alternation-count reading was
   wrong" paragraph is the shape).
4. **Verification**:
    version under test (with crates.
   io checksum,
   commit hash,
    or release tag),
    a runnable harness (shell invocation
   of the affected binary,
    a minimal source-level program in the
   target tool's language,
    or both when each surfaces different
   information),
    and at least two catalogs:
    patterns that work
   cleanly and patterns that fail.
    When multiple error variants exist,
   split the failing catalog by variant so each failure mode is
   enumerated.
5. **Verified workarounds**:
    each workaround is a runnable patch with
   its tradeoffs named (what semantics shift,
    what edge cases slip
   through).
    A workaround whose tradeoffs you have not stated is not
   verified.
6. **What does not work**:
    approaches you tried and rejected,
    with the
   reason each failed.
    Saves the next investigator from re-discovering
   dead ends.
7. **Upstream filing artifact**:
    a new-issue draft,
    an additive-comment
   draft,
    or an explicit "nothing to add" note,
    gated by the 6-constraint
   check below and the duplicate search.

## The 6-constraint upstream-filing check

Before filing upstream,
 all six must hold.
 Write an "Upstream filing
decision" subsection that walks each constraint explicitly,
 whatever the
outcome;
 the audit trail is the point.
 The neutral heading is deliberate:
it holds whether the decision is to file,
 not file,
 comment on a
duplicate,
 or post nothing.
 Default policy:
 do not file.
 Every reported
issue that does not satisfy all six is treated as a publicity incident.

1. **Is it really upstream's fault?
   ** Distinguish behavior from
   wording from architectural restriction.
2. **Can upstream fix it?
   ** This fails only when upstream genuinely
   cannot:
    an architectural or algebraic-core limitation that makes the
   fix impossible.
    A fix that is large,
    structural,
    or spread across
   several files still meets this constraint.
    Trace the depth to scope
   the prototype,
    not to decide pass/fail.
3. **Are they supporting this use case?
   ** Look for docs,
    examples,
   tests,
    or stated value propositions covering the combination.
4. **Would the repo welcome our contribution?
   ** Check `CONTRIBUTING.md`,
   issue templates,
    pull request templates,
    README,
    maintainer docs,
   repository policies,
    and recent maintainer responses for signals
   about external contributions and AI-assisted reports.
    This fails
   when the repo says not to file issues or patches from outside
   contributors,
    requires a fully human-authored filing we cannot
   truthfully provide,
    bans AI-assisted or heavily-AI-assisted issue
   filings,
    or maintainers have rejected comparable AI-assisted
   contributions.
    Absence of a policy is not a fail:
    cite the files and
   tracker searches checked,
    and say no ban was found.
    When the policy
   allows AI assistance only with disclosure,
    the draft must disclose
   assistance and name the reproduction,
    source trace,
    prototype,
    and
   workaround checks a human verified.
5. **Will they likely fix it?
   ** This fails only when upstream is
   actively leaning no,
    shown by their docs (a documented won't-fix or
   stated non-goal) or direct communication (a maintainer declining a
   comparable request).
    Absence of signal is not a fail:
    no existing
   issue,
    no recent commits in the path,
    or silence all still meet this
   constraint.
    Check the upstream tracker,
    commit history,
    and release
   deltas to inform the draft,
    not to gate it;
    cite what you find.
6. **Have we prototyped a minimal fix compatible with their
   architecture?
   ** "Minimal" means the smallest set of changes that
   actually solves the issue;
    it does not constrain the fix's
   complexity.
    The minimal set may span several files or be intricate.
   Speculative "suggested fix" prose without code,
    a correctness
   argument,
    or tests against a nontrivial set does not count.

### Candidate-fix applicability gate

Before applying the auto-prototype requirement to a known upstream issue,
verify candidate fix is absent from installed source,
and candidate issue's version,
signal/stack,
and inputs match observed failure.
A reproduction that fails only on pre-fix source proves upstream history,
not current incident.
Treat it as an excluded hypothesis:
record it in "What does not work",
do not create or keep its patch,
and collect missing incident evidence instead.

### Auto-prototype when constraints 1-5 hold or sorta-hold

When constraints 1-5 are all "yes" or "yes-with-a-soft-yes" (e.g. #5
reads "plausible" or "likely" rather than a definitive yes),
 do not
stop at "constraint 6:
 not yet" and report the gate failed.
 Prototype
the minimal fix yourself before declaring the audit done,
 working either
in a fork under our own account (our code,
 not a third-party clone;
prefer this when the prototype is headed for a pull request) or in a
disposable upstream clone.
 The disposable-clone path:

1. Before first use,
    ensure current scratch root exists with private permissions:
   `mkdir --parents "${HOME}/temp/agent"; chmod 700 "${HOME}/temp/agent"`.
   Clone upstream source into a fresh,
    private,
    unpredictable directory created with
   `mktemp --directory "${HOME}/temp/agent/upstream-prototype.XXXXXXXX"`
   (or an equivalently private throwaway workspace under `~/temp/agent/`).
   Never reuse an existing `~/temp/agent/<repo>` clone or any other
   pre-existing directory for this step.
2. Confirm the clone's `origin` URL and checked-out commit/tag match the
   upstream source cited in the doc before editing.
3. Apply the minimal set of changes that solves the cause identified in
   "Root cause":
    the smallest complete fix,
    not the simplest-looking
   one.
    Do not pad it with unrelated cleanup,
    and do not shrink it below
   what actually solves the issue because it looks large or spans
   several files.
4. Verify with the least-trusting harness that proves the change:
    prefer
   a targeted minimal program or existing reproduction harness over the
   upstream package's full test/build scripts.
    Run upstream scripts only
   inside a secret-free sandbox or container with no ambient credentials
   and no write access to this repository.
    Whatever you pick must surface
   the failure pre-patch and the success post-patch;
    "it compiles" is
   not verification.
5. Record the result,
    the verification command,
    and the verification
   output inline in the doc.
    For the diff itself:
    if it is small
   (single hunk,
    roughly under 20 lines including context),
    embed it
   inline in a fenced `diff` block.
    If it is larger (multiple hunks,
   multiple files,
    or long enough that inline embedding would crowd
   the doc),
    save it beside the doc as
   `doc/troubleshooting/<topic>.patch` (matching the doc's `<topic>`,
   prefix dropped since the directory names the family) and link to it
   from the doc with a relative path (the bare filename,
    same directory).
    The inline-vs-file
   threshold is about readability,
    not significance;
    either form
   satisfies constraint 6 as long as the patch is reproducible from
   what is recorded.
    With this recording in place,
    the audit ends
   with all six constraints "yes" and the draft becomes fileable.

The trigger is "constraints 1-5 hold or sorta-hold";
 not "the fix is
already known to be small" and not "we are sure upstream will accept
this.
" A large or multi-file minimal set is still in scope:
 prototype
it.
 Sometimes the minimal fix is a one-line terminfo or config-table
change that costs less than the audit itself,
 but size is never the
gate,
 neither to skip the prototype nor to require it.

If prototyping reveals the change breaks the architectural core or
fails the tool's existing tests in unrelated places,
 re-evaluate
constraints 2 and 5 with the new evidence and revise the audit.
 A fix
that is merely large or intricate is not grounds to re-evaluate;
 only
an architectural blocker is.
 The prototype is also a probe;
 a failed
probe is useful data,
 not a wasted step.

The third-party clone boundary above still applies to local workarounds,
where editing source bypasses the intended boundary.
 Two paths are open
for the prototype.
 A fork under our own account is our code,
 not a
third-party repository,
 so editing it is unrestricted;
 prefer the fork
when the prototype is headed for a pull request.
 Otherwise this skill
permits one narrow exception for an unforked upstream clone:
 the
disposable prototype clone described here,
 created fresh for the upstream
patch diff,
 after origin verification,
 and verified without exposing
credentials or this repository to third-party scripts.

The consumer-side workaround belongs at our boundary (e.g. a parse-time
guard in the consuming crate) where it solves the user-facing problem
regardless of upstream movement.

## Check `.out-of-scope/` before filing upstream

Before filing or drafting an upstream issue/comment,
 read `.out-of-scope/`
for the tool or bug class.
 A listed exemption means upstream tracking is
out of scope:
 still write or update `doc/troubleshooting/<topic>.md`,
still record symptoms,
 root cause,
 verification,
 and workarounds,
 but skip
GitHub issue/comment filing.

Record the exemption path and reason in the "Upstream filing decision"
subsection.
 If no exemption matches,
 continue with duplicate search and
the 6-constraint check.

## Check for an existing issue before filing

A duplicate report is itself a publicity incident,
 so before opening
anything new,
 search the upstream tracker for the same behavior:
`gh search issues` and `gh search prs` across open and closed state,
with terms drawn from the symptom and the root cause,
 not just the tool
name.
 Constraint 5 already runs this search to gauge movement;
 this step
acts on a hit.
 Read the matching thread and its comments in full,
 not
just the title.

If a matching issue exists,
 do not open a second one.
 The contribution
becomes a comment on that issue,
 and the comment carries only what
genuinely advances the thread beyond what is already there:
 a sharper
root-cause trace,
 a reproduction they lack,
 affected-version data,
 the
prototyped fix and diff,
 or a workaround not yet mentioned.
 Diff your
findings against the existing thread before writing.

If nothing you have is absent from the thread,
 the comment is empty:
post nothing.
 A bare "+1",
 "me too",
 "still happens",
 or a subscribe is
not additive and is not a comment worth making.
 An empty comment is the
correct outcome when the thread already says everything you would,
 not a
gap to fill.

Record the duplicate in the doc:
 link the existing issue,
 and when there
is additive content keep the drafted comment the same way the draft
issue is kept (in a `~~~md` fence,
 ready to post).
 When there is nothing
to add,
 say so explicitly so a future session does not re-derive the
same empty comment.

## Risk to the poster before any filing

Filing puts a real person's account in front of strangers,
 so weigh that person's risk before drafting an artifact and before
saying anything about whether to file.

- Some repositories run trust or blocklist systems.
   Vouch is one;
   others exist,
   some more thorough,
   and this skill does not survey them.
   They close or lock issues and pull requests from unvouched users,
   and a maintainer can denounce a poster.
- The lists are files in a repository or published service,
   and projects can aggregate each other's lists,
   so a denouncement can follow a person from one project to others.
   Other people may see it:
   interviewers,
   employers,
   colleagues,
   and maintainers of unrelated projects.
   Do not assume it stays private or can be undone.
- An agent-written report that a maintainer judges low-effort is exactly
  the case these systems exist for.
   A reproduction, a source trace and a prototype lower that risk
   and do not remove it.
- A recommendation to file is allowed,
   and only when the work earns it:
   all six constraints hold,
   the reproduction was re-run,
   the prototype was verified against a baseline,
   the duplicate search and the trust-system search were done and
  recorded with dates,
   and no known limit leaves the evidence weak.
   When any of those is missing,
   say plainly what is missing and make no recommendation.
- Every statement about filing,
   a recommendation included,
   names this risk next to it and says filing is the poster's decision.
   A recommendation that skips the risk,
   or that rests on unchecked work,
   is the bad recommendation this section exists to prevent.
- Before the draft is marked fileable,
   search the target repository for a trust or blocklist system
   (workflow files,
   a `VOUCHED` or similar list,
   contribution-policy text),
   record what was searched with the date and the commit,
   and tell the filer to check the live repository again on the day of
   posting.

## Upstream filing artifact format

Keep the draft whatever the decision:
 when all six constraints hold it
is fileable as-is,
 and when they do not it stays as an auditable record
a future session can re-evaluate if upstream signal changes.
 Mark it
"do not file as-is" only in that second case.
 Wrap the draft body in a
`~~~md` fence so the future filer can copy it cleanly.
 The new-issue
draft contains:
 title,
 labels,
description with the same source trace as the "Root cause" section,
reproduction code from "Verification",
 and a "Suggested fix" naming
concrete code locations.

When "Check for an existing issue" found a duplicate,
 the kept artifact
is the additive comment draft (or an explicit note that there was
nothing to add),
 not a new-issue draft presented as fileable.

## Quality checks before declaring the doc done

- Every "the source does X" claim has a file path,
   line number,
   and
  code excerpt next to it.
- Every citation of a source this repository does not own links to its
  selection (a `#:~:text=` fragment or a commit-pinned line-range
  permalink),
   and `agent-browser` confirmed each link lands on the quoted text.
- If a filing artifact exists, the doc states the risk to the poster
  (see "Risk to the poster before any filing"),
   and any recommendation to file meets the conditions listed there.
- The harness reproduces the failure when run as written.
- Every workaround names a tradeoff.
- The 6-constraint subsection is present even when you decide to file,
  so the rationale is auditable.
- `.out-of-scope/` was checked before upstream filing.
   If an exemption
  matched,
   the doc names its path and does not keep a fileable draft.
- The draft issue is wrapped in a fenced block and marked "do not file
  as-is" unless all six constraints hold.
- The upstream tracker was searched for a duplicate.
   If one exists,
   the
  doc links it and either keeps a fenced,
   additive-only comment draft or
  states explicitly there is nothing to add;
   it does not also present a
  new-issue draft as fileable.
- If constraints 1-5 held or sorta-held but constraint 6 read "not
  yet,
  " the audit is incomplete;
   either run the auto-prototype step
  and record the result,
   or document why prototyping was attempted and
  abandoned.
   "Not yet" without that follow-up is the failure mode this
  skill explicitly catches.
