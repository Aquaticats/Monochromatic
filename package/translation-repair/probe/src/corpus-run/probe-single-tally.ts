import type {
  IntroducedDefectReport,
  RegionDefectTally,
} from '@monochromatic-dev/translation-repair-adjudication/ts';

//region Probe single tally
// The one screened tally a probe over a single region leaves, read the same
// way by every runner that asks the introduced-defect probe about one region
// at a time.

/**
 Screened tally of the one region a report was asked about.

 @param report - report of a probe given exactly one region

 @returns The region's tally

 @throws {@link Error} when the report holds no tally, which a probe given a
 region cannot produce: it screens every region it is given and answers the
 empty report only for an empty list

 @example
 ```ts
 const tally = singleRegionTally({ report, },);
 ```
 */
export function singleRegionTally({ report, }: { readonly report: IntroducedDefectReport; },): RegionDefectTally {
  /**
   Tally of the single region.
   */
  const [tally,] = report.regions;
  if (tally === undefined) {
    throw new Error(
      'unreachable: the probe answered with no region tally although it was given one region, '
        + 'and it screens every region it is given',
    );
  }

  return tally;
}

//endregion Probe single tally
