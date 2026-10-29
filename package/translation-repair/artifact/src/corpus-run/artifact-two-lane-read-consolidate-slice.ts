import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireBoolean,
  requireCount,
  requireRecord,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ArtifactKeyVocabulary, } from '../artifact-key-vocabulary.ts';
import type {
  ArtifactConsolidateSlice,
  ArtifactConsolidationTerminal,
} from './artifact-two-lane-consolidate.ts';
import {
  parseGateBallot,
  parseShipped,
  parseVerdict,
} from './artifact-two-lane-read-consolidate-parts.ts';
import { parseRecordedTenseCandidates, } from './artifact-two-lane-read-tense.ts';
import {
  shippedTenseLabelOf,
  type TenseCandidateGroup,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import { parseConsolidationPolish, } from './artifact-two-lane-read-polish.ts';

//region Artifact consolidated slice read

/**
 Ways settlement can leave stage, including retired artifact spelling.

 KEYED BY `ArtifactConsolidationTerminal` rather than listed, so the compiler
 refuses a terminal the stage gains that this reader lacks, which would
 otherwise leave every artifact carrying it refused whole. The order is the
 order a refusal names them in.
 */
const TERMINAL_NAMES: Readonly<Record<ArtifactConsolidationTerminal, true>> = {
  'incumbent-only': true,
  'no-standing-text': true,
  'slate-endorsed-standing': true,
  'slate-unjudged-standing': true,
  'slate-declined-standing': true,
  'slate-kept-standing': true,
  'gate-kept-standing': true,
  'wrap-erased-difference': true,
  consolidated: true,
};

/**
 Whether a stored value names a way settlement can leave stage.

 @param value - terminal as the artifact carries it

 @returns Whether it is one of the names `TERMINAL_NAMES` holds

 @example
 ```ts
 const known = isTerminalName('consolidated',);
 ```
 */
function isTerminalName(value: unknown,): value is ArtifactConsolidationTerminal {
  return ((typeof value) === 'string')
    && Object.hasOwn(
      TERMINAL_NAMES,
      value,
    );
}

/**
 Reads what fidelity gate settled or that it was never asked.

 @param value - gate field

 @param path - artifact path

 @returns Parsed gate record

 @example
 ```ts
 const gate = parseGate({ value, path, });
 ```
 */
function parseGate(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactConsolidateSlice['gate'] {
  /**
   Gate field as record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  if (record.kind === 'not-asked') {
    requireExactKeys({
      record,
      allowed: ['kind',],
      path,
    },);
    return { kind: 'not-asked', };
  }
  if (record.kind !== 'asked') {
    throw new ArtifactParseError({
      path: `${path}.kind`,
      reason: 'one of asked, not-asked',
    },);
  }
  requireExactKeys({
    record,
    allowed: [
      'kind',
      'ballots',
      'usable',
    ],
    path,
  },);
  /**
   Every gate ballot.
   */
  const ballotRows = requireArray({
    value: record.ballots,
    path: `${path}.ballots`,
  },);
  /**
   Parsed gate ballots.
   */
  const ballots = ballotRows.map(function readOne(
    entry,
    at,
  ) {
    return parseGateBallot({
      value: entry,
      path: `${path}.ballots[${String(at,)}]`,
    },);
  },);
  /**
   Recorded usable voice count.
   */
  const usable = requireCount({
    value: record.usable,
    path: `${path}.usable`,
  },);
  if (usable !== ballots.length) {
    throw new ArtifactParseError({
      path: `${path}.usable`,
      reason: `${String(ballots.length,)} matching stored ${
        wordForCount({
          count: ballots.length,
          one: 'ballot',
          many: 'ballots',
        },)
      }`,
    },);
  }
  return {
    kind: 'asked',
    ballots,
    usable,
  };
}

/**
 Reads the mark of a slice that bought no slate because the slate bench read below its floor.

 THE MARK SAYS NO SLATE WAS ASKED, so a record carrying it must look like one: the terminal that keeps the standing
 with nothing asked, no verdicts and no gate. A mark on a slice that asked a slate is two records in one, and is
 refused rather than believed.

 @param value - recorded `benchBelowFloor` value, absent when the slice carries none

 @param terminal - how the slice left the stage, already read

 @param verdicts - structural verdicts the slice records, already read

 @param gate - what the gate settled, already read

 @param path - artifact path of the value, for refusals

 @returns Mark as a record fragment, empty when the slice carries none

 @throws {@link ArtifactParseError} when the value is anything but true, or the slice asked a slate

 @example
 ```ts
 const mark = parseBenchFloorMark({ value: undefined, terminal: 'incumbent-only', verdicts: [], gate, path, },);
 ```
 */
function parseBenchFloorMark(
  {
    value,
    terminal,
    verdicts,
    gate,
    path,
  }: {
    readonly value: unknown;
    readonly terminal: ArtifactConsolidationTerminal;
    readonly verdicts: ArtifactConsolidateSlice['verdicts'];
    readonly gate: ArtifactConsolidateSlice['gate'];
    readonly path: string;
  },
): { readonly benchBelowFloor?: true; } {
  if (value === undefined)
    return {};
  if (value !== true) {
    throw new ArtifactParseError({
      path,
      reason: 'true or absent',
    },);
  }
  /**
   What the record holds that a slice with no slate asked cannot.
   */
  const askedWhat = (terminal !== 'incumbent-only')
    ? `terminal ${terminal}`
    : ((verdicts.length > 0)
      ? `${String(verdicts.length,)} ${
        wordForCount({
          count: verdicts.length,
          one: 'verdict',
          many: 'verdicts',
        },)
      }`
      : ((gate.kind === 'asked') ? 'a gate that was asked' : undefined));
  if (askedWhat !== undefined) {
    throw new ArtifactParseError({
      path,
      reason: `a slice that asked no slate (terminal incumbent-only, no verdicts, no gate), rather than ${askedWhat}`,
    },);
  }
  return { benchBelowFloor: true, };
}

/**
 Reads the tense finding a consolidation slice records for the wording it ships.

 A SLICE THAT SHIPS NO WORDING OF ITS OWN (the archive kept, or nothing
 changed) has no shipped text to read a finding from; the contest record holds
 the candidates of what it leaves, so a finding here is refused rather than
 dropped unread.

 @param value - recorded `tenseCandidates` value, absent when the record carries none

 @param shipped - wording the slice ships, already read

 @param path - artifact path of the value, for refusals

 @returns The finding as a record fragment, empty when the slice carries none

 @throws {@link ArtifactParseError} when a slice that ships no wording of its own carries a finding, or when the finding
 is not a non-empty list of groups under the one label the sheets gave the shipped wording

 @example
 ```ts
 const tense = parseShippedTense({ value: undefined, shipped: { kind: 'unchanged', }, path: 'x.tenseCandidates', },);
 ```
 */
function parseShippedTense(
  {
    value,
    shipped,
    path,
  }: {
    readonly value: unknown;
    readonly shipped: ArtifactConsolidateSlice['shipped'];
    readonly path: string;
  },
): { readonly tenseCandidates?: readonly TenseCandidateGroup[]; } {
  if (value === undefined)
    return {};
  // The translate lane's winner is the contest record's translate candidate already, and its tense candidates are
  // recorded there.
  if ((shipped.kind !== 'consolidated') && (shipped.kind !== 'incumbent')) {
    throw new ArtifactParseError({
      path,
      reason: 'no tense candidates, since a slice that ships no wording of its own has no shipped text to read them from',
    },);
  }
  return {
    tenseCandidates: parseRecordedTenseCandidates({
      value,
      path,
      labels: [shippedTenseLabelOf({ kind: shipped.kind, }),],
    },),
  };
}

/**
 Reads one consolidated slice under generation-selected shape.

 @param value - slice record

 @param path - artifact path

 @param keys - generation key spellings

 @param polishRequired - whether generation records final body polish

 @param reviewRequired - whether polish record carries absolute naturalness review

 @param correctionChainRequired - whether review carries digest-bound corrections

 @param everyBodyBlockReviewed - whether reviewed paragraphs are every body
 block rather than the refinable paragraphs alone

 @param quorumBasisRequired - whether review rounds retain their wider quorum

 @param lastSeatGrace - whether review rounds were decided with the last-seat
 grace of 2026-10-22 (generation twenty on)

 @param tenseRecorded - whether the generation records the tense candidates of the shipped wording

 @param benchFloorRecorded - whether the generation records a slice kept for want of a slate bench

 @param translateWinnerRecorded - whether the generation records why the archive keeps a slice and the shipped kind of
 a translate lane winner

 @returns Parsed consolidation slice

 @example
 ```ts
 const slice = parseConsolidateSlice({ value, path, keys, polishRequired: true, });
 ```
 */
export function parseConsolidateSlice(
  {
    value,
    path,
    keys,
    polishRequired,
    reviewRequired = false,
    correctionChainRequired = false,
    everyBodyBlockReviewed = false,
    quorumBasisRequired = false,
    lastSeatGrace = false,
    tenseRecorded = false,
    benchFloorRecorded = false,
    translateWinnerRecorded = false,
  }: {
    readonly value: unknown;
    readonly path: string;
    readonly keys: ArtifactKeyVocabulary;
    readonly polishRequired: boolean;
    readonly reviewRequired?: boolean;
    readonly correctionChainRequired?: boolean;
    readonly everyBodyBlockReviewed?: boolean;
    readonly quorumBasisRequired?: boolean;
    readonly lastSeatGrace?: boolean;
    readonly tenseRecorded?: boolean;
    readonly benchFloorRecorded?: boolean;
    readonly translateWinnerRecorded?: boolean;
  },
): ArtifactConsolidateSlice {
  /**
   Slice as exact record.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      keys.sliceIndex,
      'terminal',
      'shipped',
      'rewrapped',
      'demoted',
      'verdicts',
      'gate',
      ...(polishRequired ? ['polish',] : []),
      ...(tenseRecorded ? ['tenseCandidates',] : []),
      ...(benchFloorRecorded ? ['benchBelowFloor',] : []),
    ],
    path,
  },);
  /**
   Terminal the artifact names, before it is known to be a name this reader
   holds.
   */
  const { terminal, } = record;
  if (!isTerminalName(terminal,)) {
    throw new ArtifactParseError({
      path: `${path}.terminal`,
      reason: `one of ${
        Object.keys(TERMINAL_NAMES,)
          .join(', ',)
      }`,
    },);
  }
  /**
   Unknown proposal verdict rows.
   */
  const verdictRows = requireArray({
    value: record.verdicts,
    path: `${path}.verdicts`,
  },);
  /**
   Parsed structural verdicts.
   */
  const verdicts = verdictRows.map(function readOne(
    entry,
    at,
  ) {
    return parseVerdict({
      value: entry,
      path: `${path}.verdicts[${String(at,)}]`,
    },);
  },);
  /**
   Slice index this record answers.
   */
  const sliceIndex = requireCount({
    value: record[keys.sliceIndex],
    path: `${path}.${keys.sliceIndex}`,
  },);
  /**
   Wording the slice ships, read before the tense finding that is about it.
   */
  const shipped = parseShipped({
    value: record.shipped,
    terminal,
    translateWinnerRecorded,
    path: `${path}.shipped`,
  },);
  /**
   The tense finding the record carries, none when it carries none.
   */
  const tense = parseShippedTense({
    value: record.tenseCandidates,
    shipped,
    path: `${path}.tenseCandidates`,
  },);
  /**
   What the gate settled, read here so the floor mark can be held against it.
   */
  const gate = parseGate({
    value: record.gate,
    path: `${path}.gate`,
  },);
  /**
   The floor mark the record carries, none when it carries none.
   */
  const floorMark = parseBenchFloorMark({
    value: record.benchBelowFloor,
    terminal,
    verdicts,
    gate,
    path: `${path}.benchBelowFloor`,
  },);
  return {
    sliceIndex,
    terminal,
    shipped,
    rewrapped: requireBoolean({
      value: record.rewrapped,
      path: `${path}.rewrapped`,
    },),
    demoted: requireBoolean({
      value: record.demoted,
      path: `${path}.demoted`,
    },),
    verdicts,
    gate,
    ...(polishRequired
      ? {
        polish: parseConsolidationPolish({
          value: record.polish,
          path: `${path}.polish`,
          reviewRequired,
          correctionChainRequired,
          everyBodyBlockReviewed,
          quorumBasisRequired,
          lastSeatGrace,
        },),
      }
      : {}),
    ...tense,
    ...floorMark,
  };
}

//endregion Artifact consolidated slice read
