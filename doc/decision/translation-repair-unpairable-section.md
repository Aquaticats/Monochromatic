# Unpairable sections are reported and skipped, not aligned proportionally

Ratified 2026-08-13 by the user.
Supersedes the open destination question in
 the aligner-scoring finding and in `doc/planning/wire-the-heading-aligner.md`.

## The decision

When the rebuilt heading matcher cannot confidently pair a section,
that section
 gets NO critic work and the refusal is recorded as a finding.
It is not paired
 by character fraction,
and it does not wait for a translate stage.

The ranked alternative was ROUTE,
sending an unpairable section to a translate
 stage.
That stage does not exist,
so today ROUTE and REPORT behave identically
 while ROUTE adds machinery for a case nothing can yet serve.
REPORT is
 available now and can become ROUTE later without anything being undone.

## Why not keep the proportional fallback

Because a confident wrong pairing costs more than an honest gap.
Proportional
 assignment slid every section of `XingZ60` by two,
so every critic call on that
 entry compared the wrong original against the wrong translation,
every issue
 filed was noise,
and the introduced-defect probe agreed because it was handed
 the same wrong source.

`XIEPT2` is the case that made the decision hard,
and on inspection it argues
 the same way.
It is a partial translation with six empty target bodies.
Pairing
 it feeds critics Chinese prose against bare English headings and manufactures
 omission claims for content nobody ever translated.
Producing nothing for that
 entry is more truthful than producing noise,
and cheaper.

## What this costs

`XIEPT2` produces no critic work at all until a translate stage exists.
That is
 accepted rather than overlooked.

## Scope of the change

Measured before ratifying:
90 of 92 entries align identically to production
 under the rebuilt matcher,
so only two entries re-slice.
`XingZ60` keeps 12 of
 its 13 pairs and loses only the wrong one.
`XIEPT2` refuses all eight.
