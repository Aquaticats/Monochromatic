import {
  cp,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  attemptDirectories,
  mergeScoredNames,
  readScoredNames,
  reportDirectoryName,
  scoredCount,
  scoredListNames,
  type ScoredNames,
} from './campaign-ledger.ts';
import {
  missedExitStatus,
  timeoutExitStatus,
} from './constants.ts';
import {
  isMissing,
  isRecord,
} from './fs-guards.ts';
import { mapInSequence } from './in-sequence.ts';

/**
 Parsed `outcomes.json` of one attempt.
 */
type AttemptOutcomes = Readonly<Record<string, unknown>>;

/**
 Parse one attempt's `outcomes.json`, which a kill can leave torn.

 @param attempt - attempt directory

 @returns the parsed object alone, or nothing when the file is absent or not complete JSON

 @example
 ```ts
 await readAttemptOutcomes('/e/attempt-001');
 ```
 */
async function readAttemptOutcomes(attempt: string): Promise<readonly AttemptOutcomes[]> {
  try {
    /**
     Parsed file.
     */
    const parsed: unknown = JSON.parse(await readFile(
      join(
        attempt,
        reportDirectoryName,
        'outcomes.json',
      ),
      'utf8',
    ));
    return isRecord(parsed) ? [parsed] : [];
  }
  catch (error) {
    // A missing file means the attempt never scored; a torn file means a kill landed mid-write.
    console.error(`No complete outcomes.json in ${attempt}: ${String(error)}`);
    return [];
  }
}

/**
 Read the outcome entries of one attempt's `outcomes.json`.

 @param outcomes - parsed file

 @returns the entries, or none when the field is not a list

 @example
 ```ts
 outcomeEntries({ outcomes: [] });
 ```
 */
function outcomeEntries(outcomes: AttemptOutcomes): readonly unknown[] {
  /**
   The field before it is known to be a list.
   */
  const entries: unknown = outcomes.outcomes;
  return Array.isArray(entries)
    ? entries.map(function keep(entry: unknown) {
      return entry;
    })
    : [];
}

/**
 Read the count of succeeded mutants of one attempt's `outcomes.json`, which only a check-only run produces.

 @param outcomes - parsed file

 @returns the count, or zero when the field is absent

 @example
 ```ts
 succeededCount({ success: 2 });
 ```
 */
function succeededCount(outcomes: AttemptOutcomes): number {
  /**
   The field before it is known to be a number.
   */
  const success: unknown = outcomes.success;
  return (typeof success) === 'number' ? success : 0;
}

/**
 Copy files of one report subdirectory from every attempt into the composed report, later attempts overwriting.

 @param attempts - attempt directories in order

 @param composed - composed report directory

 @param name - subdirectory name

 @example
 ```ts
 await copySubdirectory({ attempts, composed: '/e/mutants.out', name: 'log' });
 ```
 */
async function copySubdirectory({
  attempts,
  composed,
  name,
}: {
  readonly attempts: readonly string[];
  readonly composed: string;
  readonly name: string;
}): Promise<void> {
  await mkdir(
    join(
      composed,
      name,
    ),
    { recursive: true },
  );
  await mapInSequence({
    items: attempts,
    run: async function copyAttempt(attempt) {
      try {
        await cp(
          join(
            attempt,
            reportDirectoryName,
            name,
          ),
          join(
            composed,
            name,
          ),
          {
            recursive: true,
            force: true,
          },
        );
      }
      catch (error) {
        if (!isMissing(error))
          throw error;
      }
    },
  });
}

/**
 Copy the list of mutants the first attempt that wrote one found, which holds every mutant of the scope.

 @param attempts - attempt directories in order

 @param composed - composed report directory

 @example
 ```ts
 await copyMutantList({ attempts, composed: '/e/mutants.out' });
 ```
 */
async function copyMutantList({
  attempts,
  composed,
}: {
  readonly attempts: readonly string[];
  readonly composed: string;
}): Promise<void> {
  /**
   Texts of the attempts' lists, the first of which is kept.
   */
  const texts = await Promise.all(attempts.map(async function readList(attempt) {
    try {
      return [await readFile(
        join(
          attempt,
          reportDirectoryName,
          'mutants.json',
        ),
        'utf8',
      )];
    }
    catch (error) {
      if (isMissing(error))
        return [];
      throw error;
    }
  }));
  /**
   The kept text, or none when no attempt wrote one.
   */
  const [first] = texts.flat();
  if (first !== undefined) {
    await writeFile(
      join(
        composed,
        'mutants.json',
      ),
      first,
    );
  }
}

