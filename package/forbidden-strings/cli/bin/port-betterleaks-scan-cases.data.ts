/**
 The cases `port-betterleaks-scan-test.ts` runs: one table per walk the betterleaks porter reads a rule body or
 rewrites a pattern with.

 Each row names its subject, so a walk that drifts fails with the shape it answered wrongly rather than with an
 index. The tables are declarations only, which is why they live in a data module beside the runner.
 */

import type { RuleBodyLine, } from './port-betterleaks-toml-lines.ts';

//region Types

/**
 One line of the vendored configuration and what reading it must answer.
 */
export type LineCase = {
  readonly subject: string;
  readonly line: string;
  readonly expected: RuleBodyLine;
};

/**
 One pattern and the text a rewrite pass must yield for it.
 */
export type TextCase = {
  readonly subject: string;
  readonly pattern: string;
  readonly expected: string;
};

//endregion Types

//region The `[[rules]]` table header

/**
 The line every rule entry opens with, in the shapes a configuration can write it.
 */
export const headerCases: readonly (readonly [string, string, boolean])[] = [
  ['the header', '[[rules]]', true],
  ['the header with trailing spaces', '[[rules]]  ', true],
  ['the header ended by a CRLF carriage return', '[[rules]]\r', true],
  ['a sub-table header', '[[rules.required]]', false],
  ['a header indented', ' [[rules]]', false],
  ['a header with text after it', '[[rules]] then', false],
  ['a header missing one bracket', '[[rules]', false],
  ['an ordinary field line', 'id = "a"', false],
  ['an empty line', '', false],
];

//endregion The `[[rules]]` table header

//region A rule body's own lines

/**
 The sub-table headers, the two quoted fields, and the lines the walk must leave to the remaining extractors.
 */
export const bodyLineCases: readonly LineCase[] = [
  { subject: 'a required sub-table', line: '[[rules.required]]', expected: { kind: 'sub-table', name: 'required', }, },
  { subject: 'an allowlists sub-table', line: '[[rules.allowlists]]', expected: { kind: 'sub-table', name: 'allowlists', }, },
  { subject: 'a sub-table with trailing spaces', line: '[[rules.allowlists]]  ', expected: { kind: 'sub-table', name: 'allowlists', }, },
  { subject: 'a sub-table ended by a CRLF carriage return', line: '[[rules.allowlists]]\r', expected: { kind: 'sub-table', name: 'allowlists', }, },
  { subject: 'a sub-table whose name holds digits and underscores', line: '[[rules.a_1]]', expected: { kind: 'sub-table', name: 'a_1', }, },
  { subject: 'a sub-table with no name', line: '[[rules.]]', expected: { kind: 'no-field', }, },
  { subject: 'a sub-table whose name holds a space', line: '[[rules.a b]]', expected: { kind: 'no-field', }, },
  { subject: 'a sub-table with text after it', line: '[[rules.required]] then', expected: { kind: 'no-field', }, },
  { subject: 'a sub-table missing one bracket', line: '[[rules.required', expected: { kind: 'no-field', }, },
  { subject: 'an id', line: 'id = "aws-key"', expected: { kind: 'id', value: 'aws-key', }, },
  { subject: 'an id with no spaces', line: 'id="aws-key"', expected: { kind: 'id', value: 'aws-key', }, },
  { subject: 'an id with tabs around the equals sign', line: 'id\t=\t"aws-key"', expected: { kind: 'id', value: 'aws-key', }, },
  { subject: 'an empty id', line: 'id = ""', expected: { kind: 'id', value: '', }, },
  { subject: 'an id with no equals sign', line: 'id "aws-key"', expected: { kind: 'no-field', }, },
  { subject: 'an unterminated id', line: 'id = "aws-key', expected: { kind: 'no-field', }, },
  { subject: 'a key that only begins with id', line: 'identity = "aws-key"', expected: { kind: 'no-field', }, },
  { subject: 'an indented id', line: ' id = "aws-key"', expected: { kind: 'no-field', }, },
  { subject: 'a description', line: 'description = "a plain sentence"', expected: { kind: 'description', value: 'a plain sentence', }, },
  { subject: 'a description with no spaces', line: 'description="tight"', expected: { kind: 'description', value: 'tight', }, },
  { subject: 'a description holding escaped quotes', line: String.raw`description = "a \"quoted\" word"`, expected: { kind: 'description', value: String.raw`a \"quoted\" word`, }, },
  { subject: 'a description holding an escaped backslash', line: String.raw`description = "a\\b"`, expected: { kind: 'description', value: String.raw`a\\b`, }, },
  { subject: 'a description that ends on an escaped quote', line: String.raw`description = "a\"`, expected: { kind: 'no-field', }, },
  { subject: 'a description that never closes', line: 'description = "open', expected: { kind: 'no-field', }, },
  { subject: 'a description followed by other fields', line: 'description = "a" then', expected: { kind: 'description', value: 'a', }, },
  { subject: 'a key that only ends with description', line: 'longdescription = "a"', expected: { kind: 'no-field', }, },
  { subject: 'a field this reader leaves to its own reader', line: 'secretGroup = 1', expected: { kind: 'no-field', }, },
  { subject: 'the skipReport flag', line: 'skipReport = true', expected: { kind: 'no-field', }, },
];

