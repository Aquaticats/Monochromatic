/**
 The betterleaks porter's character walks, held to the answers the porter's regular expressions gave: the
 rule-body line readers, the `[[rules]]` header reader and the two PCRE rewrite passes {@link pcreToResharp}
 composes.

 The package's scripts are TypeScript files node runs directly and it carries no test-harness dependency, so the
 cases run as a node script that exits non-zero on the first answer a walk gets wrong. The `test:scripts` task runs
 it, and so does this package's `test` task.

 @module
 */

import assert from 'node:assert/strict';

import {
  lazyQuantifierMarkersDropped,
  namedCaptureGroupsMadeNonCapturing,
  pcreToResharp,
} from './port-betterleaks-pcre.ts';
import {
  type LineCase,
  type TextCase,
  bodyLineCases,
  composeCases,
  headerCases,
  lazyMarkerCases,
  namedCaptureCases,
} from './port-betterleaks-scan-cases.data.ts';
import {
  readRuleBodyLine,
  rulesTableHeaderPresent,
} from './port-betterleaks-toml-lines.ts';

//region Cases

/**
 What one rewrite pass answers for a pattern.
 */
type RewritePass = (arguments_: { readonly pattern: string; },) => string;

/**
 Runs one rule-body line case.
 
 @param testCase - the line and what reading it must answer
 */
function runBodyLineCase({ testCase, }: { readonly testCase: LineCase; },): void {
  assert.deepEqual(
    readRuleBodyLine({ line: testCase.line, },),
    testCase.expected,
    testCase.subject,
  );
}

/**
 Runs one rewrite case through a pass.
 
 @param testCase - the pattern and the text the pass must yield
 
 @param pass - the rewrite the case runs
 */
function runTextCase({ testCase, pass, }: { readonly testCase: TextCase; readonly pass: RewritePass; },): void {
  assert.equal(
    pass({ pattern: testCase.pattern, },),
    testCase.expected,
    testCase.subject,
  );
}

for (const [subject, line, expected,] of headerCases) {
  assert.equal(
    rulesTableHeaderPresent({ line, },),
    expected,
    subject,
  );
}
for (const testCase of bodyLineCases) {
  runBodyLineCase({ testCase, },);
}
for (const testCase of namedCaptureCases) {
  runTextCase({
    testCase,
    pass: namedCaptureGroupsMadeNonCapturing,
  },);
}
for (const testCase of lazyMarkerCases) {
  runTextCase({
    testCase,
    pass: lazyQuantifierMarkersDropped,
  },);
}
for (const testCase of composeCases) {
  runTextCase({
    testCase,
    pass: pcreToResharp,
  },);
}
/**
 Every case this file ran, so a run that answered nothing is visible as a count of zero.
 */
const caseCount = headerCases.length + bodyLineCases.length + namedCaptureCases.length + lazyMarkerCases.length
  + composeCases.length;
console.log(`betterleaks porter walks: ${String(caseCount,)} cases answered as the porter requires.`,);
//endregion Cases
