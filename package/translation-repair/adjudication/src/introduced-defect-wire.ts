import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import {
  communityRenderingsBlock,
  type RenderingCandidate,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  DECLARED_IDENTITY_RULES,
  declaredNamesBlock,
} from './declared-identity-rule.ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  flattenSpace,
  MEASUREMENT_POLICY_BLOCK,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  type AdjudicatedIssue,
  APPARATUS_KINDS,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import { proberShownTexts, } from './introduced-defect-shown.ts';
import type { RepairRegion, } from './repair-region.ts';

//region Introduced-defect probe wire
// The checker stage asks whether each KNOWN issue is gone. Nothing asked
// whether the repair broke something nobody had raised, so `regressedKnownIssues`
// scores a patch that fixes its target and mangles the sentence beside it as
// clean. This wire asks the differential question instead: did THIS replacement
// cause a defect that the baseline did not have.
//
// The failure mode is known before the first call: every region here contains a
// defect by construction, since that is why it was edited, so a model asked
// "is anything wrong" will always find something. Three things fight it, and
// none of them is the prompt alone: the verdict vocabulary refuses to offer
// `clean`, which would be false of a region whose original defect survives; the
// pre-existing issues are shown and labelled as NOT findings; and every claim
// must quote verbatim wording, which `introduced-defect-screen.ts` then checks
// deterministically.
//
// A claim anchors in one of two directions and never both, because damage comes
// in two shapes. Added wording is quoted from the AFTER text; content the edit
// DROPPED has nothing in the AFTER text to quote, its absence being the defect,
// so it is quoted from the BEFORE text instead. Accepting only forward quotes
// would have left omission, the likeliest damage a rewriting editor causes,
// permanently unprovable.


/**
 Every verdict a prober may cast on one region, closed vocabulary.

 There is deliberately no `clean`. A region can be free of introduced damage
 while remaining defective, because the accepted issue it was cut for may
 still be unfixed, and a vocabulary that made a prober choose between `clean`
 and `new-defect` would push every such region into the second bucket. The
 long name says exactly what a negative verdict proves: this prober found no
 introduced defect, not that the region is well translated.

 @example
 ```ts
 INTRODUCED_DEFECT_VERDICTS.includes('introduced-defect',);
 ```
 */
export const INTRODUCED_DEFECT_VERDICTS = [
  'introduced-defect',
  'no-introduced-defect-found',
  'uncertain',
] as const;

/**
 One prober verdict on one region.

 @example
 ```ts
 const verdict: IntroducedDefectVerdict = 'introduced-defect';
 ```
 */
export type IntroducedDefectVerdict = typeof INTRODUCED_DEFECT_VERDICTS[number];

/**
 Guards untrusted verdict strings from model JSON.

 @param value - candidate from unvalidated model output

 @returns Whether value names one listed verdict

 @example
 ```ts
 isIntroducedDefectVerdict('uncertain',);
 ```
 */
export function isIntroducedDefectVerdict(value: unknown,): value is IntroducedDefectVerdict {
  if ((typeof value) !== 'string')
    return false;

  return (INTRODUCED_DEFECT_VERDICTS as readonly string[]).includes(value,);
}

/**
 Which edit a probe run is auditing.

 The question is identical for both and the probe's rules are shared; only what
 the editor was TRYING to do differs, and that changes what a prober should
 expect to see. Telling a prober that an edit was fixing defects, when it was
 actually rewriting already-repaired text for fluency, invites them to read
 every rephrasing as a failed repair.

 @example
 ```ts
 const kind: ProbedEditKind = 'naturalness-refinement';
 ```
 */
export type ProbedEditKind = 'accuracy-repair' | 'naturalness-refinement';

/**
 Opening framing per edit kind, prepended to the shared rules.
 */
