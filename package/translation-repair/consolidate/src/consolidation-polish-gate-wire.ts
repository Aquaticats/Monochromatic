import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import {
  CANDIDATE_APPARATUS_RULE,
  CANDIDATE_DECLARED_NAMES_RULES,
  HOUSE_CORRECTION_IS_AN_IMPROVEMENT,
  JUDGE_LINE_STRUCTURE_CLAUSE,
  POLISH_GATE_HOUSE_RULES,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { citedReferenceCandidateLines, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { communityRenderingsBlock, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  CONTEST_REFUSAL,
  findingsOrNone,
  namesOneOf,
  readCandidateNames,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import {
  isJsonRecord,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  objectingJudgesOf,
  type ObjectionGroup,
  objectionsHeading,
  type RefineStageMode,
} from '@monochromatic-dev/translation-repair-repair/ts';
import { foldSoftBreaks, } from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consolidation polish gate wire

/**
 Names accepted in polish ballot.

 THE TYPE IS DERIVED FROM THIS LIST, so a name cannot be added to the type
 without being listed here, and the reply guard and the ballot reader cannot
 fall behind it.
 The tuple type is written out beside the value because without it
 `lint:types` reports TS9039 (type containing private name
 `CONTEST_REFUSAL`) and the build reports TS9013 (expression type cannot be
 inferred), both under `--isolatedDeclarations`, since the exported type is
 derived from this list. The compiler refuses either side without the other.
 */
export const POLISH_NAMES: readonly [
  'polished',
  'base',
  typeof CONTEST_REFUSAL,
] = [
  'polished',
  'base',
  CONTEST_REFUSAL,
];

/**
 Choice one naturalness judge may make.

 @example
 ```ts
 const choice: PolishChoice = 'polished';
 ```
 */
export type PolishChoice = typeof POLISH_NAMES[number];

/**
 Fidelity-first policy when approved base remains available, with the
 structure the polish must keep: the source's lines where the line rule
 governs the slice, and on prose the Markdown alone.

 PROSE LINE BREAKS ARE THE PAGE'S WRAP. Class one hundred fifty-two
 (yingying12, 2026-09-26): both candidates on a prose slice are wrapped at
 their semantic boundaries by the same rule before the gate
 (`consolidation-polish-round.ts`), so where they break lines follows from
 the wording and is nobody's choice; the sheet named "line structure" among
 what the polish may not change, and the polish replacing the calque "It is
 a pity that all this stopped abruptly" tied 2 to 2 with both base ballots
 citing it.

 SHOWN ONE LINE A PARAGRAPH. Class one hundred fifty-three (XingZ6014,
 2026-09-26): the base may stand as the archive's own unwrapped paragraph, so
 "both candidates by the same rule" was false on slices 36, 61 and 64 and three
 ballots still weighed the polish's added breaks. The sheet now shows both
 candidates folded (`soft-break-fold.ts`), and the policy says so.

 @param lineStructured - whether source line boundaries must survive

 @returns Policy for comparative gate

 @example
 ```ts
 const policy = comparativePolishPolicy({ lineStructured: false, },);
 ```
 */
function comparativePolishPolicy(
  { lineStructured, }: { readonly lineStructured: boolean; },
): string {
  /**
   What the polish must keep, and on prose what the judge must not weigh.
   */
  const kept = lineStructured
    ? 'Markdown structure, or line structure'
    : 'or Markdown structure. LINE BREAKS INSIDE A PARAGRAPH ARE THE PAGE\'S OWN WRAP and render as spaces, so each candidate\'s paragraphs are shown here on one line each: never weigh line breaks';
  return `You are deciding whether a polished English memorial passage may replace its already-approved base.

THE ORIGINAL CHINESE IS THE FIDELITY STANDARD. First check both candidates for unsupported statements and dropped content. Naturalness can never compensate for either fault.

Only if both candidates are equally faithful, judge natural English. Reject literal Chinese collocations, calqued verb-object combinations, stiff emotional descriptions, and grammar that a careful native editor would rewrite. Prefer polished only when it is clearly more idiomatic without changing meaning, detail, tone, names, links, ${kept}. Otherwise choose base. Answer neither when no clear naturalness improvement exists.

${HOUSE_CORRECTION_IS_AN_IMPROVEMENT}`;
}

/**
 Fidelity-first policy when judges objected to the base and it still ships
 as the fallback (owner, 2026-09-27, the fourteenth and fifteenth addenda of
 `doc/decision/translation-repair-ineligible-standing.md`).

 @param groups - objections by the judges they come from

 @returns Policy naming the objections as claims and the base as the fallback

 @example
 ```ts
 const policy = objectionCorrectionPolishPolicy({ groups, },);
 ```
 */
function objectionCorrectionPolishPolicy(
  { groups, }: { readonly groups: readonly ObjectionGroup[]; },
): string {
  return `You are deciding whether a correction may replace an English memorial passage the ${
    objectingJudgesOf({ groups, },)
  } objected to.

THE ORIGINAL CHINESE IS THE FIDELITY STANDARD. First check both candidates for unsupported statements and dropped content, and check each objection against the ORIGINAL: an objection is a claim, not a fact.

The base ships if you refuse the correction, with the objections recorded. Choose polished when it resolves an objection the ORIGINAL supports and adds, drops, softens or sharpens nothing else. Choose base when polished acts on an objection the ORIGINAL does not support, or changes anything the ORIGINAL does not ask for. Answer neither when polished resolves nothing the ORIGINAL supports.

${HOUSE_CORRECTION_IS_AN_IMPROVEMENT}`;
}

/**
 Subject shown to naturalness gate.

 @example
 ```ts
 const subject: ConsolidationPolishGateSubject = { sourceText: '猫睡了。', archiveText: 'The cat slept.', baseText: 'The cat slept.', polishedText: 'The cat was asleep.', mode: { kind: 'comparative' }, lineStructured: false };
 ```
 */
export type ConsolidationPolishGateSubject = {
  /**
   Original Chinese passage.
   */
  readonly sourceText: string;

  /**
   Archive wording as supporting evidence.
   */
  readonly archiveText: string;

  /**
   Standing wording, which ships unless the polish wins.
   */
  readonly baseText: string;

  /**
   Naturalness rewrite seeking to replace base.
   */
  readonly polishedText: string;

  /**
   Whether the polish explores freely or corrects what judges objected to.
   */
  readonly mode: RefineStageMode;

  /**
   Whether the source's line boundaries must survive; false on prose, whose
   line breaks are the page's wrap and which the gate reads folded one line a
   paragraph (classes one hundred fifty-two and fifty-three).
   */
  readonly lineStructured: boolean;

  /**
   Declared names and handles, when documents provide them.
   */
  readonly identityContext?: string;

  /**
   What the pages the original cites say, when it cites any, so the gate
   reads the attested details under the same rule the slate gate read them
   (class forty-one).
   */
  readonly referenceContext?: string;

  /**
   The accepted claims against the archive rendering on a disputed slice, the
   note the consolidate gate read before approving the base (ledger S12).
   */
  readonly archiveDisputeNote?: string;

  /**
   The claims the repair panel accepted about this slice, whether or not
   they dispute the archive, as the block `acceptedRepairClaimsNote` writes
   them: the evidence the producers wrote against, which a judge deciding a
   rewording of their text reads too (CuspariaKLSY21 slice 0, 2026-10-21), and
   which the polish writers and the judges of their rewrites read as well
   (ledger B805).
   Absent where the panel accepted nothing.
   */
  readonly acceptedRepairClaimsNote?: string;
};

/**
 Raw reply shape before candidate names are narrowed.
 */
export type ConsolidationPolishGateWire = {
  /**
   Why the judge answered as it did, written first. NAMED `assessment` so that it sorts first (ledger B762); the
   ballot built from it keeps the name `reason`.
   */
  readonly assessment: string;
  readonly dropped: unknown;
  readonly unsupported: unknown;

  /**
   Text this judge would ship, a name the guard has already checked, so the
   reader takes it as it stands. NAMED `verdict` so that it sorts after the findings (ledger B762); the ballot built
   from it keeps the name `choice`.
   */
  readonly verdict: PolishChoice;
};

/**
 Read naturalness ballot.
 */
export type ConsolidationPolishBallot = {
  readonly choice: PolishChoice;
  readonly unsupported: readonly PolishChoice[];
  readonly unsupportedRaw: readonly string[];
  readonly dropped: readonly PolishChoice[];
  readonly droppedRaw: readonly string[];
  readonly reason: string;
};

/**
 Narrows candidate name.

 @param value - reply candidate name

 @returns Whether value names polish candidate or refusal

 @example
 ```ts
 if (isPolishChoice(value)) use(value);
 ```
 */
function isPolishChoice(value: unknown,): value is PolishChoice {
  return namesOneOf({
    value,
    names: POLISH_NAMES,
  },);
}

/**
 Checks shape of polish gate reply.

 @param value - parsed provider value

 @returns Whether reply can be read as ballot

 @example
 ```ts
 const usable = isConsolidationPolishGateWire(value);
 ```
 */
export function isConsolidationPolishGateWire(
  value: unknown,
): value is ConsolidationPolishGateWire {
  if (!isJsonRecord(value,))
    return false;
  // THE FINDINGS ARE NOT CHECKED HERE, as the consolidate gate's are not: a
  // finding's shape never costs a voice (ledger B46), and the reader takes
  // each through `findingsOrNone`.
  return isPolishChoice(value.verdict,) && ((typeof value.assessment) === 'string');
}

/**
 Reads validated provider reply as naturalness ballot.

 @param wire - reply passing shape guard

 @returns Narrow ballot preserving raw findings

 @example
 ```ts
 const ballot = readConsolidationPolishBallot({ wire, });
 ```
 */
export function readConsolidationPolishBallot(
  { wire, }: { readonly wire: ConsolidationPolishGateWire; },
): ConsolidationPolishBallot {
  /**
   Unsupported findings, empty when model wrote another shape.
   */
  const unsupported = findingsOrNone(wire.unsupported,);
  /**
   Dropped findings, empty when model wrote another shape.
   */
  const dropped = findingsOrNone(wire.dropped,);
  return {
    choice: wire.verdict,
    unsupported: readCandidateNames({
      findings: unsupported,
      names: POLISH_NAMES,
    },),
    unsupportedRaw: unsupported,
    dropped: readCandidateNames({
      findings: dropped,
      names: POLISH_NAMES,
    },),
    droppedRaw: dropped,
    reason: wire.assessment,
  };
}

/**
 Builds fidelity-first final naturalness question.

 @param subject - original, archive, base and proposed polish

 @returns Messages for one polish judge

 @example
 ```ts
 const messages = buildConsolidationPolishGateMessages({ subject, });
 ```
 */
export function buildConsolidationPolishGateMessages(
  { subject: held, }: { readonly subject: ConsolidationPolishGateSubject; },
): readonly ChatMessage[] {
  /**
   The subject as a judge reads it: the archive, the base and the polish
   without the notes the archive's translators left each other, which a reader
   never sees and which therefore can neither win nor lose a vote. The
   original's own comments stay in view.
   */
  const subject: ConsolidationPolishGateSubject = {
    ...held,
    archiveText: withoutTranslatorsNotes({
      text: held.archiveText,
      original: held.sourceText,
    },),
    baseText: withoutTranslatorsNotes({
      text: held.baseText,
      original: held.sourceText,
    },),
    polishedText: withoutTranslatorsNotes({
      text: held.polishedText,
      original: held.sourceText,
    },),
  };
  /**
   Gate mode naming whether base remains available.
   */
  const { mode, } = subject;
  /**
   Whether this is exploratory comparison against approved base.
   */
  const comparative = mode.kind === 'comparative';
  /**
   What the gate or slate judges objected to, on an objection correction.
   */
  const objectionGroups = (mode.kind === 'objection-correction')
    ? mode.groups
    : [];
  /**
   Every objection, whichever judges raised it, for the fence.
   */
  const objections = objectionGroups.flatMap(function objectionsOf(group,): readonly string[] {
    return group.objections;
  },);
  /**
   Base as the judge reads it: on prose, each paragraph as it renders.
   */
  const shownBase = subject.lineStructured ? subject.baseText : foldSoftBreaks({ text: subject.baseText, },);
  /**
   Polish as the judge reads it: on prose, each paragraph as it renders.
   */
  const shownPolished = subject.lineStructured
    ? subject.polishedText
    : foldSoftBreaks({ text: subject.polishedText, },);
  /**
   Fence absent from every enclosed passage and finding.
   */
  const fence = selectFence({
    texts: [
      subject.sourceText,
      subject.archiveText,
      shownBase,
      shownPolished,
      ...objections,
      ...((subject.identityContext === undefined) ? [] : [subject.identityContext,]),
      ...((subject.referenceContext === undefined) ? [] : [subject.referenceContext,]),
      ...((subject.archiveDisputeNote === undefined) ? [] : [subject.archiveDisputeNote,]),
      ...((subject.acceptedRepairClaimsNote === undefined) ? [] : [subject.acceptedRepairClaimsNote,]),
    ],
  },);
  /**
   Why the archive rendering is disputed, fenced as the consolidate gate
   fences it, or nothing on an undisputed slice.
   */
  const disputeBlock = ((subject.archiveDisputeNote === undefined) || (subject.archiveDisputeNote === ''))
    ? []
    : [
      `${fence} ARCHIVE RENDERING DISPUTED ${fence}`,
      subject.archiveDisputeNote,
      fence,
      '',
    ];
  /**
   What the repair panel accepted about the slice, fenced as the dispute note
   is, or nothing where it accepted nothing.
   */
  const repairClaimsBlock = ((subject.acceptedRepairClaimsNote === undefined)
    || (subject.acceptedRepairClaimsNote === ''))
    ? []
    : [
      `${fence} REPAIR PANEL CLAIMS ACCEPTED ${fence}`,
      subject.acceptedRepairClaimsNote,
      fence,
      '',
    ];
  /**
   Community renderings a candidate lacks where the original carries the
   term, the evidence the consolidate gate weighs (ledger S12).
   */
  const communityBlock = communityRenderingsBlock({
    sourceText: subject.sourceText,
    candidates: [
      {
        label: 'ARCHIVE RENDERING',
        text: subject.archiveText,
      },
      {
        label: 'CANDIDATE "base"',
        text: subject.baseText,
      },
      {
        label: 'CANDIDATE "polished"',
        text: subject.polishedText,
      },
    ],
  },);
  /**
   Declared names fenced like every other enclosed text, or no lines.
   */
  const identity = (subject.identityContext === undefined)
    ? []
    : [
      'DECLARED NAMES:',
      `${fence}\n${subject.identityContext}\n${fence}`,
      '',
    ];
  /**
   The references and their rule, or nothing when the original cites nowhere.
   */
  const referenceBlock = citedReferenceCandidateLines({
    fence,
    ...((subject.referenceContext === undefined) ? {} : { referenceContext: subject.referenceContext, }),
  },);
  /**
   The judges' objections as quoted review data, absent on any other mode.
   */
  const objectionEvidence = objectionGroups.flatMap(function groupEvidence(group,): readonly string[] {
    return [
      `${objectionsHeading({ origin: group.origin, },)}:`,
      `${fence}\n${
        group.objections
          .map(function listed(objection,): string {
            return `- ${objection}`;
          },)
          .join('\n',)
      }\n${fence}`,
      '',
    ];
  },);
  /**
   Base label matching whether it remains publishable.
   */
  const baseLabel = comparative
    ? 'CANDIDATE "base" (already approved):'
    : 'CANDIDATE "base" (ships if the correction is refused):';
  /**
   Policy matching the mode.
   */
  const policy = (mode.kind === 'comparative')
    ? comparativePolishPolicy({ lineStructured: subject.lineStructured, },)
    : objectionCorrectionPolishPolicy({ groups: mode.groups, },);
  return [
    {
      role: 'system',
      // The page rules every other candidate sheet reads (ledger S12), the
      // line rule where it governs in every mode (ledger S15), then the house
      // rules.
      content: [
        policy,
        CANDIDATE_APPARATUS_RULE,
        CANDIDATE_DECLARED_NAMES_RULES,
        subject.lineStructured ? JUDGE_LINE_STRUCTURE_CLAUSE : '',
        POLISH_GATE_HOUSE_RULES,
      ]
        .filter(function hasRule(part,): boolean {
          return part !== '';
        },)
        .join('\n\n',),
    },
    {
      role: 'user',
      content: [
        ...identity,
        'ORIGINAL (Chinese):',
        `${fence}\n${subject.sourceText}\n${fence}`,
        '',
        'ARCHIVE RENDERING (evidence only):',
        `${fence}\n${subject.archiveText}\n${fence}`,
        '',
        baseLabel,
        `${fence}\n${shownBase}\n${fence}`,
        '',
        'CANDIDATE "polished":',
        `${fence}\n${shownPolished}\n${fence}`,
        '',
        ...referenceBlock,
        ...disputeBlock,
        ...repairClaimsBlock,
        ...communityBlock,
        ...objectionEvidence,
        'Return JSON: assessment one sentence, given before the verdict;',
        'dropped and unsupported each a list naming any of "polished", "base";',
        `verdict one of "polished", "base", "${CONTEST_REFUSAL}".`,
      ].join('\n',),
    },
  ];
}

//endregion Consolidation polish gate wire
