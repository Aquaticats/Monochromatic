import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type {
  IntroducedDefectReport,
  RegionDefectTally,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import type { RelabelCase, } from './probe-relabel-case.ts';

//region Probe relabel print
// What the probe relabel runner says, as text a case can read whole: the two
// counts that open the run, the header and run-recorded lines of each case,
// the counts of each probe, and the notes that close the run.

/**
 Line saying how many damaged regions were rebuilt.

 @param count - damaged regions rebuilt

 @returns The line, without a newline

 @example
 ```ts
 console.log(relabelRebuilt({ count: 2, },),);
 ```
 */
export function relabelRebuilt({ count, }: { readonly count: number; },): string {
  return `RELABEL rebuilt ${String(count,)} distinct damaged ${
    wordForCount({
      count,
      one: 'region',
      many: 'regions',
    },)
  }`;
}

/**
 Line saying how many unflagged control regions were gathered.

 @param count - control regions gathered

 @returns The line, without a newline

 @example
 ```ts
 console.log(relabelGathered({ count: 1, },),);
 ```
 */
export function relabelGathered({ count, }: { readonly count: number; },): string {
  return `RELABEL gathered ${String(count,)} unflagged control ${
    wordForCount({
      count,
      one: 'region',
      many: 'regions',
    },)
  }`;
}

/**
 Lines that head one case: the edit under test, then what the run recorded.

 @param relabelCase - rebuilt region case

 @returns The two lines, without newlines

 @example
 ```ts
 for (const line of relabelCaseLines({ relabelCase, },)) console.log(line,);
 ```
 */
export function relabelCaseLines({ relabelCase, }: { readonly relabelCase: RelabelCase; },): readonly string[] {
  return [
    `RELABEL ${relabelCase.entryId} positions=${
      relabelCase.positions
        .join(
          '+',
        )
    } issuesServed=${
      String(relabelCase.issues
        .length,)
    } beforeChars=${
      String(relabelCase.region
        .before
        .length,)
    } afterChars=${
      String(relabelCase.region
        .editorAfter
        .length,)
    }`,
    `  run-recorded  ${relabelCase.recorded}`,
  ];
}

/**
 Lines one probe leaves under its case: one per claim, naming the prober.

 Which prober spoke is printed rather than only how many did, because the
 corpus telemetry shows the three disagree by more than an order of magnitude
 about how often an edit is worth a claim, 0.095 against 0.006, and that
 disagreement cannot be settled from counts that hide the speaker.

 @param tally - screened tally of the single region

 @returns One line per claim, without newlines

 @example
 ```ts
 for (const line of relabelClaimLines({ tally, },)) console.log(line,);
 ```
 */
export function relabelClaimLines({ tally, }: { readonly tally: RegionDefectTally; },): readonly string[] {
  return tally.claims
    .map(function claimLine(claim,): string {
      return `    ${claim.modelId} ${claim.admissibility} (${claim.category})`;
    },);
}

/**
 Counts one probe printed beside its label.

 @param report - what the probe said about the region

 @param tally - screened tally of that region

 @returns One line of counts

 @example
 ```ts
 const counts = relabelCounts({ report, tally, },);
 ```
 */
export function relabelCounts(
  {
    report,
    tally,
  }: {
    readonly report: IntroducedDefectReport;
    readonly tally: RegionDefectTally;
  },
): string {
  return `heard=${String(report.heardProbers,)}/${
    String(report.configuredProbers,)
  } corroborated=${String(tally.corroborated,)} removal=${
    String(tally.removalCorroborated,)
  } contradicted=${String(tally.contradicted,)} unanchored=${
    String(tally.unanchored,)
  } preExisting=${
    String(tally.preExisting,)
  } none=${String(tally.noneFound,)} uncertain=${
    String(tally.uncertain,)
  }`;
}

/**
 Notes that close the run, saying how to read its lines.

 @param production - disclosure production sends

 @param other - disclosure the second arm sends instead

 @returns The notes, one per line printed

 @example
 ```ts
 for (const note of relabelNotes({ production: 'withheld', other: 'rendered', },)) console.log(note,);
 ```
 */
export function relabelNotes(
  {
    production,
    other,
  }: {
    readonly production: string;
    readonly other: string;
  },
): readonly string[] {
  return [
    `NOTE production sends issues-${production}. Compare it against `
      + `issues-${other} on each region: a region that reports damage under one prompt `
      + 'only is one the other prompt talks the probe out of. Compare it against issues-absent: a '
      + 'region that reports damage only with no list known is one whose claims the screen '
      + 'dismisses as restating a prior issue. A region dark under all three exonerates the label '
      + 'and indicts the difficulty of the judgement.',
    'NOTE a control line prints positions= empty. Read the issues-absent arm across controls '
      + 'against the issues-absent arm across damaged regions: similar rates mean the unlabelled '
      + 'prober is re-reporting pre-existing defects and the damaged result proves nothing, and a '
      + 'much lower control rate means the issue list is suppressing real detections.',
  ];
}

//endregion Probe relabel print
