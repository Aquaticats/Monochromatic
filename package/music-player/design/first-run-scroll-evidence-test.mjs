import {
  assertFirstRunScrollEvidence,
  FirstRunScrollEvidenceError
} from './first-run-scroll-evidence.mjs';

//region Synthetic protocol controls, no native gesture or image claim follows
const node = {
  text: 'Exact authored analysis body',
  package: 'dev.monochromatic.musicplayer',
  bounds: [
    100,
    200,
    400,
    600
  ]
};
const moved = {
  ...node,
  bounds: [
    100,
    150,
    400,
    550
  ]
};
const positive = {
  appRgbChanged: true,
  beforeNodes: [node],
  afterNodes: [moved],
  sameEnvironment: true
};
if (assertFirstRunScrollEvidence(positive)
  .verticalDisplacement
  !== (-50)) throw new Error('Positive scroll proof failed.');
const cases = [
  {
    ...positive,
    appRgbChanged: false,
    afterNodes: [node]
  },
  {
    ...positive,
    afterNodes: [node]
  },
  {
    ...positive,
    appRgbChanged: false
  },
  {
    ...positive,
    sameEnvironment: false
  },
  {
    ...positive,
    beforeNodes: []
  },
  {
    ...positive,
    afterNodes: [
      moved,
      moved
    ]
  },
  {
    ...positive,
    afterNodes: [{
      ...moved,
      text: 'Different body'
    }]
  },
  {
    ...positive,
    afterNodes: [{
      ...moved,
      package: 'different.application'
    }]
  },
  {
    ...positive,
    afterNodes: [{
      ...moved,
      bounds: [
        100,
        150
      ]
    }]
  },
  {
    ...positive,
    afterNodes: [{
      ...moved,
      bounds: [
        100,
        Number.NaN,
        400,
        550
      ]
    }]
  },
];
for (const input of cases) {
  let rejected = false;
  try {
    assertFirstRunScrollEvidence(input);
  } catch (error) {
    if (!(error instanceof FirstRunScrollEvidenceError)) throw error;
    rejected = true;
  }
  if (!rejected) throw new Error('Invalid scroll proof was accepted.');
}
console.log('First-run scroll proof positive, signal-isolation, identity, multiplicity, environment and rectangle controls passed.');
//endregion
