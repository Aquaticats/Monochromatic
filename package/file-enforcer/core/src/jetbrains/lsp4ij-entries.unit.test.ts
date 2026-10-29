import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ABSENT_XML_ENTRY,
  ABSENT_XML_VALUE,
  buildLanguageSettingsEntry,
  buildUserDefinedEntry,
  findXmlEntryByKey,
  getXmlOptionValue,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region buildLanguageSettingsEntry

    describe({
      name: buildLanguageSettingsEntry.name,
      children: [
        it({
          name: 'wraps config and schema as options under one settings entry, round-tripping escaped JSON',
          fn: async () => {
            /**
             Rendered settings entry with an embedded quoted JSON value.
             */
            const block = buildLanguageSettingsEntry({
              serverId: 'srv',
              configContent: '{"a":1}',
              schemaContent: '{}',
            },);
            expect(findXmlEntryByKey({ xml: block, key: 'srv', },),).not.toBe(ABSENT_XML_ENTRY,);
            expect(getXmlOptionValue({ block, optionName: 'configurationContent', },),).toBe('{"a":1}',);
            expect(getXmlOptionValue({ block, optionName: 'configurationSchemaContent', },),).toBe('{}',);
            expect(getXmlOptionValue({ block, optionName: 'experimentalContent', },),).toBe('{}',);
            expect(getXmlOptionValue({ block, optionName: 'initializationOptionsContent', },),).toBe('{}',);
            expect(block.includes('<LanguageServerDefinitionSettings>',),).toBe(true,);
          },
        },),
      ],
    },),

    //endregion buildLanguageSettingsEntry

    //region buildUserDefinedEntry

    describe({
      name: buildUserDefinedEntry.name,
      children: [
        it({
          name: 'copies present options, skips absent ones, and declares id, name, and mappings',
          fn: async () => {
            /**
             Source server block holding one copyable option.
             */
            const sourceBlock = '<option name="commandLine" value="harper-ls" />';
            /**
             Rendered user-defined entry derived from the source block.
             */
            const block = buildUserDefinedEntry({
              copyOptions: ['commandLine', 'missing',],
              fileNames: ['AGENTS.md', 'CLAUDE.md',],
              languageId: 'markdown',
              serverId: 'cav',
              serverName: 'A & B',
              sourceBlock,
            },);
            expect(findXmlEntryByKey({ xml: block, key: 'cav', },),).not.toBe(ABSENT_XML_ENTRY,);
            expect(getXmlOptionValue({ block, optionName: 'commandLine', },),).toBe('harper-ls',);
            expect(getXmlOptionValue({ block, optionName: 'missing', },),).toBe(ABSENT_XML_VALUE,);
            expect(getXmlOptionValue({ block, optionName: 'serverId', },),).toBe('cav',);
            expect(getXmlOptionValue({ block, optionName: 'serverName', },),).toBe('A & B',);
            expect(block.includes('languageId="markdown"',),).toBe(true,);
            expect(block.includes('<option value="AGENTS.md" />',),).toBe(true,);
            expect(block.includes('<option value="CLAUDE.md" />',),).toBe(true,);
            expect(block.includes('<UserDefinedLanguageServerItemSettings>',),).toBe(true,);
          },
        },),
      ],
    },),

    //endregion buildUserDefinedEntry
  ],
},);