const PROBE_FRAMING: Readonly<Record<ProbedEditKind, string>> = {
  'accuracy-repair':
    `Editors replaced the BEFORE text of each numbered region with its AFTER text, trying to fix defects that were ALREADY THERE.`,
  'naturalness-refinement':
    `A later pass rewrote the BEFORE text of each numbered region into its AFTER text for NATURALNESS ALONE.
It was NOT fixing defects. Any listed issue was already repaired in the BEFORE text, so a rephrasing that keeps the meaning is the edit working as intended, not a failure.`,
};

/**
 The one rule that names what the editor was doing, per edit kind.

 Kept per kind rather than neutralised into "the edit" because the two lanes
 really are doing different things, and a prober told the wrong one reads
 every rephrasing as a failed repair.

 This comment used to say the accuracy prompt was byte-identical to the one
 every artifact had been produced under, so telemetry stayed comparable across
 runs. That is no longer true and the comparability is no longer worth having.
 The old prompt asked whether the replacement introduced a defect the BEFORE
 text did not have, which made the pre-edit TRANSLATION the standard of
 accuracy. Read back, every claim it produced argued from the before text, and
 one called a corrected mistranslation an introduced inaccuracy. A series of
 readings taken through that question measured whether an edit CHANGED
 anything, so preserving comparability with it would preserve a measurement of
 the wrong quantity.
 */
const PROBE_CREATED_CLAUSE: Readonly<Record<ProbedEditKind, string>> = {
  'accuracy-repair':
    `- DO report a distinct defect created while attempting the repair, even where it concerns the same source passage as a pre-existing issue.`,
  'naturalness-refinement':
    `- DO report a distinct defect the rewrite created, even where it concerns the same source passage as a pre-existing issue.`,
};

/**
 Rules preceding the per-kind clause.
 */
const PROBE_RULES_HEAD =
  `Judge ONLY this: does the AFTER text misrepresent the ORIGINAL in a way the BEFORE text did not?

THE ORIGINAL IS THE ONLY STANDARD OF ACCURACY. The BEFORE text is a translation and may itself be wrong. Correcting it is the edit working, not damage.

Rules:
- A change that brings the AFTER text CLOSER to the ORIGINAL is NEVER damage, however much text it rewrites.
- Content the AFTER text drops is damage ONLY IF THE ORIGINAL SUPPORTS IT. Dropping wording the ORIGINAL never had is a correct repair, not an omission, unless it is page apparatus (${APPARATUS_KINDS}): the page keeps its apparatus, so dropping it is damage. ${NARRATIVE_DETAIL_IS_NOT_APPARATUS}
- Wording the AFTER text adds is damage only if the ORIGINAL does not support it.
- "It was in the BEFORE text" is NOT a reason. Say what the ORIGINAL says and how the AFTER text departs from it.
- Do NOT report a listed pre-existing issue merely because the replacement failed to fix it. That is not damage.
- Do NOT report a defect that is present in BOTH the BEFORE text and the AFTER text. It was not introduced.`;

/**
 Rules and reply contract following the per-kind clause.
 */
const PROBE_RULES_TAIL = `- Stylistic preference is NOT a defect. A different word being nicer is not damage. Report only concrete loss of accuracy, grammar, coherence, or consistency.

Verdicts:
- introduced-defect: the replacement caused a specific defect absent from BEFORE
- no-introduced-defect-found: you looked and found no defect the replacement caused; the region may still be imperfect
- uncertain: you cannot tell from what you were shown

For introduced-defect, anchor the claim with EXACTLY ONE of these, never both:
- "evidence": the exact damaged wording quoted FROM THE AFTER TEXT, when the edit ADDED or altered something. Leave "omittedText" empty.
- "omittedText": the exact wording quoted FROM THE BEFORE TEXT that the edit DROPPED, when the defect is missing content. Leave "evidence" empty.
Quote verbatim. A paraphrase cannot be checked and the claim will be discarded.
Say in "verdictReason" what the ORIGINAL says and how the AFTER text departs from it. A reason that only cites the BEFORE text has not judged accuracy at all.
Leave "evidence", "omittedText", "category", "severity" and "verdictReason" as empty strings for other verdicts.
Reply with ONLY a JSON object of shape {"checks": [{"category": "", "evidence": "", "omittedText": "", "region": 1, "severity": "", "verdict": "no-introduced-defect-found", "verdictReason": ""}]}. No prose, no code fences.
Every region number must appear exactly once in checks.`;

