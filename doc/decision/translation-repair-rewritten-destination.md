# A reference the archive rendered another way: a candidate owes one rendering, not both

Decided by the owner on 2026-09-04 ("Either rendering"),
asked with three options after the luxuanwen3
re-run of that day stopped at its first paragraph.

## What happened

- luxuanwen3's original links `https://twitter.com/Deaver1229`;
  the archive's page links
    `https://x.com/Deaver1229`.
  `mergeAtoms` in `translate-atom-floor.ts` (commit `3ca134e57`:
  "protected
    atoms take the larger demand of the two references") owed a candidate the per-key maximum of both,
  so
    every rendering was told it lacked the other URL.
  The archive's own paragraph failed the same rule,
  the
    withheld-standing rule of the same day found nothing valid to ship,
  and the entry stopped
    (`ConsolidationStandingIneligibleError`,
  terminal `incumbent-only`,
  34 minutes,
  2.01 USD).
- Measured over the pinned corpus (`~/temp/agent/atom-census-20260904.mjs`,
  `link-census-20260904.mjs`):
    8 of 93 entries carry a link the archive rewrote (MizuharaNagisa,
  Rentable_A,
  SS3B_0016,
  XIEPT2,
  aiyysk,
    homoyamakaze,
  luxuanwen3,
  shihai4h):
  a moved domain,
  a trailing slash,
  a same-language Wikipedia article.
    Footnotes diverge in one direction only,
  never both.
  None of the eight could ship under the maximum.

## The rule

- Per atom kind,
  the atoms only the original carries and the atoms only the page carries form one pool
    (`renderingPoolsOf` in `translate-atom-rendering.ts`).
  A candidate owes exactly the larger side's count
    from the pool,
  drawn from either side:
  one link where each side has one,
  two where the page split one
    into two.
  Carrying neither and carrying both are refused,
  with one finding naming both renderings and the
    count owed.
- A kind that diverges in one direction only is not a pool:
  an atom the page added stays owed,
  an atom the
    page dropped stays owed,
  an atom neither carries stays refused.
  With no page there are no pools and the
    original alone governs,
  as before.
- The translators' rule says to keep the existing translation's destination where it differs from the
    original's and never carry both;
  the repair rule says the check accepts either and refuses both.

## What this is not

- Not an alignment of links by position or label.
  A page that dropped one source link and added an unrelated
    one reads as a rewrite of that kind,
  and the dropped link is not demanded there (XIEPT2's profile link);
    the owner accepted that in the option chosen.

## Options rejected

- The original's destination governs:
  reverts the archive's localizations (English Wikipedia back to Chinese,
    `x.com` back to `twitter.com`) and makes the archive's paragraph ineligible in all eight,
  so each must be
    rewritten to change one URL.
- Keep the maximum:
  the eight entries fail at consolidation on every pass.

## What landed

- `translate-atom-rendering.ts` (`AtomRenderingPool`,
  `renderingPoolsOf`,
  `atomFindings`),
  replacing the
    validator's local atom comparison;
  guards in `translate-atom-rendering.unit.test.ts` and
    `translate-validate.unit.test.ts` (either destination accepted,
  neither and both refused),
  shown to fail
    with the pools neutralised.
- `corpus-run/destination-renderings.ts` (`judgeDestinationRenderings`),
  the same rule over the whole
    would-ship page for the publisher's `DroppedDestinationError` (commit `1c9663666`).
  The first record of
    this decision said the publisher was unchanged;
  the luxuanwen3 re-run of 13:18 UTC the same day showed
    that reading wrong:
  every slice passed and the publisher,
  comparing the page to the source alone,
  refused
    the `x.com` page 56 minutes and 4.24 USD in (`~/temp/agent/luxuanwen3-shapes3-20260904.log`).
  The
    document pool is the source destinations the archive never carried plus the archive destinations the source
    never carried;
  the page owes the larger side from either;
  a destination both carry as written is owed
    outright;
  an archive addition the page lost is not a source destination and is not reported.
  Findings
    `destinations-archive-rendering` and `destinations-both-renderings` go on the `DESTINATIONS` line.
  Guards in
    `destination-renderings.unit.test.ts`,
  `dropped-destinations.unit.test.ts` and `publish-fixed.unit.test.ts`
    (the archive rendering published and named;
  neither refused;
  both named),
  shown to fail with the archive
    side of the pool neutralised.
