import {
  summarizeProbeTelemetry,
  type TelemetryProbeReading,
} from '@monochromatic-dev/translation-repair-lane-contest/ts';
import type { StageRosterCoverage, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Probe telemetry report
// The lines score-probe prints about a run, split out when the driver crossed
// its size cap.
//
// Kept apart from the JOIN half on purpose. Everything here is a property of
// the run alone and prints with no human grades in hand; the join needs a
// graded sheet and a manifest and answers a different question.
//
// COUNTS ONLY, like the driver: artifacts quote unlicensed corpus text, and
// this output is meant to be pasteable into a verdict.

/**
 Probe readings of one artifact, kept under the entry that carried them.

 Grouped rather than flattened because envelope ids are derived from the text
 they cover, so two entries sharing a paragraph name one id for regions that
 serve different issues.

 @example
 ```ts
 const group: EntryReadings = { entryId: 'Kitten.json', readings: [], };
 ```
 */
export type EntryReadings = {
  /**
   Artifact the readings came from.
   */
  readonly entryId: string;

  /**
   That artifact's probe readings.
   */
  readonly readings: readonly TelemetryProbeReading[];
};

/**
 Everything a run's artifacts yield, before any human grade is joined.

 @example
 ```ts
 const gathered: GatheredProbe = await gatherReadings({ artifactsDir, },);
 ```
 */
export type GatheredProbe = {
  /**
   Accuracy-probe readings, grouped by entry.
   */
  readonly readings: readonly EntryReadings[];

  /**
   Readings keyed by the issue whose record carried them.
   */
  readonly byIssueId: ReadonlyMap<string, TelemetryProbeReading>;

  /**
   Issues whose slice the naturalness lane rewrote after probing.
   */
  readonly refinedIssueIds: ReadonlySet<string>;

  /**
   Naturalness-lane readings, grouped by entry.
   */
  readonly refinementReadings: readonly EntryReadings[];

  /**
   How the editor stage fared against its configured roster.
   */
  readonly editorRoster: StageRosterCoverage;

  /**
   How the refine stage fared against its configured roster.
   */
  readonly refineRoster: StageRosterCoverage;

  /**
   Artifacts carrying at least one rewritten slice.
   */
  readonly entriesWithRewrites: number;

  /**
   Artifacts read.
   */
  readonly entries: number;

  /**
   Repair-lane records seen across every artifact read, probed or not.

   THE DENOMINATOR EVERY PROBE RATE DIVIDES BY. Named for its lane because
   "shipped" is the repair lane's word for its own output, not a claim that
   any page carries it: the contest and the consolidation both sit downstream
   of the disposition this counts.
   */
  readonly repairShippedRecords: number;

  /**
   Repair-lane records carrying no probe field, a subset of the count beside
   it.
   */
  readonly repairUnprobedRecords: number;
};

/**
 Prints a run's probe telemetry, roster coverage, and the notes that keep
 each number from being read as the wrong thing.

 @param gathered - readings and coverage across every settled artifact

 @example
 ```ts
 reportProbeTelemetry({ gathered, },);
 ```
 */
export function reportProbeTelemetry(
  { gathered, }: { readonly gathered: GatheredProbe; },
): void {
  /**
   Summary over distinct shipped regions.
   */
  const summary = summarizeProbeTelemetry({ entries: gathered.readings, },);

  console.log(
    `PROBE entries=${String(gathered.entries,)} repairShippedRecords=${
      String(gathered.repairShippedRecords,)
    } repairUnprobedRecords=${String(gathered.repairUnprobedRecords,)} regions=${
      String(summary.regions,)
    } majorityIntroduced=${String(summary.majorityIntroduced,)} minorityIntroduced=${
      String(summary.minorityIntroduced,)
    } noneIntroduced=${String(summary.noneIntroduced,)}`,
  );
  console.log(
    `CLAIMS added=${String(summary.corroborated,)} dropped=${
      String(summary.removalCorroborated,)
    } contradicted=${String(summary.contradicted,)} unanchored=${
      String(summary.unanchored,)
    } degradedRosterRegions=${String(summary.degradedRosterRegions,)}`,
  );

  // NOT GATED ON `entries` AS WELL, and the redundant guard should not be
  // added back. `resolvePool` raises EmptyPoolError before any artifact is
  // read when generation filtering leaves nothing, so a report reaching this
  // line has already read at least one artifact. A zero here therefore says
  // something about what those artifacts hold, never that none were found.
  if (gathered.repairShippedRecords === 0)
    console.log(
      'NOTE repairShippedRecords=0 means no artifact read here carries a '
        + 'repair-lane record that lane applied, so every PROBE and CLAIMS '
        + 'figure is zero by construction rather than by measurement. '
        + 'majorityIntroduced=0 is NOT evidence the probe cleared anything: it '
        + 'had nothing to look at. Read editorOffered on the ROSTER line to '
        + 'tell the two cases apart, since it counts slices the editor was '
        + 'asked to repair. Above zero there, the lane was asked and nothing it '
        + 'produced shipped, which is worth a look; at zero, the critics filed '
        + 'nothing to repair and this run is quiet rather than clean.',
    );

  /**
   Summary over the naturalness lane's own rewrites.

   Reported on its own line rather than folded into the accuracy figures,
   because the two audit different edits against different baselines. Its
   region count is rewritten SLICES, not replaced envelopes, so the two are
   not comparable as rates either.
   */
  const refinement = summarizeProbeTelemetry({
    entries: gathered.refinementReadings,
  },);
  console.log(
    `REFINEMENT rewrittenSlices=${String(refinement.regions,)} majorityIntroduced=${
      String(refinement.majorityIntroduced,)
    } minorityIntroduced=${String(refinement.minorityIntroduced,)} noneIntroduced=${
      String(refinement.noneIntroduced,)
    } added=${String(refinement.corroborated,)} dropped=${
      String(refinement.removalCorroborated,)
    } contradicted=${String(refinement.contradicted,)} unanchored=${
      String(refinement.unanchored,)
    }`,
  );
  // Reported for the stages whose degradation is otherwise INVISIBLE. Critic
  // and panel announce their heard counts per chunk in the run log and carry a
  // quorum rule; these two do not, and a stage that ran short here produces
  // output shaped exactly like a healthy stage with less to do.
  console.log(
    `ROSTER editorOffered=${String(gathered.editorRoster
      .offered,)} editorDegraded=${
      String(gathered.editorRoster
        .degraded,)
    } editorSilent=${String(gathered.editorRoster
      .silent,)} refineOffered=${
      String(gathered.refineRoster
        .offered,)
    } refineDegraded=${String(gathered.refineRoster
      .degraded,)} refineSilent=${
      String(gathered.refineRoster
        .silent,)
    } entriesWithRewrites=${
      String(gathered.entriesWithRewrites,)
    }/${String(gathered.entries,)}`,
  );
  if (gathered.editorRoster
    .degraded
    > 0)
    console.log(
      'NOTE editorDegraded counts chunks repaired with FEWER editors than the '
        + 'roster configures. The editor ensemble exists so no single model '
        + 'writes the shipped text, so on those chunks that property does not '
        + 'hold: judges still chose, but they chose among one model\'s '
        + 'proposals.',
    );
  if (gathered.refineRoster
    .silent
    > 0)
    console.log(
      'NOTE refineSilent counts slices where NO refiner answered, so the '
        + 'naturalness lane could not run there. One model refines, and a '
        + 'roster of one has no quorum to lose, so its failure shows up '
        + 'nowhere else: the audit simply does not grow, which is also what a '
        + 'run with nothing worth rewriting looks like.',
    );
  if (refinement.regions === 0)
    console.log(
      'NOTE rewrittenSlices=0 means no artifact here carries a refinement '
        + 'audit. That is what artifacts written before the lane was audited '
        + 'look like, and it is NOT evidence the lane rewrote nothing: read '
        + 'the ROSTER line to tell the two apart.',
    );
}

//endregion Probe telemetry report
