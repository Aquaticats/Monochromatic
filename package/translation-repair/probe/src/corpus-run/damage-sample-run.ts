import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

import { runIntroducedDefectProbe, } from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  StatedRefusalError,
  writeSheetPair,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  buildCase,
  drawRegions,
} from './damage-sample-draw.ts';
import {
  emptyPoolSays,
  poolLines,
  wroteLine,
} from './damage-sample-lines.ts';
import type { PoolPolicy, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { collectShippedRegions, } from './damage-sample-pool.ts';
import {
  formatVerifyManifest,
  formatVerifySheet,
  type VerifyItem,
} from './probe-verify-sheet.ts';
import type { RunClient, } from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Damage sample run
// Draws shipped regions at random and asks a human the SAME source-anchored
// question the probe now asks, so the two answers are comparable.
//
// Every earlier repair measurement failed on one of two counts. The round-three
// repair sheet asked whether the returned wording fixed its issue, which is a
// different question from whether the edit damaged anything. The verification
// sheet asked the right question but drew only regions someone already believed
// were bad, and re-reading those five showed all five were correct repairs, so
// the selection carried the answer.
//
// This draws from every shipped region in the settled artifacts, by seed, with
// no reference to what anyone thought of them. The probe's verdict is recorded
// in the manifest and NEVER on the sheet: a reader shown a machine claim is
// answering a different question, and agreement measured that way is worthless.

/**
 Draws a sample, probes each region, and writes the sheet and manifest.

 @param runsDir - run artifact root for this checkout

 @param seed - draw seed, which a later round overrides to draw a fresh sample

 @param openClient - builds the client one region's probe asks through: `createProbeRunClient` in a run, called once per
 region and never for a pool with nothing to draw

 @param proberModelIds - models asked about each region

 @param perCallTimeoutMs - how long each call may run before it is abandoned

 @param l - logger the probe's lines are written to, tagged here

 @param policy - generation policy the entry read from its environment
 (`readPoolPolicy`), handed to the pool

 @param repository - checkout a required commit is resolved in and every
 recorded commit is placed against, which the entry names: the package's own
 in a run, a throwaway one in a case

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @throws {@link StatedRefusalError} When the settled entries ship no replacement
 over an archive wording to draw from, or no prober answered for any region
 drawn, since a sheet with no item would be kept and refuse the next run; and
 for what the pool, the client and the sheet write refuse with

 @example
 ```ts
 await sampleDamage({
   runsDir,
   seed: 'damage-round-one',
   openClient: createProbeRunClient,
   proberModelIds,
   perCallTimeoutMs: 360_000,
   l,
   policy,
   repository,
   stageSettings,
 },);
 ```
 */
export async function sampleDamage(
  {
    runsDir,
    seed,
    openClient,
    proberModelIds,
    perCallTimeoutMs,
    l,
    policy,
    repository,
    stageSettings,
  }: {
    readonly runsDir: string;
    readonly seed: string;
    readonly openClient: () => RunClient;
    readonly proberModelIds: readonly RosterModelId[];
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly policy: PoolPolicy;
    readonly repository: string;
    readonly stageSettings: StageSettings;
  },
): Promise<void> {
  /**
   Every distinct shipped region in the settled artifacts.
   */
  const {
    regions: pool,
    filledWithoutIncumbent,
  } = await collectShippedRegions({
    runsDir,
    policy,
    repository,
  },);
  if (pool.length === 0) {
    // AN EMPTY SHEET IS KEPT: both files are refused for as long as either
    // stands, so a sheet of no item would stop the draw that follows a pass
    // which settled something to draw from.
    throw new StatedRefusalError({ says: emptyPoolSays({ filledWithoutIncumbent, },), },);
  }
  for (const line of poolLines({
    regionCount: pool.length,
    filledWithoutIncumbent,
    seed,
  },))
    console.log(line,);

  /**
   Sheet items, one per drawn region that could be placed and probed.
   */
  const items: VerifyItem[] = [];
  /* oxlint-disable no-await-in-loop -- sequential by design so this never competes with a running corpus pass for per-model stream slots */
  for (const ref of drawRegions({
    regions: pool,
    seed,
  },)) {
    /**
     Case this region makes.
     */
    const built = buildCase({ ref, },);

    /**
     What the probe says about it, issues withheld as production now runs.
     */
    const report = await runIntroducedDefectProbe({
      client: openClient(),
      proberModelIds,
      sourceText: built.sourceText,
      baselineText: built.baselineText,
      regions: [built.region,],
      issues: [],
      identityContext: '',
      signal: new AbortController().signal,
      perCallTimeoutMs,
      l: tagged({
        tag: 'damage-sample',
        l,
      },),
      stageSettings,
    },);

    /**
     Screened tally of the single region.
     */
    const [tally,] = report.regions;
    if (report.heardProbers === 0) {
      // NO PROBER ANSWERED, so the probe said nothing about this region, which
      // is not the same as the probe saying it is clean: recorded as silent it
      // would read as a negative the human grade is then scored against.
      console.log(`DAMAGE ${ref.entryId} ${ref.regionId} probe=unheard, left off the sheet`,);
      continue;
    }
    if (tally === undefined) {
      throw new Error(
        `unreachable: the probe of one region of ${ref.entryId} returned no tally for it, `
          + 'though it screens every region it is given',
      );
    }

    /**
     Whether the probe corroborated damage at this region.
     */
    const flagged = (tally.corroborated + tally.removalCorroborated) > 0;

    items.push({
      relabelCase: built,
      claims: tally.claims,
      // Labelled by what the PROBE said, not by anything a reader believes,
      // because on this sheet the probe's verdict is the thing under test and
      // the human grade is the truth it is scored against.
      kind: flagged
        ? 'probe-flagged'
        : 'probe-silent',
    },);
    console.log(
      `DAMAGE ${ref.entryId} ${ref.regionId} probe=${
        flagged
          ? 'flagged'
          : 'silent'
      }`,
    );
  }
  /* oxlint-enable no-await-in-loop */
  if (items.length === 0) {
    throw new StatedRefusalError({
      says: 'no prober answered for any region drawn, so none of them can be called silent and no sheet is written',
    },);
  }

  await writeSheetPair({
    dir: runsDir,
    sheetName: 'damage-sheet.md',
    manifestName: 'damage-manifest.json',
    // Claims are stripped so the sheet shows the reader nothing the probe
    // concluded. The manifest keeps them for scoring.
    sheet: formatVerifySheet({
      items: items.map(function withoutClaims(item,) {
        return {
          ...item,
          claims: [],
        };
      },),
      // Half of these the probe never flagged, so the grader is told a mix
      // is coming and nothing about which is which.
      framing: 'blind',
    },),
    manifest: formatVerifyManifest({ items, },),
  },);
  console.log(wroteLine({
    items: items.length,
    runsDir,
  },),);
}

//endregion Damage sample run
