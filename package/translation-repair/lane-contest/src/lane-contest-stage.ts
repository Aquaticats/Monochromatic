import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import type {
  JsonSchemaResponseFormat,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { contestResponseFormat, } from './contest-ballot-wire.ts';
import {
  ARCHIVE_VERDICTS,
  buildLaneContestMessages,
  CANDIDATE_NAMES,
  isLaneContestWire,
  type LaneChoice,
  type LaneContestBallot,
  type LaneContestSubject,
  readLaneContestBallot,
} from './lane-contest-wire.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type FanOutMode,
  runWindowedRounds,
  shortBenchStageFinding,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Lane contest stage
// SETTLES ONE CONTESTED SLICE by asking the roster which candidate the original
// supports, and counting the answers.
//
// NO SINGLE MODEL DECIDES, which is this package's rule wherever a model is
// asked anything. It matters here because the thing being decided is what a
// reader will see on a memorial page, with no later stage to catch a mistake.
//
// DECLINING IS COUNTED, NOT DISCARDED. A judge answering `neither` has said
// something: that the two candidates differ only in wording. Treating that as a
// lost voice would make an undecidable slice look like an unanswered one, and
// those need opposite handling.

/**
 Voices that must back a candidate before it is called the winner.

 TWO, matching every other agreement rule in this package. One judge is an
 opinion; two reaching the same reading of the same original is corroboration.

 FROZEN, like the version 2 comparison rules. The settled-artifact reader
 recomputes every recorded verdict against this number, so raising or lowering
 it re-decides every contest already on disk and makes artifacts settled under
 the old value refuse to parse. A different quorum is a different question and
 needs a new artifact generation, not a tuned constant.
 */
export const LANE_CONTEST_QUORUM = 2;

/**
 Schema a reply must satisfy before it reaches the reader.
 */
export const LANE_CONTEST_RESPONSE_FORMAT: JsonSchemaResponseFormat = contestResponseFormat({
  schemaName: 'lane_contest',
  verdicts: CANDIDATE_NAMES,
  asksArchive: true,
  archiveVerdicts: ARCHIVE_VERDICTS,
},);

/**
 What the roster settled on for one contested slice.

 @example
 ```ts
 const outcome: LaneContestOutcome = { choice: 'neither', ballots: [], usable: 0, findings: [], };
 ```
 */
export type LaneContestOutcome = {
  /**
   Candidate enough voices backed, or `neither`.
   */
  readonly choice: LaneChoice;

  /**
   Every usable ballot, for the audit trail.
   */
  readonly ballots: readonly LaneContestBallot[];

  /**
   Voices whose answer arrived and could be read as a ballot.

   ONE COUNT RATHER THAN TWO. Other stages separate voices heard from voices
   whose reply survived their reader, because those readers can refuse a
   well-shaped reply. This one cannot: anything passing the shape guard reads
   as a ballot, so a second count would always equal the first and invite a
   reader to believe otherwise.
   */
  readonly usable: number;

  /**
   What went wrong, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Counts how many ballots named each candidate.

 @param ballots - usable ballots

 @returns Voice count per candidate name

 @example
 ```ts
 const votes = countChoices({ ballots, },);
 ```
 */
function countChoices(
  { ballots, }: { readonly ballots: readonly LaneContestBallot[]; },
): ReadonlyMap<LaneChoice, number> {
  /**
   Votes so far.
   */
  const votes = new Map<LaneChoice, number>();
  for (const ballot of ballots)
    votes.set(
      ballot.choice,
      (votes.get(ballot.choice,) ?? 0) + 1,
    );
  return votes;
}

/**
 Reads the winner out of the votes, or `neither`.

 A CLEAR WINNER OR NONE. A candidate that ties with the other has not been
 chosen, and shipping either on a tie would be picking by list order.

 @param votes - voice count per candidate

 @returns Candidate to ship, or `neither`

 @example
 ```ts
 const choice = settleVotes({ votes, },);
 ```
 */
function settleVotes(
  { votes, }: { readonly votes: ReadonlyMap<LaneChoice, number>; },
): LaneChoice {
  /**
   Votes for the repair candidate.
   */
  const repair = votes.get('repair',) ?? 0;

  /**
   Votes for the translate candidate.
   */
  const translate = votes.get('translate',) ?? 0;
  if ((repair >= LANE_CONTEST_QUORUM) && (repair > translate))
    return 'repair';
  if ((translate >= LANE_CONTEST_QUORUM) && (translate > repair))
    return 'translate';
  return 'neither';
}

/**
 Reads the winner out of a set of ballots, or `neither`.

 SHARED WITH THE ARTIFACT READER rather than kept private, so a stored verdict
 can be recomputed from the ballots stored beside it and refused when the two
 disagree, exactly as the recorded lane comparison already is.

 @param ballots - usable ballots

 @returns Candidate to ship, or `neither`

 @example
 ```ts
 const choice = settleLaneContestBallots({ ballots, },);
 ```
 */
export function settleLaneContestBallots(
  { ballots, }: { readonly ballots: readonly LaneContestBallot[]; },
): LaneChoice {
  return settleVotes({ votes: countChoices({ ballots, },), },);
}

/**
 What the roster made of the archive rendering at one slice.

 `unjudged` IS NOT A DECLINE. It covers a roster whose voices omitted the
 field, one too small to settle anything, and one that split evenly. None of
 those say the archive is flawed, and recording them as a decline would
 invent a verdict nobody gave.

 @example
 ```ts
 const archive: ArchiveOutcome = 'endorsed';
 ```
 */
export type ArchiveOutcome = 'endorsed' | 'declined' | 'unjudged';

/**
 Reads what the roster made of the archive, or that it settled nothing.

 MIRRORS {@link settleVotes} RATHER THAN SETTING ITS OWN BAR. Two voices and
 a strict lead is the agreement rule everywhere else in this package, and a
 second frozen number would be a second thing every stored verdict is
 recomputed against.

 SHARED WITH THE ARTIFACT READER, like the choice rule beside it, so a
 recorded archive verdict can be re-derived from the ballots stored with it
 and refused when the two disagree.

 @param ballots - usable ballots

 @returns What the roster made of the archive

 @example
 ```ts
 const archive = settleArchiveBallots({ ballots, },);
 ```
 */
export function settleArchiveBallots(
  { ballots, }: { readonly ballots: readonly LaneContestBallot[]; },
): ArchiveOutcome {
  /**
   Voices that would publish the archive as it stands.
   */
  const publishable = ballots
    .filter(function endorses(ballot,): boolean {
      return ballot.archive === 'publishable';
    },)
    .length;

  /**
   Voices that found something wrong with it.
   */
  const flawed = ballots
    .filter(function declines(ballot,): boolean {
      return ballot.archive === 'flawed';
    },)
    .length;
  if ((publishable >= LANE_CONTEST_QUORUM) && (publishable > flawed))
    return 'endorsed';
  if ((flawed >= LANE_CONTEST_QUORUM) && (flawed > publishable))
    return 'declined';
  return 'unjudged';
}

/**
 A ballot this stage read from a seat it asked, which always names that seat.

 SEATED BY CONSTRUCTION. `LaneContestBallot` leaves the seat optional for a
 ballot read back from an artifact settled before seats were recorded; every
 ballot this stage builds comes from a reply its round received from one
 seat, so the seat is always there, and the stage's own helpers take this
 type rather than asking.
 */
type SeatedBallot = LaneContestBallot & { readonly modelId: string; };

/**
 Drops one ballot's archive answer, keeping everything else it said.

 @param ballot - ballot that answered about an archive that was not there

 @returns Same ballot with no archive verdict

 @remarks
 LISTS THE KEPT FIELDS rather than spreading a rest binding. The rest form
 leaves an unused `archive` binding and a declaration the TSDoc rule has
 nowhere to attach to, and naming the survivors makes the one omission the
 point of the function.

 @example
 ```ts
 const stripped = withoutArchiveAnswer({ ballot, },);
 ```
 */
function withoutArchiveAnswer(
  { ballot, }: { readonly ballot: SeatedBallot; },
): SeatedBallot {
  return {
    choice: ballot.choice,
    unsupported: ballot.unsupported,
    unsupportedRaw: ballot.unsupportedRaw,
    dropped: ballot.dropped,
    droppedRaw: ballot.droppedRaw,
    reason: ballot.reason,
    modelId: ballot.modelId,
  };
}

/**
 Logs one line per ballot naming the seat that cast it.

 ONE LINE A JUDGE, because the question a reader brings to a contested slice
 is which models chose what and why (hulicaijia26 footnote 7: four of five
 ballots called the archive's wordplay note an addition, and the one summary
 line the stage printed could not say whose ballots they were).

 @param ballots - ballots as recorded, each carrying its seat

 @param l - logger to write through

 @example
 ```ts
 logBallots({ ballots: recorded, l: cl, },);
 ```
 */
function logBallots(
  {
    ballots,
    l,
  }: {
    readonly ballots: readonly SeatedBallot[];
    readonly l: Logger;
  },
): void {
  for (const ballot of ballots) {
    /**
     Candidates this judge called unsupported, as one clause.
     */
    const unsupported = ballot.unsupported
      .join(', ',);

    /**
     Candidates this judge said dropped page content, as one clause.
     */
    const dropped = ballot.dropped
      .join(', ',);
    l.info(
      `lane contest ballot by ${ballot.modelId}: chose ${ballot.choice}, archive ${
        ballot.archive ?? 'unanswered'
      }, unsupported [${unsupported}], dropped [${dropped}]: ${ballot.reason}`,
    );
  }
}

/**
 Asks the roster which candidate one contested slice should ship.

 @param client - synthetic chat client

 @param modelIds - roster to ask

 @param subject - passage, archive rendering and both candidates

 @param signal - abort shared with the rest of the entry

 @param exchangeTimeoutMs - per-call ceiling

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param l - logger to tag

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns What the roster settled on, with every usable ballot

 @example
 ```ts
 const outcome = await contestLaneSlice({ client, modelIds, subject, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function contestLaneSlice(
  {
    client,
    modelIds,
    subject,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly subject: LaneContestSubject;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  },
): Promise<LaneContestOutcome> {
  /**
   Logger naming this stage.
   */
  const cl = tagged({
    l,
    tag: contestLaneSlice.name,
  },);

  // EXACT-HALF VOICES ARE NEEDED BEFORE GRACE BEGINS, sized by the rounds on
  // the bench. Deterministic exclusions can make fast ballots inadmissible,
  // but they do not make unreliable whole-roster participation mandatory. An
  // eligible side lacking corroboration therefore fails closed as neither.
  /**
   One reply per voice, heard or lost.
   */
  const {
    outcomes,
    quorum,
  } = await runWindowedRounds({
    client,
    modelIds,
    messages: buildLaneContestMessages({ subject, },),
    signal,
    exchangeTimeoutMs,
    responseFormat: LANE_CONTEST_RESPONSE_FORMAT,
    validate: isLaneContestWire,
    stage: 'lane-contest',
    l: cl,
    graceMs: stageSettings
      .grace
      .roundMs,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
  },);

  /**
   Ballots read out of the replies that arrived and validated in shape.

   FLAT-MAPPED RATHER THAN FILTERED AND MAPPED, so the narrowing on `heard`
   reaches the value: a filtered array is still typed as the whole union.
   */
  const ballots = outcomes.flatMap(function toBallot(
    outcome,
  ): readonly SeatedBallot[] {
    /**
     This seat and its voice, heard or lost.
     */
    const {
      modelId,
      voice,
    } = outcome;
    return voice.heard
      ? [
        {
          ...readLaneContestBallot({ wire: voice.value, },),
          modelId,
        },
      ]
      : [];
  },);

  /**
   Ballots as they will be recorded and counted.

   AN ABSENT INCUMBENT LEAVES NOTHING TO JUDGE. `IncumbentKind` admits
   `absent`, and such a slice shows judges an empty archive block while the
   schema still asks whether the archive is publishable, so an answer arrives
   about nothing at all. It is dropped HERE rather than where the record is
   built, because the verdict is derived from exactly the ballots that get
   stored, and stripping in one place keeps those two from disagreeing.
   */
  const recorded = (subject.incumbentText === '')
    ? ballots.map(function strip(ballot,): SeatedBallot {
      return withoutArchiveAnswer({ ballot, },);
    },)
    : ballots;

  /**
   Candidate enough voices backed.
   */
  const choice = settleLaneContestBallots({ ballots: recorded, },);
  logBallots({
    ballots: recorded,
    l: cl,
  },);
  cl.info(
    `lane contest: ${String(ballots.length,)}/${String(outcomes.length,)} usable, settled on ${choice}`,
  );
  return {
    choice,
    ballots: recorded,
    usable: recorded.length,
    findings: [
      ...((ballots.length < LANE_CONTEST_QUORUM)
        ? [
          `lane-contest heard ${String(ballots.length,)} usable ${
            wordForCount({
              count: ballots.length,
              one: 'ballot',
              many: 'ballots',
            },)
          }, below the ${String(LANE_CONTEST_QUORUM,)} needed to settle`,
        ]
        : []),
      // A SHORT BENCH IS SAID IN THE FINDINGS, as the gathers say it (ledger X8).
      ...(quorum.short
        ? [shortBenchStageFinding({
          stage: 'lane-contest',
          quorum,
          benchSize: modelIds.length,
        },),]
        : []),
    ],
  };
}

//endregion Lane contest stage
