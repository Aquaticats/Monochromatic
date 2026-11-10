// Checks the field-line readers the betterleaks TOML walker uses in place of pattern matching.
// What: the value after an assignment, the body a triple-quoted field opens, the number a
// `field = digits` line carries, and the `true` marker, over every field line the replaced
// patterns were given, plus a whole configuration parsed through the walker that uses them.
// Why: the walker turns the vendored upstream configuration into the scanner's embedded builtin
// rules, so a reader that answers differently from the pattern it replaced changes every rule
// compiled into the scanner without changing a line of the configuration.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
  NO_VALUE_AFTER_ASSIGNMENT,
  assignmentHoldsTrue,
  numberAfterAssignment,
  tripleQuotedBodyAfterAssignment,
  valueAfterAssignment,
} from './port-betterleaks-field-line.ts';
import { parseRules } from './port-betterleaks-parse.ts';

// Every field line the replaced patterns were given, with what they answered for it: the text after
// the assignment. The answers were recorded from the patterns themselves before they were
// replaced, so this table is their behaviour, not a reading of it.
const ASSIGNMENT_LINES = [
  ["regex = '''body'''", "regex", "'''body'''"],
  ["regex='''body'''", "regex", "'''body'''"],
  ["regex   =   '''body'''", "regex", "'''body'''"],
  [" regex = '''a'''", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regexx = '''a'''", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex = 'x'''", "regex", "'x'''"],
  ["regex = ''''''", "regex", "''''''"],
  ["regex = ", "regex", ""],
  ["regex", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex =", "regex", ""],
  ["regex = = '''a'''", "regex", "= '''a'''"],
  ["regex = '''", "regex", "'''"],
  ["regex = '''a'''", "regex", "'''a'''"],
  ["regex =  '''a'''", "regex", "'''a'''"],
  ["regex =\u0009'''a'''", "regex", "'''a'''"],
  ["regex =\u00A0'''a'''", "regex", "'''a'''"],
  ["regex =\u3000'''a'''", "regex", "'''a'''"],
  ["regex =\u2028'''a'''", "regex", "'''a'''"],
  ["regex =\u000D'''a'''", "regex", "'''a'''"],
  ["regex =\u200A'''a'''", "regex", "'''a'''"],
  ["regex =\uFEFF'''a'''", "regex", "'''a'''"],
  ["regex =\u000D\u000A'''a'''", "regex", "'''a'''"],
  ["path = '''body'''", "path", "'''body'''"],
  ["path='''body'''", "path", "'''body'''"],
  ["path   =   '''body'''", "path", "'''body'''"],
  [" path = '''a'''", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["pathx = '''a'''", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path = 'x'''", "path", "'x'''"],
  ["path = ''''''", "path", "''''''"],
  ["path = ", "path", ""],
  ["path", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path =", "path", ""],
  ["path = = '''a'''", "path", "= '''a'''"],
  ["path = '''", "path", "'''"],
  ["path = '''a'''", "path", "'''a'''"],
  ["path =  '''a'''", "path", "'''a'''"],
  ["path =\u0009'''a'''", "path", "'''a'''"],
  ["path =\u00A0'''a'''", "path", "'''a'''"],
  ["path =\u3000'''a'''", "path", "'''a'''"],
  ["path =\u2028'''a'''", "path", "'''a'''"],
  ["path =\u000D'''a'''", "path", "'''a'''"],
  ["path =\u200A'''a'''", "path", "'''a'''"],
  ["path =\uFEFF'''a'''", "path", "'''a'''"],
  ["path =\u000D\u000A'''a'''", "path", "'''a'''"],
  ["secretGroup = 3", "secretGroup", "3"],
  ["secretGroup=003", "secretGroup", "003"],
  ["secretGroup = 3x", "secretGroup", "3x"],
  ["secretGroup = ", "secretGroup", ""],
  ["secretGroup = x3", "secretGroup", "x3"],
  ["secretGroup = 3 4", "secretGroup", "3 4"],
  ["secretGroup =", "secretGroup", ""],
  ["secretGroup", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  ["secretGroupx = 3", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  [" secretGroup = 3", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  ["skipReport = true", "skipReport", "true"],
  ["skipReport=true", "skipReport", "true"],
  ["skipReport = trueish", "skipReport", "trueish"],
  ["skipReport =  true", "skipReport", "true"],
  ["skipReport = tru", "skipReport", "tru"],
  ["skipReport = false", "skipReport", "false"],
  ["skipReport =", "skipReport", ""],
  ["skipReport", "skipReport", NO_VALUE_AFTER_ASSIGNMENT],
  ["skipReportx = true", "skipReport", NO_VALUE_AFTER_ASSIGNMENT],
  [" skipReport = true", "skipReport", NO_VALUE_AFTER_ASSIGNMENT],
] as const;

for (const [line, key, value] of ASSIGNMENT_LINES) {
  void test(`the value after the assignment of ${JSON.stringify(line)}`, () => {
    assert.equal(valueAfterAssignment({
      key,
      line,
    },), value,);
  },);
}

// The lines that open a triple-quoted value, with the body the pattern answered for each: the text
// after the opening `'''`, including a closing `'''` that stays on the same line.
const TRIPLE_QUOTED_LINES = [
  ["regex = '''body'''", "regex", "body'''"],
  ["regex='''body'''", "regex", "body'''"],
  ["regex   =   '''body'''", "regex", "body'''"],
  [" regex = '''a'''", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regexx = '''a'''", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex = 'x'''", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex = ''''''", "regex", "'''"],
  ["regex = ", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex =", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex = = '''a'''", "regex", NO_VALUE_AFTER_ASSIGNMENT],
  ["regex = '''", "regex", ""],
  ["regex = '''a'''", "regex", "a'''"],
  ["regex =  '''a'''", "regex", "a'''"],
  ["regex =\u0009'''a'''", "regex", "a'''"],
  ["regex =\u00A0'''a'''", "regex", "a'''"],
  ["regex =\u3000'''a'''", "regex", "a'''"],
  ["regex =\u2028'''a'''", "regex", "a'''"],
  ["regex =\u000D'''a'''", "regex", "a'''"],
  ["regex =\u200A'''a'''", "regex", "a'''"],
  ["regex =\uFEFF'''a'''", "regex", "a'''"],
  ["regex =\u000D\u000A'''a'''", "regex", "a'''"],
  ["path = '''body'''", "path", "body'''"],
  ["path='''body'''", "path", "body'''"],
  ["path   =   '''body'''", "path", "body'''"],
  [" path = '''a'''", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["pathx = '''a'''", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path = 'x'''", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path = ''''''", "path", "'''"],
  ["path = ", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path =", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path = = '''a'''", "path", NO_VALUE_AFTER_ASSIGNMENT],
  ["path = '''", "path", ""],
  ["path = '''a'''", "path", "a'''"],
  ["path =  '''a'''", "path", "a'''"],
  ["path =\u0009'''a'''", "path", "a'''"],
  ["path =\u00A0'''a'''", "path", "a'''"],
  ["path =\u3000'''a'''", "path", "a'''"],
  ["path =\u2028'''a'''", "path", "a'''"],
  ["path =\u000D'''a'''", "path", "a'''"],
  ["path =\u200A'''a'''", "path", "a'''"],
  ["path =\uFEFF'''a'''", "path", "a'''"],
  ["path =\u000D\u000A'''a'''", "path", "a'''"],
] as const;

for (const [line, key, body] of TRIPLE_QUOTED_LINES) {
  void test(`the triple-quoted body of ${JSON.stringify(line)}`, () => {
    assert.equal(tripleQuotedBodyAfterAssignment({
      key,
      line,
    },), body,);
  },);
}

// A configuration that uses every field the readers answer, including a triple-quoted value that
// spans lines, so the walker is exercised where it calls them.
const CONFIGURATION = [
  '[[rules]]',
  'id = "one"',
  'description = "One rule"',
  'regex = \'\'\'',
  '(?i)cat',
  '\'\'\'',
  'path = \'\'\'',
  '**/*.txt',
  '\'\'\'',
  'secretGroup = 3',
  'skipReport = true',
  '',
  '[[rules]]',
  'id = "two"',
  'description = "Two"',
  'regex = \'\'\'single line\'\'\'',
  'secretGroup = 12abc',
  'skipReport = trueish',
  '',
  '[[rules]]',
  'id = "three"',
  'description = "Three"',
  'regex = \'\'\'third\'\'\'',
].join('\n',);

void test('the walker carries every field the readers answer into the rule it emits', () => {
  const rules = parseRules({ toml: CONFIGURATION, },);
  assert.deepEqual(rules.map(rule => rule.id,), ['one', 'two', 'three',],);
  assert.deepEqual(rules[0], {
    id: 'one',
    description: 'One rule',
    regex: '\n(?i)cat\n',
    pathScope: '\n**/*.txt\n',
    secretGroup: 3,
    skipReport: true,
    hasRequired: false,
  },);
  // A digit run ends where the digits do, and the marker is a prefix of the value, because the
  // replaced patterns stopped there too.
  assert.equal(rules[1]?.secretGroup, 12,);
  assert.equal(rules[1]?.skipReport, true,);
  assert.equal(rules[2]?.secretGroup, undefined,);
  assert.equal(rules[2]?.skipReport, false,);
},);

// The lines that carry a capture-group selector, with the number the pattern answered for each: the
// digit run after the assignment, and nothing for a line that starts with something else.
const NUMBER_LINES = [
  ["secretGroup = 3", "secretGroup", 3],
  ["secretGroup=003", "secretGroup", 3],
  ["secretGroup = 3x", "secretGroup", 3],
  ["secretGroup = ", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  ["secretGroup = x3", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  ["secretGroup = 3 4", "secretGroup", 3],
  ["secretGroup =", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  ["secretGroup", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  ["secretGroupx = 3", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
  [" secretGroup = 3", "secretGroup", NO_VALUE_AFTER_ASSIGNMENT],
] as const;

for (const [line, key, number] of NUMBER_LINES) {
  void test(`the number on ${JSON.stringify(line)}`, () => {
    assert.equal(numberAfterAssignment({
      key,
      line,
    },), number,);
  },);
}

// The lines that carry the report-suppression marker, with the answer the pattern gave for each. A
// value that only starts with `true` still carries the marker, because the pattern stopped there.
const TRUE_LINES = [
  ["skipReport = true", "skipReport", true],
  ["skipReport=true", "skipReport", true],
  ["skipReport = trueish", "skipReport", true],
  ["skipReport =  true", "skipReport", true],
  ["skipReport = tru", "skipReport", false],
  ["skipReport = false", "skipReport", false],
  ["skipReport =", "skipReport", false],
  ["skipReport", "skipReport", false],
  ["skipReportx = true", "skipReport", false],
  [" skipReport = true", "skipReport", false],
] as const;

for (const [line, key, holdsTrue] of TRUE_LINES) {
  void test(`the true marker on ${JSON.stringify(line)}`, () => {
    assert.equal(assignmentHoldsTrue({
      key,
      line,
    },), holdsTrue,);
  },);
}
