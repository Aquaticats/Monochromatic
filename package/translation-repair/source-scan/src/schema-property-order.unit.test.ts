/**
 Guards the order the properties of every answer schema are declared in (ledger B762).

 THE OWNER'S RULING OF 2026-10-21 ("Reason first everywhere") puts the reason before the choice in every ballot, so that
 a reply cut short loses its choice and not its reason. Charm Hyper returns the keys of a tool call in alphabetical order
 (200 of 227 multi-key objects in zheermao22; 102 of 103 vote replies in CuspariaKLSY22), while NeuralWatt, OpenRouter and
 Synthetic follow the declared order, and the repo sorts nothing on the way (`canonicalPromptValue` digests the messages
 for the uniqueness check and never reaches a request body; `buildAnthropicBody` and `renderToolSystemPrompt` place the
 schema as declared). Whether Hyper sorts what it sends the model or what the model said is not known from the stored
 replies. So the only order that holds on every provider is one where declaring and sorting agree: a schema whose
 declared order is its alphabetical order reads the same wherever it is sorted.

 WHAT THE SCAN READS, in the package's source (tests and fixtures build mock shapes of their own and are left out): every
 object literal that is the value of a `properties` key, which is how a JSON schema names its members. A finding names the
 file and line, the order declared and the order alphabetical. A member spread into the literal is passed over, since its
 keys are another literal's, and that literal is read where it is written.

 WHAT IT CANNOT READ: a schema built by `Object.fromEntries` or by a spread, whose keys are not written in a literal. The
 finding schema of the rendering audit (built from `FINDING_FIELDS`) is pinned by `rendering-audit-wire.unit.test.ts`, and the
 archive answer a contest adds by spread (`verdictOnArchive`) by `contest-ballot-wire.unit.test.ts`.

 THE SCAN IS GREEN since the sixteen schemas it first listed were renamed or reordered so that their declared order is
 alphabetical (ledger B762 lists the schemas and the names chosen). A new schema declared out of order is a finding.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a scan shown able to find each form (ledger M21).
 Fixtures are cat-themed; the package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { compareCodePoints, } from '@monochromatic-dev/translation-repair-markup';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  literalText,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Schema property order
// The scan itself: which object literals are a `properties` member, the keys each declares in order, and the line of each.

/**
 The name a property is declared under, empty for a computed key or any form this scan cannot read.

 @param property - property node of an object literal

 @returns Its key as written, bare or quoted

 @example
 ```ts
 const name = declaredName({ property, },);
 ```
 */
function declaredName({ property, }: { readonly property: TreeNode; },): string {
  if ((property.type !== 'Property') || (property.computed === true))
    return '';
  return identifierName({ node: property.key, },) || literalText({ node: property.key, },);
}

/**
 Every `properties` member in the source given, as the keys it declares in order.

 @param file - file read

 @returns One entry for each, with the line it starts on

 @example
 ```ts
 const members = propertiesMembers({ file, },);
 ```
 */
function propertiesMembers(
  { file, }: { readonly file: SourceText; },
): readonly { readonly line: number; readonly keys: readonly string[]; }[] {
  return nodesUnder({ root: parseSource({ file, },).program, },)
    .filter(function isPropertiesMember(node,): boolean {
      return (node.type === 'Property')
        && (declaredName({ property: node, },) === 'properties')
        && isTreeNode(node.value,)
        && (node.value.type === 'ObjectExpression');
    },)
    .toSorted(function byPlace(left, right,): number {
      return left.start - right.start;
    },)
    .map(function toMember(node,): { readonly line: number; readonly keys: readonly string[]; } {
      /**
       The object literal that holds the members.
       */
      const literal = node.value as TreeNode;
      /**
       The members of the literal, spreads among them.
       */
      const members = literal.properties as readonly TreeNode[];
      return {
        line: file.text.slice(0, node.start,).split('\n',).length,
        keys: members
          .map(function nameOf(property,): string {
            return declaredName({ property, },);
          },)
          .filter(function named(name,): boolean {
            return name !== '';
          },),
      };
    },);
}

/**
 Every `properties` member of the package source whose declared order is not its alphabetical order.

 @param files - files read; tests among them are passed over

 @returns Findings naming file, line and both orders, in the order the files are given

 @example
 ```ts
 const findings = unsortedSchemas({ files, },);
 ```
 */
function unsortedSchemas({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return files
    .filter(function isSource(file,): boolean {
      return !file.isTest;
    },)
    .flatMap(function findings(file,): readonly string[] {
      return propertiesMembers({ file, },)
        .flatMap(function unsorted({
          line,
          keys,
        },): readonly string[] {
          /**
           The same keys in alphabetical order, by code point.
           */
          const sorted = keys.toSorted(function byCodePoint(left, right,): number {
            return compareCodePoints({
              left,
              right,
            },);
          },);
          if (keys.every(function inPlace(key, index,): boolean {
            return key === sorted[index];
          },))
            return [];
          return [`${file.path}:${String(line,)} declared [${keys.join(', ',)}], alphabetical [${sorted.join(', ',)}]`,];
        },);
    },);
}

//endregion Schema property order

await describe({
  name: 'schema property order (ledger B762)',
  children: [
    it({
      name: 'FINDS a properties member declared out of alphabetical order, bare or quoted keys alike, and leaves a sorted '
        + 'one, a one-key one, a spread, a computed key, other objects that merely hold keys and tests',
      fn: async () => {
        expect(unsortedSchemas({
          files: [
            {
              path: 'litter.ts',
              text: [
                'export const sorted = { properties: { best: 1, reason: 2, }, };',
                'export const reasonFirst = {',
                '  type: \'object\',',
                '  properties: { reason: { type: \'string\', }, best: { type: \'integer\', }, },',
                '};',
                'export const quoted = { properties: { \'zebra\': 1, \'apple\': 2, }, };',
                'export const single = { properties: { only: 1, }, };',
                'const extra = { reason: 1, };',
                'export const spread = { properties: { ...extra, apple: 1, zebra: 2, }, };',
                'export const spreadUnsorted = { properties: { zebra: 1, ...extra, apple: 2, }, };',
                'export const computed = { properties: { [extra.key]: 1, apple: 2, zebra: 3, }, };',
                'export const notProperties = { fields: { zebra: 1, apple: 2, }, };',
                'export const notAnObject = { properties: [\'zebra\', \'apple\',], };',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'catnip.unit.test.ts',
              text: 'export const mock = { properties: { zebra: 1, apple: 2, }, };',
              isTest: true,
            },
          ],
        },),).toEqual([
          'litter.ts:4 declared [reason, best], alphabetical [best, reason]',
          'litter.ts:6 declared [zebra, apple], alphabetical [apple, zebra]',
          'litter.ts:10 declared [zebra, apple], alphabetical [apple, zebra]',
        ],);
      },
    },),

    it({
      name: 'DECLARES EVERY ANSWER SCHEMA IN THE ORDER THE KEYS SORT, so the order the owner ruled holds on a provider that '
        + 'returns a tool call\'s keys alphabetically as it does on one that follows the declaration',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: unsortedSchemas({ files, },), },);
      },
    },),
  ],
},);
