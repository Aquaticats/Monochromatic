import { hasTagAttribute } from './evidence-html-scan.mjs';

// The review refuses to pass when a tag can load something from outside the document, so a scan that misses one shape
// admits an offline review that reaches the network, and one that reports a shape the document does not carry blocks a
// review for nothing. The cases below are the answers the removed patterns gave for the same text: they were recorded
// from `/<script\b[^>]*\bsrc=/i`, `/<link\b[^>]*\bhref=/i` and `/<img\b[^>]*\bsrc="https?:/i`.

// Script tag shapes the review forbids: a `src` attribute anywhere in the tag body.
function scriptTagLoadsSource({ text }) {
  return hasTagAttribute({
    text,
    tagName: 'script',
    attributeName: 'src',
    valuePrefixes: ['='],
    ignoreAsciiCase: true
  });
}

// Link tag shapes the review forbids: an `href` attribute anywhere in the tag body.
function linkTagLoadsSource({ text }) {
  return hasTagAttribute({
    text,
    tagName: 'link',
    attributeName: 'href',
    valuePrefixes: ['='],
    ignoreAsciiCase: true
  });
}

// Image tag shapes the review forbids: an `src` attribute holding an `http:` or `https:` URL.
function imageTagLoadsRemoteSource({ text }) {
  return hasTagAttribute({
    text,
    tagName: 'img',
    attributeName: 'src',
    valuePrefixes: ['="http:', '="https:'],
    ignoreAsciiCase: true
  });
}

// A bare attribute name, bounded on both sides, which is how the scan viewer asked for a `src` at all.
function scriptTagMentionsSource({ text }) {
  return hasTagAttribute({
    text,
    tagName: 'script',
    attributeName: 'src',
    valuePrefixes: [],
    ignoreAsciiCase: true
  });
}

function expectFinding({ scan, text, expected }) {
  const actual = scan({ text });
  if (actual !== expected) {
    throw new Error(`${scan.name} returned ${String(actual)} for ${JSON.stringify(text)}, expected ${String(expected)}.`);
  }
}

// Positive control: every shape is seen when it is present before any absence is trusted.
expectFinding({ scan: scriptTagLoadsSource, text: '<script src="app.js"></script>', expected: true });
expectFinding({ scan: linkTagLoadsSource, text: '<link rel="stylesheet" href="print.css">', expected: true });
expectFinding({
  scan: imageTagLoadsRemoteSource,
  text: '<img src="https://example.test/a.png">',
  expected: true
});
expectFinding({ scan: scriptTagMentionsSource, text: '<script src>', expected: true });

// The tag name must end at a boundary, so a longer tag name is not this tag.
expectFinding({ scan: scriptTagLoadsSource, text: '<scripted src="app.js">', expected: false });
expectFinding({ scan: linkTagLoadsSource, text: '<linkhref="print.css">', expected: false });
expectFinding({ scan: imageTagLoadsRemoteSource, text: '<imgx src="https://example.test/a.png">', expected: false });

// The boundary in the patterns is a word boundary, not a name boundary: the hyphen in a `data-` attribute is not a
// word character, so `data-src=` still carries a `\bsrc=`. A word character before the name does suppress it.
expectFinding({ scan: scriptTagLoadsSource, text: '<script data-src="app.js">', expected: true });
expectFinding({ scan: linkTagLoadsSource, text: '<link data-href="print.css">', expected: true });
expectFinding({ scan: imageTagLoadsRemoteSource, text: '<img data-src="https://example.test/a.png">', expected: true });
expectFinding({ scan: scriptTagLoadsSource, text: '<script xsrc="app.js">', expected: false });
expectFinding({ scan: scriptTagMentionsSource, text: '<script srcx="app.js">', expected: false });

// The scan folds ASCII case the way the `i` flag did, in the tag name, the attribute name and the URL scheme.
expectFinding({ scan: scriptTagLoadsSource, text: '<SCRIPT SRC="app.js"></SCRIPT>', expected: true });
expectFinding({ scan: linkTagLoadsSource, text: '<link HREF="print.css">', expected: true });
expectFinding({ scan: imageTagLoadsRemoteSource, text: '<IMG SRC="HTTPS://example.test/a.png">', expected: true });

// A newline or a tab inside the tag body is whitespace like any other, so the attribute is still found.
expectFinding({ scan: scriptTagLoadsSource, text: '<script\n  src="app.js"\n></script>', expected: true });
expectFinding({ scan: scriptTagMentionsSource, text: '<script\tsrc=app.js>', expected: true });

// The tag body ends at the first `>`, which is where the pattern's `[^>]*` run ended: an attribute in the next tag is
// not this tag's.
expectFinding({
  scan: scriptTagLoadsSource,
  text: '<script>inline(' + ') > 0</script><script src="app.js">',
  expected: true
});
expectFinding({ scan: scriptTagLoadsSource, text: '<script>inline()</script>', expected: false });
expectFinding({
  scan: scriptTagLoadsSource,
  text: '<div data-x="<script src=1>">',
  expected: true
});

// The remote-source shape needs the `http:` or `https:` scheme with its quotes, so a protocol-relative URL, a `data:`
// URL and a scheme without the quote are all accepted.
expectFinding({
  scan: imageTagLoadsRemoteSource,
  text: '<img src="//example.test/a.png">',
  expected: false
});
expectFinding({
  scan: imageTagLoadsRemoteSource,
  text: '<img src="data:image/png;base64,AA">',
  expected: false
});
expectFinding({
  scan: imageTagLoadsRemoteSource,
  text: '<img src=https://example.test/a.png>',
  expected: false
});

// A bare attribute name is bounded on the right too, so `src=` is a mention and `srcx` is not.
expectFinding({ scan: scriptTagMentionsSource, text: '<script src="app.js">', expected: true });
expectFinding({ scan: scriptTagMentionsSource, text: '<script data-src>', expected: true });
expectFinding({ scan: scriptTagMentionsSource, text: '<script data-srcx>', expected: false });

// A document that never closes the tag leaves the body open to its end, as the pattern's `[^>]*` run did.
expectFinding({ scan: scriptTagLoadsSource, text: '<script src="app.js"', expected: true });
expectFinding({ scan: linkTagLoadsSource, text: '<link rel="stylesheet"', expected: false });

console.log('Evidence HTML scan: tag attributes keep their tag body, word boundaries, ASCII case folding and value prefixes.');
