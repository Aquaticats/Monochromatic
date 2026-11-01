# Translation repair history 2026-08-12 to 2026-08-15: the channel marker that came back and the run that recorded nothing

The channel-marker fix that stopped working when the leaked tail grew by a character and the shape rule that replaced it,
the guard shown to fail,
the run whose log went into a pipe with no reader,
the durable record of lost voices and the callers that dropped it,
and the silent commands met along the way.

Part of [Translation repair history: 2026-08-12 to 2026-08-15](translation-repair-history-2026-08-12-to-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Session 2026-08-13, evening: the channel marker recurred and the run was recording nothing

### The marker came back one character longer

`#64` closed on 2026-08-12 with a parser fix that matched the exact string `|>`,
 which is what the provider's token filter left in front of Kimi-K3's JSON that
 day.
The filter is not atomic across SSE delta boundaries,
so what survives is
 a TAIL of a `<|word|>` token,
and the tail length is not stable.
By 2026-08-13
 it was `p|>` and `ep|>`,
both suffixes of `<|im_sep|>`,
and the exact match no
 longer fired.

Of the 23 voices lost in the most recent run window,
21 opened with one of those
 two.
State that as a WINDOW rather than a population:
the log spans three
 `START` lines across about 100 minutes.
The separate per-model tally in
 `translation-repair-runs-pass13/voice-loss.log` counts mentions,
not events,
 and cannot be read as a share of anything.

The fix matches the SHAPE,
not a vocabulary:
a bounded leading run of
 marker-name characters closing with `|>`,
then content that opens an object,
an
 array or a code fence.
A vocabulary rule would need the provider's tokenizer,
 which this code cannot read,
and would break again on the next token that
 leaks.
The shape rule covers every tail of every marker,
including ones nobody
 has seen.

What it deliberately does NOT become is a "skip junk until the first brace"
 rule,
which would swallow a model prefixing an apology and turn content the
 refusal detector must classify into a silent parse success.
Refusal
 classification is untouched regardless,
since it reads the unstripped answer.

### Three further gaps, found by review rather than by me

-   A fence hidden BEHIND a marker still lost the voice.
    The fence stripper runs
    first and sees the marker,
    so it does nothing;
    the marker rule then demanded
    a brace and found a backtick.
-   Several leaked markers in a row were not consumed.
    Now handled
    transactionally:
    a run that never reaches real content leaves the input
    untouched rather than half repaired.
-   The refusal test proved nothing.
    `|> I cannot help` carries no brace,
    so the
    rejected rule would leave it alone too.
    Replaced with an apology FOLLOWED BY
    valid JSON,
    which the rejected rule mends and this one must not.

### The guard was shown to fail before being trusted

Reverting `MARKER_TAIL_LIMIT` to 2,
which reproduces the old exact-`|>`
 behaviour,
makes the rule return no marker for `p|>`,
`ep|>` and `<|im_sep|>`
 while the shipped rule strips all three.
Run on a copy,
so the live worktree
 was never mutated while a pass could pick it up.

### Verified at the user boundary, in production

Seven minutes after the restart onto the fixed code:
six markers stripped,
two
 of them `p|>` which the old rule could not touch,
and zero voices lost in that
 window.
Short window,
and "zero lost" also depends on the roster being healthy
 in it;
the load-bearing evidence is the `p|>` strips.

### The run was writing its log into a pipe with no reader

Worse than the marker,
and found only by operating the thing.
A pass launched by
 a supervisor inherits a pipe to that supervisor,
which appends it to a file.
 Kill the supervisor to swap it,
which is safe for the pass and was verified
 earlier today,
and the pass keeps running while its output goes to a socket
 nobody holds.
Twenty minutes of a run produced no record at all.

Two consequences worth carrying forward.

-   `voice-loss.log` in the runs directory is NOT written by the pipeline.
    It is
    an operator artifact I produced by grepping a captured log.
    Nothing in the
    code emits it.
-   So voice loss is now recorded where it survives:
    `gatherStageVoices` emits a
    `stage-voice-lost` finding naming every model that never answered,
    and
    findings travel into the durable per-entry artifact.
    Emitted even when
    quorum was MET,
    which is the case the old findings dropped entirely and the
    one that hides a model degrading quietly while the stage still looks
    healthy.
    The test asserting empty findings for a two-of-four gather was
    asserting exactly that gap.

### Two silent failures in my own commands

-   `kill --signal KILL <pid>` is not valid for bash's builtin `kill`,
    and I had
    the stderr redirected away.
    Three kills reported success and none ran.
    Use
    `/usr/bin/kill --signal TERM`,
    and check `/proc/<pid>` rather than trusting
    the exit code.
-   `pgrep --full 'corpus-pass.ts'` matches the shell wrapper running the pgrep
    itself.
    Piping that to `kill` killed my own command,
    not the pass.
    Use the
    known pid.

Both belong to the same family as the two silent probes recorded earlier today:
 a command that answers a question you did not ask,
and reads as success.

### The silent probe, finally explained

Earlier today `find -newermt '-60 minutes'` reported nothing while the target
 existed,
and it was recorded as an unexplained silent probe.
It happened again
 tonight with `-newermt '-10 minutes'`,
which reported no slice-cache activity
 while the run was actively writing.
Measured directly,
in one directory at one
 moment:

-   `find slice-cache -type f -mmin -10` reports 3 files.
    Correct.
-   `find slice-cache -type f -newermt '10 minutes ago'` reports 0.
-   `find slice-cache -type f -newermt '-10 minutes'` reports 0,
    exit status 0.

So `-mmin` is reliable here and `-newermt` with a RELATIVE expression is not,
 in either spelling,
and neither wrong form errors.
Use `-mmin -N`.
The general
 form of the lesson is the one already written down:
validate a probe on a case
 that must match before trusting it to report nothing,
because an empty result
 and a broken query are the same two characters on screen.

### The durable record was itself not connected

Landing `stage-voice-lost` was not enough,
and I nearly declared it done.
A
 reviewer asked whether all eight `gatherStageVoices` callers actually thread
 `gather.findings` onward.
Two did not.

-   `candidate-select.ts` had no findings channel at all,
    so EVERY judge vote,
    per envelope and per chunk,
    built findings and dropped them.
-   `derivability-probe.ts` still has none.

This was symptomless by construction.
Under the old semantics findings were
 documented as "empty when quorum was met",
so a caller that discarded them
 looked exactly like one with nothing to pass on.
That is the same shape as the
 alignment findings nobody read and the rejection reasons that reached only a
 line count.
The lesson generalizes:
when a channel is usually empty,
a caller
 that drops it is invisible,
so adding a producer is only half the change.

`SelectionOutcome` now carries findings,
`editor-ensemble` collects them across
 every envelope vote,
`selectChunkPatch` returns them beside its patch rather
 than widening the shared `PatchOutcome`,
and the editor and refine stages
 spread them into findings they already write.
`derivability-probe.ts` is left
 alone deliberately:
it is reached only through `recall-barrel.ts` for the
 recall benchmark,
which writes no per-entry artifact for a finding to land in.

One finding per unheard model,
not one naming a list,
so counting findings
 counts voices lost.
A list-valued finding counts GATHERS that lost at least one
 voice,
which is a different number,
and reading the first as the second is
 exactly the mistake that made the old per-model tally unusable.

### What the positive control does and does not cover

Against the two rejected implementations,
the previous shipped version and the
 skip-to-the-first-brace rule:

-   apology then JSON:
    the shipped rule leaves it alone,
    the skip-to-brace rule
    MENDS it.
    This is the load-bearing case and it discriminates.
-   fence behind a marker:
    shipped strips,
    the previous version loses the voice.
-   two markers in a row:
    shipped strips both,
    the previous version loses it.
-   a marker run reaching prose:
    all three agree,
    so this test discriminates
    against NEITHER.
    It guards the transactional property against a
    partial-strip implementation,
    which neither comparison has.
    Weaker test,
    recorded as such rather than counted as proof.
