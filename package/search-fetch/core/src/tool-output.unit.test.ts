/**
 Unit tests for Pi Search Fetch tool output helpers.
 
 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  createJsonContent,
  createLinkupToolOutput,
  createWarningContent,
  LINKUP_VISIBLE_JSON_MAX_BYTES,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Tool name fixture.
 */
const TOOL_NAME = 'linkup_web_search';

/**
 Fixed behavior warning fixture.
 */
const FIXED_BEHAVIOR = 'This extension always uses fixed behavior.';

/**
 Response fixture.
 */
const RESPONSE = { results: [], };

/**
 Response fixture with object search results.
 */
const RESULTS_RESPONSE = {
  results: [
    {
      title: 'First',
      url: 'https://example.com/first',
    },
    {
      title: 'Second',
      url: 'https://example.com/second',
    },
  ],
} as const;

/**
 Response fixture with results plus metadata.
 */
const RESULTS_WITH_EXTRA_KEY_RESPONSE = {
  results: [
    {
      title: 'First',
      url: 'https://example.com/first',
    },
  ],
  total: 1,
} as const;

/**
 Search response fixture carrying Linkup metadata in a noncanonical property order.
 */
const METADATA_RESULTS_RESPONSE = {
  costDollars: {
    search: {
      neural: 0.007,
    },
    total: 0.007,
  },
  requestId: '4b6f791ad7a70eb585c93832ab8af1f3',
  results: RESULTS_RESPONSE.results,
  resolvedSearchType: '',
  searchTime: 577.5,
} as const;

/**
 Metadata search response fixture with unexpected top-level data.
 */
const METADATA_RESULTS_WITH_EXTRA_KEY_RESPONSE = {
  ...METADATA_RESULTS_RESPONSE,
  diagnostic: 'upstream diagnostic',
} as const;

/**
 Metadata search response fixture missing required cost data.
 */
const METADATA_RESULTS_WITH_MISSING_KEY_RESPONSE = {
  requestId: METADATA_RESULTS_RESPONSE.requestId,
  results: METADATA_RESULTS_RESPONSE.results,
  resolvedSearchType: METADATA_RESULTS_RESPONSE.resolvedSearchType,
  searchTime: METADATA_RESULTS_RESPONSE.searchTime,
} as const;

/**
 Metadata search response fixture containing an invalid scalar result item.
 */
const METADATA_RESULTS_WITH_SCALAR_ITEM_RESPONSE = {
  ...METADATA_RESULTS_RESPONSE,
  results: [
    ...METADATA_RESULTS_RESPONSE.results,
    'invalid scalar result',
  ],
} as const;

/**
 Metadata search response fixture containing an invalid null result item.
 */
const METADATA_RESULTS_WITH_NULL_ITEM_RESPONSE = {
  ...METADATA_RESULTS_RESPONSE,
  results: [
    ...METADATA_RESULTS_RESPONSE.results,
    null,
  ],
} as const;

/**
 Metadata search response fixture containing an invalid array result item.
 */
const METADATA_RESULTS_WITH_ARRAY_ITEM_RESPONSE = {
  ...METADATA_RESULTS_RESPONSE,
  results: [
    ...METADATA_RESULTS_RESPONSE.results,
    [],
  ],
} as const;

/**
 Metadata search response fixture containing a non-array results value.
 */
const METADATA_RESULTS_WITH_NON_ARRAY_RESULTS_RESPONSE = {
  ...METADATA_RESULTS_RESPONSE,
  results: 'invalid results value',
} as const;

/**
 Metadata response shapes that must retain complete pretty JSON rather than JSONL.
 */
const METADATA_RESULTS_JSON_FALLBACK_CASES = [
  {
    name: 'unexpected top-level data',
    value: METADATA_RESULTS_WITH_EXTRA_KEY_RESPONSE,
  },
  {
    name: 'a missing required top-level key',
    value: METADATA_RESULTS_WITH_MISSING_KEY_RESPONSE,
  },
  {
    name: 'a scalar result item',
    value: METADATA_RESULTS_WITH_SCALAR_ITEM_RESPONSE,
  },
  {
    name: 'a null result item',
    value: METADATA_RESULTS_WITH_NULL_ITEM_RESPONSE,
  },
  {
    name: 'an array result item',
    value: METADATA_RESULTS_WITH_ARRAY_ITEM_RESPONSE,
  },
  {
    name: 'a non-array results value',
    value: METADATA_RESULTS_WITH_NON_ARRAY_RESULTS_RESPONSE,
  },
] as const;

/**
 Markdown-only response fixture.
 */
const MARKDOWN_ONLY_RESPONSE = { markdown: '# Meow', } as const;

/**
 Markdown response fixture with additional metadata.
 */
const MARKDOWN_WITH_EXTRA_KEY_RESPONSE = {
  markdown: '# Meow',
  title: 'Meow title',
} as const;

/**
 Bytes in one kibibyte, matching Pi's truncation utilities.
 */
const BYTES_PER_KIBIBYTE = 1_024;

/**
 Expected Linkup visible JSON cap in kibibytes.
 */