/**
 Read the name of the mutant an `outcomes.json` entry scored.

 @param entry - one element of the `outcomes` list

 @returns the mutant's name as the list files spell it, or the empty text for the baseline entry or an unreadable entry

 @example
 ```ts
 mutantNameOf({ scenario: { Mutant: { name: 'src/a.rs:1:1: replace f' } } });
 ```
 */
function mutantNameOf(entry: unknown): string {
  if (!isRecord(entry))
    return '';
  /**
   The scenario field before it is known to be a mutant.
   */
  const {scenario} = entry;
  if ((!isRecord(scenario)) || (!isRecord(scenario.Mutant)))
    return '';
  /**
   The mutant's name field.
   */
  const {name} = scenario.Mutant;
  return (typeof name) === 'string' ? name : '';
}

/**
 Lay the scores of a later pass over those of an earlier one, so each mutant counts as the later pass scored it.

 A mutant the later pass scored takes the later outcome. A mutant it did not score keeps the earlier one, which is how
 a report composed before the later pass finished still lists every survivor.

 @param earlier - names the earlier pass scored

 @param later - names the later pass scored

 @returns the names of the whole campaign

 @example
 ```ts
 overlayScoredNames({ earlier, later });
 ```
 */
export function overlayScoredNames({
  earlier,
  later,
}: {
  readonly earlier: ScoredNames;
  readonly later: ScoredNames;
}): ScoredNames {
  /**
   Every mutant the later pass scored.
   */
  const rescored = new Set([
    ...later.caught,
    ...later.missed,
    ...later.timeout,
    ...later.unviable,
  ]);
  /**
   Keep the names the later pass did not score.

   @param names - names of one outcome in the earlier pass

   @returns those the later pass left alone
   */
  function untouched(names: readonly string[]): readonly string[] {
    return names.filter(function notRescored(name) {
      return !rescored.has(name);
    });
  }
  return mergeScoredNames([
    {
      caught: untouched(earlier.caught),
      missed: untouched(earlier.missed),
      timeout: untouched(earlier.timeout),
      unviable: untouched(earlier.unviable),
    },
    later,
  ]);
}

/**
 Count the mutants the first attempt of a campaign set out to score, from the `mutants.json` of its composed report.

 @param evidence - the campaign's evidence directory, whose `mutants.out` was composed

 @returns the length of the list as the only element, or no element when no attempt wrote one or it is not a list

 @example
 ```ts
 await plannedMutantCount('/e/native-mutation-0123456789ab/pass-1');
 ```
 */
export async function plannedMutantCount(evidence: string): Promise<readonly number[]> {
  try {
    /**
     The parsed list.
     */
    const parsed: unknown = JSON.parse(await readFile(
      join(
        evidence,
        reportDirectoryName,
        'mutants.json',
      ),
      'utf8',
    ));
    return Array.isArray(parsed) ? [parsed.length] : [];
  }
  catch (error) {
    // No attempt wrote the list, or a kill tore it: the plan is unknown.
    if (!isMissing(error))
      console.error(`No complete mutants.json in ${evidence}: ${String(error)}`);
    return [];
  }
}

/**
 Write the campaign's whole report where callers read a finished campaign: `<evidence>/mutants.out`.

 The four list files are the union over every attempt, so the report is complete however many times the campaign was
 interrupted and resumed. `outcomes.json` takes the last readable attempt's fields, every readable attempt's outcome
 entries and counts recomputed from the lists, which a kill cannot tear. `log`, `diff` and `mutants.json` are copied
 from the attempts.

 A campaign whose first pass ran in another evidence directory passes that directory as `earlier`: its attempts come
 first, and the later pass's result replaces the earlier one for each mutant the later pass scored (its list entries,
 its outcome entry, its log and its diff), so the report reads as one campaign.

 @param evidence - the campaign's evidence directory

 @param earlier - evidence directory of the pass that ran first in a two-pass campaign, absent in a one-pass campaign

 @returns the scored names the report holds

 @example
 ```ts
 await composeReport({ evidence: '/e/native-mutation-0123456789ab' });
 ```
 */
