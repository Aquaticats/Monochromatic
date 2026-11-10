// Checks of the cursor scans that stand in for the design tools' regular expressions.
// What: each case names the answer the pattern the scan replaced gave, read back from the pattern
// itself before it was deleted, or the answer the printed builder shape demands.
// Why: every scan replaces a pattern that decided whether an evidence page is accepted, so a scan that
// silently answers differently must fail here rather than pass an unchecked page.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { needRuleNames, ruleNameAt } from './text-scan.mjs';

//region The `need` rule field of a builder's own source

test('reads the rule name from a one-line need call', () => {
  // The deleted pattern `/need\(\{ rule: '([a-z-]+)',/g` answered exactly this.
  assert.deepEqual([...needRuleNames({ source: "need({ rule: 'no-input-control', holds: true });" })], ['no-input-control']);
});

test('reads the rule name when the call is laid out over several lines', () => {
  // The printed builder puts `need({` and `rule:` on separate lines, which the deleted pattern could
  // not see at all.
  const source = "  need({\n    rule: 'png-size',\n    holds: true,\n    detail: 'x'\n  });";
  assert.deepEqual([...needRuleNames({ source })], ['png-size']);
});

test('tolerates any whitespace between the call, its object and its field', () => {
  assert.deepEqual([...needRuleNames({ source: "need({rule: 'a-b'," })], ['a-b']);
  assert.deepEqual([...needRuleNames({ source: "need(  {\n\trule:\n\t\t'x'," })], ['x']);
});

test('keeps the order of the calls and their repeats', () => {
  const source = "need({ rule: 'b',\nneed({ rule: 'a',\nneed({ rule: 'b',";
  assert.deepEqual([...needRuleNames({ source })], [
    'b',
    'a',
    'b'
  ]);
});

test('skips a need call that declares no rule field', () => {
  assert.deepEqual([...needRuleNames({ source: "need({ holds: true, rule: 'late' });" })], []);
  assert.deepEqual([...needRuleNames({ source: 'need({ rule: 1 });' })], []);
  assert.deepEqual([...needRuleNames({ source: "need({ rule: 'A'," })], []);
  assert.deepEqual([...needRuleNames({ source: "need({ rule: ''," })], []);
});

test('skips a use of the name that is not a rule call', () => {
  assert.deepEqual([...needRuleNames({ source: 'function need({ rule, holds }) {}' })], []);
  assert.deepEqual([...needRuleNames({ source: "needsIt({ rule: 'a'," })], []);
});

test('reads a rule name whose detail holds template text with brackets', () => {
  const source = "  need({\n    rule: 'label-clearance-pixels',\n    holds: clearanceMatchesPixels(capture),\n"
    + '    detail: `${capture.file}: stated label clearance differs from the embedded pixels`\n  });';
  assert.deepEqual([...needRuleNames({ source })], ['label-clearance-pixels']);
});

test('names the call that declares a rule, and nothing for one that does not', () => {
  assert.equal(ruleNameAt({
    source: "need({ rule: 'a-b',",
    callStart: 0
  }), 'a-b');
  assert.equal(ruleNameAt({
    source: 'need({ holds: 1,',
    callStart: 0
  }), '');
});

test('finds every rule the committed scan-indicator builder declares', () => {
  // The builder and its test must agree on the rule names; the deleted pattern found none of them
  // because the printed builder puts `rule:` on its own line, which is why the test task failed.
  const design = import.meta.dirname;
  const builder = readFileSync(
    join(
      design,
      'scan-indicator.mjs'
    ),
    'utf8'
  );
  const names = [...needRuleNames({ source: builder })];
  assert.ok(names.length > 0);
  assert.equal(new Set(names).size, names.length);
  for (const name of names) {
    assert.ok(name.length > 0);
    for (const character of name) {
      const allowed = ((character >= 'a') && (character <= 'z')) || (character === '-');
      assert.ok(allowed, `rule name ${name} holds ${character}`);
    }
  }
});
//endregion