/**
 How the house rules land on this sheet's own verdicts.

 NAMED SEPARATELY FROM `PROBE_RULES_HEAD` because it CONTRADICTS one of its
 bullets on purpose. That bullet says content the AFTER text drops is damage
 only if the ORIGINAL supports it, and on a protected detail the ORIGINAL does
 support it, which is the whole reason the rule exists. Left unqualified the
 prober has two live rules that disagree, and the older one is the one written
 as a numbered rule.

 @example
 ```ts
 const clause = PROBE_HOUSE_RULE_CLAUSE;
 ```
 */
const PROBE_HOUSE_RULE_CLAUSE =
  `WHERE THE AFTER TEXT IS VAGUER THAN THE ORIGINAL BECAUSE A HOUSE RULE ASKS FOR IT, the verdict is no-introduced-defect-found, even though the ORIGINAL supports the wording that went missing. Where the AFTER text RESTORED a detail a house rule keeps out, the verdict is introduced-defect and the reason says which rule.`;

/**
 Composes the prober's system prompt for one edit kind.

 Wording follows the design review: the negative case is named as a search
 that found nothing rather than as a clean bill of health, and the four
 exclusions are stated as rules rather than as hints, because the pre-existing
 defect sitting in every region is the thing a general "review this" prompt
 reports first.

 Composed rather than substituted into a placeholder: the framing is prose
 built from a closed set, and a template with a marker in it is a small
 grammar that arbitrary text could later be interpolated into.

 @param editKind - which edit this run audits

 @returns System prompt with the matching framing

 @example
 ```ts
 const prompt = probeSystemPrompt({ editKind: 'accuracy-repair', declaresNames: false, },);
 ```
 */
function probeSystemPrompt(
  {
    editKind,
    declaresNames,
  }: {
    readonly editKind: ProbedEditKind;
    readonly declaresNames: boolean;
  },
): string {
  /**
   Rules for reading the declared names, where the page declares any
   (ledger H8).
   */
  const identityRules = declaresNames
    ? `\n\n${DECLARED_IDENTITY_RULES}\n- A declared name the AFTER text uses to refer to its entity is never an introduced defect.`
    : '';
  return `You are a strict bilingual translation reviewer auditing an edit for collateral damage.
${PROBE_FRAMING[editKind]}
${PROBE_RULES_HEAD}
${PROBE_CREATED_CLAUSE[editKind]}

${MEASUREMENT_POLICY_BLOCK}

${PROBE_HOUSE_RULE_CLAUSE}${identityRules}

${PROBE_RULES_TAIL}`;
}

/**
 Messages plus the region order checks resolve through:
 region number N on the wire means `envelopeIds[N - 1]`.

 @example
 ```ts
 const plan: IntroducedDefectPromptPlan = buildIntroducedDefectMessages({ ... },);
 ```
 */
export type IntroducedDefectPromptPlan = {
  /**
   Messages ready for `chatJson`.
   */
  readonly messages: readonly ChatMessage[];

  /**
   Envelope ids in prompt numbering order.
   */
  readonly envelopeIds: readonly string[];
};

/**
 Whether a prober is shown the accepted issues its region was cut for.

 `rendered` was the only behaviour until it was measured. Listing the issues
 and forbidding a prober from re-reporting them silenced the stage: across 45
 verdicts on regions a reader called damaged it raised 2 admissible claims,
 and the same regions with the list withheld raised 18. It answered
 undamaged regions the same way, 0.033 against 0.044, so its output barely
 depended on its input.

 `withheld` moves that defence to `introduced-defect-screen.ts`, which
 dismisses a claim quoting wording an accepted issue already complained
 about. The prober then reads the text without being told what to excuse, and
 the excusing happens where it can be checked.

 @example
 ```ts
 const disclosure: PriorIssueDisclosure = 'withheld';
 ```
 */
