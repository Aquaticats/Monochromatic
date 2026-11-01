import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { spawnSync, } from 'node:child_process';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseStatuslineInput,
  renderStatusline,
} from '../dist/final/node/statusline.mjs';

/**
 Stable render timestamp used by rate-limit tests.
 */
const RENDERED_AT_MS = Date.parse('2026-06-01T12:00:00Z',);

/**
 Milliseconds in one hour for fixture reset timestamps.
 */
const HOUR_MS = 3_600_000;

/**
 ANSI reset sequence.
 */
const RESET = '\u001B[0m';

/**
 ANSI red sequence.
 */
const RED = '\u001B[31m';

/**
 ANSI yellow sequence.
 */
const YELLOW = '\u001B[33m';

/**
 Temporary workspace handle that removes itself at the end of an `await using`
 scope.
 */
type TempWorkspace = {
  /**
   Absolute path to temporary workspace root.
   */
  readonly path: string;
  /**
   Removes temporary workspace root recursively.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Environment variable restore handle.
 */
type EnvRestore = {
  /**
   Restores original environment variable value.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 Creates isolated temporary workspace.
 
 @returns disposable temporary workspace handle
 
 @example
 ```ts
 await using workspace = await makeWorkspace();
 ```
 */
async function makeWorkspace(): Promise<TempWorkspace> {
  /**
   Temporary root path.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    'claude-statusline-',
  ),);

  return {
    path,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Temporarily sets HOME.
 
 @param value - HOME value
 
 @returns disposable restore handle
 
 @example
 ```ts
 using restore = homeEnv('/tmp/home');
 ```
 */
function homeEnv(value: string,): EnvRestore {
  /**
   Original HOME value.
   */
  const original = process.env.HOME;
  process.env.HOME = value;

  return {
    [Symbol.dispose]: function restore(): void {
      if (original === undefined) {
        delete process.env.HOME;
        return;
      }

      process.env.HOME = original;
    },
  };
}

/**
 Writes Claude settings fixture under temporary HOME.
 
 @param home - temporary HOME path
 
 @param effortLevel - effort level setting value
 
 @example
 ```ts
 await writeSettings({ home: '/tmp/home', effortLevel: 'low' });
 ```
 */
async function writeSettings({
  home,
  effortLevel,
}: Readonly<{
  home: string;
  effortLevel: string;
}>,): Promise<void> {
  await mkdir(
    join(
      home,
      '.claude',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      home,
      '.claude',
      'settings.json',
    ),
    JSON.stringify({ effortLevel, },),
  );
}

/**
 Converts epoch milliseconds to epoch seconds.
 
 @param epochMs - epoch timestamp in milliseconds
 
 @returns epoch timestamp in seconds
 
 @example
 ```ts
 epochSeconds(Date.now());
 ```
 */
function epochSeconds(epochMs: number,): number {
  return epochMs / 1_000;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: parseStatuslineInput.name,
      children: [
        it({
          name: 'parses trusted Claude statusline JSON',
          fn: async function testParseStatuslineInput(): Promise<void> {
            const input = parseStatuslineInput('{"model":{"display_name":"Opus"}}',);
            expect(input.model?.display_name,).toBe('Opus',);
          },
        },),
      ],
    },),
    describe({
      name: renderStatusline.name,
      children: [
        it({
          name: 'renders model, context, rate limit, effort, and activity segments',
          fn: async function testRenderStatusline(): Promise<void> {
            await using workspace = await makeWorkspace();
            using _home = homeEnv(workspace.path,);
            await writeSettings({
              home: workspace.path,
              effortLevel: 'low',
            },);
            /**
             Transcript path fixture.
             */
            const transcriptPath = join(
              workspace.path,
              'session.jsonl',
            );
            await writeFile(
              transcriptPath,
              `${JSON.stringify({
                type: 'assistant',
                message: {
                  content: [
                    {
                      type: 'text',
                      text: 'Assistant is compiling, then testing the statusline.',
                    },
                  ],
                },
              },)}\n`,
            );

            const line = await renderStatusline({
              input: {
                transcript_path: transcriptPath,
                model: {
                  display_name: 'Opus 4.8 (1M context)',
                },
                context_window: {
                  context_window_size: 1_000_000,
                  current_usage: {
                    input_tokens: 50_000,
                    cache_creation_input_tokens: 1_000,
                    cache_read_input_tokens: 45,
                    output_tokens: 0,
                  },
                },
                rate_limits: {
                  five_hour: {
                    used_percentage: 30,
                    resets_at: epochSeconds(RENDERED_AT_MS + (4 * HOUR_MS),),
                  },
                },
              },
              renderedAtMs: RENDERED_AT_MS,
            },);

            expect(line,).toBe(
              `Opus ${YELLOW}○${RESET}     51,045/1,000,000    ${RED}70% left →150%${RESET} (4h)    Testing`,
            );
          },
        },),
      ],
    },),
    describe({
      name: 'claude-code-statusline CLI',
      children: [
        it({
          name: 'reads stdin and writes one statusline',
          fn: async function testCli(): Promise<void> {
            await using workspace = await makeWorkspace();
            await writeSettings({
              home: workspace.path,
              effortLevel: 'high',
            },);

            const result = spawnSync(
              process.execPath,
              [
                '../dist/final/node/statusline.mjs',
              ],
              {
                cwd: new URL('.', import.meta.url,),
                encoding: 'utf8',
                env: {
                  ...process.env,
                  HOME: workspace.path,
                },
                input: JSON.stringify({
                  model: {
                    display_name: 'Sonnet 4.6 (200K context)',
                  },
                },),
              },
            );

            expect(result.status,).toBe(0,);
            expect(result.stdout,).toBe('Sonnet\n',);
            expect(result.stderr,).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
