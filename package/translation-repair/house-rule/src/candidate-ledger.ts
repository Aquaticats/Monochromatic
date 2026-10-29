import { mkdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import {
  type Candidate,
  type CandidateProducer,
  producerModelIds,
  type SelectionBallot,
} from './candidate-select-model.ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  currentLogContext,
  type LogContext,
} from '@monochromatic-dev/translation-repair-typography/ts';

//region Candidate ledger
// WHAT EACH MODEL ACTUALLY WROTE, and what the judges said about it.
//
// WHY THIS EXISTS. Every contest in this pipeline ran through
// `selectBestCandidate`, counted ballots, reported a standing, and then threw
// the text away. A standing says a model was preferred on 3% of ballots; it
// cannot say whether that model wrote something wrong or merely something
// nobody picked as the single best of ten. Those are different findings with
// different remedies, and no run this project has ever made could tell them
// apart after the fact.
//
// IT COST A REAL INVESTIGATION. Asked to confirm or reject that one seat was
// weak, the only honest answer available from the archive was that no run had
// kept a single line that seat produced, and re-deriving it meant buying fresh
// calls. A roster decision should read evidence the run already paid for.
//
// ONE HOOK, EVERY CONTEST. This is written from `selectBestCandidate`, which
// the translate lane, both editor paths, the refiner and the fidelity judge all
// route through, so no caller has to remember to record anything.
//
// NEVER RAISES INTO THE SELECTION PATH. A pipeline that failed a slice because
// its telemetry could not write would be worse than one with no telemetry, so
// every failure here is caught, named and swallowed. The absence shows up in
// the report rather than in the run.
//
// HOLDS UNLICENSED CORPUS WORDING, exactly as the settled artifacts already do,
// because candidate text is a rendering of a corpus passage. It lands under the
// run directory, which is outside this repository, and it must never be
// committed. `TRANSLATION_REPAIR_RUNS_DIR` names that directory; with the
// variable unset nothing is written at all, which is what unit runs want.
//
// READ BY THE ENTRY FILE, NOT HERE (`readContestLedger`). The recorder read
// the variable itself under every selection, so a case chose where contests
// landed by writing the process's environment; it now writes where the
// `ContestLedger` it is handed says, which every stage carries down from the
// entry inside `StageSettings`.

/**
 Directory under a runs dir holding one file per judged contest.
 */
export const LEDGER_DIR = 'ledger';

/**
 What ends a contest's file name. The atomic write's temporary name carries
 more after it, so a file still being written never ends in it.
 */
const LEDGER_SUFFIX = '.json';

/**
 Width the ordinal is padded to, so a thousand contests still sort as text.
 */
const ORDINAL_DIGITS = 6;

/**
 Stamp naming this launch, fixed at module load: the moment as an ISO time
 with its separators made file-safe, then the process id.

 ONE PER PROCESS, so two launches into one runs directory never write the
 same name. The ordinal alone once restarted at zero per process, and a
 relaunch into the same directory, which is the documented resume path,
 overwrote the earlier launch's contests one by one with no reader able to
 tell. Names still sort as text into contest order: launches by their stamp,
 contests within a launch by their ordinal.

 @example
 ```ts
 const name = ledgerFileName({ ordinal: 3, launch: LAUNCH_STAMP, },);
 ```
 */
export const LAUNCH_STAMP: string = `${launchMoment()}-${String(process.pid,)}`;

/**
 This launch's moment as an ISO time with its separators made file-safe.

 @returns Time text carrying neither colons nor dots

 @example
 ```ts
 const moment = launchMoment();
 ```
 */
function launchMoment(): string {
  return new Date()
    .toISOString()
    .replaceAll(
    ':',
    '-',
    )
    .replaceAll(
    '.',
    '-',
    );
}

/**
 File name of one recorded contest.

 @param ordinal - contest number within the launch, from zero

 @param launch - stamp of the launch writing it

 @returns Name that sorts by launch, then by ordinal

 @example
 ```ts
 const name = ledgerFileName({ ordinal: 0, launch: LAUNCH_STAMP, },);
 ```

 @internal
 */