export async function composeReport({
  evidence,
  earlier,
}: {
  readonly evidence: string;
  readonly earlier?: string;
}): Promise<ScoredNames> {
  /**
   Attempts of the earlier pass, in order.
   */
  const earlierAttempts = earlier === undefined ? [] : await attemptDirectories(earlier);
  /**
   Attempts in order, the earlier pass's first.
   */
  const attempts = [
    ...earlierAttempts,
    ...await attemptDirectories(evidence)
  ];
  /**
   Names scored by the attempts of this directory.
   */
  const own = await readScoredNames(evidence);
  /**
   Scored names over all attempts, each mutant as the last pass that scored it left it.
   */
  const names = earlier === undefined ? own : overlayScoredNames({
    earlier: await readScoredNames(earlier),
    later: own,
  });
  /**
   Mutants this directory's attempts scored, whose earlier outcome entries are replaced.
   */
  const rescored = new Set([
    ...own.caught,
    ...own.missed,
    ...own.timeout,
    ...own.unviable,
  ]);
  /**
   Composed report directory.
   */
  const composed = join(
    evidence,
    reportDirectoryName,
  );
  await mkdir(
    composed,
    { recursive: true },
  );
  /**
   Names per list file name.
   */
  const byList: Readonly<Record<(typeof scoredListNames)[number], readonly string[]>> = {
    'caught.txt': names.caught,
    'missed.txt': names.missed,
    'timeout.txt': names.timeout,
    'unviable.txt': names.unviable,
  };
  await Promise.all(scoredListNames.map(function writeList(listName) {
    return writeFile(
      join(
        composed,
        listName,
      ),
      byList[listName]
        .map(function withNewline(name) {
          return `${name}\n`;
        })
        .join(''),
    );
  }));
  /**
   Readable outcome files of the attempts, in attempt order.
   */
  const outcomesByAttempt = await Promise.all(attempts.map(readAttemptOutcomes));
  /**
   The same, flattened.
   */
  const outcomes = outcomesByAttempt.flat();
  /**
   Outcome entries over all attempts, without those of the earlier pass for a mutant the later pass scored.
   */
  const entries = outcomesByAttempt.flatMap(function keptEntries(
    attemptOutcomes,
    index
  ) {
    return attemptOutcomes.flatMap(outcomeEntries)
      .filter(function isCurrent(entry) {
        return (index >= earlierAttempts.length) || (!rescored.has(mutantNameOf(entry)));
      });
  });
  /**
   Mutants that succeeded over all attempts.
   */
  const success = outcomes.reduce(
    function addSuccess(
      total,
      outcome
    ) {
      return total + succeededCount(outcome);
    },
    0,
  );
  await writeFile(
    join(
      composed,
      'outcomes.json',
    ),
    `${JSON.stringify(
      {
        ...outcomes.at(-1),
        outcomes: entries,
        total_mutants: scoredCount(names) + success,
        missed: names.missed
          .length,
        caught: names.caught
          .length,
        timeout: names.timeout
          .length,
        unviable: names.unviable
          .length,
        success,
        start_time: outcomes.at(0)
          ?.start_time,
        end_time: outcomes.at(-1)
          ?.end_time,
      },
      null,
      2,
    )}\n`,
  );
  await copySubdirectory({
    attempts,
    composed,
    name: 'log',
  });
  await copySubdirectory({
    attempts,
    composed,
    name: 'diff',
  });
  await copyMutantList({
    attempts,
    composed,
  });
  return names;
}

/**
 Work out the exit status of a campaign from everything it scored.

 cargo-mutants reports only the attempt it ran, and an attempt that was handed no mutants to score reports success; a
 finished campaign's status must follow the whole report. A timeout wins over missed mutants, as in cargo-mutants.
 Any other status (a failed baseline, an out-of-memory kill, a signal, cargo-mutants' own interrupted exit) is the
 attempt's own and stays.

 @param attemptStatus - exit status of the last attempt's container

 @param names - scored names of the whole campaign

 @returns the status the campaign ends with

 @example
 ```ts
 cumulativeStatus({ attemptStatus: 0, names });
 ```
 */
export function cumulativeStatus({
  attemptStatus,
  names,
}: {
  readonly attemptStatus: number;
  readonly names: ScoredNames;
}): number {
  if (![
    0,
    missedExitStatus,
    timeoutExitStatus
  ].includes(attemptStatus))
    return attemptStatus;
  if (names.timeout
    .length
    > 0)
    return timeoutExitStatus;
  return names.missed
    .length
    > 0 ? missedExitStatus : 0;
}
