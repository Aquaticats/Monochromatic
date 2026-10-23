import type { ChatMessage, } from '@monochromatic-dev/module-llm-type/ts';

import { communityRenderingsBlock, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { JUDGE_LINE_STRUCTURE_CLAUSE, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { FRONT_MATTER_DECISION_RULE, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  isJsonRecord,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  CONTEST_POLICY,
  findingsOrNone,
  namesOneOf,
  readCandidateNames,
} from './contest-ballot-wire.ts';
import { contestSizeNote, } from './contest-size-note.ts';
import { selectFence, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  renderedBreakContract,
  renderedBreakFacts,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { laneContestTenseGroups, } from './lane-contest-tense.ts';
import { tenseCandidateLines, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import {
  citedReferenceCandidateLines,
  type SliceSyntax,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';

//region Lane contest wire
// ASKS THE QUESTION THAT ACTUALLY SEPARATES THE TWO LANES, rather than which
// candidate reads better.
//
// WHY THAT DISTINCTION IS THE WHOLE DESIGN. Eight entries read complete against
// the Chinese, in `doc/audit/eight-entries-read-against-the-original.md`, found
// the lanes failing in opposite directions rather than at different points on
// one scale. The repair lane edits the archive, so an invention no critic
// flagged ships untouched. The translate lane owes the archive nothing, so
// accurate detail the archive knew and the source does not carry is lost, and
// where content lives only in pictures it has nothing to translate at all.
//
// A judge asked "which is better?" is being asked to weigh those against each
// other with nothing to weigh them WITH, which is consistent with the 82%
// decline rate the scratch lane-contest instrument measured. So this asks for
// the two findings that decide it, per candidate:
//
//   UNSUPPORTED: does this candidate say something the original does not?
//   DROPPED: does this candidate omit something the original does say?
//
// and takes the choice as a consequence of those, with the candidate's own
// index named so a caller can act on it.
//
// THE ORIGINAL IS THE STANDARD, per `doc/decision/translation-repair-output-goal.md`.
// The archive rendering is shown as EVIDENCE about what the original says and a
// starting point worth keeping where it is right, never as the thing a
// candidate is scored against. Accurate detail the archive adds is kept rather
// than stripped, which is the one place a candidate may exceed the original.

/**
 Candidate names a judge may use, which are the lanes plus the refusal.

 THE TYPE IS DERIVED FROM THIS LIST, so a name cannot be added to the type
 without being listed here, and the reply guard, the ballot reader and the
 artifact reader that import this list cannot fall behind it.
 */
export const CANDIDATE_NAMES = [
  'repair',
  'translate',
  'neither',
] as const;

/**
 Which candidate a judge chose, or that it could not choose.

 DECLINING IS A VERDICT, not a failure to answer. Two candidates that differ
 only in wording have no better one, and a judge forced to pick would be
 inventing a preference the evidence does not carry.

 @example
 ```ts
 const choice: LaneChoice = 'neither';
 ```
 */
export type LaneChoice = typeof CANDIDATE_NAMES[number];

/**
 Verdicts a judge may give the archive rendering.

 THE TYPE IS DERIVED FROM THIS LIST, for the reason {@link CANDIDATE_NAMES}
 gives.
 */
export const ARCHIVE_VERDICTS = [
  'publishable',
  'flawed',
] as const;

/**
 What a judge thinks of the archive rendering shown beside the candidates.

 ORTHOGONAL TO {@link LaneChoice}, not a fourth member of it. The choice asks
 which candidate to ship; this asks whether the text already published is fit
 to keep. A slice can have a clear winner AND a sound archive, or no winner
 and a sound archive, and one field cannot say both.

 TWO VALUES, matching the two questions the candidates are judged on. A judge
 that cannot tell omits the field rather than answering a third way, and the
 settling rule reads that absence as a voice that did not speak.

 @example
 ```ts
 const verdict: ArchiveVerdict = 'flawed';
 ```
 */
export type ArchiveVerdict = typeof ARCHIVE_VERDICTS[number];

/**
 One judge's reading of one contested slice.

 @example
 ```ts
 const ballot: LaneContestBallot = { choice: 'repair', unsupported: [], unsupportedRaw: [], dropped: [], droppedRaw: [], reason: 'x', };
 ```
 */
export type LaneContestBallot = {
  /**
   Candidate this judge would ship.
   */
  readonly choice: LaneChoice;

  /**
   Candidates saying something the original does not support.
   */
  readonly unsupported: readonly LaneChoice[];

  /**
   Unsupported findings exactly as this judge wrote them.

   KEPT BESIDE THE NARROWED LIST rather than instead of it. A judge that
   answers with the offending phrases rather than with candidate names has
   still said something, and keeping only the narrowed list would leave an
   audit trail reading as though that judge had found nothing.
   */
  readonly unsupportedRaw: readonly string[];

  /**
   Candidates omitting something the original says.
   */
  readonly dropped: readonly LaneChoice[];

  /**
   Dropped findings exactly as this judge wrote them.
   */
  readonly droppedRaw: readonly string[];

  /**
   Why, for the audit trail rather than for validity.
   */
  readonly reason: string;

  /**
   Whether this judge would publish the archive rendering as it stands.

   OPTIONAL BECAUSE ITS ABSENCE IS NOT A FAULT. The schema asks for it on
   every ballot, but a judge that omits it has still chosen between the
   candidates, and refusing the whole ballot over a missing archive answer
   would trade a lane verdict for an archive one.
   */
  readonly archive?: ArchiveVerdict;

  /**
   Seat that cast this ballot.

   RECORDED SO A SETTLED PAGE CAN SAY WHO JUDGED IT. hulicaijia26
   (2026-09-26) shipped a footnote whose translator's wordplay note four of
   five contest ballots called an addition, and neither the log, the artifact
   nor the stored replies named the models that cast them. OPTIONAL because a
   ballot read back from an artifact settled before this field existed has no
   seat to report, and the verdict never depends on it.
   */
  readonly modelId?: string;
};

/**
 Reply shape a judge is asked for, before it is read.
 */
export type LaneContestWire = {
  /**
   Why the judge answered as it did, written first. NAMED `assessment` so that it sorts first (ledger B762); the
   ballot built from it keeps the name `reason`.
   */
  readonly assessment: string;

  /**
   Findings of dropped content, a list of strings when the model followed
   the schema and whatever it wrote otherwise; the reader narrows (ledger
   B46).
   */
  readonly dropped: unknown;

  /**
   Findings against a candidate, read the same way.
   */
  readonly unsupported: unknown;

  /**
   Lane this judge would publish, a name the guard has already checked, so
   the reader takes it as it stands. NAMED `verdict` so that it sorts after the findings (ledger B762); the ballot
   built from it keeps the name `choice`.
   */
  readonly verdict: LaneChoice;

  /**
   Archive answer as the model wrote it, which the guard does not check and
   the reader narrows to a verdict. NAMED `verdictOnArchive` so that it sorts after `verdict` (ledger B762); the ballot
   built from it keeps the name `archive`.
   */
  readonly verdictOnArchive?: unknown;
};

/**
 Whether a value is one of the names a judge may use.

 @param value - candidate name from a reply

 @returns Whether it names a lane or the refusal

 @example
 ```ts
 const named = isLaneChoice('repair',);
 ```
 */
function isLaneChoice(value: unknown,): value is LaneChoice {
  return namesOneOf({
    value,
    names: CANDIDATE_NAMES,
  },);
}

/**
 Whether a value is one of the archive verdicts.

 @param value - archive answer from a reply

 @returns Whether it names a verdict

 @example
 ```ts
 const named = isArchiveVerdict('publishable',);
 ```
 */
function isArchiveVerdict(value: unknown,): value is ArchiveVerdict {
  return namesOneOf({
    value,
    names: ARCHIVE_VERDICTS,
  },);
}

/**
 Whether a reply carries the shape a ballot is read from.

 A READABLE CHOICE AND REASON ONLY. Whether the findings are consistent with
 the choice is the reader's question, because an inconsistent ballot is still
 a ballot that was cast and is worth recording as one; and since 2026-09-30
 their shape is too (ledger B46): findings missing or not a list of strings
 read as none, where they used to cost the whole ballot.

 @param value - parsed reply

 @returns Whether it can be read as a ballot

 @example
 ```ts
 const usable = isLaneContestWire(reply,);
 ```
 */
export function isLaneContestWire(value: unknown,): value is LaneContestWire {
  if (!isJsonRecord(value,))
    return false;
  return isLaneChoice(value.verdict,) && ((typeof value.assessment) === 'string');
}

/**
 Reads a validated reply as a ballot.

 @param wire - reply that passed the shape guard

 @returns Ballot with its findings narrowed

 @example
 ```ts
 const ballot = readLaneContestBallot({ wire, },);
 ```
 */
export function readLaneContestBallot(
  { wire, }: { readonly wire: LaneContestWire; },
): LaneContestBallot {
  /**
   Archive answer this judge gave, present only when it gave a readable one.

   SPREAD RATHER THAN SET TO UNDEFINED, because the property is optional
   under `exactOptionalPropertyTypes` and an explicit `undefined` would not
   typecheck against it.
   */
  const archive = isArchiveVerdict(wire.verdictOnArchive,)
    ? { archive: wire.verdictOnArchive, }
    : {};

  /**
   Findings against a candidate, none where the model wrote no list.
   */
  const unsupported = findingsOrNone(wire.unsupported,);

  /**
   Findings of dropped content, read the same way.
   */
  const dropped = findingsOrNone(wire.dropped,);

  return {
    ...archive,
    choice: wire.verdict,
    unsupported: readCandidateNames({
      findings: unsupported,
      names: CANDIDATE_NAMES,
    },),
    unsupportedRaw: unsupported,
    dropped: readCandidateNames({
      findings: dropped,
      names: CANDIDATE_NAMES,
    },),
    droppedRaw: dropped,
    reason: wire.assessment,
  };
}

/**
 What a judge is shown for one contested slice.
 */
export type LaneContestSubject = {
  /**
   Original passage, which is the standard.
   */
  readonly sourceText: string;

  /**
   Syntax role requiring dedicated decision rules.
   */
  readonly syntax?: SliceSyntax;

  /**
   Archive rendering, as evidence rather than as the standard.
   */
  readonly incumbentText: string;

  /**
   What the repair lane would ship.
   */
  readonly repairText: string;

  /**
   What the translate lane would ship.
   */
  readonly translateText: string;

  /**
   Candidates deterministic syntax guard has already made unpublishable.

   Judges still see bytes for comparison and cast raw ballots, but are told
   these names cannot be selected. Driver independently excludes violating
   choices, so prompt is liveness guidance rather than publication guard.
   */
  readonly ineligibleCandidates?: readonly ('archive' | 'repair' | 'translate')[];

  /**
   Names and handles both documents' front matter declares, when either does.

   WITHOUT THIS THE JUDGE CANNOT TELL AN ATTESTED NAME FROM AN INVENTION.
   Front matter is document-level while this stage sees one slice, so a name
   the source document declares appears, to a judge shown only the slice, in
   the archive and in the repair candidate and nowhere in the Chinese. Calling
   it unsupported is the correct inference from that evidence and the wrong
   answer about the passage. Measured on `Zha_Ke` slice 0, where the source
   front matter declares an alias, the archive renders it, the repair lane
   keeps it, the translate lane drops it, and the contest chose the lane that
   dropped it.

   Every other model-facing stage in this package is already given this:
   critics, refiners, translators, translate judges and the rendering audit.
   This stage was the only one that was not.
   */
  readonly identityContext?: string;

  /**
   Whether the original's entry records a death, which shows the judges the
   tense candidates of each text on offer (`tense-candidate-block.ts`).
   Absent for an entry that records none.
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

   WITHOUT THIS THE JUDGE CANNOT TELL ACCURATE ARCHIVE DETAIL FROM AN
   INVENTION. Mio20: the panel rejected "who is also trans" as an addition
   with the blog in front of it, the repair lane kept the clause, and three
   of five contest judges shown only the original called the repair
   candidate unsupported for it.
   */
  readonly referenceContext?: string;

  /**
   Which details of the archive rendering the repair lane's adjudicators
   accepted as additions, so a candidate leaving one out is not read as
   dropping page content (class one hundred eight, 2026-09-24).
   */
  readonly archiveDisputeNote?: string;

  /**
   Whether the line-structure rule governs this slice, so the judges read
   what a line-structured original asks (ledger S15). REQUIRED, NOT
   DEFAULTED: a default of false would hide every governed slice, the H2
   failure again.
   */
  readonly lineStructured: boolean;

  /**
   Corroborated added-damage claims the introduced-defect probe raised
   against the repair candidate, one line each, absent when none.

   EVIDENCE, NOT A VERDICT. The probe runs in shadow mode because its
   precision is measured at six of ten (2026-09-03), so nothing acts on a
   claim; the judges that choose between the lanes are shown it and check it
   against the ORIGINAL, which the contest that chose a tense-damaged repair
   lane 7 of 7 on keyword233 could not do without it.
   */
  readonly repairDamageClaims?: readonly string[];
};

/**
 Builds the exchange asking one judge to settle one contested slice.

 @param subject - passage, archive rendering and both candidates

 @returns Messages for one exchange

 @example
 ```ts
 const messages = buildLaneContestMessages({ subject, },);
 ```
 */
export function buildLaneContestMessages(
  { subject: held, }: { readonly subject: LaneContestSubject; },
): readonly ChatMessage[] {
  /**
   The subject as a judge reads it: the archive and both candidates without
   the notes the archive's translators left each other (XingZ60 slice 6: all
   three judges chose the lane that kept one). A reader never sees such a
   note, so it can neither win nor lose a vote; the original's own comments
   stay in view.
   */
  const subject: LaneContestSubject = {
    ...held,
    incumbentText: withoutTranslatorsNotes({
      text: held.incumbentText,
      original: held.sourceText,
    },),
    repairText: withoutTranslatorsNotes({
      text: held.repairText,
      original: held.sourceText,
    },),
    translateText: withoutTranslatorsNotes({
      text: held.translateText,
      original: held.sourceText,
    },),
  };
  /**
   Tense candidates of each text on offer, shown only where the entry records
   a death, after the passages with the other evidence about them.
   */
  const tenseGroups = laneContestTenseGroups({
    lifeEnded: subject.lifeEnded === true,
    ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
    incumbentText: subject.incumbentText,
    repairText: subject.repairText,
    translateText: subject.translateText,
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
      subject.repairText,
      subject.translateText,
      subject.identityContext ?? '',
      subject.referenceContext ?? '',
      subject.archiveDisputeNote ?? '',
      tenseLines.join('\n',),
    ],
  },);

  /**
   Declared names as one block, empty when neither side declares any.
   */
  const declared = subject.identityContext ?? '';

  /**
   Declared names and their fence, or nothing when neither side declares any.

   PLACED BEFORE THE PASSAGES, as in the critic prompt, so the declarations
   read as given facts rather than as a footnote to evidence already weighed.
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

   PLACED AFTER THE PASSAGES, like the damage claims: a judge reads the
   texts before the evidence about what they may carry.
   */
  const referenceBlock = citedReferenceCandidateLines({
    fence,
    referenceContext: subject.referenceContext ?? '',
  },);

  /**
   Accepted additions against the archive rendering, or nothing on an
   undisputed slice (class one hundred eight).

   PLACED AFTER THE PASSAGES, like the references: a judge reads the texts
   before the evidence about what they may carry.
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
   Size evidence, or nothing when every rendering is in proportion.
   */
  const sizeNote = contestSizeNote({
    sourceText: subject.sourceText,
    renderings: [
      {
        label: 'ARCHIVE RENDERING',
        text: subject.incumbentText,
      },
      {
        label: 'CANDIDATE "repair"',
        text: subject.repairText,
      },
      {
        label: 'CANDIDATE "translate"',
        text: subject.translateText,
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
   Deterministic syntax admission result, or nothing for ordinary contests.
   */
  const ineligible = subject.ineligibleCandidates ?? [];
  /**
   Guidance preventing panel from spending votes on candidates guard rejects.
   */
  const admissionBlock = (ineligible.length === 0)
    ? []
    : [
      `DETERMINISTIC PUBLICATION ADMISSION rejects: ${ineligible.join(', ')}.`,
      'Do not choose a rejected repair or translate candidate.',
      'Judge a rejected archive as flawed. If no eligible lane is faithful, choose neither.',
      'Raw ballots are audited and violating choices are excluded rather than redirected.',
      '',
    ];

  /**
   Probe claims against the repair candidate, or nothing when none were
   corroborated.
   */
  const damageClaims = subject.repairDamageClaims ?? [];
  /**
   Those claims as a block the judge reads after both candidates and before
   the sizes, or nothing at all.

   PLACED AFTER THE PASSAGES, like the size note, so a judge reads the texts
   before being handed a complaint about one of them.
   */
  const damageBlock = (damageClaims.length === 0)
    ? []
    : [
      'CORROBORATED ADDED-DAMAGE CLAIMS against CANDIDATE "repair", evidence to weigh, not a verdict:',
      ...damageClaims,
      'Each claim came from one prober auditing one of the two edits behind the repair candidate,',
      'named on its line: the accuracy repair, which started from the archive rendering, or the',
      'naturalness rewrite, which started from the repaired text. The quoted wording is in the repair',
      'candidate and absent from the text that edit started from. Probers are wrong about four claims',
      'in ten, so check each against the ORIGINAL yourself before it counts as unsupported wording.',
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
        label: 'CANDIDATE "repair"',
        text: subject.repairText,
      },
      {
        label: 'CANDIDATE "translate"',
        text: subject.translateText,
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
        label: 'CANDIDATE "repair"',
        text: subject.repairText,
      },
      {
        label: 'CANDIDATE "translate"',
        text: subject.translateText,
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
        ...admissionBlock,
        'ORIGINAL (Chinese), the standard:',
        `${fence}\n${subject.sourceText}\n${fence}`,
        '',
        'ARCHIVE RENDERING, evidence only:',
        `${fence}\n${subject.incumbentText}\n${fence}`,
        '',
        'CANDIDATE "repair":',
        `${fence}\n${subject.repairText}\n${fence}`,
        '',
        'CANDIDATE "translate":',
        `${fence}\n${subject.translateText}\n${fence}`,
        '',
        ...((breakFacts === '')
          ? []
          : [
            'RENDERED LINE STRUCTURE, measured facts:',
            `${fence}\n${breakFacts}\n${fence}`,
            '',
          ]),
        ...disputeBlock,
        ...referenceBlock,
        ...damageBlock,
        ...tenseBlock,
        ...communityBlock,
        ...sizeBlock,
        'SEPARATELY from that choice, judge the ARCHIVE RENDERING shown above.',
        'It is the text already published. Ask of it the same two questions you',
        'asked of the candidates: does it say anything the original does not',
        'support, and does it omit anything the original says? Answer',
        '"publishable" only if it passes both, and "flawed" otherwise. This',
        'answer does not name a candidate and does not affect your verdict.',
        '',
        'Return JSON: assessment one sentence, given before the verdict;',
        'dropped and unsupported each a list naming any of "repair", "translate";',
        'verdict one of "repair", "translate", "neither";',
        'verdictOnArchive one of "publishable", "flawed".',
      ].join('\n',),
    },
  ];
}

//endregion Lane contest wire