export type PriorIssueDisclosure = 'rendered' | 'withheld';

/**
 Disclosure the production stages send, for every instrument that claims to
 measure the production condition.

 ONE CONSTANT, so an instrument's "production" arm cannot drift from what the
 pass sends. Two instruments were once found labelling an arm "shown" while
 relying on a default that had meanwhile flipped to `withheld`, so their
 closing notes described a prompt effect that was a screen effect.

 @example
 ```ts
 disclosure: PRODUCTION_PRIOR_ISSUE_DISCLOSURE,
 ```
 */
export const PRODUCTION_PRIOR_ISSUE_DISCLOSURE: PriorIssueDisclosure = 'withheld';

/**
 Renders the pre-existing issues a region was cut for, so a prober can
 recognise and discount them.

 @param region - region whose served issues are named

 @param issues - accepted issues of the chunk

 @returns Claim summary lines, or a line saying none were resolvable

 @example
 ```ts
 const lines = renderPriorIssues({ region, issues, },);
 ```
 */
function renderPriorIssues(
  {
    region,
    issues,
  }: {
    readonly region: RepairRegion;
    readonly issues: readonly AdjudicatedIssue[];
  },
): string {
  /**
   Summary lines of every accepted issue this region served.
   */
  const lines = issues
    .filter(function isServed(issue,) {
      return region.issueIds
        .includes(issue.issueId,);
    },)
    .flatMap(function toLines(issue,) {
      return issue.claims
        .map(function toLine(member,) {
          return `- (${member.claim
            .category}, ${issue.severity}): ${flattenSpace({ text: member.claim
              .summary, },)}`;
        },);
    },);
  if (lines.length === 0)
    return '- (none recorded)';
  return lines.join('\n',);
}

/**
 What the neighbouring blocks are for, stated inside the probe sheet.

 THE PROBE IS AN AUDITOR, so its rule differs from the editor's: it is not
 being asked to change anything, only to say what the edit broke. What the
 neighbours buy it is the ability to tell a passage that MOVED from one that
 was invented or lost, which is the distinction a reading of shipped damage found it making wrongly
 by never making it at all.
 */
const NEARBY_RULE = 'THE TWO NEARBY BLOCKS ARE CONTEXT, not text under review. '
  + 'Use them to tell whether wording that looks added or dropped here belongs '
  + 'to a neighbouring passage instead of being new or lost';

/**
 Builds the prober sheet: the original, the baseline translation, and every
 replaced region with the pre-existing defects it was meant to fix.

 Both whole texts appear once rather than per region, so a prober can see
 where a region sits without the sheet repeating the chunk for every edit.

 @param sourceText - original chunk text

 @param baselineText - translation as it stood before any replacement

 @param regions - replaced regions in prompt numbering order

 @param issues - accepted issues of the chunk, for the pre-existing lists

 @param editKind - which edit is being audited; defaults to the accuracy
 repair so the stage this prompt was written for is unchanged

 @returns Messages plus region numbering order

 @example
 ```ts
 const plan = buildIntroducedDefectMessages({ sourceText, baselineText, regions, issues, },);
 ```
 */
