/**
 Property-based fuzz tests for the XML entry editor in `./xml.ts`.
 
 Properties: `listXmlEntries` and `findXmlEntryByKey` are total over
 arbitrary input (the Lezer parser error-recovers, so they return an
 array or the absence sentinel and never throw), and their results agree;
 and inserting a well-formed entry into a document with a `<map>` makes
 the entry findable by key, with its option value surviving the
 parse-and-decode round-trip for arbitrary (escaped) keys and values.
 
 Run plan and seed policy: see `../fuzz-budget-fixture.ts`.
 
 @module
 */

import {
  assert,
  asyncProperty,
  constantFrom,
  oneof,
  record,
  string,
} from 'fast-check';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fuzzRunPlan, } from '../fuzz-budget-fixture.ts';
import {
  ABSENT_XML_ENTRY,
  escapeXmlAttribute,
  findXmlEntryByKey,
  getXmlOptionValue,
  listXmlEntries,
  replaceOrInsertXmlEntry,
  type XmlEntry,
} from '../../dist/final/node/index.mjs';

//region Constants and arbitraries

/**
 Run plan resolved once for every property in this file.
 */
const RUN = fuzzRunPlan();

/**
 A minimal JetBrains-shaped document with an empty `<map>` to insert into.
 */
const MAP_DOCUMENT = [
  '<application>',
  '  <component name="c">',
  '    <map>',
  '    </map>',
  '  </component>',
  '</application>',
  '',
].join('\n',);

/**
 Arbitrary XML-ish text: arbitrary strings unioned with concrete document
 fragments, some valid and some malformed, to drive the parser's
 error-recovery branches as well as the entry-bearing path.
 */
const xmlArbitrary = oneof(
  string(),
  constantFrom(
    MAP_DOCUMENT,
    '<map><entry key="a"><option name="opt" value="1" /></entry></map>',
    '<map><entry><option name="x" value="y" /></entry></map>',
    '<map><entry key="unterminated">',
    '<<<not xml&&&',
    '',
  ),
);

/**
 Arbitrary key or value text limited to non-control characters (including
 XML delimiters and an astral code point) so the constructed document
 stays valid XML once escaped.
 */
const safeTextArbitrary = string({
  unit: constantFrom(
    'a',
    'B',
    '0',
    ' ',
    '<',
    '>',
    '&',
    '"',
    '\'',
    '/',
    '=',
    '\u{1F600}',
  ),
},);

//endregion Constants and arbitraries

await describe({
  name: '',
  children: [
    //region listXmlEntries

    describe({
      name: listXmlEntries.name,
      children: [
        it({
          name: 'returns a well-formed entry array for arbitrary input without throwing',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                xmlArbitrary,
                async function listTotality(xml,) {
                  /**
                   Entries parsed from the document.
                   */
                  const entries = listXmlEntries({ xml, },);
                  expect(Array.isArray(entries,),).toBe(true,);
                  entries.forEach(function wellFormed(entry,) {
                    expect(typeof entry.key,).toBe('string',);
                    expect(typeof entry.block,).toBe('string',);
                    expect(typeof entry.start,).toBe('number',);
                    expect(typeof entry.end,).toBe('number',);
                  },);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion listXmlEntries

    //region findXmlEntryByKey

    describe({
      name: findXmlEntryByKey.name,
      children: [
        it({
          name: 'never throws and agrees with listXmlEntries for arbitrary input',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  xml: xmlArbitrary,
                  key: oneof(string(), constantFrom('a', 'x', 'unterminated',),),
                },),
                async function findConsistent({
                  xml,
                  key,
                },) {
                  /**
                   Lookup result for the requested key.
                   */
                  const found = findXmlEntryByKey({
                    xml,
                    key,
                  },);
                  if (found === ABSENT_XML_ENTRY) return;
                  expect(found.key,).toBe(key,);
                  expect(
                    listXmlEntries({ xml, },).some(function matches(entry,) {
                      return entry.key === key;
                    },),
                  ).toBe(true,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion findXmlEntryByKey

    //region replaceOrInsertXmlEntry

    describe({
      name: replaceOrInsertXmlEntry.name,
      children: [
        it({
          name: 'inserts a findable entry whose option value survives the round-trip',
          timeout: RUN.timeout,
          fn: async () => {
            await assert(
              asyncProperty(
                record({
                  key: safeTextArbitrary,
                  value: safeTextArbitrary,
                },),
                async function insertRoundTrip({
                  key,
                  value,
                },) {
                  /**
                   Well-formed entry block carrying one escaped option.
                   */
                  const block = `<entry key="${escapeXmlAttribute({ value: key, },)}">`
                    + `<option name="opt" value="${escapeXmlAttribute({ value, },)}" />`
                    + '</entry>';
                  /**
                   Document after inserting the entry into the map.
                   */
                  const updated = replaceOrInsertXmlEntry({
                    xml: MAP_DOCUMENT,
                    key,
                    block,
                  },);
                  /**
                   Entry located by key in the updated document.
                   */
                  const found = findXmlEntryByKey({
                    xml: updated,
                    key,
                  },);
                  expect(found,).not.toBe(ABSENT_XML_ENTRY,);
                  /**
                   Located entry narrowed past the absence sentinel.
                   */
                  const entry = found as XmlEntry;
                  expect(entry.key,).toBe(key,);
                  expect(
                    getXmlOptionValue({
                      block: entry.block,
                      optionName: 'opt',
                    },),
                  ).toBe(value,);
                },
              ),
              RUN.params,
            );
          },
        },),
      ],
    },),

    //endregion replaceOrInsertXmlEntry
  ],
},);
