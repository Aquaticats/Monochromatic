import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  IntroducedDefectReport,
  RegionDefectTally,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { SensitivityArm, } from './probe-sensitivity-arms.ts';

//region Probe sensitivity print
// What the probe sensitivity runner says, as text a case can read whole: the
// opening line, one line per arm with one per claim under it, and the notes
// that close the run.

/**
 Opening line naming the list production sends and how many arms follow.

 @param production - list production sends, which every arm is read against

 @param armCount - arms the run will ask about

 @returns The line, without a newline

 @example
 ```ts
 console.log(sensitivityOpening({ production: 'withheld', armCount: 3, },),);
 ```
 */
export function sensitivityOpening(
  {
    production,
    armCount,
  }: {
    readonly production: string;
    readonly armCount: number;
  },
): string {
  return `SENSITIVITY production sends list=${production}; ${String(armCount,)} ${
    wordForCount({
      count: armCount,
      one: 'arm',
      many: 'arms',
    },)
  } ${
    wordForCount({
      count: armCount,
      one: 'follows',
      many: 'follow',
    },)
  }`;
}

/**
 Lines one arm leaves: the tally line, then one line per screened claim.

 @param arm - region, list, issue and framing that were sent

 @param report - what the probe said about the arm's one region

 @param tally - screened tally of that region

 @returns The lines, without newlines, in the order they print

 @example
 ```ts
 for (const line of sensitivityArmLines({ arm, report, tally, },)) console.log(line,);
 ```
 */
export function sensitivityArmLines(
  {
    arm,
    report,
    tally,
  }: {
    readonly arm: SensitivityArm;
    readonly report: IntroducedDefectReport;
    readonly tally: RegionDefectTally;
  },
): readonly string[] {
  /**
   Envelope of the region the arm asks about.
   */
  const { envelopeId, } = arm.region;

  return [
    `SENSITIVITY ${envelopeId} list=${arm.list} issue=${arm.issue} expected=${
      arm.expectation
    } heard=${
      String(report.heardProbers,)
    }/${String(report.configuredProbers,)} corroborated=${
      String(tally.corroborated,)
    } removal=${String(tally.removalCorroborated,)} contradicted=${
      String(tally.contradicted,)
    } unanchored=${String(tally.unanchored,)} preExisting=${
      String(tally.preExisting,)
    } noneFound=${
      String(tally.noneFound,)
    } uncertain=${String(tally.uncertain,)}`,
    ...tally.claims
      .map(function claimLine(claim,): string {
        return `  claim ${claim.admissibility} (${claim.category}/${claim.severity})`;
      },),
  ];
}

/**
 Notes that close the run, saying how to read its lines.

 @param production - list production sends, which the first note closes on

 @returns The notes, one per line printed

 @example
 ```ts
 for (const note of sensitivityNotes({ production: 'withheld', },)) console.log(note,);
 ```
 */
export function sensitivityNotes({ production, }: { readonly production: string; },): readonly string[] {
  return [
    `NOTE compare each accuracy region's three lines. list=none against list=withheld differ only `
      + 'in the deterministic screen, which dismisses a claim restating the prior issue; '
      + 'list=withheld against list=rendered differ only in the prompt, and rendered is the prompt '
      + 'production abandoned because it silenced the stage. A region that reports damage with '
      + 'list=none and goes quiet with list=withheld is one whose claims merely restate the prior '
      + 'issue; one that goes quiet only with list=rendered is one the rendered prompt silences, '
      + `and its zeros would mean nothing in a run that rendered. Production sends list=${production}.`,
    'NOTE the refinement/* lines test the naturalness framing under production\'s list. Its '
      + 'control is refinement/clean: a claim there means the probe reads mere rephrasing as '
      + 'damage, which would flag every refinement the lane ever ships.',
    'NOTE the deletion/* lines vary what the issue list SAYS, under both lists that carry one. '
      + 'With list=rendered the prober reads the label; with list=withheld only the screen does. '
      + 'deletion/unlabelled and deletion/mislabelled delete identical source-supported text; a '
      + 'gap between them under list=rendered measures how far a false accepted issue can talk '
      + 'the probe out of seeing real damage, and under list=withheld how far the screen dismisses '
      + 'a real claim as restating it. deletion/licensed is the negative control, where silence '
      + 'is correct.',
  ];
}

//endregion Probe sensitivity print