export function buildIntroducedDefectMessages(
  {
    sourceText,
    baselineText,
    regions,
    issues,
    editKind = 'accuracy-repair',
    disclosure = 'rendered',
    neighbouringIncumbentText,
    neighbouringSourceText,
    identityContext,
  }: {
    readonly sourceText: string;
    readonly baselineText: string;
    readonly regions: readonly RepairRegion[];
    readonly issues: readonly AdjudicatedIssue[];
    readonly editKind?: ProbedEditKind;
    readonly disclosure?: PriorIssueDisclosure;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly identityContext?: string;
  },
): IntroducedDefectPromptPlan {
  /**
   The baseline, the window and every region without the notes between
   translators, as the prober reads them (`introduced-defect-shown.ts`).
   */
  const {
    baseline: baselineShown,
    incumbent: incumbentShown,
    regions: regionsShown,
  } = proberShownTexts({
    sourceText,
    baselineText,
    regions,
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
  },);

  /**
   Fence no enclosed text can reproduce.

   Chosen against every text this sheet carries rather than fixed, because all
   of them are arbitrary prose: a setext heading underline is an ordinary row
   of equals signs, and a translation may contain one. A fixed fence would let
   a region close its own block and have the rest of its text read as sheet
   structure, which is how a corpus paragraph gets to invent a region. Each
   text counts as written and as shown, since cutting a note can join two runs
   of fence characters.
   */
  const fence = selectFence({
    texts: [
      sourceText,
      baselineText,
      baselineShown,
      // THE NEIGHBOURS COUNT TOWARDS THE FENCE like every other text this sheet
      // carries. They are arbitrary corpus prose, so a fence chosen without
      // them could be one a neighbouring paragraph reproduces, and that
      // paragraph would close its own block and have the rest read as sheet
      // structure. Adding text to the sheet without adding it here is exactly
      // the hole the dynamic fence exists to close.
      neighbouringSourceText ?? '',
      neighbouringIncumbentText ?? '',
      incumbentShown,
      identityContext ?? '',
      ...regions.flatMap(function toTexts(region,) {
        return [
          region.before,
          region.editorAfter,
        ];
      },),
      ...regionsShown.flatMap(function toShownTexts(region,) {
        return [
          region.before,
          region.editorAfter,
        ];
      },),
    ],
  },);

  /**
   The passages either side, or nothing when this slice stands alone.

   A MEASUREMENT IS WHY THIS IS HERE. Probed against `lintong`'s duplicated
   farewell, this stage reported nothing, and it could not have reported
   otherwise: it compares one region against itself, and the wording it should
   have objected to sits in the slice next door. A prober that can see the
   neighbour can at least recognise a passage that has moved.
   */
  const nearbyBlock = ((neighbouringSourceText === undefined)
    || (neighbouringSourceText === ''))
    && (incumbentShown === '')
    ? ''
    : `${fence} NEARBY ORIGINAL, CONTEXT ONLY ${fence}
${neighbouringSourceText ?? ''}
${fence} NEARBY EXISTING TRANSLATION, CONTEXT ONLY ${fence}
${incumbentShown}
${fence} ${NEARBY_RULE} ${fence}
`;

  /**
   Rendered region blocks in numbering order.
   */
  const blocks = regionsShown.map(function toBlock(
    {
      region,
      before,
      editorAfter,
    },
    index,
  ) {
    return `${fence} REGION ${index + 1} ${fence}${
      disclosure === 'rendered'
        ? `
PRE-EXISTING DEFECTS THIS EDIT TARGETED (these are NOT your findings):
${renderPriorIssues({
          region,
          issues,
        },)}`
        : ''
    }
${fence} BEFORE ${String(index + 1,)} ${fence}
${before}
${fence} AFTER ${String(index + 1,)} ${fence}
${editorAfter}`;
  },);

  /**
   Declared names, or nothing on a page declaring none (ledger H8).
   */
  const identityBlock = declaredNamesBlock({
    fence,
    ...((identityContext === undefined) ? {} : { identityContext, }),
  },);

  /**
   Community renderings each region's BEFORE or AFTER text lacks, so an edit
   losing the community word reads as a loss (ledger H8).
   */
  const communityBlock = communityRenderingsBlock({
    sourceText,
    candidates: regionsShown.flatMap(function toCandidates(
      region,
      index,
    ): readonly RenderingCandidate[] {
      return [
        {
          label: `BEFORE ${String(index + 1,)}`,
          text: region.before,
        },
        {
          label: `AFTER ${String(index + 1,)}`,
          text: region.editorAfter,
        },
      ];
    },),
  },)
    .join('\n',);

  return {
    messages: [
      {
        role: 'system',
        content: probeSystemPrompt({
          editKind,
          declaresNames: identityBlock !== '',
        },),
      },
      {
        role: 'user',
        content: `${identityBlock}${fence} ORIGINAL ${fence}
${sourceText}
${fence} BASELINE TRANSLATION ${fence}
${baselineShown}
${nearbyBlock}${fence} REPLACED REGIONS ${fence}
${blocks.join('\n\n',)}
${communityBlock}${fence} END ${fence}`,
      },
    ],
    envelopeIds: regions.map(function toId(region,) {
      return region.envelopeId;
    },),
  };
}

