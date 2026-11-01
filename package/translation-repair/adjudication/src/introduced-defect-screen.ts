import type {
  AdjudicatedIssue,
  ReadonlyEntry,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { withoutTranslatorsNotes, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { RepairRegion, } from './repair-region.ts';
import {
  type IntroducedDefectCheckWire,
  isIntroducedDefectVerdict,
} from './introduced-defect-wire.ts';
import {
  normalizePunctuation,
  rendersAsNothing,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { flattenSpace, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Introduced-defect screening
// The deterministic half of the probe. A prober claiming the edit introduced a
// defect must quote the damaged wording from the AFTER text; this module then
// decides what that quote actually proves, without asking any model.
//
// The precedent is `screenNonTranslationVotes`: deterministic evidence gets to
// DISMISS a claim it contradicts, and never has to positively prove one. There
// is no mechanical test for mistranslation, so demanding one would make the
// probe blind to the defects it exists to find. What is mechanical is the
// differential premise: a quote that already occurs in the BEFORE text cannot
// have been introduced by replacing that text, whatever the prober says.

/**
 What the deterministic check made of one claim.

 @example
 ```ts
 const admissibility: ClaimAdmissibility = 'contradicted';
 ```
 */
export type ClaimAdmissibility =
  | 'corroborated'
    | 'removal-corroborated'
    | 'contradicted'
    | 'unanchored'
    | 'pre-existing';

/**
 Admissibility values that uphold a claim that the edit caused damage: a
 DELIBERATE SUBSET of `ClaimAdmissibility`, typed by it so a misspelt member
 fails to compile.

 LEFT OUT are `contradicted` (the differential refuted the claim),
 `unanchored` (it carried no usable anchor) and `pre-existing` (the claim
 re-reports a defect the region was cut for), none of which says the edit
 caused anything. A member `ClaimAdmissibility` gains is therefore not upheld
 until it is added here.

 @example
 ```ts
 const upheld = UPHELD_ADMISSIBILITY.has('corroborated',);
 ```
 */
export const UPHELD_ADMISSIBILITY: ReadonlySet<ClaimAdmissibility> = new Set<ClaimAdmissibility>([
  'corroborated',
  'removal-corroborated',
],);

/**
 One prober claim of introduced damage, after screening.

 @example
 ```ts
 const claim: ScreenedDefectClaim = { modelId, admissibility: 'corroborated', ... };
 ```
 */
export type ScreenedDefectClaim = {
  /**
   Prober that made the claim.
   */
  readonly modelId: string;

  /**
   Defect class in the prober's words.
   */
  readonly category: string;

  /**
   Severity as claimed, unvalidated: recorded so a later calibration can ask
   whether probers use the vocabulary the panel uses.
   */
  readonly severity: string;

  /**
   Wording the prober quoted from the AFTER text, for damage the edit added.
   */
  readonly evidence: string;

  /**
   Wording the prober quoted from the BEFORE text, for content the edit
   dropped; empty on claims of added damage.
   */
  readonly omittedText: string;

  /**
   Why the prober says the BEFORE text lacked this defect.
   */
  readonly reason: string;

  /**
   What the deterministic check made of the quote.
   */
  readonly admissibility: ClaimAdmissibility;
};

/**
 Everything screening decided about one replaced region.

 The five counts are kept apart rather than reduced to a verdict because the
 question this probe was built to answer is which of them a human agrees with,
 and collapsing them now would destroy the evidence for that.

 @example
 ```ts
 const tally: RegionDefectTally = { envelopeId, corroborated: 1, ... };
 ```
 */
export type RegionDefectTally = {
  /**
   Envelope the region replaced.
   */
  readonly envelopeId: string;

  /**
   Accepted issues the region served.
   */
  readonly issueIds: readonly string[];

  /**
   Claims of ADDED damage whose quote is in the AFTER text and absent from
   BEFORE.
   */
  readonly corroborated: number;

  /**
   Claims of DROPPED content whose quote is in the BEFORE text and absent from
   AFTER, so the edit demonstrably removed it.
   */
  readonly removalCorroborated: number;

  /**
   Claims the differential refutes in the direction claimed: added wording
   that already occurred before the edit, or dropped wording still present
   after it.
   */
  readonly contradicted: number;

  /**
   Claims carrying no usable anchor, neither anchor, or both at once, leaving
   nothing to check them against.
   */
  readonly unanchored: number;

  /**
   Claims quoting wording an accepted issue already complained about, so the
   prober is re-reporting the defect the region was cut for rather than
   damage the edit caused.

   Counted apart from every other outcome because it is the only one that
   says the claim is about the WRONG THING rather than wrong. Folding it into
   `contradicted` would say the differential refuted the claim, which it did
   not, and folding it into a damage count would credit the probe for finding
   the defect it was told to ignore.
   */
  readonly preExisting: number;

  /**
   Probers that looked and reported no introduced defect.
   */
  readonly noneFound: number;

  /**
   Probers that declined to judge this region.
   */
  readonly uncertain: number;

  /**
   Every claim of introduced damage, screened.
   */
  readonly claims: readonly ScreenedDefectClaim[];
};

/**
 Text as the screen compares it: through the evidence fold, then with its
 whitespace collapsed, so a prober's quote written with straight marks, or
 rewrapped, still reads as the words it quotes (ledger B24).

 @param text - a prober's quote, or a region's text

 @returns Folded, collapsed text

 @example
 ```ts
 asScreened({ text: 'The  cat’s nap', },); // "The cat's nap"
 ```
 */
function asScreened({ text, }: { readonly text: string; },): string {
  return flattenSpace({ text: normalizePunctuation({ text, },), },);
}

/**
 A region's text as the prober read it: without the notes between translators
 when its sheet cut them, as it stands when it cut nothing.

 @param text - the region's replaced or replacement text

 @param original - the texts whose comments are the original's own on the
 sheet, absent when the sheet cut nothing

 @returns The text the prober's quote was copied out of

 @example
 ```ts
 const shown = shownTextOf({ text: region.editorAfter, original: '', },);
 ```
 */
function shownTextOf(
  {
    text,
    original,
  }: {
    readonly text: string;
    readonly original?: string;
  },
): string {
  return (original === undefined)
    ? text
    : withoutTranslatorsNotes({
      text,
      original,
    },);
}

/**
 Decides what a claim's anchors prove about one region.

 The differential runs in BOTH directions, because collateral damage comes in
 two shapes and only one of them can be quoted from the new text. Wording the
 edit ADDED is checkable as present in AFTER and absent from BEFORE. Wording
 the edit DROPPED cannot be quoted from AFTER at all, since its absence is the
 defect, and is checkable as present in BEFORE and absent from AFTER. Judging
 only the first direction would have made every omission claim unanchored,
 which is the failure mode worth guarding hardest against: dropping a clause
 while rewriting is among the likeliest ways an editor causes damage.

 A claim carrying BOTH anchors is a wire fault rather than a stronger claim.
 Screening each and taking the better answer would let a prober launder a
 contradicted anchor by attaching a second one.

 A side is an anchor when it shows a reader something, which is asked of
 `rendersAsNothing` and never of an empty-string comparison: a side holding
 only spaces, a zero-width space or a filler quotes nothing, and counting it
 made a one-sided claim read as anchored both ways (ledger B128).

 @param evidence - wording quoted from the replacement, for added damage

 @param omittedText - wording quoted from the replaced text, for dropped
 content

 @param region - region the claim is about

 @param original - the texts whose comments are the original's own on the
 sheet the prober read (`proberOriginalOf`): both texts the quote is checked
 against are read as that sheet showed them, without the notes between
 translators, since the prober copied the quote out of them so; absent when the
 sheet cut nothing

 @returns Admissibility of the claim

 @example
 ```ts
 const admissibility = screenEvidence({ evidence, omittedText: '', region, original, },);
 ```
 */
export function screenEvidence(
  {
    evidence,
    omittedText,
    region,
    original,
  }: {
    readonly evidence: string;
    readonly omittedText: string;
    readonly region: RepairRegion;
    readonly original?: string;
  },
): ClaimAdmissibility {
  /**
   Whether the claim quotes wording the edit added. A side that shows a
   reader nothing is no anchor, asked as the package asks it of any text a
   model wrote (ledgers B40 and B128).
   */
  const quotesAdded = !rendersAsNothing({ text: evidence, },);

  /**
   Whether the claim quotes wording the edit dropped, asked the same way.
   */
  const quotesDropped = !rendersAsNothing({ text: omittedText, },);
  if (quotesAdded === quotesDropped)
    return 'unanchored';

  /**
   Replacement text both directions are checked against, as the prober read it.
   */
  const after = asScreened({
    text: shownTextOf({
      text: region.editorAfter,
      ...((original === undefined) ? {} : { original, }),
    },),
  },);

  /**
   Replaced text both directions are checked against, as the prober read it.
   */
  const before = asScreened({
    text: shownTextOf({
      text: region.before,
      ...((original === undefined) ? {} : { original, }),
    },),
  },);
  if (quotesAdded) {
    /**
     Added-wording anchor, folded and collapsed as both texts are compared.
     */
    const added = asScreened({ text: evidence, },);
    if (!after.includes(added,))
      return 'unanchored';
    return before.includes(added,) ? 'contradicted' : 'corroborated';
  }

  /**
   Dropped-wording anchor, folded and collapsed likewise.
   */
  const dropped = asScreened({ text: omittedText, },);
  if (!before.includes(dropped,))
    return 'unanchored';
  return after.includes(dropped,) ? 'contradicted' : 'removal-corroborated';
}

/**
 Target-side wording every accepted issue a region serves complained about.

 Read from the claims' own evidence rather than from the region, because the
 region records what was REPLACED and an issue records what was WRONG, and a
 replacement is routinely wider than any single complaint it answers.

 @param region - region whose served issues are collected

 @param issues - accepted issues of the chunk

 @returns Target-side quotes as the screen compares text, those that show a reader nothing dropped

 @example
 ```ts
 const quotes = collectPriorQuotes({ region, issues, },);
 ```
 */
function collectPriorQuotes(
  {
    region,
    issues,
  }: {
    readonly region: RepairRegion;
    readonly issues: readonly AdjudicatedIssue[];
  },
): readonly string[] {
  return issues
    .filter(function isServed(issue,) {
      return region.issueIds
        .includes(issue.issueId,);
    },)
    .flatMap(function toQuotes(issue,) {
      return issue.claims
        .flatMap(function toSpans(member,) {
          return member.claim
            .spans
            .filter(function isTargetSide(span,) {
              return span.side === 'target';
            },)
            .map(function toText(span,) {
              return asScreened({ text: span.quotedText, },);
            },);
        },);
    },)
    .filter(function isUsable(quote,) {
      return !rendersAsNothing({ text: quote, },);
    },);
}

/**
 Whether an anchored verdict would otherwise count as damage the edit caused.

 The pre-existing check only ever DOWNGRADES a claim that survived the
 differential. A contradicted claim has already been refuted mechanically, by
 the stronger fact that its wording was present before the edit, and an
 unanchored one quotes nothing checkable; relabelling either would replace a
 precise verdict with a vaguer one.

 @param anchored - what the differential made of the claim

 @returns Whether the verdict is one the pre-existing check may replace

 @example
 ```ts
 const replaceable = countsAsDamage({ anchored: 'removal-corroborated', },);
 ```
 */
function countsAsDamage(
  { anchored, }: { readonly anchored: ClaimAdmissibility; },
): boolean {
  return UPHELD_ADMISSIBILITY.has(anchored,);
}

/**
 Decides whether a claim is pointing at a defect that was already reported.

 This is the defence the PROMPT used to provide by listing the accepted issues
 and forbidding a prober from re-reporting them. Listing them measurably
 silenced the stage: with the list shown the probe raised 2 admissible claims
 across 45 verdicts on regions a reader called damaged, and with it withheld,
 18. Moving the same defence here keeps it while letting the prober look at
 the text without being told what to excuse, which is the split
 `screenNonTranslationVotes` already set as precedent: deterministic evidence
 dismisses a claim rather than a prompt preventing it.

 Containment is checked BOTH ways for ADDED wording, because those two quotes
 are cut by different parties. A critic quotes the phrase it objected to, and a
 prober quotes as much of the surrounding wording as it thinks damaged, so
 neither is reliably the longer one.

 REMOVAL CLAIMS TAKE ONLY ONE DIRECTION, and the difference is the whole point.
 A removal claim quotes the wording that DISAPPEARED, drawn from the before
 text, which is the same side the critic quoted, on a region that exists
 precisely because the critic quoted something in it. So containment is close
 to guaranteed, and which way it runs is the entire signal:

 - Dropped wording INSIDE the prior quote was licensed to disappear. Removing
   the objected-to phrase is what the repair was for, so a prober reporting its
   absence is restating the accepted issue. Discounted.
 - Dropped wording CONTAINING the prior quote means the edit took the
   objected-to phrase AND unrelated content with it. That is the over-deletion
   shape a human grader found as a deleted contributor credit, and it is a NEW
   defect the critic never asked for. It must survive.

 Checking both ways here suppressed exactly the second case. Measured:
 removal-corroborated ran 159 across the original 56-entry run and 0 across
 every run after this reclassification landed, while corroborated held its rate
 per region, because added-wording claims quote the AFTER text and never
 collided.

 Both quotes arrive as the differential read them, through `asScreened`
 (ledgers B24 and B128), and the claim's quote is the side the differential
 corroborated. A quote chosen here by any other rule can disagree with the
 verdict it is checked under: a claim corroborated on its evidence, with an
 omitted side of spaces, was once checked on those spaces and so restated
 nothing.

 @param quoted - side the differential corroborated the claim on, as
 `asScreened` returns it

 @param priorQuotes - target-side quotes of the served issues, likewise

 @param removal - whether the claim anchors on wording the edit dropped

 @returns Whether the claim restates an accepted issue

 @throws {@link Error} when the quote is blank, which no claim the
 differential corroborated carries

 @example
 ```ts
 const known = restatesPriorIssue({ quoted, priorQuotes, removal: false, },);
 ```
 */
function restatesPriorIssue(
  {
    quoted,
    priorQuotes,
    removal,
  }: {
    readonly quoted: string;
    readonly priorQuotes: readonly string[];
    readonly removal: boolean;
  },
): boolean {
  // A blank quote would sit inside every prior quote and dismiss the claim.
  if (quoted === '') {
    throw new Error(
      'unreachable: a claim counted as damage quotes nothing, though screenEvidence corroborates a claim only on a side that is not blank once screened',
    );
  }

  return priorQuotes
    .some(function overlaps(prior,) {
      if (removal)
        return prior.includes(quoted,);

      return prior.includes(quoted,) || quoted.includes(prior,);
    },);
}

/**
 Counts screened claims sharing one admissibility.

 @param claims - screened claims of one region

 @param wanted - admissibility to count

 @returns Claims carrying that admissibility

 @example
 ```ts
 countAdmissibility({ claims, wanted: 'contradicted', },);
 ```
 */
function countAdmissibility(
  {
    claims,
    wanted,
  }: {
    readonly claims: readonly ScreenedDefectClaim[];
    readonly wanted: ClaimAdmissibility;
  },
): number {
  return claims.filter(function matches(claim,) {
    return claim.admissibility === wanted;
  },)
    .length;
}

/**
 One prober's check on a region, paired with who cast it.

 Named rather than inferred from the mapping that builds it: an inferred
 object literal carries writable properties, and every later reader of the
 cast list then takes a mutable parameter it never mutates.
 */
type CastCheck = Readonly<{
  /**
   Prober that cast this check.
   */
  modelId: string;

  /**
   Check as the wire carried it.
   */
  check: IntroducedDefectCheckWire;
}>;

/**
 Every prober's checks resolved to one per region, with what the resolution
 passed over.

 @example
 ```ts
 const resolved: ResolvedProberChecks = { checks: new Map(), findings: [], };
 ```
 */
export type ResolvedProberChecks = {
  /**
   Per prober, the check kept for each one-based region number.
   */
  readonly checks: ReadonlyMap<string, ReadonlyMap<number, IntroducedDefectCheckWire>>;

  /**
   Wire irregularities in scorecard-stable wording, each under its prober.
   */
  readonly findings: readonly string[];
};

/**
 Resolves each prober's checks once into the first check per region carrying
 a verdict the screen knows.

 A check carrying a verdict outside the closed vocabulary is a wire fault and
 is passed over, as the screen always did. A later check on a region the
 prober already answered is a `duplicate-check` finding and is not counted,
 as `resolveResolutionChecks` (`tally-resolution.ts`) does for the checkers.

 @param ballots - checks per prober, keyed by model id

 @returns Kept checks per prober and the findings, `<model id>: duplicate-check (<region>)`

 @example
 ```ts
 const { checks, findings, } = resolveProberChecks({ ballots, },);
 ```
 */
export function resolveProberChecks(
  { ballots, }: { readonly ballots: Readonly<Record<string, readonly IntroducedDefectCheckWire[]>>; },
): ResolvedProberChecks {
  /**
   Findings accumulated across every prober.
   */
  const findings: string[] = [];

  /**
   Kept checks per prober, in the order the probers are keyed. A map until
   handed back, as every record filled by a key is (ledger B77).
   */
  const checks = new Map<string, ReadonlyMap<number, IntroducedDefectCheckWire>>();
  for (const [modelId, wire,] of Object.entries(ballots,)) {
    /**
     This prober's kept check per region; first occurrence wins.
     */
    const kept = new Map<number, IntroducedDefectCheckWire>();
    for (const check of wire) {
      if (!isIntroducedDefectVerdict(check.verdict,))
        continue;
      if (kept.has(check.region,)) {
        findings.push(`${modelId}: duplicate-check (${check.region})`,);
        continue;
      }
      kept.set(
        check.region,
        check,
      );
    }
    checks.set(
      modelId,
      kept,
    );
  }
  return {
    checks,
    findings,
  };
}

/**
 Screens every prober ballot into one tally per region.

 A check naming a region outside the sheet, or carrying a verdict outside the
 closed vocabulary, is dropped rather than counted anywhere: it is a wire
 fault, and folding it into `uncertain` would make schema noise look like
 model doubt.

 @param regions - replaced regions in prompt numbering order

 @param ballots - checks per prober, keyed by model id

 @param issues - accepted issues the regions serve, whose quotes a prober's may restate

 @param original - the texts whose comments are the original's own on the
 sheet the probers read (`proberOriginalOf`), so each quote is checked against
 the regions as the sheet showed them; absent when the sheet cut nothing

 @returns Tally per region, in region order

 @example
 ```ts
 const tallies = screenIntroducedDefects({ regions, ballots, original, },);
 ```
 */
export function screenIntroducedDefects(
  {
    regions,
    ballots,
    issues = [],
    original,
  }: {
    readonly regions: readonly RepairRegion[];
    readonly ballots: Readonly<Record<string, readonly IntroducedDefectCheckWire[]>>;
    readonly issues?: readonly AdjudicatedIssue[];
    readonly original?: string;
  },
): readonly RegionDefectTally[] {
  /**
   Each prober's checks resolved once, so every region reads the same first
   check of a prober.
   */
  const resolved = resolveProberChecks({ ballots, },);
  return regions.map(function toTally(
    region,
    index,
  ): RegionDefectTally {
    /**
     Wording the accepted issues of this region already complained about.
     */
    const priorQuotes = collectPriorQuotes({
      region,
      issues,
    },);
    /**
     One check per prober on this region, paired with its prober.

     ONE PER PROBER, THE FIRST WITH A VERDICT THE SCREEN KNOWS, resolved once
     for every region by `resolveProberChecks`. The sheet asks for one check
     per region; a prober answering twice used to count twice and a prober
     skipping the region counted nowhere, so the printed tallies could exceed
     the probers heard or fall short of them.
     */
    const cast = [...resolved.checks,]
      .flatMap(function toChecks([
        modelId,
        byRegion,
      ],): readonly CastCheck[] {
        /**
         First check this prober cast here with a known verdict.
         */
        const first = byRegion.get(index + 1,);
        return (first === undefined)
          ? []
          : [
            {
              modelId,
              check: first,
            },
          ];
      },);

    /**
     Probers that cast no check on this region at all, counted as uncertain:
     a prober that skipped a region has not cleared it. A prober whose only
     check here carried a verdict outside the vocabulary is schema noise, not
     doubt, and stays dropped as before.
     */
    const silentProbers = Object
      .entries(ballots,)
      .filter(function castNothingHere([
        ,
        checks,
      ]: ReadonlyEntry<string, readonly IntroducedDefectCheckWire[]>,): boolean {
        return !checks.some(function isThisRegion(check,): boolean {
          return check.region === (index + 1);
        },);
      },)
      .length;

    /**
     Screened claims of introduced damage on this region.
     */
    const claims = cast
      .filter(function isClaim(entry,) {
        return entry.check
          .verdict
          === 'introduced-defect';
      },)
      .map(function toClaim(entry,): ScreenedDefectClaim {
        /**
         What the differential makes of this claim's anchors.
         */
        const anchored = screenEvidence({
          evidence: entry.check
            .evidence,
          omittedText: entry.check
            .omittedText,
          region,
          ...((original === undefined) ? {} : { original, }),
        },);

        /**
         Both anchors as the prober wrote them.
         */
        const {
          evidence,
          omittedText,
        } = entry.check;

        /**
         Whether the differential corroborated the claim on wording the edit
         dropped.
         */
        const removal = anchored === 'removal-corroborated';

        /**
         Wording the differential corroborated the claim on, read as it read
         it. Choosing the side by the raw omitted text took a side of spaces
         for the quote (ledger B128). Read only where the claim counts as
         damage.
         */
        const quoted = asScreened({ text: removal ? omittedText : evidence, },);

        return {
          modelId: entry.modelId,
          category: entry.check
            .category,
          severity: entry.check
            .severity,
          evidence: entry.check
            .evidence,
          omittedText: entry.check
            .omittedText,
          reason: entry.check
            .verdictReason,
          admissibility: countsAsDamage({ anchored, },)
            && restatesPriorIssue({
              quoted,
              priorQuotes,
              removal,
            },)
            ? 'pre-existing'
            : anchored,
        };
      },);

    return {
      envelopeId: region.envelopeId,
      issueIds: region.issueIds,
      corroborated: countAdmissibility({
        claims,
        wanted: 'corroborated',
      },),
      removalCorroborated: countAdmissibility({
        claims,
        wanted: 'removal-corroborated',
      },),
      contradicted: countAdmissibility({
        claims,
        wanted: 'contradicted',
      },),
      unanchored: countAdmissibility({
        claims,
        wanted: 'unanchored',
      },),
      preExisting: countAdmissibility({
        claims,
        wanted: 'pre-existing',
      },),
      noneFound: cast.filter(function foundNone(entry,) {
        return entry.check
          .verdict
          === 'no-introduced-defect-found';
      },)
        .length,
      uncertain: cast.filter(function declined(entry,) {
        return entry.check
          .verdict
          === 'uncertain';
      },)
        .length
        + silentProbers,
      claims,
    };
  },);
}

//endregion Introduced-defect screening
