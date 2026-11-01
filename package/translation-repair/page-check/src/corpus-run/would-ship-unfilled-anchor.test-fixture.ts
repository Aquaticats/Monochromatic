import type { WouldShipSource, } from '../../dist/final/node/index.mjs';

//region Would-ship unfilled anchor
// AN ARTIFACT WHOSE ONE SLICE IS AN ANCHOR NOBODY FILLED, reached through a
// declined contest over an archive that holds nothing: the translate lane
// backed no candidate and recorded the slice unfilled, leaving the contest
// two blank lanes to choose between and no archive wording to fall back on.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. The publish-fixed-replacements and
// publish-fixed tests kept their own copy of this artifact; both now import
// it from here.

/**
 Builds an artifact whose one slice is an ANCHOR nobody filled.

 @returns Artifact whose one slice is an unfilled anchor

 @example
 ```ts
 const artifact = artifactWithAnUnfilledAnchor();
 ```
 */
export function artifactWithAnUnfilledAnchor(): WouldShipSource {
  return {
    comparison: [
      {
        sliceIndex: 1,
        incumbentKind: 'absent',
        incumbentText: '',
        repairText: '',
        translateText: '',
        laneRelation: 'both-differ',
        repairOutcome: { kind: 'unfilled', },
        translateOutcome: { kind: 'unfilled', },
        decisionComparison: {
          kind: 'comparable',
          verdict: 'same',
        },
        repairDelivery: { kind: 'gap-remains', },
        translateDelivery: { kind: 'gap-remains', },
      },
    ],
    consolidation: { kind: 'not-run', },
    laneSelection: {
      kind: 'contested',
      slices: [
        {
          sliceIndex: 1,
          verdict: { kind: 'settled-neither', },
          ballots: [],
          usable: 3,
        },
      ],
    },
  };
}

//endregion Would-ship unfilled anchor