export function ledgerFileName(
  {
    ordinal,
    launch,
  }: {
    readonly ordinal: number;
    readonly launch: string;
  },
): string {
  /**
   Ordinal padded so a thousand contests still sort as text.
   */
  const padded = String(ordinal,)
    .padStart(
    ORDINAL_DIGITS,
    '0',
    );
  return `${launch}-${padded}${LEDGER_SUFFIX}`;
}

/**
 Whether a file name in the ledger directory names a recorded contest. A
 contest still being written sits under its atomic write's temporary name,
 which does not end in the suffix, and is no contest until it is renamed
 (ledger B65).

 @param name - file name, without a directory

 @returns Whether it ends in the contest suffix

 @example
 ```ts
 const recorded = isLedgerFileName({ name: '000001.json', },);
 ```
 */
export function isLedgerFileName({ name, }: { readonly name: string; },): boolean {
  return name.endsWith(LEDGER_SUFFIX,);
}

/**
 Environment variable naming the run directory, matching `run-config.ts`.

 READ AS IT STANDS RATHER THAN THROUGH `resolveRunsDir`, because that lives
 in `corpus-run/` and this is core pipeline code, and because it falls back to
 a default directory where this keeps nothing: a launch that named no runs
 directory records no contest. A core module reaching into the runner family
 to write telemetry would invert the dependency the rest of the package keeps.
 */
const RUNS_DIR_VARIABLE = 'TRANSLATION_REPAIR_RUNS_DIR';

/**
 Where a launch keeps its judged contests: under the runs directory it named,
 or nowhere when it named none.

 @example
 ```ts
 const ledger: ContestLedger = { kind: 'kept', runsDir: '/srv/cattery-runs', };
 ```
 */
export type ContestLedger =
  | {
    /**
     The launch named no runs directory, so no contest is written.
     */
    readonly kind: 'unkept';
  }
    | {
    /**
     The launch named a runs directory, under which every contest is written.
     */
      readonly kind: 'kept';

    /**
     Runs directory the launch named.
     */
      readonly runsDir: string;
    };

/**
 Reads where a launch keeps its judged contests from the environment an entry
 file hands in.

 UNSET AND EMPTY ALIKE KEEP NOTHING, as the recorder always read them: the
 ordinary case for unit runs and for probes that set no runs directory.

 @param env - environment of the launch

 @returns Ledger under the named runs directory, or the unkept one

 @example
 ```ts
 const contestLedger = readContestLedger({ env: process.env, },);
 ```
 */
export function readContestLedger(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): ContestLedger {
  /**
   Run directory the launch named, empty when it named none.
   */
  const runsDir = env[RUNS_DIR_VARIABLE] ?? '';
  if (runsDir === '')
    return { kind: 'unkept', };
  return {
    kind: 'kept',
    runsDir,
  };
}

/**
 How many contests this process has recorded, used to order the files.

 A HOLDER RATHER THAN A BARE BINDING, matching the `state` object in
 `anthropic-delta-scan.ts`: the count has to change, and a mutable module
 binding is the shape this codebase avoids.
 */
const state = { recorded: 0, };

/**
 One candidate as it was shown to the judges.
 */
export type LedgerCandidate = {
  /**
   One-based position in the slate, which is what a ballot names.
   */
  readonly index: number;

  /**
   Every model with a hand in this candidate, composites expanded.
   */
  readonly producers: readonly string[];

  /**
   Exactly the text judges compared.
   */
  readonly rendered: string;
};

/**
 One judged contest, whole.
 */
export type LedgerRound = {
  /**
   What the judges were asked to decide, verbatim from the caller.
   */
  readonly task: string;

  /**
   When it was judged.
   */
  readonly at: string;

  /**
   Every candidate, in the order the judges saw them.
   */
  readonly candidates: readonly LedgerCandidate[];

  /**
   Every ballot, reasons included.
   */
  readonly ballots: readonly SelectionBallot[];

  /**
   Winning position, or that nothing was chosen.
   */
  readonly selectedIndex: number | 'declined';

  /**
   Entry, pipeline, lane and slice the contest was judged for, each empty
   where it was judged outside one (ledger A11): without them a round could
   not be joined to the artifact that shipped its winner.
   */
  readonly context: LogContext;
};