/**
 One check as a prober reports it.

 @example
 ```ts
 const wire: IntroducedDefectCheckWire = { region: 1, verdict: 'uncertain', ... };
 ```
 */
export type IntroducedDefectCheckWire = {
  /**
   One-based region number from the prompt sheet.
   */
  readonly region: number;

  /**
   Verdict string; validated against the closed vocabulary at resolution.
   */
  readonly verdict: string;

  /**
   Defect class in the prober's own words; free text, telemetry only.
   */
  readonly category: string;

  /**
   Claimed severity; free text, validated where it is read.
   */
  readonly severity: string;

  /**
   Wording quoted from the AFTER text, which the screen checks against the
   baseline before the claim counts as corroborated.
   */
  readonly evidence: string;

  /**
   Wording quoted from the BEFORE text that the edit dropped.

   Omission damage has no wording in the AFTER text to point at, since its
   absence IS the defect, so a probe accepting only forward quotes could never
   corroborate the likeliest kind of collateral damage an editor causes.
   */
  readonly omittedText: string;

  /**
   Why the BEFORE text did not carry this defect.

   NAMED `verdictReason` so that it sorts after `verdict`, the one place the owner's ruling of 2026-10-21 ("Reason first
   everywhere") leaves the reason last (ledger B732, B762).
   */
  readonly verdictReason: string;
};

/**
 Whole prober reply on the wire.

 @example
 ```ts
 const report: IntroducedDefectReportWire = { checks: [], };
 ```
 */
export type IntroducedDefectReportWire = {
  /**
   Every check cast.
   */
  readonly checks: readonly IntroducedDefectCheckWire[];
};

/**
 Guards one wire check.

 @param value - candidate from parsed model JSON

 @returns Whether value carries the required check fields

 @example
 ```ts
 isIntroducedDefectCheckWire({ region: 1, verdict: 'uncertain', ... },);
 ```
 */
function isIntroducedDefectCheckWire(value: unknown,): value is IntroducedDefectCheckWire {
  if (!isJsonRecord(value,))
    return false;

  /**
   Region reference as reported; integerness checked on the primitive copy.
   */
  const { region, } = value;
  if ((typeof region) !== 'number')
    return false;
  if ((region % 1) !== 0)
    return false;
  return [
    'verdict',
    'category',
    'severity',
    'evidence',
    'omittedText',
    'verdictReason',
  ].every(function isString(field,) {
    return (typeof value[field]) === 'string';
  },);
}

/**
 Guards a whole prober reply.

 @param value - parsed model JSON

 @returns Whether value is a wire report

 @example
 ```ts
 const outcome = await client.chatJson({ ..., validate: isIntroducedDefectReportWire, },);
 ```
 */
export function isIntroducedDefectReportWire(value: unknown,): value is IntroducedDefectReportWire {
  if (!isJsonRecord(value,))
    return false;
  if (!isJsonArray(value.checks,))
    return false;
  return value.checks
    .every(function eachCheck(check,) {
      return isIntroducedDefectCheckWire(check,);
    },);
}

//endregion Introduced-defect probe wire
