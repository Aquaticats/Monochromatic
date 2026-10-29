/**
 Tests for terminal title engine entries and registry lookup.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildTerminalTitle,
  buildToolTerminalTitle,
  buildToolTitle,
  genericUnknownToolTitle,
  lookupToolTitleEntry,
  pathTitleEntry,
  shellCommandTitleEntry,
  staticTitleEntry,
  textTitleEntry,
  TOOL_TITLE_ENTRY_MISSING,
  type ToolTitleRegistry,
} from '@monochromatic-dev/agent-harness-shared-terminal-title';

await describe({
  name: buildToolTitle.name,
  children: [
    it({
      name: 'formats static entries by lifecycle tense',
      fn: async () => {
        /**
         Registry with one static no-input title entry.
         */
        const registry: ToolTitleRegistry = {
          TaskList: staticTitleEntry({ pre: 'Listing tasks', post: 'Listed tasks', },),
        };
        expect(buildToolTitle({ registry, toolName: 'TaskList', input: {}, tense: 'post', },),).toBe(
          'Listed tasks',
        );
      },
    },),
    it({
      name: 'formats path entries with smart relative path',
      fn: async () => {
        /**
         Registry with one path title entry.
         */
        const registry: ToolTitleRegistry = {
          Read: pathTitleEntry({
            field: 'file_path',
            labels: { pre: 'Reading', post: 'Read', },
            noun: 'file',
          },),
        };
        expect(
          buildToolTitle({
            registry,
            toolName: 'Read',
            input: { file_path: '/repo/src/index.ts', },
            tense: 'pre',
            context: { cwd: '/repo', },
          },),
        ).toBe('Reading src/index.ts',);
      },
    },),
    it({
      name: 'uses fallback when field entry has no string value',
      fn: async () => {
        /**
         Registry with one text title entry.
         */
        const registry: ToolTitleRegistry = {
          Grep: textTitleEntry({
            field: 'pattern',
            labels: { pre: 'Searching for', post: 'Searched for', },
            fallback: { pre: 'Searching', post: 'Searched', },
          },),
        };
        expect(buildToolTitle({ registry, toolName: 'Grep', input: {}, tense: 'pre', },),).toBe(
          'Searching',
        );
      },
    },),
    it({
      name: 'formats shell command entries with lifecycle verbs',
      fn: async () => {
        /**
         Registry with one shell command title entry.
         */
        const registry: ToolTitleRegistry = {
          Bash: shellCommandTitleEntry({ field: 'command', },),
        };
        expect(
          buildToolTitle({
            registry,
            toolName: 'Bash',
            input: { command: 'env timeout 10 npm test', },
            tense: 'pre',
          },),
        ).toBe('Running npm test',);
      },
    },),
    it({
      name: 'uses generic unknown-tool lifecycle fallback',
      fn: async () => {
        expect(
          buildToolTitle({
            registry: {},
            toolName: 'mcp__weather',
            input: {},
            tense: 'post',
          },),
        ).toBe('Ran mcp__weather',);
      },
    },),
  ],
},);

await describe({
  name: lookupToolTitleEntry.name,
  children: [
    it({
      name: 'does not resolve inherited object properties',
      fn: async () => {
        expect(lookupToolTitleEntry({ registry: {}, toolName: '__proto__', },),).toBe(
          TOOL_TITLE_ENTRY_MISSING,
        );
      },
    },),
  ],
},);

await describe({
  name: buildTerminalTitle.name,
  children: [
    it({
      name: 'prefixes non-empty body',
      fn: async () => {
        expect(buildTerminalTitle({ prefix: 'π', body: 'Reading src/index.ts', },),).toBe(
          'π Reading src/index.ts',
        );
      },
    },),
    it({
      name: 'keeps only prefix for empty body',
      fn: async () => {
        expect(buildTerminalTitle({ prefix: 'π', body: '', },),).toBe('π',);
      },
    },),
    it({
      name: 'does not display-cap long titles',
      fn: async () => {
        /**
         Long body that used to exceed display-length caps.
         */
        const body = 'a'.repeat(200,);
        expect(buildTerminalTitle({ prefix: 'π', body, },),).toBe(`π ${body}`,);
      },
    },),
  ],
},);

await describe({
  name: buildToolTerminalTitle.name,
  children: [
    it({
      name: 'formats and prefixes tool titles',
      fn: async () => {
        /**
         Registry with one path title entry.
         */
        const registry: ToolTitleRegistry = {
          Read: pathTitleEntry({
            field: 'file_path',
            labels: { pre: 'Reading', post: 'Read', },
            noun: 'file',
          },),
        };
        expect(
          buildToolTerminalTitle({
            prefix: '✳',
            registry,
            toolName: 'Read',
            input: { file_path: 'src/index.ts', },
            tense: 'pre',
          },),
        ).toBe('✳ Reading src/index.ts',);
      },
    },),
  ],
},);

await describe({
  name: genericUnknownToolTitle.name,
  children: [
    it({
      name: 'uses running verb for unknown pre-tool title',
      fn: async () => {
        expect(
          genericUnknownToolTitle({
            toolName: 'custom',
            input: {},
            tense: 'pre',
            context: {},
          },),
        ).toBe('Running custom',);
      },
    },),
  ],
},);