/**
 Names every model behind a candidate, composites expanded.

 @param producer - provenance the slate recorded

 @returns Model ids, one entry per contributor

 @example
 ```ts
 const names = producersOf({ producer, },);
 ```
 */
function producersOf(
  { producer, }: { readonly producer: CandidateProducer; },
): readonly string[] {
  // ALREADY THE ANSWER. `RosterModelId` is a union of string literals, so the
  // ids need no conversion; mapping them through an identity would only add a
  // step for a reader to check.
  return producerModelIds(producer,);
}

/**
 Records one judged contest, text and ballots together.

 SWALLOWS EVERY FAILURE. See the module note: telemetry must not be able to
 fail a slice the run already paid for.

 @param task - what the judges were asked, used to tell contests apart

 @param candidates - slate exactly as the judges saw it

 @param ballots - what each judge said, reasons verbatim

 @param selectedIndex - winning position, or that the round declined

 @param l - pipeline logger

 @param contestLedger - where the launch keeps its contests, which the entry
 file read (`readContestLedger`)

 @example
 ```ts
 await recordContest({ task, candidates, ballots, selectedIndex, l, contestLedger: stageSettings.contestLedger, },);
 ```
 */
export async function recordContest<ValueT,>(
  {
    task,
    candidates,
    ballots,
    selectedIndex,
    l,
    contestLedger,
  }: {
    readonly task: string;
    readonly candidates: readonly Candidate<ValueT>[];
    readonly ballots: readonly SelectionBallot[];
    readonly selectedIndex: number | 'declined';
    readonly l: Logger;
    readonly contestLedger: ContestLedger;
  },
): Promise<void> {
  // NOTHING TO WRITE OUTSIDE A RUN, which is the ordinary case for unit tests
  // and for probes that set no run directory. Silent because it is not a fault.
  if (contestLedger.kind === 'unkept')
    return;

  /**
   Run directory to write under.
   */
  const { runsDir, } = contestLedger;

  /**
   Logger tagged with this recorder.
   */
  const rl = tagged({
    tag: recordContest.name,
    l,
  },);

  /**
   Position of this contest in the process, so files sort in judging order.
   */
  const ordinal = state.recorded;
  state.recorded += 1;

  /**
   Everything this contest held.
   */
  const round: LedgerRound = {
    task,
    at: new Date().toISOString(),
    candidates: candidates.map(function asLedger(
      candidate,
      at,
    ): LedgerCandidate {
      return {
        index: at + 1,
        producers: producersOf({ producer: candidate.producer, },),
        rendered: candidate.rendered,
      };
    },),
    ballots,
    selectedIndex,
    context: currentLogContext(),
  };

  /**
   File this contest is written to, unique across launches.
   */
  const fileName = ledgerFileName({
    ordinal,
    launch: LAUNCH_STAMP,
  },);

  try {
    /**
     Directory the ledger lives in, created on first write.
     */
    const dir = join(
      runsDir,
      LEDGER_DIR,
    );
    await mkdir(
      dir,
      { recursive: true, },
    );
    // ATOMIC, so a report listing the directory mid-run never parses half a
    // round (ledger A11).
    await writeFileAtomic({
      path: join(
        dir,
        fileName,
      ),
      text: JSON.stringify(
        round,
        null,
        2,
      ),
    },);
  } catch (error) {
    // NAMED, NOT QUOTED. A filesystem error quotes a path, and a run directory
    // path can name a person.
    rl.warn(
      `could not record contest ${String(ordinal,)}: ${errorName({ error, },)}. `
        + 'The run is unaffected; this contest will be missing from the ledger',
    );
  }
}

//endregion Candidate ledger
