import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  escapeXmlAttribute,
  unescapeXmlAttribute,
  xmlOptionLine,
} from './xml-coding.ts';
import {
  ABSENT_XML_ENTRY,
  ABSENT_XML_VALUE,
  findXmlEntryByKey,
  getXmlOptionValue,
  listXmlEntries,
  replaceOrInsertXmlEntry,
  type XmlEntry,
} from './xml.ts';

/**
 JetBrains persistent-state map document with two entries, used across editing tests.
 */
const SAMPLE_MAP = [
  '<application>',
  '  <component name="X">',
  '    <option name="map">',
  '      <map>',
  '        <entry key="alpha">',
  '          <value>',
  '            <option name="commandLine" value="run &amp; go" />',
  '          </value>',
  '        </entry>',
  '        <entry key="beta">',
  '          <value />',
  '        </entry>',
  '      </map>',
  '    </option>',
  '  </component>',
  '</application>',
].join('\n',);

/**
 Narrows an entry lookup to a present entry, throwing a labelled error otherwise.
 
 @param entry - Entry lookup result.
 
 @param label - Identifier surfaced when the entry is absent.
 
 @returns Present entry.
 
 @throws Error when entry is the absent sentinel.
 
 @example
 ```ts
 requireEntry(findXmlEntryByKey({ xml, key: 'a' }), 'a');
 ```
 */
function requireEntry(entry: XmlEntry | typeof ABSENT_XML_ENTRY, label: string,): XmlEntry {
  if (entry === ABSENT_XML_ENTRY) throw new Error(`expected entry '${label}' to be present`,);
  return entry;
}

