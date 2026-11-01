# Translation repair history, first handover document: the bench pricing prompt and completion tokens separately, and the width sweep re-read, 2026-08-15

The bench changed to price sending and answering separately,
with the tests it lacked,
the re-read of the width sweep by response schema at no new cost,
and the correction of what neither measurement may claim about which half of a ballot grows.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Bench token pricing split into prompt and completion, and the width sweep re-read

-   THE BENCH NOW PRICES SENDING AND ANSWERING SEPARATELY (`95b93ff9b`),
    which
    was the first remaining item of `#92`.
    `BenchCall` carries `promptTokens`
    and `completionTokens` beside the server's own total,
    and the summary prints
    all three.
    The wrapper had NO tests at all before this;
    five cases cover
    both halves,
    the missing-total fallback,
    the no-usage case,
    schema naming,
    a
    recorded and rethrown transport failure,
    and the quota read that stays off
    the rows.
    The 602 exchanges already bought keep only their totals and cannot
    be re-split.
-   AND THE WIDTH SWEEP WAS RE-READ PER STAGE (`a4ba5b505`,
    `601a3af1f`) at no
    new cost,
    by grouping the same rows by response schema.
    The budget audit had
    said the judge round "is the same size at every width":
    true of its calls,
    which sit at 5.4 per slice at every width from three up,
    and FALSE of its
    tokens,
    which rise 58% from width 2 to width 6.
    It still dominates at both
    ends,
    60% of a slice at width 2 and 52% at width 6,
    so a cheaper decision
    procedure saves more than a narrower producing roster.
    The character
    arithmetic in Question 1 had predicted a ballot growing 69% from width 3 to
    6;
    the measured figure is 38%,
    because a ballot also carries the policy,
    the
    source and the incumbent,
    none of which widen.
-   WHAT NEITHER MEASUREMENT MAY CLAIM,
    corrected the same morning after a
    review caught it:
    WHICH HALF of a ballot grows is not knowable from rows
    carrying one total per exchange,
    and the first bench under the split is what
    settles it.
    The `CallTokens` TSDoc,
    the audit and the decisions doc had each
    asserted a mechanism (a prompt repeating every candidate) and a provider
    behaviour (a total exceeding both halves) that nothing here measured.
    Both
    now read as open.
    Keeping the stated total needs no such claim anyway:
    report what the provider billed rather than a derivation,
    and fall back to
    the sum only for servers that state no total.
