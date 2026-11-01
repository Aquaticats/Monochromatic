/**
 Unit tests for `runSearchFetchCli` over the built artifact.

 Every case drives the built entry with injected streams and stub
 dependencies, so grammar, streams, and exit-status mapping are pinned without
 touching the network or the real configuration.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  SearchFetchCliConfigurationError,
  runSearchFetchCli,
  successExitCode,
  usageExitCode,
  type SearchFetchCliRunDependencies,
  type SearchFetchCliSettledStatus,
  type SearchFetchCliTextWriter,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Host fixture matching the blocklist entry in the config fixture.
 */
const BLOCKED_HOST = 'blocked.example';

/**
 Fixed config fixture carrying one blocked host.
 */
const CONFIG = {
  blocklist: [BLOCKED_HOST,],
  source: {
    path: '/fixture/pi-search-fetch.json',
    loaded: true,
  },
} satisfies SearchFetchCliRunDependencies['config'];

/**
 Search response fixture rendered as one JSONL result line.
 */
const SEARCH_RESPONSE = {
  results: [
    {
      url: 'https://example.com/good',
      title: 'good',
    },
  ],
};

/**
 Markdown fetch response fixture rendered as raw page markdown.
 */
const MARKDOWN_FETCH_RESPONSE = {
  markdown: '# fetched',
};

/**
 Oversized markdown response forcing the truncation path without touching the network.
 */
const OVERSIZED_MARKDOWN = 'x'
  .repeat(200_000,);

/**
 Oversized fetch response over {@link OVERSIZED_MARKDOWN}.
 */
const OVERSIZED_FETCH_RESPONSE = {
  markdown: OVERSIZED_MARKDOWN,
};

/**
 Sentinel kept as the thrown value when a run expected to throw settled instead.
 */
const RUN_SETTLED_SENTINEL: unique symbol = Symbol('run settled instead of throwing',);

//endregion Fixtures

//region Types

/**
 Stub dependencies plus the inputs they recorded.
 */
type StubRun = {
  /**
   Dependencies the run loads.
   */
  readonly dependencies: SearchFetchCliRunDependencies;
  /**
   Search inputs the stub client received.
   */
  readonly searchInputs: unknown[];
  /**
   Fetch inputs the stub client received.
   */
  readonly fetchInputs: unknown[];
  /**
   Failure the dependency loader throws instead of loading, when supplied.
   */
  readonly loadError?: Error;
};

/**
 Streams and captured text of one run.
 */
type CapturedStreams = {
  /**
   Captured stdout writer.
   */
  readonly stdout: SearchFetchCliTextWriter;
  /**
   Captured stderr writer.
   */
  readonly stderr: SearchFetchCliTextWriter;
  /**
   Concatenated stdout text.
   */
  readonly stdoutText: () => string;
  /**
   Concatenated stderr text.
   */
  readonly stderrText: () => string;
};

/**
 Outcome and captured text of a run that settled.
 */
type SettledCapture = {
  /**
   Settled exit status.
   */
  readonly status: SearchFetchCliSettledStatus;
  /**
   Captured stdout text.
   */
  readonly stdout: string;
  /**
   Captured stderr text.
   */
  readonly stderr: string;
};

/**
 Outcome and captured text of a run that threw.
 */
type ThrownCapture = {
  /**
   Thrown failure, or {@link RUN_SETTLED_SENTINEL} when the run settled.
   */
  readonly thrown: unknown;
  /**
   Captured stdout text.
   */
  readonly stdout: string;
  /**
   Captured stderr text.
   */
  readonly stderr: string;
};

//endregion Types

//region Helpers

/**
 Build stub dependencies recording the inputs they receive.

 @param searchResponse - canned search response, defaulting to {@link SEARCH_RESPONSE}

 @param fetchResponse - canned fetch response, defaulting to {@link MARKDOWN_FETCH_RESPONSE}

 @param searchError - failure the stub search call throws instead of answering

 @param fetchError - failure the stub fetch call throws instead of answering

 @param loadError - failure the dependency loader throws instead of loading

 @returns stub dependencies with their recorded inputs

 @example
 ```ts
 stubRun({ fetchResponse: MARKDOWN_FETCH_RESPONSE });
 ```
 */
