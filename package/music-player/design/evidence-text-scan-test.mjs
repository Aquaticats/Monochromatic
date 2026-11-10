import {
  findPngDataPayloads,
  isSearchFilenameCaptureName
} from './evidence-text-scan.mjs';

// The review validators read payloads out of their own committed HTML, so a scan that reports one payload too many,
// drops one, or mis-slices a payload admits a broken artifact. Each case pins a property the removed
// `data:image/png;base64,` pattern had: document order, one payload per prefix, the maximal run, an empty run
// reporting nothing, and payload text without the prefix.

function expectPayloads({
  text,
  expected
}) {
  const actual = findPngDataPayloads({ text });
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(
      `findPngDataPayloads returned ${JSON.stringify(actual)} for ${JSON.stringify(text)},`
        + ` expected ${JSON.stringify(expected)}.`
    );
  }
}

// Fixture text holding one hundred prefixed payloads, one per space-separated word.
function manyPayloadText() {
  const parts = [];
  for (let index = 0; index < 100; index += 1)
    parts.push(`data:image/png;base64,P${index.toString(16)}`);
  return parts.join(' ');
}

// Payloads `manyPayloadText` embeds, in document order.
function manyPayloadExpectation() {
  const labels = [];
  for (let index = 0; index < 100; index += 1)
    labels.push(`P${index.toString(16)}`);
  return labels;
}

// Positive control: the scan can see a payload at all before its silence is trusted.
expectPayloads({
  text: '<img src="data:image/png;base64,AAEC">',
  expected: ['AAEC']
});

// Document order, and a second prefix after the first payload's run.
expectPayloads({
  text: 'data:image/png;base64,AA data:image/png;base64,BB',
  expected: [
    'AA',
    'BB'
  ]
});

// The run is maximal and carries every base64 symbol, padding included.
expectPayloads({
  text: 'data:image/png;base64,aZ0+/=!',
  expected: ['aZ0+/=']
});

// A prefix with no base64 code unit after it reports nothing, and a later prefix is still found.
expectPayloads({
  text: 'data:image/png;base64,<data:image/png;base64,CC',
  expected: ['CC']
});

// A prefix at the very end of the document has no payload.
expectPayloads({
  text: 'data:image/png;base64,',
  expected: []
});

// Adjacent payloads separate only where a code unit outside the alphabet sits between them; base64 letters inside the
// next prefix belong to the earlier run, which is what the pattern's greedy class did too.
expectPayloads({
  text: 'data:image/png;base64,AA"data:image/png;base64,BB',
  expected: [
    'AA',
    'BB'
  ]
});
expectPayloads({
  text: 'data:image/png;base64,AAdata:image/png;base64,BB',
  expected: ['AAdata']
});

// A prefix that carries no payload is skipped without swallowing the payload that follows it, and a prefix that is not
// at the document start is still found.
expectPayloads({
  text: 'data:image/png;base64,--data:image/png;base64,BB',
  expected: ['BB']
});
expectPayloads({
  text: 'xdata:image/png;base64,AAy',
  expected: ['AAy']
});

// A document with many payloads keeps every one of them, in order.
expectPayloads({
  text: manyPayloadText(),
  expected: manyPayloadExpectation()
});

console.log('Evidence text scan: base64 payload extraction keeps order, maximal runs and prefix boundaries.');

function expectSearchFilenameCaptureName({
  name,
  accepted
}) {
  const actual = isSearchFilenameCaptureName({ name });
  if (actual !== accepted) {
    throw new Error(
      `isSearchFilenameCaptureName returned ${String(actual)} for ${JSON.stringify(name)},`
        + ` expected ${String(accepted)}.`
    );
  }
}

// The review throws when a manifest names a capture outside this rule, so the rule is what keeps the review to
// committed evidence under `questions/evidence/`. Positive control: both source axes are accepted before any
// rejection is trusted.
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-inner-dark-s100.png',
  accepted: true
});
expectSearchFilenameCaptureName({
  name: 'search-filename-actual-player-cover-light-s200.png',
  accepted: true
});
expectSearchFilenameCaptureName({
  name: 'search-filename-actual-player-a-b-c.png',
  accepted: true
});

// An empty slug, a case-folded slug, a dot, an underscore, a space, a traversal segment and the wrong suffix tail are
// all outside the rule.
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-A-s100.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-a.PNG',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-a.b.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-a_b.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-a b.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-../etc.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-a.png.png',
  accepted: false
});

// The rule is anchored at both ends and names one of two source axes, so a name carrying neither axis, or text before
// or after it, is outside the rule.
expectSearchFilenameCaptureName({
  name: 'search-filename-single-a.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'xsearch-filename-comparison-a.png',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-comparison-actual-player-a.png',
  accepted: true
});
expectSearchFilenameCaptureName({
  name: '',
  accepted: false
});
expectSearchFilenameCaptureName({
  name: 'search-filename-',
  accepted: false
});

console.log(
  'Evidence text scan: the filename capture-name rule keeps both source axes, a lowercase-digit slug and the anchored suffix.'
);
