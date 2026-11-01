import type {
  ConsistencyWinner,
  UnlocatedReason,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type { UnplacedReason, } from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Artifact consistency
// WHAT THE PAGE-LEVEL CONSISTENCY PASS DID TO THE SETTLED PAGE (the owner's
// ruling of 2026-10-11). After the slices settle, the pass finds source
// wording or a refrain the page repeats, asks the judge bench which rendering
// every place should follow, and carries the chosen rendering to the others
// where the floors allow. The artifact records every group it looked at, every
// ballot, and the text of each slice it changed, so the artifact and the
// published page agree and a reader can recount the vote.
//
// APPLIED BELOW THE PAGE ASSEMBLY AND ABOVE THE POLISH when a slice's text is
// read (`would-ship-text.ts`): the page passes run over the page the
// consistency pass composed, and what the pass wrote stands over the polish,
// the consolidation and the contest it settled after.
//
// THREE GENERATIONS, TWO SHAPES. Generation fifteen carries the section as the
// first pass to write it recorded it: ballots counted by the exact text of a
// quote, a located place carrying the judge's string, a slot a condition of
// being located, and the chosen words carried by code alone. It stays readable
// as written. GENERATION SIXTEEN is the second shape: ballots
// agree when their quotes locate one span of the place's own English, a located
// place carries the words and the offset the English holds, a slot is no
// condition of being located, a split with no strict majority is run off among the finalists
// (`verdict.runOff`), and the chosen rendering is carried by writers and a
// choice among candidates (`group.carries`). It stays readable as written too.
// GENERATION SEVENTEEN is the section the pass writes now: the second shape,
// a place that ended `overlapped` names the place it gave way to
// (`gaveWayTo`), which the reader resolves, and a located place records the
// single marks its English holds directly beside the span (`markBefore`,
// `markAfter`), which the reader compares when it recounts a splice; an apostrophe
// the locator proved belongs to its word, so the recorded words hold it and neither
// key does, and the reader refuses recorded words that start or end with any other
// single mark. The keys a
// generation does not write are refused in it by the reader, and the outcome
// words and winner reasons it does not give are refused too.

/**
 What became of one place of a group.

 `chosen` is the place every other place follows. `replaced` is a place whose
 slice now carries the chosen rendering. `consistent` is a place that already
 carried it. `kept-context` is a place a strict majority kept for its
 surroundings. `kept-floor` is a place whose replacement a floor refused.
 `overlapped` is a place whose replacement overlaps one of an earlier group in
 the same slice, or would change the words of a place another group located
 and does not itself replace in that slice, and was left out for it; or a
 place whose own located words a replacement that was made changed. It never
 names an overlap with a place of its own group, whose places in one block are
 carried by one replacement. From generation seventeen it carries `gaveWayTo`,
 the place of another group in the same slice that it gave way to. `unlocated` is a place whose rendering the ballots did not pin
 down. `untouched` is a place of a group that decided nothing. `kept-current`
 (generation sixteen) is a place whose carry ended with its own English
 standing: the choice among the candidates kept it, reached no decision, had no
 candidate to choose or failed.
 */
export type ConsistencyOccurrenceOutcome =
  | 'chosen'
    | 'replaced'
    | 'consistent'
    | 'kept-context'
    | 'kept-floor'
    | 'kept-current'
    | 'overlapped'
    | 'unlocated'
    | 'untouched';

/**
 Every outcome a place can have, keyed by the union so the compiler refuses a
 member the record lacks, for a reader checking a recorded one.
 */
export const CONSISTENCY_OCCURRENCE_OUTCOMES: Readonly<Record<ConsistencyOccurrenceOutcome, true>> = {
  chosen: true,
  replaced: true,
  consistent: true,
  'kept-context': true,
  'kept-floor': true,
  'kept-current': true,
  overlapped: true,
  unlocated: true,
  untouched: true,
};

/**
 Why the pass left a place as it was after the splice was chosen and the recorded words let code move the chosen words in.

 The five `gain-` reasons are those of a splice that carries an edge apostrophe into the place and was left out on the final
 page: `gain-pairs` (another quotation mark of the page could pair with the apostrophe), `gain-unproves` (making the splice
 would unprove the apostrophe a splice that stands carries in), `gain-floors` (the floors refuse the slice with the splice),
 `gain-overlaps` (its words overlap a replacement that is made) and `gain-refused` (the floors refused the other
 replacements of the slice, which keeps its wording whole). The three `edge-` reasons are those of a splice refused at the edge
 of the span on the English around it, or on the page: `edge-second-mark` (a single mark past a run of markup beyond the edge
 the splice brings an apostrophe to), `edge-reading-changes` (a single mark outside the words that would read differently) and
 `edge-reference-mark` (the splice brings an apostrophe, and the page holds a character reference that decodes to a
 quotation-like character, such as `&#39;` or `&rsquo;`, which no reader of the raw English sees as a mark).
 */
export type ConsistencyKeptReason =
  | 'gain-pairs'
    | 'gain-unproves'
    | 'gain-floors'
    | 'gain-overlaps'
    | 'gain-refused'
    | 'edge-second-mark'
    | 'edge-reading-changes'
    | 'edge-reference-mark';

/**
 Every reason a place can have been left as it was for, keyed by the union so the compiler refuses a member the record
 lacks, in the order a reader lists them.
 */
export const CONSISTENCY_KEPT_REASONS: Readonly<Record<ConsistencyKeptReason, true>> = {
  'gain-pairs': true,
  'gain-unproves': true,
  'gain-floors': true,
  'gain-overlaps': true,
  'gain-refused': true,
  'edge-second-mark': true,
  'edge-reading-changes': true,
  'edge-reference-mark': true,
};

/**
 Every reason a bench can go unasked, keyed by the union, for a reader.
 */
export const CONSISTENCY_NOT_ASKED_REASONS: Readonly<Record<ConsistencyNotAskedReason, true>> = {
  'too-few-placed': true,
  'no-english': true,
  consistent: true,
};

/**
 Why a group was not put to the bench.
 */
export type ConsistencyNotAskedReason = 'too-few-placed' | 'no-english' | 'consistent';

/**
 Where a place's rendering stands, as recorded.

 @example
 ```ts
 const rendering: ArtifactConsistencyRendering = { kind: 'unlocated', reason: 'not-found', };
 ```
 */
export type ArtifactConsistencyRendering =
  | {
    /**
     A strict majority quoted words that occur once in the place's English.
     */
    readonly kind: 'located';

    /**
     Generation fifteen: the words the majority quoted, as they quoted them.
     Generation sixteen: the words the place's English holds where their quotes
     stand, without the punctuation that closes them.
     */
    readonly quote: string;

    /**
     Generation sixteen: offset in the place's English where the span starts.
     */
    readonly start?: number;

    /**
     Generation seventeen: the single mark, one character, the place's English
     holds directly before the span, as the English writes it; absent where none
     stands there, and where the mark is a proven apostrophe, which the recorded
     words hold. Generations fifteen and sixteen record none.
     */
    readonly markBefore?: string;

    /**
     Generation seventeen: the single mark the place's English holds directly
     after the span; absent where none stands there, and where the mark is a
     proven apostrophe, which the recorded words hold.
     */
    readonly markAfter?: string;

    /**
     For a frame: the slot words. Generation fifteen: as the majority quoted
     them. Generation sixteen: as the English holds them inside the span, absent
     where the ballots did not pin a slot down.
     */
    readonly slot?: string;

    /**
     Generation eighteen, for a frame: the link and footnote definitions of the page that the slot was read with, the smallest set
     of them that gives the slot, present only where reading the span without them gives another slot or none (`a[b]c` is one
     word when `[b]` is defined in another slice, so the slot `b` is no stand). In the form a page's definitions are read in
     (`pageDefinitionsOf`). The file holds no page, so the recount reads the slot with these.
     */
    readonly definitions?: string;
  }
    | {
    /**
     The rendering could not be pinned down.
     */
      readonly kind: 'unlocated';

    /**
     Why.
     */
      readonly reason: UnlocatedReason;
    };

/**
 A place of a group of the section, named by where its group stands and its label.

 @example
 ```ts
 const name: ArtifactConsistencyPlaceName = { group: 1, label: 'OCCURRENCE 3', };
 ```
 */
export type ArtifactConsistencyPlaceName = {
  /**
   Position, from zero, of the place's group among the section's groups.
   */
  readonly group: number;

  /**
   Label the sheet gave the place within its group.
   */
  readonly label: string;
};

/**
 One place of a group.

 @example
 ```ts
 const occurrence: ArtifactConsistencyOccurrence = { label: 'OCCURRENCE 1', sliceIndex: 0, startOffset: 0, endOffset: 12, line: 1, rendering: { kind: 'unlocated', reason: 'not-asked', }, outcome: 'untouched', };
 ```
 */
export type ArtifactConsistencyOccurrence = {
  /**
   Label the sheet gave the place.
   */
  readonly label: string;

  /**
   Slice (the comparison row's index) holding the place.
   */
  readonly sliceIndex: number;

  /**
   Absolute offset in the original where the place starts.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive offset where it ends.
   */
  readonly endOffset: number;

  /**
   One-based line the place starts on.
   */
  readonly line: number;

  /**
   For a frame: absolute offset where the slot starts.
   */
  readonly slotStartOffset?: number;

  /**
   For a frame: absolute exclusive offset where the slot ends.
   */
  readonly slotEndOffset?: number;

  /**
   Where the place's rendering stands in its English.
   */
  readonly rendering: ArtifactConsistencyRendering;

  /**
   What became of the place.
   */
  readonly outcome: ConsistencyOccurrenceOutcome;

  /**
   Generation seventeen: for a place that ended `overlapped`, and no other, the
   place of another group in the same slice that it gave way to: the place of
   the replacement made over its words, or the place whose located words its
   replacement would have changed. Absent from every other place and from
   every place of an older generation.
   */
  readonly gaveWayTo?: ArtifactConsistencyPlaceName;

  /**
   Generation seventeen: for a place that ended `kept-current` because the pass left it as it was after the splice was
   chosen and the recorded words let code move the chosen words in, why. The reader cannot check the reason, which rests
   on English the record does not hold, but it accepts `kept-current` beside a replacement plan only with one from the
   closed set. Absent from every other place and from every place of an older generation.
   */
  readonly keptBecause?: ConsistencyKeptReason;
};

/**
 A place no slice holds whole, which stays as it stands.

 @example
 ```ts
 const unplaced: ArtifactConsistencyUnplaced = { startOffset: 22, endOffset: 34, line: 3, reason: 'in-no-slice', };
 ```
 */
export type ArtifactConsistencyUnplaced = {
  /**
   Absolute offset in the original where the place starts.
   */
  readonly startOffset: number;

  /**
   Absolute exclusive offset where it ends.
   */
  readonly endOffset: number;

  /**
   One-based line it starts on.
   */
  readonly line: number;

  /**
   Why no slice holds it whole.
   */
  readonly reason: UnplacedReason;
};

/**
 One heard ballot as recorded.

 @example
 ```ts
 const ballot: ArtifactConsistencyBallot = { seat: 'glm-5.3', reason: 'Same.', quotes: [], choice: 'OCCURRENCE 1', keep: [], };
 ```
 */
export type ArtifactConsistencyBallot = {
  /**
   Seat that answered.
   */
  readonly seat: string;

  /**
   Why the judge chose as it did, written before the choice.
   */
  readonly reason: string;

  /**
   The quotes the judge gave, one per place it quoted.
   */
  readonly quotes: readonly {
    /**
     Label of the place.
     */
    readonly label: string;

    /**
     Words quoted.
     */
    readonly quote: string;

    /**
     For a frame: the slot words quoted.
     */
    readonly slot?: string;
  }[];

  /**
   Label of the place every place should follow.
   */
  readonly choice: string;

  /**
   Labels of the places kept for their surroundings.
   */
  readonly keep: readonly string[];
};

/**
 One heard ballot that only chooses: the run-off's and the choice among a
 place's candidates.

 @example
 ```ts
 const ballot: ArtifactConsistencyChoiceBallot = { seat: 'glm-5.3', reason: 'Reads best.', choice: 'OCCURRENCE 3', };
 ```
 */
export type ArtifactConsistencyChoiceBallot = {
  /**
   Seat that answered.
   */
  readonly seat: string;

  /**
   Why the judge chose as it did, written before the choice.
   */
  readonly reason: string;

  /**
   Label of the place (a run-off) or of the candidate (a carry) chosen.
   */
  readonly choice: string;
};

/**
 The bench asked once more among the finalists of a first round that gave no strict majority.

 @example
 ```ts
 const runOff: ArtifactConsistencyRunOff = { finalists: ['OCCURRENCE 3', 'OCCURRENCE 4',], ballots: [], usable: 0, quorum: 2, };
 ```
 */
export type ArtifactConsistencyRunOff = {
  /**
   Labels of the finalists, in sheet order, the only places the run-off's
   sheet showed.
   */
  readonly finalists: readonly string[];

  /**
   Every ballot heard, whether or not it could be counted.
   */
  readonly ballots: readonly ArtifactConsistencyChoiceBallot[];

  /**
   Ballots that could be counted.
   */
  readonly usable: number;

  /**
   Voices the run-off's gather closed on.
   */
  readonly quorum: number;
};

/**
 What a candidate text of a carry is: the English the block has now, the chosen
 place's words moved into every receiving place of the block by code, or a text
 a writer wrote.
 */
export type ConsistencyCandidateKind = 'current' | 'splice' | 'writer';

/**
 Every kind a candidate can be, keyed by the union so the compiler refuses a
 member the record lacks, for a reader checking a recorded one.
 */
export const CONSISTENCY_CANDIDATE_KINDS: Readonly<Record<ConsistencyCandidateKind, true>> = {
  current: true,
  splice: true,
  writer: true,
};

/**
 One text the carry to the receiving places of a block chose among.

 @example
 ```ts
 const candidate: ArtifactConsistencyCandidate = { label: 'CANDIDATE 1', kind: 'current', writers: [], text: 'The cat naps.', };
 ```
 */
export type ArtifactConsistencyCandidate = {
  /**
   Label the choice sheet gave the candidate, counted from one in sheet order.
   */
  readonly label: string;

  /**
   `current` is the English the block has now; `splice` is the chosen place's
   words moved into every receiving place of the block by code; `writer` is a
   text a writer wrote.
   */
  readonly kind: ConsistencyCandidateKind;

  /**
   Seats whose reply was exactly this text, which is none for a splice no
   writer wrote and for a current text no writer returned.
   */
  readonly writers: readonly string[];

  /**
   The block's English as this candidate has it: the paragraph or heading that
   holds the receiving places' words, or the whole English of the slice for
   words that lie in no single block.
   */
  readonly text: string;
};

/**
 Why a carry reached no decision among its candidates.
 */
export type ConsistencyCarryNoDecisionReason = 'no-alternative' | 'no-quorum' | 'no-majority';

/**
 Every reason a carry can reach no decision, keyed by the union, for a reader.
 */
export const CONSISTENCY_CARRY_NO_DECISION_REASONS: Readonly<Record<ConsistencyCarryNoDecisionReason, true>> = {
  'no-alternative': true,
  'no-quorum': true,
  'no-majority': true,
};

/**
 What a receiving place's carry decided.

 @example
 ```ts
 const decision: ArtifactConsistencyCarryDecision = { kind: 'none', reason: 'no-alternative', };
 ```
 */
export type ArtifactConsistencyCarryDecision =
  | {
    /**
     A strict majority of the usable ballots chose a candidate.
     */
    readonly kind: 'chosen';

    /**
     Label of the candidate.
     */
    readonly label: string;
  }
    | {
    /**
     No candidate was chosen, so the place's English stands.
     */
      readonly kind: 'none';

    /**
     Why.
     */
      readonly reason: ConsistencyCarryNoDecisionReason;
    }
    | {
    /**
     Trouble stopped the carry before a choice could be counted.
     */
      readonly kind: 'failed';

    /**
     What went wrong, in words naming no corpus text.
     */
      readonly detail: string;
    };

/**
 Every kind of decision a carry can record, keyed by the union, for a reader.
 */
export const CONSISTENCY_CARRY_DECISION_KINDS: Readonly<Record<ArtifactConsistencyCarryDecision['kind'], true>> = {
  chosen: true,
  none: true,
  failed: true,
};

/**
 How the chosen rendering was carried to one receiving place: the texts
 offered, the ballots that chose among them and what they decided. The
 receiving places of one block are carried to together and share one choice, so
 each of them carries the same texts, ballots and decision under its own label.

 @example
 ```ts
 const carry: ArtifactConsistencyCarry = { label: 'OCCURRENCE 2', candidates: [], ballots: [], usable: 0, quorum: 2, decision: { kind: 'none', reason: 'no-alternative', }, };
 ```
 */
export type ArtifactConsistencyCarry = {
  /**
   Label of the receiving place.
   */
  readonly label: string;

  /**
   The texts the choice was among, in sheet order; the current English first
   when no alternative was offered.
   */
  readonly candidates: readonly ArtifactConsistencyCandidate[];

  /**
   Every ballot heard, whether or not it could be counted; none when the bench
   was not asked.
   */
  readonly ballots: readonly ArtifactConsistencyChoiceBallot[];

  /**
   Ballots that could be counted.
   */
  readonly usable: number;

  /**
   Voices the choice's gather closed on; zero when the bench was not asked.
   */
  readonly quorum: number;

  /**
   The bench asked once more among the finalists, present only when the first ballots reached their quorum
   and no candidate had a strict majority (the lead's adoption of 2026-10-15 of the owner's rule of 2026-10-11 for this ask,
   open to the owner's veto); its finalists are those candidates, its ballots are the second ask's, and `decision` is then
   what the run-off decided. A carry recorded before the rule has none, and a split it recorded stays as recorded.
   */
  readonly runOff?: ArtifactConsistencyRunOff;

  /**
   What the ballots decided, or the run-off when there was one.
   */
  readonly decision: ArtifactConsistencyCarryDecision;
};

/**
 What was asked of the bench about a group and what it answered.

 @example
 ```ts
 const verdict: ArtifactConsistencyVerdict = { kind: 'not-asked', reason: 'consistent', };
 ```
 */
export type ArtifactConsistencyVerdict =
  | {
    /**
     The bench was not asked.
     */
    readonly kind: 'not-asked';

    /**
     Why.
     */
    readonly reason: ConsistencyNotAskedReason;
  }
    | {
    /**
     Trouble stopped the group before the bench's answer could be counted;
     the group's places stay as they stand.
     */
      readonly kind: 'failed';

    /**
     What went wrong, in words naming no corpus text.
     */
      readonly detail: string;
    }
    | {
    /**
     The bench was asked.
     */
      readonly kind: 'asked';

    /**
     Every ballot heard, whether or not it could be counted.
     */
      readonly ballots: readonly ArtifactConsistencyBallot[];

    /**
     Ballots that could be counted.
     */
      readonly usable: number;

    /**
     Voices the gather closed on, which the usable ballots had to reach.
     */
      readonly quorum: number;

    /**
     The place every place followed, or why none. When the choices had
     finalists, the place the run-off chose, or the run-off's reason for none.
     */
      readonly winner: ConsistencyWinner;

    /**
     Generation sixteen: the bench asked once more among the finalists;
     absent when the choices had none.
     */
      readonly runOff?: ArtifactConsistencyRunOff;
    };

/**
 The repeated source wording or frame of a group.

 @example
 ```ts
 const repeat: ArtifactConsistencyRepeat = { kind: 'maximal', wording: '小猫在窗台上打盹', };
 ```
 */
export type ArtifactConsistencyRepeat =
  | {
    /**
     A run repeated as it stands.
     */
    readonly kind: 'maximal';

    /**
     The repeated run.
     */
    readonly wording: string;
  }
    | {
    /**
     A frame repeated with a varying slot.
     */
      readonly kind: 'refrain';

    /**
     Characters every place begins with.
     */
      readonly prefix: string;

    /**
     Characters every place ends with.
     */
      readonly suffix: string;
    };

/**
 One repeated wording and what the pass did about it.

 @example
 ```ts
 const group: ArtifactConsistencyGroup = { repeat: { kind: 'maximal', wording: '小猫在窗台上打盹', }, occurrences: [], unplaced: [], verdict: { kind: 'not-asked', reason: 'too-few-placed', }, };
 ```
 */
export type ArtifactConsistencyGroup = {
  /**
   The repeated wording or frame.
   */
  readonly repeat: ArtifactConsistencyRepeat;

  /**
   Places a slice holds whole, in page order.
   */
  readonly occurrences: readonly ArtifactConsistencyOccurrence[];

  /**
   Places no slice holds whole, which stay as they stand.
   */
  readonly unplaced: readonly ArtifactConsistencyUnplaced[];

  /**
   What was asked of the bench and what it answered.
   */
  readonly verdict: ArtifactConsistencyVerdict;

  /**
   Generation sixteen: how the chosen rendering was carried to each receiving
   place, in the order of the places, the places of one block sharing one
   record; empty where no place received one.
   */
  readonly carries?: readonly ArtifactConsistencyCarry[];
};

/**
 The text a changed slice now carries.

 @example
 ```ts
 const slice: ArtifactConsistencySlice = { sliceIndex: 3, text: 'The cat naps all day.', };
 ```
 */
export type ArtifactConsistencySlice = {
  /**
   Slice (the comparison row's index) that changed.
   */
  readonly sliceIndex: number;

  /**
   Text the slice carries, before the archive's typography is restored.
   */
  readonly text: string;
};

/**
 The consistency pass's outcome recorded in a settled artifact.

 @example
 ```ts
 const consistency: ArtifactConsistency = { groups: [], slices: [], findings: [], };
 ```
 */
export type ArtifactConsistency = {
  /**
   Every group the pass looked at, in the page order of its first place.
   */
  readonly groups: readonly ArtifactConsistencyGroup[];

  /**
   Slices whose text the pass changed, one row per slice.
   */
  readonly slices: readonly ArtifactConsistencySlice[];

  /**
   What the pass reports beside its groups: what the gathers reported, why a
   run-off was asked, what the carries reported (a writer's text a floor
   refused or that could not be read, writers that were not asked and why, a
   carry that failed), a replacement a floor refused, and trouble that stopped
   the pass.
   */
  readonly findings: readonly string[];
};

/**
 Consistency of an artifact whose pass changed nothing, and of every artifact
 written before the pass existed.
 */
export const NO_CONSISTENCY: ArtifactConsistency = {
  groups: [],
  slices: [],
  findings: [],
};

/**
 What the consistency pass says about one slice.

 @example
 ```ts
 const override: ConsistencyOverride = { kind: 'replaced', text: 'The cat naps.', };
 ```
 */
export type ConsistencyOverride =
  | {
    /**
     The page carries this text, the settled text with chosen wording carried in.
     */
    readonly kind: 'replaced';

    /**
     Text the page carries.
     */
    readonly text: string;
  }
    | {
    /**
     The pass left this slice as the stages settled it.
     */
      readonly kind: 'untouched';
    };

/**
 Reads what the consistency pass did to one slice.

 @param consistency - recorded section

 @param sliceIndex - slice asked about

 @returns The override, or that there is none

 @example
 ```ts
 const override = consistencyOverrideAt({ consistency, sliceIndex: 14, },);
 ```
 */
export function consistencyOverrideAt(
  {
    consistency,
    sliceIndex,
  }: {
    readonly consistency: ArtifactConsistency;
    readonly sliceIndex: number;
  },
): ConsistencyOverride {
  /**
   The recorded row for this slice, at most one: the reader refuses a file
   holding two.
   */
  const replaced = consistency
    .slices
    .find(function namesIt(row,): boolean {
      return row.sliceIndex === sliceIndex;
    },);
  if (replaced === undefined)
    return { kind: 'untouched', };
  return {
    kind: 'replaced',
    text: replaced.text,
  };
}

//endregion Artifact consistency