function stubRun(
  {
    searchResponse = SEARCH_RESPONSE,
    fetchResponse = MARKDOWN_FETCH_RESPONSE,
    searchError,
    fetchError,
    loadError,
  }: {
    readonly searchResponse?: unknown;
    readonly fetchResponse?: unknown;
    readonly searchError?: Error;
    readonly fetchError?: Error;
    readonly loadError?: Error;
  } = {},
): StubRun {
  /**
   Search inputs the stub client received.
   */
  const searchInputs: unknown[] = [];
  /**
   Fetch inputs the stub client received.
   */
  const fetchInputs: unknown[] = [];
  return {
    dependencies: {
      config: CONFIG,
      client: {
        async search(options,) {
          searchInputs.push(options.input,);
          if (searchError !== undefined)
            throw searchError;
          return {
            provider: 'exa',
            response: searchResponse,
          };
        },
        async fetch(options,) {
          fetchInputs.push(options.input,);
          if (fetchError !== undefined)
            throw fetchError;
          return {
            provider: 'linkup',
            response: fetchResponse,
          };
        },
      },
    },
    searchInputs,
    fetchInputs,
    ...(loadError === undefined ? {} : { loadError, }),
  };
}

/**
 Build empty captured streams for one run.

 @returns writers and text accessors for the run

 @example
 ```ts
 captureStreams();
 ```
 */
function captureStreams(): CapturedStreams {
  /**
   Captured stdout chunks.
   */
  const stdoutChunks: string[] = [];
  /**
   Captured stderr chunks.
   */
  const stderrChunks: string[] = [];
  return {
    stdout: {
      write(text,) {
        stdoutChunks.push(text,);
      },
    },
    stderr: {
      write(text,) {
        stderrChunks.push(text,);
      },
    },
    stdoutText: function stdoutText() {
      return stdoutChunks.join('',);
    },
    stderrText: function stderrText() {
      return stderrChunks.join('',);
    },
  };
}

/**
 Run one command line against captured streams and stub dependencies.

 @param argv - arguments after the script path

 @param stub - stub dependencies for the run

 @returns settled status and captured stream text

 @throws whatever the run throws, for cases asserting on the thrown failure

 @example
 ```ts
 await captureSettledRun({ argv: ['search', 'docs'], stub: stubRun() });
 ```
 */
async function captureSettledRun(
  {
    argv,
    stub,
  }: {
    readonly argv: readonly string[];
    readonly stub: StubRun;
  },
): Promise<SettledCapture> {
  /**
   Captured run streams.
   */
  const streams = captureStreams();
  /**
   Settled status of the run.
   */
  const status = await runSearchFetchCli({
    argv,
    stdout: streams.stdout,
    stderr: streams.stderr,
    loadRunDependencies: async function loadStubDependencies() {
      if (stub.loadError !== undefined)
        throw stub.loadError;
      return stub.dependencies;
    },
  },);
  return {
    status,
    stdout: streams
      .stdoutText(),
    stderr: streams
      .stderrText(),
  };
}

/**
 Run one command line expected to fail through a thrown error.

 @param argv - arguments after the script path

 @param stub - stub dependencies for the run

 @returns thrown failure and captured stream text

 @example
 ```ts
 await captureThrownRun({ argv: ['search', 'docs'], stub: stubRun({ searchError: new Error('down') }) });
 ```
 */
async function captureThrownRun(
  {
    argv,
    stub,
  }: {
    readonly argv: readonly string[];
    readonly stub: StubRun;
  },
): Promise<ThrownCapture> {
  /**
   Captured run streams.
   */
  const streams = captureStreams();
  /**
   Thrown failure, or the sentinel when the run settled instead.
   */
  const thrown = await caughtRunFailure({
    argv,
    stub,
    streams,
  },);
  return {
    thrown,
    stdout: streams
      .stdoutText(),
    stderr: streams
      .stderrText(),
  };
}

/**
 Run one command line against captured streams and keep its thrown failure.

 @param argv - arguments after the script path

 @param stub - stub dependencies for the run

 @param streams - captured run streams

 @returns thrown failure, or {@link RUN_SETTLED_SENTINEL} when the run settled

 @example
 ```ts
 await caughtRunFailure({ argv: ['search', 'docs'], stub: stubRun(), streams: captureStreams() });
 ```
 */
async function caughtRunFailure(
  {
    argv,
    stub,
    streams,
  }: {
    readonly argv: readonly string[];
    readonly stub: StubRun;
    readonly streams: CapturedStreams;
  },
): Promise<unknown> {
  try {
    await runSearchFetchCli({
      argv,
      stdout: streams.stdout,
      stderr: streams.stderr,
      loadRunDependencies: async function loadStubDependencies() {
        if (stub.loadError !== undefined)
          throw stub.loadError;
        return stub.dependencies;
      },
    },);
    return RUN_SETTLED_SENTINEL;
  }
  catch (error: unknown) {
    return error;
  }
}