//endregion A rule body's own lines

//region Named-capture openings

/**
 A rule body's `(?P<name>` openings, the shapes a name may take, and the openings that are not one.
 */
export const namedCaptureCases: readonly TextCase[] = [
  { subject: 'one named capture', pattern: '(?P<name>a)', expected: '(?:a)', },
  { subject: 'a name with digits and underscores', pattern: '(?P<a_1>b)', expected: '(?:b)', },
  { subject: 'a one-character name', pattern: '(?P<_>c)', expected: '(?:c)', },
  { subject: 'two named captures', pattern: '(?P<one>a)(?P<two>b)', expected: '(?:a)(?:b)', },
  { subject: 'a named capture inside other text', pattern: 'x(?P<n>z)y', expected: 'x(?:z)y', },
  { subject: 'a name that opens with a digit', pattern: '(?P<1a>b)', expected: '(?P<1a>b)', },
  { subject: 'a name with no characters', pattern: '(?P<>b)', expected: '(?P<>b)', },
  { subject: 'an opening with no closing bracket', pattern: '(?P<name', expected: '(?P<name', },
  { subject: 'another dialect’s named form', pattern: '(?<name>a)', expected: '(?<name>a)', },
  { subject: 'an opening cut short', pattern: '(?P<', expected: '(?P<', },
  { subject: 'no capture at all', pattern: '(?:a)|b', expected: '(?:a)|b', },
];

//endregion Named-capture openings

//region Lazy quantifier markers

/**
 Quantifier suffixes, and the `?` that opens a group, which is no lazy marker.
 */
export const lazyMarkerCases: readonly TextCase[] = [
  { subject: 'a lazy star', pattern: 'a*?b', expected: 'a*b', },
  { subject: 'a lazy plus', pattern: 'a+?b', expected: 'a+b', },
  { subject: 'a lazy question mark', pattern: 'a??b', expected: 'a?b', },
  { subject: 'a lazy open count', pattern: 'a{2,}?b', expected: 'a{2,}b', },
  { subject: 'a lazy closed count', pattern: 'a{2,3}?b', expected: 'a{2,3}b', },
  { subject: 'a lazy marker ending the body', pattern: '*?', expected: '*', },
  { subject: 'two lazy markers in a row', pattern: '??', expected: '?', },
  { subject: 'a non-capturing group opening', pattern: '(?:a)', expected: '(?:a)', },
  { subject: 'a lookahead opening', pattern: '(?=a)', expected: '(?=a)', },
  { subject: 'a negative lookahead opening', pattern: '(?!a)', expected: '(?!a)', },
  { subject: 'a lookbehind opening', pattern: '(?<=a)', expected: '(?<=a)', },
  { subject: 'an escaped quantifier', pattern: String.raw`a\*?b`, expected: String.raw`a\*b`, },
  { subject: 'no quantifier at all', pattern: 'abc', expected: 'abc', },
];

//endregion Lazy quantifier markers

//region Whole bodies through pcreToResharp

/**
 Bodies through {@link pcreToResharp}, which composes both passes before escaping resharp-only meta characters.
 */
export const composeCases: readonly TextCase[] = [
  { subject: 'a named capture with a lazy quantifier', pattern: '(?i)(?P<name>a*?)', expected: '(?i)(?:a*)', },
  { subject: 'two passes over one body', pattern: '(?P<one>a+?)(?P<two>b??)', expected: '(?:a+)(?:b?)', },
  { subject: 'a body with no named capture and no lazy marker', pattern: '(?:a){2}', expected: '(?:a){2}', },
];

//endregion Whole bodies through pcreToResharp
