import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { CommandLineOf, } from '@monochromatic-dev/translation-repair-run/ts';
import {
  type DigestGroup,
  groupByDigest,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import { judgedRoundsOf, } from './editor-calibrate-standing.ts';
import { noJudgedRoundLine, } from './editor-standing-absence.ts';
import {
  readStandingArtifact,
  type StandingArtifactOutcome,
  type StandingReading,
} from './editor-standing-read-artifact.ts';
import {
  standingArtifactPaths,
  type StandingListing,
} from './editor-standing-read-listing.ts';
import {
  everyStandingRound,
  printDigestStanding,
} from './editor-standing-read-seat.ts';

//region Editor standing read run
// THE EDITOR STANDING THAT COSTS NOTHING, read off work already paid for.
//
// `editor-calibrate.ts` buys a controlled measurement: every model in every
// seat on every slice. This reads the record instead. Every settled artifact's
// repair chunks carry `rounds`, each with the slate judges were shown, each
// candidate's producer, and every ballot cast, which is exactly what
// `producerStandings` counts. That telemetry was being written and never read.
//
// IT IS OBSERVATIONAL, AND THAT BOUNDS WHAT IT CAN SAY. Only models that held
// a seat ever wrote a candidate, so it ranks whoever was seated against each
// other and is silent about everyone else. A model that never held the seat is
// not last here; it is absent. That survivorship is the exact thing the
// controlled calibration exists to defeat, so this corroborates it and cannot
// replace it.
//
// THE PROCEDURE RETURNS ITS EXIT CODE and the entry sets it on the process, so a
// caller importing it keeps its own process status (ledger B379). Until
// 2026-10-10 the whole procedure lived in the entry file and set the code itself.

/**
 Exit code returned when no artifact carried a judged round: none carried a
 round at all, or every round it carried drew no ballot.
 */
const NOTHING_RECORDED = 1;

/**
 Exit code returned when at least one digest carried a judged round and its
 standing was printed.
 */
const STANDINGS_READ = 0;

/**
 Reads every named archive and reports both seats, per digest.

 @param line - the report's command line, read whole by `reportingRefusals`

 @param resolveRuns - names the runs directory, called only when the command
 line names no directory, since named directories need none

 @returns `NOTHING_RECORDED` (1) when no digest carried a judged round, else
 `STANDINGS_READ` (0); the report itself is on stdout and stderr

 @example
 ```ts
 process.exitCode = await reportStandings({ line, resolveRuns: resolveRunsDir, },);
 ```
 */
export async function reportStandings(
  {
    line,
    resolveRuns,
  }: {
    readonly line: CommandLineOf<'editor-standing-read'>;
    readonly resolveRuns: () => Promise<string>;
  },
): Promise<number> {
  /**
   Directories to read, named on the command line or defaulted to this run.
   */
  const roots = line.positionals;

  /**
   What listing each named directory found, in the order they were named.
   */
  const listings = await Promise.all(
    (roots.length === 0
      ? [await resolveRuns(),]
      : roots).map(async function one(path,): Promise<StandingListing> {
        return await standingArtifactPaths({ path, },);
      },),
  );

  /**
   Every artifact path under every named directory.
   */
  const paths = listings.flatMap(function pathsOf(listing,): readonly string[] {
    return listing.paths;
  },);

  /**
   What every artifact turned out to be, read or not, in the order listed.
   */
  const outcomes = await Promise.all(paths.map(async function one(path,): Promise<StandingArtifactOutcome> {
    return await readStandingArtifact({ path, },);
  },),);

  // EVERY STDERR LINE ONCE EVERYTHING IS READ, in the order the directories were
  // named and their artifacts listed, so one archive gives one report whichever
  // read ends first.
  for (
    const said of [
      ...listings,
      ...outcomes,
    ].flatMap(function saidBy(found,): readonly string[] {
      return found.said;
    },)
  ) {
    console.error(said,);
  }

  /**
   Every artifact that parsed under the current roster.
   */
  const readings = outcomes.flatMap(function parsed(outcome,): readonly StandingReading[] {
    return (outcome.kind === 'read') ? [outcome.reading,] : [];
  },);

  /**
   Artifacts settled under an earlier seating, counted apart from defects.
   */
  const offRoster = outcomes.filter(function earlier(outcome,): boolean {
    return outcome.kind === 'off-roster';
  },)
    .length;

  /**
   Artifacts settled before the lane recorded rounds, counted apart for the
   same reason: a record that cannot answer this question is not a broken one.
   */
  const earlierSchema = outcomes.filter(function beforeRounds(outcome,): boolean {
    return outcome.kind === 'earlier-schema';
  },)
    .length;

  /**
   Every round the readings recorded, at either seat, judged or not.
   */
  const recorded = readings.flatMap(function roundsOf(reading,): readonly SelectionRound[] {
    return everyStandingRound({ reading, },);
  },);

  /**
   Every digest the readings were built by, each with whether any of its
   rounds drew a ballot.
   */
  const digests = groupByDigest({ readings, },)
    .map(function judgedOrNot(group,): {
      readonly group: DigestGroup<StandingReading>;
      readonly judged: boolean;
    } {
      return {
        group,
        judged: group
          .readings
          .some(function any(reading,): boolean {
            return judgedRoundsOf({ rounds: everyStandingRound({ reading, },), },)
              .length
              > 0;
          },),
      };
    },);

  /**
   Groups carrying at least one judged round, the ones a standing is printed
   for.
   */
  const groups = digests
    .filter(function judged(digest,): boolean {
      return digest.judged;
    },)
    .map(function groupOf(digest,): DigestGroup<StandingReading> {
      return digest.group;
    },);

  /**
   Groups none of whose rounds drew a ballot, which print no standing and are
   counted on the summary line instead, so their rounds are counted somewhere.
   Until 2026-10-10 they were dropped whenever another digest had a judged
   round, and the line's `digestsWithRounds` counted the judged ones alone.
   */
  const unjudged = digests.filter(function unjudgedDigest(digest,): boolean {
    return !digest.judged;
  },);

  /**
   Rounds the unjudged groups recorded, none of which drew a ballot.
   */
  const unjudgedDigestRounds = unjudged.flatMap(function roundsOfDigest(digest,): readonly SelectionRound[] {
    return digest.group
      .readings
      .flatMap(function roundsOf(reading,): readonly SelectionRound[] {
        return everyStandingRound({ reading, },);
      },);
  },);

  console.log(
    `editor-standing-read: archives=${String(roots.length,)} artifacts=${String(paths.length,)} `
      + `read=${String(readings.length,)} earlierRoster=${String(offRoster,)} `
      + `earlierSchema=${String(earlierSchema,)} `
      + `digestsJudged=${String(groups.length,)} digestsUnjudged=${String(unjudged.length,)} `
      + `unjudgedDigestRounds=${String(unjudgedDigestRounds.length,)}`,
  );
  console.log(
    '  OBSERVATIONAL. Only models that held a seat ever wrote a candidate, so an absent model '
      + 'is unmeasured rather than last. Rounds inside one entry are correlated, so read the '
      + 'entry count, not the round count. Digests are never pooled.',
  );

  if (groups.length === 0) {
    console.log(noJudgedRoundLine({
      offRoster,
      unjudged: recorded.length,
    },),);
    return NOTHING_RECORDED;
  }

  for (const group of groups) {
    printDigestStanding({ group, },);
  }
  return STANDINGS_READ;
}

//endregion Editor standing read run