//endregion Helpers

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runSearchFetchCli.name,
      children: [
        //region Verb dispatch and argument grammar

        it({
          name: 'runs web_search for one positional query',
          fn: async () => {
            /**
             Stub dependencies recording the search call.
             */
            const stub = stubRun();
            /**
             Captured run.
             */
            const capture = await captureSettledRun({
              argv: ['search', 'docs',],
              stub,
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(stub.searchInputs,).toEqual([{
              query: 'docs',
            },],);
            expect(stub.fetchInputs,).toEqual([],);
            expect(capture.stdout,).toContain('"title":"good"',);
            expect(capture.stderr,).toBe('',);
          },
        },),
        it({
          name: 'runs web_fetch for one positional URL',
          fn: async () => {
            /**
             Stub dependencies recording the fetch call.
             */
            const stub = stubRun();
            /**
             Captured run.
             */
            const capture = await captureSettledRun({
              argv: ['fetch', 'https://example.com/page',],
              stub,
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(stub.fetchInputs,).toEqual([{
              url: 'https://example.com/page',
            },],);
            expect(stub.searchInputs,).toEqual([],);
            expect(capture.stdout,).toBe('# fetched\n',);
            expect(capture.stderr,).toBe('',);
          },
        },),
        it({
          name: 'joins words after -- with single spaces into the one query',
          fn: async () => {
            /**
             Stub dependencies recording the search call.
             */
            const stub = stubRun();
            /**
             Captured run.
             */
            const capture = await captureSettledRun({
              argv: ['search', '--', 'cachyos', 'kernel', '2026',],
              stub,
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(stub.searchInputs,).toEqual([{
              query: 'cachyos kernel 2026',
            },],);
          },
        },),
        it({
          name: 'treats tokens after -- as data, never as options',
          fn: async () => {
            /**
             Stub dependencies recording the search call.
             */
            const stub = stubRun();
            /**
             Captured run.
             */
            const capture = await captureSettledRun({
              argv: ['search', '--', '--help',],
              stub,
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(stub.searchInputs,).toEqual([{
              query: '--help',
            },],);
            expect(capture.stdout.includes('Usage:'),).toBe(false,);
          },
        },),

        //endregion Verb dispatch and argument grammar

        //region Grammar refusals

        ...[
          {
            name: 'refuses a positional together with -- words (documented unsupported)',
            argv: ['search', 'cachyos', '--', 'kernel',],
          },
          {
            name: 'refuses several positionals before --',
            argv: ['search', 'cachyos', 'kernel',],
          },
          {
            name: 'refuses zero arguments after the verb',
            argv: ['search',],
          },
          {
            name: 'refuses an unknown verb',
            argv: ['frobnicate', 'docs',],
          },
          {
            name: 'refuses a missing flag value',
            argv: ['search', 'docs', '--include-domains',],
          },
          {
            name: 'refuses an unknown flag',
            argv: ['search', '--nope', 'docs',],
          },
          {
            name: 'refuses a repeated single-valued flag',
            argv: [
              'search',
              '--from-date',
              '2026-01-01',
              '--from-date',
              '2026-02-01',
              'docs',
            ],
          },
          {
            name: 'refuses a repeated boolean flag',
            argv: ['--help', '--help',],
          },
          {
            name: 'refuses a bare invocation with no verb',
            argv: [],
          },
        ].map(function grammarRefusalCase(testCase,) {
          return it({
            name: testCase.name,
            fn: async () => {
              /**
               Stub dependencies the refusal must never reach.
               */
              const stub = stubRun();
              /**
               Captured refusal.
               */
              const capture = await captureSettledRun({
                argv: testCase.argv,
                stub,
              },);
              expect(capture.status,).toBe(usageExitCode,);
              expect(capture.stdout,).toBe('',);
              expect(capture.stderr.includes('Usage:'),).toBe(true,);
              expect(stub.searchInputs,).toEqual([],);
              expect(stub.fetchInputs,).toEqual([],);
            },
          },);
        },),

        //endregion Grammar refusals

        //region Option handling

        it({
          name: 'trims --include-domains entries and drops empty ones',
          fn: async () => {
            /**
             Stub dependencies recording the search call.
             */
            const stub = stubRun();
            /**
             Captured run.
             */
            await captureSettledRun({
              argv: [
                'search',
                '--include-domains',
                ' a.com , ,b.com ,, ',
                'docs',
              ],
              stub,
            },);
            expect(stub.searchInputs,).toEqual([{
              query: 'docs',
              includeDomains: ['a.com', 'b.com',],
            },],);
          },
        },),
        it({
          name: 'treats an all-empty --include-domains list as the flag being absent',
          fn: async () => {
            /**
             Stub dependencies recording the search call.
             */
            const stub = stubRun();
            /**
             Captured run.
             */
            await captureSettledRun({
              argv: ['search', '--include-domains', ',,', 'docs',],
              stub,
            },);
            expect(stub.searchInputs,).toEqual([{
              query: 'docs',
            },],);
          },
        },),
        it({
          name: 'passes --from-date and --to-date through exactly as the shared schema takes them',
          fn: async () => {
            /**
             Stub dependencies recording the search call.
             */
            const stub = stubRun();
            /**
             Captured run.
             */
            await captureSettledRun({
              argv: [
                'search',
                '--from-date',
                '2026-01-01',
                '--to-date',
                '2026-02-02T03:04:05Z',
                'docs',
              ],
              stub,
            },);
            expect(stub.searchInputs,).toEqual([{
              query: 'docs',
              fromDate: '2026-01-01',
              toDate: '2026-02-02T03:04:05Z',
            },],);
          },
        },),
        it({
          name: 'lets the shared tool schema warn and strip keys a verb does not accept',
          fn: async () => {
            /**
             Stub dependencies recording the fetch call.
             */
            const stub = stubRun();
            /**
             Captured run carrying the tool warning and the page markdown.
             */
            const capture = await captureSettledRun({
              argv: ['fetch', '--from-date', '2026-01-01', 'https://example.com/page',],
              stub,
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(stub.fetchInputs,).toEqual([{
              url: 'https://example.com/page',
            },],);
            expect(
              capture
                .stdout
                .includes('ignored extension-unsupported web_fetch parameters: fromDate',),
            ).toBe(true,);
            expect(capture.stdout,).toContain('# fetched',);
          },
        },),

        //endregion Option handling

        //region Furniture

        it({
          name: 'prints global usage on stdout and exits 0 for --help',
          fn: async () => {
            /**
             Captured help run.
             */
            const capture = await captureSettledRun({
              argv: ['--help',],
              stub: stubRun(),
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(capture.stdout.includes('Usage:'),).toBe(true,);
            expect(capture.stdout.includes('search-fetch search <query>'),).toBe(true,);
            expect(capture.stdout.includes('search-fetch fetch <url>'),).toBe(true,);
            expect(capture.stdout.includes('--include-domains'),).toBe(true,);
            expect(capture.stderr,).toBe('',);
          },
        },),
        it({
          name: 'prints verb usage on stdout and exits 0 for a verb --help',
          fn: async () => {
            /**
             Captured help run.
             */
            const capture = await captureSettledRun({
              argv: ['search', '--help',],
              stub: stubRun(),
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(capture.stdout.includes('search-fetch search <query>'),).toBe(true,);
            expect(capture.stdout.includes('search-fetch fetch <url>'),).toBe(false,);
            expect(capture.stderr,).toBe('',);
          },
        },),
        it({
          name: 'prints the package version on stdout and exits 0 for --version',
          fn: async () => {
            /**
             Captured version run.
             */
            const capture = await captureSettledRun({
              argv: ['--version',],
              stub: stubRun(),
            },);
            /**
             Manifest version this package reports.
             */
            const manifest: unknown = JSON.parse(
              await readFile(new URL('../package.json', import.meta.url,), 'utf8',),
            );
            if (((typeof manifest) !== 'object')
              || (manifest === null)
              || (!('version' in manifest))
              || ((typeof manifest.version) !== 'string'))
              throw new Error('package.json has no string version',);
            expect(capture.status,).toBe(successExitCode,);
            expect(capture.stdout,).toBe(`${manifest.version}\n`,);
            expect(capture.stderr,).toBe('',);
          },
        },),
        it({
          name: 'prefers --help when --help and --version appear together',
          fn: async () => {
            /**
             Captured furniture run.
             */
            const capture = await captureSettledRun({
              argv: ['--help', '--version',],
              stub: stubRun(),
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(capture.stdout.includes('Usage:'),).toBe(true,);
          },
        },),

        //endregion Furniture

        //region Exit-status mapping

        it({
          name: 'refuses a blocklist URL with the usage status before any provider call',
          fn: async () => {
            /**
             Stub dependencies whose client must stay unused.
             */
            const stub = stubRun();
            /**
             Captured refusal.
             */
            const capture = await captureSettledRun({
              argv: ['fetch', `https://${BLOCKED_HOST}/page`,],
              stub,
            },);
            expect(capture.status,).toBe(usageExitCode,);
            expect(capture.stdout,).toBe('',);
            expect(stub.fetchInputs,).toEqual([],);
          },
        },),
        it({
          name: 'reports a configuration load failure with the usage status and empty stdout',
          fn: async () => {
            /**
             Stub dependencies whose loader refuses to load.
             */
            const stub = stubRun({
              loadError: new SearchFetchCliConfigurationError({
                message: 'config fixture refused to load',
              },),
            },);
            /**
             Captured refusal.
             */
            const capture = await captureSettledRun({
              argv: ['search', 'docs',],
              stub,
            },);
            expect(capture.status,).toBe(usageExitCode,);
            expect(capture.stdout,).toBe('',);
          },
        },),
        it({
          name: 'leaves stdout empty and rethrows a provider failure for the standard failure exit',
          fn: async () => {
            /**
             Captured failed run.
             */
            const capture = await captureThrownRun({
              argv: ['search', 'docs',],
              stub: stubRun({
                searchError: new Error('provider down',),
              },),
            },);
            expect(capture.thrown,).toBeInstanceOf(Error,);
            expect((capture.thrown as Error).message,).toBe('provider down',);
            expect(capture.stdout,).toBe('',);
          },
        },),
        it({
          name: 'leaves stdout empty and rethrows a deadline expiry for the standard failure exit',
          fn: async () => {
            /**
             Captured failed run.
             */
            const capture = await captureThrownRun({
              argv: ['fetch', 'https://example.com/slow',],
              stub: stubRun({
                fetchError: new Error('search-fetch call exceeded the 120s deadline',),
              },),
            },);
            expect(capture.thrown,).toBeInstanceOf(Error,);
            expect((capture.thrown as Error).message.includes('120s deadline',),).toBe(true,);
            expect(capture.stdout,).toBe('',);
          },
        },),

        //endregion Exit-status mapping

        //region Output shape

        it({
          name: 'ends truncated output with the truncation notice naming a readable full-response path',
          fn: async () => {
            /**
             Captured run over the oversized response.
             */
            const capture = await captureSettledRun({
              argv: ['fetch', 'https://example.com/big',],
              stub: stubRun({
                fetchResponse: OVERSIZED_FETCH_RESPONSE,
              },),
            },);
            expect(capture.status,).toBe(successExitCode,);
            expect(capture.stdout.includes('[Linkup response truncated:'),).toBe(true,);
            expect(capture.stdout.trimEnd().endsWith(']'),).toBe(true,);
            /**
             Marker introducing the full-response path in the truncation notice.
             */
            const noticePathMarker = 'saved to: ';
            /**
             Index of the path marker in the captured output.
             */
            const markerIndex = capture
              .stdout
              .lastIndexOf(noticePathMarker,);
            if (markerIndex === (-1))
              throw new Error('truncation notice named no full-response path',);
            /**
             Output after the marker, carrying the path up to the closing bracket.
             */
            const afterMarker = capture
              .stdout
              .slice(markerIndex + noticePathMarker.length,);
            /**
             Index closing the named path.
             */
            const closeIndex = afterMarker.indexOf(']',);
            if (closeIndex === (-1))
              throw new Error('truncation notice named no full-response path',);
            /**
             Full-response path named by the truncation notice.
             */
            const fullResponsePath = afterMarker.slice(0, closeIndex,);
            /**
             Full response text recovered from the named path.
             */
            const fullResponseText = await readFile(fullResponsePath, 'utf8',);
            expect(fullResponseText.includes(OVERSIZED_MARKDOWN,),).toBe(true,);
          },
        },),
        it({
          name: 'emits no MCP resource URIs or resource_link blocks on truncated output',
          fn: async () => {
            /**
             Captured truncated run, the case where an MCP surface would add resources.
             */
            const capture = await captureSettledRun({
              argv: ['fetch', 'https://example.com/big',],
              stub: stubRun({
                fetchResponse: OVERSIZED_FETCH_RESPONSE,
              },),
            },);
            expect(capture.stdout.includes('search-fetch://'),).toBe(false,);
            expect(capture.stdout.includes('resource_link'),).toBe(false,);
            expect(capture.stderr.includes('search-fetch://'),).toBe(false,);
          },
        },),

        //endregion Output shape
      ],
    },),
  ],
},);
