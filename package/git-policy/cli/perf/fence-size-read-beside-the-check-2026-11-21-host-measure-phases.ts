/**
 The phases of the 2026-11-21 host measurement, each one a list of adds and what it writes about them.

 A phase builds its list of additions first, makes them one at a time, writes the rows to the file the planning document
 reads its figures from, and prints the summary it read back out of those rows.
 */
import {
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  countedArmsOfRound,
  oneAdd,
  refusedArmsOfRound,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-add.ts';
import {
  isThere,
  oneAtATime,
  run,
} from './fence-size-read-beside-the-check-2026-11-21-commands.ts';
import { fixtureEnvironment, } from './fence-size-read-beside-the-check-2026-11-21-host-measure-source.ts';
import {
  ARMS,
  type Addition,
  type Arm,
  BINARIES,
  nameOf,
  SCRATCH,
  SOURCE,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-paths.ts';
import {
  type Row,
  rowAsWritten,
  summaryOfAdds,
  summaryOfRefusedAdds,
} from '../bin/fence-size-read-beside-the-check-2026-11-21-host-measure-phase.ts';

/**
 Writes one phase's rows, one line each, into the file the phase is named by.

 @param path - the file the rows go to

 @param rows - the rows the phase wrote, in the order it made the adds
 */
async function writeRows({
  path,
  rows,
}: {
  readonly path: string;
  readonly rows: readonly Row[];
}): Promise<void> {
  await writeFile(
    path,
    `${rows
      .map(function lineOf(row) {
        return JSON.stringify(rowAsWritten({ row, },),);
      },)
      .join('\n',)}\n`,
  );
}

/**
 Makes every add of one phase, one at a time.

 @param additions - the adds the phase makes, in the order it makes them

 @returns the rows the adds left behind, in that order
 */
async function measureEvery({ additions, }: { readonly additions: readonly Addition[]; }): Promise<readonly Row[]> {
  /**
   The rows this phase wrote, in the order it made the adds.
   */
  const rows: Row[] = [];
  await oneAtATime({
    items: additions,
    step: async function measureOne(addition) {
      rows.push(await oneAdd(addition,),);
    },
  },);
  return rows;
}

/**
 The adds one counted phase makes: one warm-up per arm, then every counted round's.

 @param layout - the layout the phase measures, which names the row file it writes

 @param rounds - how many counted rounds it makes

 @returns the additions, in the order the phase makes them

 @example
 ```ts
 countedAdditions({ layout: 'loose', rounds: 2 });
 ```
 */
export function countedAdditions({
  layout,
  rounds,
}: {
  readonly layout: string;
  readonly rounds: number;
}): readonly Addition[] {
  /**
   The warm-up adds, one per arm, which no median reads.
   */
  const warmUps: readonly Addition[] = ARMS.map(function warmUpOf(arm) {
    return {
      arm,
      tag: `warm-${arm.name}`,
      layout,
      counted: false,
    };
  });
  /**
   The counted adds, round by round in the order each round takes the arms.
   */
  const counted: Addition[] = [];
  for (let round = 0; round < rounds; round += 1) {
    for (const [at, arm,] of countedArmsOfRound({ round, },)
      .entries()) {
      counted.push({
        arm,
        tag: `r${String(round + 1,)}p${String(at + 1,)}`,
        layout,
        counted: true,
        round: round + 1,
        position: at + 1,
      },);
    }
  }
  return [
    ...warmUps,
    ...counted,
  ];
}

/**
 What:
  Makes every add of one counted phase, one at a time, then writes the phase's rows and prints its medians.

 Why:
  The rows are the record the planning document's figures come from, so the phase writes them before it prints the summary
  it read them into.

 @param layout - the layout the phase measures, which names the row file it writes

 @param rounds - how many counted rounds it makes

 @example
 ```ts
 await adds({ layout: 'loose', rounds: 2 });
 ```
 */
export async function adds({
  layout,
  rounds,
}: {
  readonly layout: string;
  readonly rounds: number;
}): Promise<void> {
  /**
   The rows this phase wrote, in the order it made the adds.
   */
  const rows = await measureEvery({ additions: countedAdditions({
    layout,
    rounds,
  },), },);
  await writeRows({
    path: join(
      SCRATCH,
      `${layout}-host-adds.jsonl`,
    ),
    rows,
  },);
  console.log(JSON.stringify(
    {
      phase: 'adds',
      layout,
      summary: summaryOfAdds({
        rows,
        arms: ARMS.map(function nameOfArm(arm) {
          return nameOf(arm,);
        },),
      },),
    },
    null,
    1,
  ),);
}

/**
 What:
  Makes the same adds on a source whose check refuses it: one staged path, which no rung serves on ZFS, so the add falls to
  Git's own checkout.

 Why:
  The overlap starts its read before the check, so a refused source pays a read the fence never takes; this phase prices
  that, and every arm's record shows whether the read ran at all.

 @param rounds - how many rounds it makes

 @example
 ```ts
 await refusedAdds({ rounds: 2 });
 ```
 */
export async function refusedAdds({ rounds, }: { readonly rounds: number; }): Promise<void> {
  /**
   The environment of every fixture-building Git run.
   */
  const env = fixtureEnvironment();
  /**
   The path the refused phase stages, which makes the source's index differ from its HEAD tree.
   */
  const marker = 'refusal-marker.txt';
  await writeFile(
    join(
      SOURCE,
      marker,
    ),
    'staged, so the index of the source is not its HEAD tree\n',
  );
  await run({
    args: [
      'git',
      'add',
      '--',
      marker,
    ],
    cwd: SOURCE,
    env,
  },);
  /**
   The adds this phase makes, round by round in the order each round takes the arms.
   */
  const additions: Addition[] = [];
  for (let round = 0; round < rounds; round += 1) {
    for (const [at, arm,] of refusedArmsOfRound({ round, },)
      .entries()) {
      additions.push({
        arm,
        tag: `refused-r${String(round + 1,)}p${String(at + 1,)}`,
        layout: 'refused',
        counted: true,
        round: round + 1,
        position: at + 1,
      },);
    }
  }
  /**
   The rows this phase wrote, in the order it made the adds.
   */
  const rows = await measureEvery({ additions, },);
  await run({
    args: [
      'git',
      'rm',
      '--cached',
      '--quiet',
      '--',
      marker,
    ],
    cwd: SOURCE,
    env,
  },);
  await rm(
    join(
      SOURCE,
      marker,
    ),
    { force: true, },
  );
  await writeRows({
    path: join(
      SCRATCH,
      'refused-host-adds.jsonl',
    ),
    rows,
  },);
  console.log(JSON.stringify(
    {
      phase: 'refused',
      summary: summaryOfRefusedAdds({
        rows,
        arms: ARMS.map(function nameOfArm(arm) {
          return nameOf(arm,);
        },),
      },),
    },
    null,
    1,
  ),);
}

/**
 Prints one add of one arm, whole, which is the shape check a run makes before its counted rounds.

 @param arm - the arm to add once

 @throws Error when the arm's build is not there to run

 @example
 ```ts
 await probeArm(ARMS[0]);
 ```
 */
export async function probeArm(arm: Arm): Promise<void> {
  /**
   The build this arm runs.
   */
  const binary = BINARIES[arm.binary];
  if ((binary === undefined) || (!(await isThere({ path: binary, },)))) {
    console.log(`${arm.name}: binary not built yet`,);
    return;
  }
  console.log(
    JSON.stringify(await oneAdd({
      arm,
      tag: `probe-${arm.name}`,
      layout: 'probe',
      counted: false,
    },),),
  );
}

/**
 Prints one add of every arm, whole, which is the shape check a run makes before its counted rounds.

 @example
 ```ts
 await probe();
 ```
 */
export async function probe(): Promise<void> {
  await oneAtATime({
    items: ARMS,
    step: probeArm,
  },);
}