const EXPECTED_LINKUP_VISIBLE_JSON_KIBIBYTES = 100;

/**
 JSON payload kibibytes that exceed Pi's core default but not Linkup's cap.
 */
const ABOVE_PI_DEFAULT_JSON_KIBIBYTES = 60;

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: createWarningContent.name,
      children: [
        it({
          name: 'names ignored keys and fixed behavior',
          fn: async () => {
            /**
             Local value for warning.
             */
            const warning = createWarningContent({
              toolName: TOOL_NAME,
              ignoredKeys: ['depth', 'limit',],
              fixedBehavior: FIXED_BEHAVIOR,
            },);

            expect(warning.type,).toBe('text',);
            expect(warning.text,).toContain('depth, limit',);
            expect(warning.text,).toContain(FIXED_BEHAVIOR,);
          },
        },),
      ],
    },),
    describe({
      name: createJsonContent.name,
      children: [
        it({
          name: 'returns pretty JSON text when below truncation limits',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: RESPONSE,
            },);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).toContain('"results"',);
            expect(result.fullJsonPath,).toBeUndefined();
          },
        },),
        it({
          name: 'returns inner results array as JSONL when requested',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: RESULTS_RESPONSE,
              renderResultsArrayAsJsonl: true,
            },);
            /**
             Expected model-visible JSONL text.
             */
            const expectedJsonl = [
              '{"title":"First","url":"https://example.com/first"}',
              '{"title":"Second","url":"https://example.com/second"}',
            ].join('\n',);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).toBe(expectedJsonl,);
            expect(result.content.text,).not.toContain('"results"',);
            expect(result.fullJsonPath,).toBeUndefined();
          },
        },),
        it({
          name: 'returns metadata envelope results as JSONL when requested',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: METADATA_RESULTS_RESPONSE,
              renderResultsArrayAsJsonl: true,
            },);
            /**
             Expected model-visible JSONL text.
             */
            const expectedJsonl = [
              '{"title":"First","url":"https://example.com/first"}',
              '{"title":"Second","url":"https://example.com/second"}',
            ].join('\n',);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).toBe(expectedJsonl,);
            expect(result.content.text,).not.toContain('requestId',);
            expect(result.fullJsonPath,).toBeUndefined();
          },
        },),
        it({
          name: 'ignores metadata value types when exact top-level key set matches',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: {
                costDollars: false,
                requestId: null,
                results: RESULTS_RESPONSE.results,
                resolvedSearchType: [],
                searchTime: 'unknown',
              },
              renderResultsArrayAsJsonl: true,
            },);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).not.toContain('requestId',);
            expect(result.content.text,).toContain('"title":"First"',);
          },
        },),
        ...METADATA_RESULTS_JSON_FALLBACK_CASES.map(function createMetadataFallbackTest(testCase,) {
          return it({
            name: `keeps complete JSON for metadata envelopes with ${testCase.name}`,
            fn: async () => {
              /**
               Local value for result.
               */
              const result = await createJsonContent({
                value: testCase.value,
                renderResultsArrayAsJsonl: true,
              },);

              expect(result.content.type,).toBe('text',);
              expect(result.content.text,).toBe(JSON.stringify(
                testCase.value,
                null,
                2,
              ),);
              expect(result.fullJsonPath,).toBeUndefined();
            },
          },);
        },),
        it({
          name: 'returns empty JSONL for empty results when requested',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: RESPONSE,
              renderResultsArrayAsJsonl: true,
            },);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).toBe('',);
            expect(result.fullJsonPath,).toBeUndefined();
          },
        },),
        it({
          name: 'keeps JSON for results responses with extra fields',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: RESULTS_WITH_EXTRA_KEY_RESPONSE,
              renderResultsArrayAsJsonl: true,
            },);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).toContain('"results"',);
            expect(result.content.text,).toContain('"total"',);
          },
        },),
        it({
          name: 'truncates JSONL and writes full JSONL to temp file',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: RESULTS_RESPONSE,
              renderResultsArrayAsJsonl: true,
              truncationOptions: {
                maxBytes: 10,
                maxLines: 10,
              },
            },);

            expect(result.content.text,).toContain('JSONL response truncated',);
            expect(result.fullJsonPath,).toBeDefined();
            if (result.fullJsonPath === undefined)
              throw new Error('missing full JSONL path',);
            expect(result.fullJsonPath.endsWith('/response.jsonl',),).toBe(true,);
            /**
             Local value for fullJsonl.
             */
            const fullJsonl = await readFile(result.fullJsonPath, 'utf8',);
            expect(fullJsonl,).toContain('"title":"First"',);
            expect(fullJsonl,).not.toContain('"results"',);
          },
        },),
        it({
          name: 'truncates metadata-envelope JSONL and writes full JSONL to temp file',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: METADATA_RESULTS_RESPONSE,
              renderResultsArrayAsJsonl: true,
              truncationOptions: {
                maxBytes: 10,
                maxLines: 10,
              },
            },);

            expect(result.content.text,).toContain('JSONL response truncated',);
            expect(result.fullJsonPath,).toBeDefined();
            if (result.fullJsonPath === undefined)
              throw new Error('missing full metadata-envelope JSONL path',);
            expect(result.fullJsonPath.endsWith('/response.jsonl',),).toBe(true,);
            /**
             Local value for fullJsonl.
             */
            const fullJsonl = await readFile(result.fullJsonPath, 'utf8',);
            expect(fullJsonl,).toContain('"title":"First"',);
            expect(fullJsonl,).not.toContain('"requestId"',);
          },
        },),
        it({
          name: 'returns raw markdown for single-field markdown responses',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: MARKDOWN_ONLY_RESPONSE,
            },);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).toBe(MARKDOWN_ONLY_RESPONSE.markdown,);
            expect(result.content.text,).not.toContain('"markdown"',);
            expect(result.fullJsonPath,).toBeUndefined();
          },
        },),
        it({
          name: 'keeps JSON for markdown responses with extra fields',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: MARKDOWN_WITH_EXTRA_KEY_RESPONSE,
            },);

            expect(result.content.type,).toBe('text',);
            expect(result.content.text,).toContain('"markdown"',);
            expect(result.content.text,).toContain('"title"',);
            expect(result.content.text,).not.toBe(MARKDOWN_WITH_EXTRA_KEY_RESPONSE.markdown,);
          },
        },),
        it({
          name: 'keeps JSON above Pi default when below Linkup byte limit',
          fn: async () => {
            /**
             Large string that would exceed Pi's 50 KiB default limit.
             */
            const largeText = 'x'.repeat(
              ABOVE_PI_DEFAULT_JSON_KIBIBYTES * BYTES_PER_KIBIBYTE,
            );
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: {
                largeText,
              },
            },);

            expect(LINKUP_VISIBLE_JSON_MAX_BYTES,).toBe(
              EXPECTED_LINKUP_VISIBLE_JSON_KIBIBYTES * BYTES_PER_KIBIBYTE,
            );
            expect(result.fullJsonPath,).toBeUndefined();
            expect(result.content.text,).not.toContain('JSON response truncated',);
          },
        },),
        it({
          name: 'truncates large JSON and writes full JSON to temp file',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createJsonContent({
              value: {
                long: 'abcdefghijklmnopqrstuvwxyz',
              },
              truncationOptions: {
                maxBytes: 10,
                maxLines: 10,
              },
            },);

            expect(result.content.text,).toContain('JSON response truncated',);
            expect(result.fullJsonPath,).toBeDefined();
            if (result.fullJsonPath === undefined)
              throw new Error('missing full JSON path',);
            /**
             Local value for fullJson.
             */
            const fullJson = await readFile(result.fullJsonPath, 'utf8',);
            expect(fullJson,).toContain('abcdefghijklmnopqrstuvwxyz',);
          },
        },),
      ],
    },),
    describe({
      name: createLinkupToolOutput.name,
      children: [
        it({
          name: 'returns JSON content and details for model-visible response',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createLinkupToolOutput({
              toolName: TOOL_NAME,
              linkupResponse: RESPONSE,
              rawLinkupResponse: RESPONSE,
              ignoredKeys: [],
              fixedBehavior: FIXED_BEHAVIOR,
            },);

            expect(result.content,).toHaveLength(1,);
            expect(result.content[0]?.type,).toBe('text',);
            expect(result.details.linkupResponse,).toBe(RESPONSE,);
            expect(result.details.rawLinkupResponse,).toBe(RESPONSE,);
          },
        },),
        it({
          name: 'returns markdown content and details for markdown-only response',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createLinkupToolOutput({
              toolName: TOOL_NAME,
              linkupResponse: MARKDOWN_ONLY_RESPONSE,
              rawLinkupResponse: MARKDOWN_ONLY_RESPONSE,
              ignoredKeys: [],
              fixedBehavior: FIXED_BEHAVIOR,
            },);

            expect(result.content,).toHaveLength(1,);
            expect(result.content[0]?.type,).toBe('text',);
            if (result.content[0]?.type !== 'text')
              throw new Error('markdown content was not text',);
            expect(result.content[0].text,).toBe(MARKDOWN_ONLY_RESPONSE.markdown,);
            expect(result.details.linkupResponse,).toBe(MARKDOWN_ONLY_RESPONSE,);
            expect(result.details.rawLinkupResponse,).toBe(MARKDOWN_ONLY_RESPONSE,);
          },
        },),
        it({
          name: 'returns warning before JSON when ignored keys are present',
          fn: async () => {
            /**
             Local value for result.
             */
            const result = await createLinkupToolOutput({
              toolName: TOOL_NAME,
              linkupResponse: RESPONSE,
              rawLinkupResponse: RESPONSE,
              ignoredKeys: ['depth',],
              fixedBehavior: FIXED_BEHAVIOR,
            },);

            expect(result.content,).toHaveLength(2,);
            expect(result.content[0]?.type,).toBe('text',);
            if (result.content[0]?.type !== 'text')
              throw new Error('warning content was not text',);
            expect(result.content[0].text,).toContain('depth',);
            expect(result.details.ignoredKeys,).toEqual(['depth',],);
          },
        },),
      ],
    },),
  ],
},);
