import { findPngDataPayloads, } from './evidence-text-scan.ts';

/**
 Asserts that one fixture yields exactly the payloads the removed pattern reported.
 
 @param text - document text the scan reads
 
 @param expected - payloads recorded from the removed pattern for this text
 */
function expectPayloads(
  {
    text,
    expected,
  }: {
    readonly text: string;
    readonly expected: readonly string[];
  },
): void {
  /**
   Payloads the index scan reported for the fixture.
   */
  const actual = findPngDataPayloads({ text, },);
  if (JSON.stringify(actual,)
    !== JSON.stringify(expected,)) {
    throw new Error(
      `findPngDataPayloads returned ${JSON.stringify(actual,)} for ${JSON.stringify(text,)},`
        + ` expected ${JSON.stringify(expected,)}.`,
    );
  }
}

/**
 Fixture text holding one hundred prefixed payloads, one per space-separated word.
 
 @returns document text with a payload after every prefix
 */
function manyPayloadText(): string {
  /**
   Prefixed payloads, joined with a separator outside the base64 alphabet.
   */
  const parts: string[] = [];
  for (let index = 0; index < 100; index += 1) {
    parts.push(`data:image/png;base64,P${index.toString(16,)}`,);
  }
  return parts.join(' ',);
}

/**
 Payloads {@link manyPayloadText} embeds, in document order.
 
 @returns payload labels without their `data:` URL prefix
 */
function manyPayloadExpectation(): readonly string[] {
  /**
   Payload labels, derived from the same counter the fixture uses.
   */
  const labels: string[] = [];
  for (let index = 0; index < 100; index += 1) {
    labels.push(`P${index.toString(16,)}`,);
  }
  return labels;
}

// The review validators read payloads out of their own committed HTML, so a scan
// that reports one payload too many, drops one, or slices one wrongly admits a
// broken artifact. Each case pins a property the removed
// `data:image/png;base64,` pattern had: document order, one payload per prefix,
// the maximal run, an empty run reporting nothing, and payload text without the
// prefix.

// Positive control: the scan can see a payload at all before its silence is trusted.
expectPayloads({
  text: '<img src="data:image/png;base64,AAEC">',
  expected: ['AAEC',],
},);

// Document order, and a second prefix after the first payload's run.
expectPayloads({
  text: 'data:image/png;base64,AA data:image/png;base64,BB',
  expected: [
    'AA',
    'BB',
  ],
},);

// The run is maximal and carries every base64 symbol, padding included.
expectPayloads({
  text: 'data:image/png;base64,aZ0+/=!',
  expected: ['aZ0+/=',],
},);

// A prefix with no base64 code unit after it reports nothing, and a later prefix is
// still found.
expectPayloads({
  text: 'data:image/png;base64,<data:image/png;base64,CC',
  expected: ['CC',],
},);

// A prefix at the very end of the document has no payload.
expectPayloads({
  text: 'data:image/png;base64,',
  expected: [],
},);

// Adjacent payloads separate only where a code unit outside the alphabet sits
// between them; base64 letters inside the next prefix belong to the earlier run,
// which is what the pattern's greedy class did too.
expectPayloads({
  text: 'data:image/png;base64,AA"data:image/png;base64,BB',
  expected: [
    'AA',
    'BB',
  ],
},);
expectPayloads({
  text: 'data:image/png;base64,AAdata:image/png;base64,BB',
  expected: ['AAdata',],
},);

// A prefix that carries no payload is skipped without swallowing the payload that
// follows it, and a prefix that is not at the document start is still found.
expectPayloads({
  text: 'data:image/png;base64,--data:image/png;base64,BB',
  expected: ['BB',],
},);
expectPayloads({
  text: 'xdata:image/png;base64,AAy',
  expected: ['AAy',],
},);

// A document with many payloads keeps every one of them, in order.
expectPayloads({
  text: manyPayloadText(),
  expected: manyPayloadExpectation(),
},);

console.log('Evidence text scan: base64 payload extraction keeps order, maximal runs and prefix boundaries.');
