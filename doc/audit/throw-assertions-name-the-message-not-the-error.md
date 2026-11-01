# Throw assertions name the message, not the error

Measured 2026-08-19 over `package/module/translation-repair` (moved on 2026-10-18 to the packages under
`package/translation-repair`),
zero quota,
read-only,
while the `#107` verification ran.

## What was counted

Every `.toThrow(...)` and `.rejects.toThrow(...)` across 236 unit test files,
classified by what the assertion was given:

```text
by MESSAGE SUBSTRING   161
by ERROR CLASS          68
by regex                 0
bare or other           26
```

So roughly two in three throw assertions in this package check the text of a message
rather than the identity of the error,
against 49 exported error classes.

## The actionable subset is 88, not 161

Splitting the 161 by whether the error class was already in scope in that file:

```text
in files that ALREADY name an exported error class   88
in files naming no error class at all                73
```

The 88 are where the change costs nothing.
The class is already imported and referred to in the same file,
so adding it beside the existing message check is one argument,
and it closes the gap without removing the diagnostic assertion that is there now.

Heaviest files,
with the class each already names:

-   `artifact-change-sets.unit.test.ts`,
    20,
    `ArtifactParseError`
-   `corpus-run/artifact-v2-read.unit.test.ts`,
    16,
    `ArtifactParseError`
-   `corpus-run/artifact-v2-read-vocabulary.unit.test.ts`,
    12,
    `ArtifactParseError`
-   `slice-delivery.unit.test.ts`,
    11,
    `SliceDeliveryError`
-   `delivery-coherence.unit.test.ts`,
    7,
    `DeliveryCoherenceError`

## Why it matters here specifically, in two ways

A MESSAGE ASSERTION PASSES ON THE WRONG ERROR whenever some other failure carries the
same words.
That is the ordinary argument,
and it matters most at guards,
where `GFP` already requires that a guard be shown to fail before it is trusted:
a guard test that would also pass on an unrelated error proves less than it appears to.

THE SECOND REASON IS LOCAL TO THIS REPOSITORY and is the stronger one.
`DGT` requires user-facing diagnostics to name the affected input and every remediation
path,
and says their length is unconstrained.
`DNL` requires neutral wording.
Both invite rewriting messages as they are improved.
Every rewrite that touches an asserted substring breaks a test that was not testing the
thing that changed,
so the current style puts a standing tax on obeying `DGT`.
Pinning the class and a short stable phrase leaves the prose free to improve.

## What this did NOT establish, recorded so nobody quotes it as more than it is

IT DOES NOT SHOW THAT ANY ASSERTION CURRENTLY PASSES ON THE WRONG ERROR.
That was the measurement worth having and the probe built for it could not answer it.
Searching source files for each asserted substring returns three different things at once:
generic words that occur in ordinary prose (`status` appears in 53 files),
substrings absent from source because the message is composed at runtime from a template
(`lanes.repair.whiskers` is a test fixture path echoed back by the message),
and genuine matches.
Establishing real ambiguity needs the message templates evaluated rather than grepped,
and that was not built.

So the finding is about STYLE AND EXPOSURE,
measured exactly,
and not about a defect demonstrated in a specific test.

## Recommendation, scoped deliberately

Add the class to the 88 assertions whose file already names one,
keeping the message check
beside it.
Leave the other 73 alone unless a class is introduced for them on their own merits.
Do not sweep all 161:
the remaining ones would need new imports for modest gain,
and churn across a test suite is its own risk this close to a release.

Not urgent,
and it touches package source,
so it lands after the running verification
rather than beside it.
