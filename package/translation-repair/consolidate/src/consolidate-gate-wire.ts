import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import { communityRenderingsBlock, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  CONTEST_POLICY,
  CONTEST_REFUSAL,
  contestSizeNote,
  findingsOrNone,
  namesOneOf,
  readCandidateNames,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import { JUDGE_LINE_STRUCTURE_CLAUSE, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { FRONT_MATTER_DECISION_RULE, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  isJsonRecord,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  renderedBreakContract,
  renderedBreakFacts,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  citedReferenceCandidateLines,
  type SliceSyntax,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  ARCHIVE_TENSE_LABEL,
  CONSOLIDATED_TENSE_LABEL,
  STANDING_TENSE_LABEL,
  tenseCandidateGroups,
  tenseCandidateLines,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';

//region Consolidate gate wire
// Asks whether the rendering this run WROTE should replace the one that would
// otherwise ship.
//
// SAME QUESTION AS THE LANE CONTEST, different pair. `CONTEST_POLICY` is shared
// rather than restated, because the lane contest's 10 of 13 against the
// eight-entry reading, where a general-preference control managed 8, is a
// number about that wording.
//
// THE CONSOLIDATION IS NOT TRUSTED BECAUSE IT IS NEWEST. It is a third
// candidate produced by the same kind of instrument that produced the first
// two, and it can be worse. So it is put to the roster like any other.
//
// A TIE KEEPS THE STANDING TEXT, which the counting rule enforces rather than
// this sheet: see `consolidate-gate-stage.ts`. Changing what a reader sees needs
// more evidence than leaving it, and the translate wire already records why,
// that a reader who knows this archive should not see it churn.

/**
 Names a judge may use, which are the two renderings plus the refusal.

 THE TYPE IS DERIVED FROM THIS LIST, so a name cannot be added to the type
 without being listed here, and the reply guard and the ballot reader cannot
 fall behind it.
 The tuple type is written out beside the value because without it
 `lint:types` reports TS9039 (type containing private name
 `CONTEST_REFUSAL`) and the build reports TS9013 (expression type cannot be
 inferred), both under `--isolatedDeclarations`, since the exported type is
 derived from this list. The compiler refuses either side without the other.
 */
export const CONSOLIDATE_GATE_NAMES: readonly [
  'consolidated',
  'standing',
  typeof CONTEST_REFUSAL,
] = [
  'consolidated',
  'standing',
  CONTEST_REFUSAL,
];

/**
 Which rendering a judge would publish, or that it cannot choose.

 @example
 ```ts
 const choice: GateChoice = 'standing';
 ```
 */
export type GateChoice = typeof CONSOLIDATE_GATE_NAMES[number];

/**
 Whether a value is one of the names a judge may use.

 @param value - candidate name from a reply

 @returns Whether it names a rendering or the refusal

 @example
 ```ts
 const named = isGateChoice('standing',);
 ```
 */
export function isGateChoice(value: unknown,): value is GateChoice {
  return namesOneOf({
    value,
    names: CONSOLIDATE_GATE_NAMES,
  },);
}

/**
 One judge's reading of the consolidation against the standing text.

 @example
 ```ts
 const ballot: GateBallot = { choice: 'standing', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: 'x', };
 ```
 */
export type GateBallot = {
  /**
   Rendering this judge would publish.
   */
  readonly choice: GateChoice;

  /**
   Renderings saying something the original does not support.
   */
  readonly unsupported: readonly GateChoice[];

  /**
   Unsupported findings exactly as this judge wrote them.
   */
  readonly unsupportedRaw: readonly string[];

  /**
   Renderings omitting something the original says.
   */
  readonly dropped: readonly GateChoice[];

  /**
   Dropped findings exactly as this judge wrote them.
   */
  readonly droppedRaw: readonly string[];

  /**
   Why, for the audit trail rather than for validity.
   */
  readonly reason: string;
};

/**
 Reply shape a judge is asked for, before it is read.
 */
export type GateWire = {
  /**
   Why the judge answered as it did, written first. NAMED `assessment` so that it sorts first (ledger B762); the
   ballot built from it keeps the name `reason`.
   */
  readonly assessment: string;

  /**
   Findings of dropped content, a list of strings when the model
   followed the schema and whatever it wrote otherwise; the reader narrows.
   */
  readonly dropped: unknown;

  /**
   Findings against the consolidation, read the same way.
   */
  readonly unsupported: unknown;

  /**
   Rendering this judge would publish, a name the guard has already checked,
   so the reader takes it as it stands. NAMED `verdict` so that it sorts after the findings (ledger B762); the ballot
   built from it keeps the name `choice`.
   */
  readonly verdict: GateChoice;
};

/**
 Whether a reply carries the shape a ballot is read from.

 SHAPE ONLY, on the lane contest's rule: whether the findings are consistent
 with the choice is the reader's question, because an inconsistent ballot is
 still a ballot that was cast.

 @param value - parsed reply

 @returns Whether it can be read as a ballot

 @example
 ```ts
 const usable = isConsolidateGateWire(reply,);
 ```
 */
export function isConsolidateGateWire(value: unknown,): value is GateWire {
  if (!isJsonRecord(value,))
    return false;
  // THE FINDINGS ARE NOT CHECKED HERE. `contest-ballot-wire.ts` records that
  // no wording of a finding may cost a voice; a wrong TYPE, a `null` from a
  // model that ignored the schema, used to cost the whole ballot, choice
  // included, and until 2026-09-30 a findings key left out still did (ledger
  // B46). The reader takes each through `findingsOrNone` and keeps the choice.
  return isGateChoice(value.verdict,)
    && ((typeof value.assessment) === 'string');
}

/**
 Reads a validated reply as a ballot.

 @param wire - reply that passed the shape guard

 @returns Ballot with its findings narrowed

 @example
 ```ts
 const ballot = readConsolidateGateBallot({ wire, },);
 ```
 */
export function readConsolidateGateBallot(
  { wire, }: { readonly wire: GateWire; },
): GateBallot {
  /**
   Findings against the consolidation, empty where the model wrote no list.
   */
  const unsupported = findingsOrNone(wire.unsupported,);

  /**
   Findings of dropped content, read the same way.
   */
  const dropped = findingsOrNone(wire.dropped,);

  return {
    choice: wire.verdict,
    unsupported: readCandidateNames({
      findings: unsupported,
      names: CONSOLIDATE_GATE_NAMES,
    },),
    unsupportedRaw: unsupported,
    dropped: readCandidateNames({
      findings: dropped,
      names: CONSOLIDATE_GATE_NAMES,
    },),
    droppedRaw: dropped,
    reason: wire.assessment,
  };
}

/**
 What a judge is shown for one gated slice.
 */
export type ConsolidateGateSubject = {
  /**
   Original passage, which is the standard.
   */
  readonly sourceText: string;

  /**
   Syntax role requiring dedicated gate policy.
   */
  readonly syntax?: SliceSyntax;

  /**
   Archive rendering, as evidence rather than as the standard.
   */
  readonly incumbentText: string;

  /**
   Rendering this run wrote for this slice.
   */
  readonly consolidatedText: string;

  /**
   Rendering that ships if the consolidation is refused.
   */
  readonly standingText: string;

  /**
   Whether the line-structure rule governs this slice, so the judges read
   what a line-structured original asks (ledger S15). REQUIRED, NOT
   DEFAULTED: a default of false would hide every governed slice, the H2
   failure again.
   */
  readonly lineStructured: boolean;

  /**
   Why the deterministic publication rule refused the standing, when it
   did: a standing that cannot ship is shown as such, so refusing the
   consolidation is not mistaken for keeping something (class fifty-six,
   XingZ606 slice 33, 2026-09-18).
   */
  readonly standingRefusal?: string;

  /**
   Why the archive rendering shown is the repair lane's stand-in and which
   details are accepted additions, so keeping the stand-in is not the safe
   choice on their account (class one hundred eight, 2026-09-24).
   */
  readonly archiveDisputeNote?: string;

  /**
   Names and handles both documents' front matter declares, when either does.

   WITHOUT THIS THE JUDGE CANNOT TELL AN ATTESTED NAME FROM AN INVENTION,
   which is the defect measured on `Zha_Ke` slice 0 and fixed for every other
   model-facing stage.
   */
  readonly identityContext?: string;

  /**
   Whether the original's entry records a death, which shows the judge the tense
   candidates of each text on offer (`tense-candidate-block.ts`).
   */
  readonly lifeEnded?: boolean;

  /**
   Names the translation declares for its subject, its display name and its
   aliases, which the tense candidates read as named subjects. Absent when it
   declares none.
   */
  readonly subjectNames?: readonly string[];

  /**
   What the pages the original cites say, one line per page, when the
   original links anywhere (class thirty-six, 2026-09-16).
   */
  readonly referenceContext?: string;
};

/**
 Builds the exchange asking one judge to gate one consolidation.

 @param subject - passage, archive rendering and the two renderings

 @returns Messages for one exchange

 @example
 ```ts
 const messages = buildConsolidateGateMessages({ subject, },);
 ```
 */
export function buildConsolidateGateMessages(
  { subject: held, }: { readonly subject: ConsolidateGateSubject; },
): readonly ChatMessage[] {
  /**
   The subject as a judge reads it: the archive and both candidates without
   the notes the archive's translators left each other, which a reader never
   sees and which therefore can neither win nor lose a vote (XingZ60 slice 6:
   two of three ballots praised both candidates for keeping the archive's
   comment). The original's own comments stay in view.
   */
  const subject: ConsolidateGateSubject = {
    ...held,
    incumbentText: withoutTranslatorsNotes({
      text: held.incumbentText,
      original: held.sourceText,
    },),
    consolidatedText: withoutTranslatorsNotes({
      text: held.consolidatedText,
      original: held.sourceText,
    },),
    standingText: withoutTranslatorsNotes({
      text: held.standingText,
      original: held.sourceText,
    },),
  };
  /**
   Tense candidates of the texts on offer, shown only where the entry records a
   death, after the passages with the other evidence about them.
   */
  const tenseGroups = tenseCandidateGroups({
    lifeEnded: subject.lifeEnded === true,
    ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
    texts: [
      {
        label: ARCHIVE_TENSE_LABEL,
        text: subject.incumbentText,
      },
      {
        label: CONSOLIDATED_TENSE_LABEL,
        text: subject.consolidatedText,
      },
      {
        label: STANDING_TENSE_LABEL,
        text: subject.standingText,
      },
    ],
  },);
  /**
   Lines the tense block encloses, which are the sentences as the detector
   reads them (marks dropped, links unwrapped) and so can hold a longer run of
   the fence character than any text they were read from.
   */
  const tenseLines = tenseCandidateLines({ groups: tenseGroups, },);
  /**
   Fence long enough to enclose every text without one closing early.
   */
  const fence = selectFence({
    texts: [
      subject.sourceText,
      subject.incumbentText,
      subject.consolidatedText,
      subject.standingText,
      subject.identityContext ?? '',
      subject.referenceContext ?? '',
      subject.standingRefusal ?? '',
      subject.archiveDisputeNote ?? '',
      tenseLines.join('\n',),
    ],
  },);
  /**
   What the judge is told when the standing cannot ship, or nothing.
   */
  const refusalBlock = (subject.standingRefusal === undefined)
    ? []
    : [
      `CANDIDATE "standing" CANNOT SHIP: the deterministic publication rule refused it (${
        subject.standingRefusal
      }).`,
      '"consolidated" ships whichever you choose, and your ballot records what you hold against it. Choose '
        + '"standing" only when "consolidated" misrepresents the ORIGINAL, and name how in your findings; a '
        + 'rendering the rule refuses is not the safer choice.',
      '',
    ];
  /**
   Why the archive rendering and the standing are the repair lane's stand-in,
   or nothing on an undisputed slice (class one hundred eight).
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
   Declared names as one block, empty when neither side declares any.
   */
  const declared = subject.identityContext ?? '';

  /**
   Declared names and their fence, or nothing when neither side declares any.
   */
  const identityBlock = (declared.length === 0)
    ? []
    : [
      `${fence} DECLARED NAMES ${fence}`,
      declared,
      fence,
      '',
    ];
  /**
   The pages the original cites and their rule, or nothing when it cites
   none.
   */
  const referenceBlock = citedReferenceCandidateLines({
    fence,
    referenceContext: subject.referenceContext ?? '',
  },);
  /**
   Size evidence, or nothing when every rendering is in proportion.
   */
  const sizeNote = contestSizeNote({
    sourceText: subject.sourceText,
    renderings: [
      {
        label: ARCHIVE_TENSE_LABEL,
        text: subject.incumbentText,
      },
      {
        label: CONSOLIDATED_TENSE_LABEL,
        text: subject.consolidatedText,
      },
      {
        label: STANDING_TENSE_LABEL,
        text: subject.standingText,
      },
    ],
  },);

  /**
   Size note and its separating blank line, or nothing at all.

   PLACED AFTER THE PASSAGES so a judge reads the texts before their
   sizes, rather than being handed a number to confirm.
   */
  const sizeBlock = (sizeNote.length === 0)
    ? []
    : [
      sizeNote,
      '',
    ];

  /**
   The tense block, fenced like the other blocks, or nothing when no text has a candidate.
   */
  const tenseBlock = (tenseGroups.length === 0)
    ? []
    : [
      `${fence} TENSE CANDIDATES ${fence}`,
      ...tenseLines,
      fence,
      '',
    ];

  /**
   Community renderings a candidate lacks where the original carries the
   term (owner, 2026-09-09), evidence to weigh after the passages.
   */
  const communityBlock = communityRenderingsBlock({
    sourceText: subject.sourceText,
    candidates: [
      {
        label: 'ARCHIVE RENDERING',
        text: subject.incumbentText,
      },
      {
        label: 'CANDIDATE "consolidated"',
        text: subject.consolidatedText,
      },
      {
        label: 'CANDIDATE "standing"',
        text: subject.standingText,
      },
    ],
  },);

  /**
   Policy extended for syntax-bearing visible metadata.
   */
  const policy = [
    (subject.syntax === 'front-matter')
      ? `${CONTEST_POLICY}\n\n${FRONT_MATTER_DECISION_RULE}`
      : CONTEST_POLICY,
    subject.lineStructured ? JUDGE_LINE_STRUCTURE_CLAUSE : '',
    // THE INSTRUCTION ONLY: the measured counts ride in the user turn (R4).
    renderedBreakContract({
      sourceText: subject.sourceText,
      archiveText: subject.incumbentText,
      ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
    },),
  ]
    .filter(function hasPolicy(part,): boolean {
      return part !== '';
    },)
    .join('\n\n',);

  /**
   Measured break counts of the original and of both candidates, none where the instruction is not due.
   */
  const breakFacts = renderedBreakFacts({
    sourceText: subject.sourceText,
    archiveText: subject.incumbentText,
    ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
    renderings: [
      {
        label: 'CANDIDATE "consolidated"',
        text: subject.consolidatedText,
      },
      {
        label: 'CANDIDATE "standing"',
        text: subject.standingText,
      },
    ],
  },);

  return [
    {
      role: 'system',
      content: policy,
    },
    {
      role: 'user',
      content: [
        ...identityBlock,
        'ORIGINAL (Chinese), the standard:',
        `${fence}\n${subject.sourceText}\n${fence}`,
        '',
        'ARCHIVE RENDERING, evidence only:',
        `${fence}\n${subject.incumbentText}\n${fence}`,
        '',
        'CANDIDATE "consolidated":',
        `${fence}\n${subject.consolidatedText}\n${fence}`,
        '',
        'CANDIDATE "standing":',
        `${fence}\n${subject.standingText}\n${fence}`,
        '',
        ...((breakFacts === '')
          ? []
          : [
            'RENDERED LINE STRUCTURE, measured facts:',
            `${fence}\n${breakFacts}\n${fence}`,
            '',
          ]),
        ...refusalBlock,
        ...disputeBlock,
        ...referenceBlock,
        ...tenseBlock,
        ...communityBlock,
        ...sizeBlock,
        'Return JSON: assessment one sentence, given before the verdict;',
        'dropped and unsupported each a list naming any of "consolidated", "standing";',
        `verdict one of "consolidated", "standing", "${CONTEST_REFUSAL}".`,
      ].join('\n',),
    },
  ];
}

//endregion Consolidate gate wire