await describe({
  name: '',
  children: [
    //region escapeXmlAttribute

    describe({
      name: escapeXmlAttribute.name,
      children: [
        it({
          name: 'escapes every active attribute delimiter and control character',
          fn: async () => {
            expect(escapeXmlAttribute({ value: '&"<>\n\r\t', },),)
              .toBe('&amp;&quot;&lt;&gt;&#10;&#13;&#9;',);
          },
        },),
        it({
          name: 'escapes ampersand before other entities to avoid double meaning',
          fn: async () => {
            expect(escapeXmlAttribute({ value: '&lt;', },),).toBe('&amp;lt;',);
          },
        },),
      ],
    },),

    //endregion escapeXmlAttribute

    //region unescapeXmlAttribute

    describe({
      name: unescapeXmlAttribute.name,
      children: [
        it({
          name: 'round-trips every escaped delimiter',
          fn: async () => {
            /**
             Adversarial value containing each character the encoder rewrites.
             */
            const raw = '&"<>\n\r\t';
            expect(unescapeXmlAttribute({ value: escapeXmlAttribute({ value: raw, },), },),).toBe(raw,);
          },
        },),
        it({
          name: 'decodes named and numeric entities including apostrophe and hex',
          fn: async () => {
            expect(unescapeXmlAttribute({ value: '&apos;&#65;&#x42;', },),).toBe("'AB",);
          },
        },),
        it({
          name: 'preserves unknown entities and a dangling ampersand verbatim',
          fn: async () => {
            expect(unescapeXmlAttribute({ value: '&nbsp; a & b', },),).toBe('&nbsp; a & b',);
          },
        },),
      ],
    },),

    //endregion unescapeXmlAttribute

    //region xmlOptionLine

    describe({
      name: xmlOptionLine.name,
      children: [
        it({
          name: 'renders an indented option with an escaped value',
          fn: async () => {
            expect(xmlOptionLine({ indent: '  ', name: 'serverName', value: 'A & B', },),)
              .toBe('  <option name="serverName" value="A &amp; B" />',);
          },
        },),
      ],
    },),

    //endregion xmlOptionLine

    //region listXmlEntries

    describe({
      name: listXmlEntries.name,
      children: [
        it({
          name: 'lists keyed entries with decoded keys and byte ranges',
          fn: async () => {
            /**
             Entries discovered in the sample document.
             */
            const entries = listXmlEntries({ xml: SAMPLE_MAP, },);
            expect(entries.map(function entryKey(entry,): string {
              return entry.key;
            },),)
              .toEqual(['alpha', 'beta',],);
            /**
             Present alpha entry.
             */
            const alpha = requireEntry(findXmlEntryByKey({ xml: SAMPLE_MAP, key: 'alpha', },), 'alpha',);
            expect(SAMPLE_MAP.slice(alpha.start, alpha.end,),).toBe(alpha.block,);
            expect(alpha.block.startsWith('<entry',),).toBe(true,);
          },
        },),
        it({
          name: 'returns no entries for a document without keyed entries',
          fn: async () => {
            expect(listXmlEntries({ xml: '<map><entry><value /></entry></map>', },),).toEqual([],);
          },
        },),
        it({
          name: 'reads keys regardless of quote style and spacing around equals',
          fn: async () => {
            expect(findXmlEntryByKey({ xml: "<map><entry key = 'srv' ><value /></entry></map>", key: 'srv', },),)
              .not.toBe(ABSENT_XML_ENTRY,);
          },
        },),
      ],
    },),

    //endregion listXmlEntries

    //region getXmlOptionValue

    describe({
      name: getXmlOptionValue.name,
      children: [
        it({
          name: 'reads and decodes an option value from a block',
          fn: async () => {
            /**
             Present alpha entry holding a single escaped option.
             */
            const alpha = requireEntry(findXmlEntryByKey({ xml: SAMPLE_MAP, key: 'alpha', },), 'alpha',);
            expect(getXmlOptionValue({ block: alpha.block, optionName: 'commandLine', },),).toBe('run & go',);
          },
        },),
        it({
          name: 'reads options regardless of attribute order and quote style',
          fn: async () => {
            /**
             Option with value before name and single-quoted attributes.
             */
            const block = "<value><option  value='cmd'   name='commandLine' /></value>";
            expect(getXmlOptionValue({ block, optionName: 'commandLine', },),).toBe('cmd',);
          },
        },),
        it({
          name: 'returns the absent sentinel when the option is missing',
          fn: async () => {
            expect(getXmlOptionValue({ block: '<value />', optionName: 'commandLine', },),).toBe(ABSENT_XML_VALUE,);
          },
        },),
      ],
    },),

    //endregion getXmlOptionValue

    //region replaceOrInsertXmlEntry

    describe({
      name: replaceOrInsertXmlEntry.name,
      children: [
        it({
          name: 'replaces an existing entry, dropping the old line indentation idempotently',
          fn: async () => {
            /**
             New beta block carrying a readable option.
             */
            const block = '        <entry key="beta"><value><option name="commandLine" value="cmd" /></value></entry>';
            /**
             Document after swapping the beta entry block.
             */
            const updated = replaceOrInsertXmlEntry({ xml: SAMPLE_MAP, key: 'beta', block, },);
            /**
             Present beta entry after replacement.
             */
            const beta = requireEntry(findXmlEntryByKey({ xml: updated, key: 'beta', },), 'beta',);
            expect(getXmlOptionValue({ block: beta.block, optionName: 'commandLine', },),).toBe('cmd',);
            expect(findXmlEntryByKey({ xml: updated, key: 'alpha', },),).not.toBe(ABSENT_XML_ENTRY,);
            expect(replaceOrInsertXmlEntry({ xml: updated, key: 'beta', block, },),).toBe(updated,);
          },
        },),
        it({
          name: 'inserts a new entry before the map close regardless of its indentation',
          fn: async () => {
            /**
             Map whose close tag sits at column zero.
             */
            const weird = '<map>\n    <entry key="a"><value /></entry>\n</map>';
            /**
             Document after inserting a fresh entry.
             */
            const updated = replaceOrInsertXmlEntry({ xml: weird, key: 'b', block: '<entry key="b"><value /></entry>', },);
            expect(findXmlEntryByKey({ xml: updated, key: 'b', },),).not.toBe(ABSENT_XML_ENTRY,);
            expect(updated.indexOf('key="b"',) < updated.lastIndexOf('</map>',),).toBe(true,);
          },
        },),
        it({
          name: 'throws when the document has no map element',
          fn: async () => {
            expect(function insertWithoutMap(): string {
              return replaceOrInsertXmlEntry({ xml: '<state></state>', key: 'x', block: '<entry key="x" />', },);
            },)
              .toThrow();
          },
        },),
      ],
    },),

    //endregion replaceOrInsertXmlEntry
  ],
},);
