/**
 Tests for the program that wraps shipped text at its semantic boundaries:
 how it is found, how its version is checked, what it is handed, and what
 each way it can fail says.

 THE PROGRAM IS REPLACED BY SMALL STAND-INS for every case about how it is
 called or how it fails, because a real run cannot be made to report another
 version, exit with a chosen status, or record the arguments it received
 (`lint-stand-in.test-fixture.ts`). The real program is exercised by
 `semantic-wrap-shapes.unit.test.ts` and `semantic-wrap.unit.test.ts`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';

import {
  LINT_BINARY_NAME,
  LintBinaryMissingError,
  LintBinaryRunError,
  LintBinaryTextError,
  LintBinaryVersionError,
  PINNED_LINT_BINARY_VERSION,
  wrapWithLintBinary,
} from '../dist/final/node/index.mjs';
import {
  caughtFrom,
  standIn,
  startsOf,
} from './lint-stand-in.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wrapWithLintBinary.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RAISES LintBinaryMissingError NAMING THE TOOL AND THE WAYS TO INSTALL IT when no program of '
            + 'that name is found, so a run stops at its first wrap instead of shipping unwrapped text',
          fn: async () => {
            /**
             What the wrap raised for a program nobody installed.
             */
            const thrown = caughtFrom({
              body: () =>
                wrapWithLintBinary({
                  binary: 'purr-lint-nobody-installed',
                  text: 'It naps. It wakes.',
                },),
            },);

            expect(thrown,).toBeInstanceOf(LintBinaryMissingError,);
            expect(thrown,).toHaveProperty(
              'name',
              'LintBinaryMissingError',
            );
            expect((thrown as Error).message,).toContain('purr-lint-nobody-installed',);
            expect((thrown as Error).message,).toContain('mise run toolchain:sync',);
            expect((thrown as Error).message,).toContain(
              `cargo binstall ${LINT_BINARY_NAME}@${PINNED_LINT_BINARY_VERSION}`,
            );
          },
        },),

        it({
          name: 'RAISES LintBinaryVersionError NAMING BOTH VERSIONS for a program that is not the pinned one, '
            + 'and never hands it the passage',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'wrap-binary-version-', },);
            const { program, log, } = await standIn({
              dir: owned.path,
              version: '9.9.9',
              behaviour: 'echo',
            },);

            /**
             What the wrap raised for the wrong version.
             */
            const thrown = caughtFrom({
              body: () =>
                wrapWithLintBinary({
                  binary: program,
                  text: 'It naps. It wakes.',
                },),
            },);

            expect(thrown,).toBeInstanceOf(LintBinaryVersionError,);
            expect((thrown as Error).message,).toContain('9.9.9',);
            expect((thrown as Error).message,).toContain(PINNED_LINT_BINARY_VERSION,);
            expect((thrown as Error).message,).toContain('mise run toolchain:sync',);
            expect(await startsOf({ log, },),).toEqual([['--version',],],);
          },
        },),

        it({
          name: 'RAISES LintBinaryRunError ON A NON-ZERO EXIT, naming the status and the length of what the '
            + 'program wrote to standard error but never the passage, which a finding may quote',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'wrap-binary-exit-', },);
            const { program, } = await standIn({
              dir: owned.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'fail-two',
            },);
            /**
             A passage the stand-in repeats on standard error.
             */
            const passage = 'The tabby hid the secret toebean.';

            /**
             What the wrap raised for the failing run.
             */
            const thrown = caughtFrom({
              body: () =>
                wrapWithLintBinary({
                  binary: program,
                  text: passage,
                },),
            },);

            expect(thrown,).toBeInstanceOf(LintBinaryRunError,);
            expect((thrown as Error).message,).toContain('status 2',);
            expect((thrown as Error).message,).toContain(`${String(passage.length,)} bytes`,);
            expect((thrown as Error).message,).not.toContain('toebean',);
          },
        },),

        it({
          name: 'ASKS THE VERSION ONCE PER PROGRAM and starts it for every passage with the configuration and '
            + 'the standard-input flags, the passage on standard input and never in an argument',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'wrap-binary-calls-', },);
            const { program, log, } = await standIn({
              dir: owned.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'echo',
            },);

            /**
             Two passages through the same program.
             */
            const wrapped = [
              wrapWithLintBinary({
                binary: program,
                text: '-dash. First passage.',
              },),
              wrapWithLintBinary({
                binary: program,
                text: 'Second passage, here.',
              },),
            ];
            /**
             Everything the stand-in was started with.
             */
            const starts = await startsOf({ log, },);

            expect(wrapped,).toEqual([
              '-dash. First passage.',
              'Second passage, here.',
            ],);
            expect(starts.map(function first(args,): string {
              return args.at(0,) ?? '';
            },),).toEqual([
              '--version',
              '--config',
              '--config',
            ],);
            expect(starts[1],).toEqual(starts[2],);
            /**
             The configuration path the program was handed.
             */
            const config = starts[1]?.[1] ?? '';
            /**
             The copy the build puts beside the built chunks, which is where
             the built module must read the configuration from.
             */
            const builtCopy = join(
              import.meta.dirname,
              '..',
              'dist',
              'final',
              'node',
              'semantic-wrap.config.jsonc',
            );
            expect(config,).toBe(builtCopy,);
            expect((await readFile(
              config,
              'utf8',
            )).includes('markdown/semantic-line-breaks',),).toBe(true,);
            /**
             The configuration as the package ships it in `src`.
             */
            const sourceBytes = await readFile(join(
              import.meta.dirname,
              'semantic-wrap.config.jsonc',
            ),);
            expect(await readFile(config,),).toEqual(sourceBytes,);
            expect(starts[1]?.slice(2,),).toEqual([
              '--stdin',
              '--stdin-filename',
              'passage.md',
              '--fix',
            ],);
          },
        },),

        it({
          name: 'RAISES LintBinaryTextError FOR A LONE SURROGATE before starting anything, because it cannot '
            + 'cross standard input as UTF-8 without becoming another character',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'wrap-binary-surrogate-', },);
            const { program, log, } = await standIn({
              dir: owned.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'echo',
            },);

            /**
             What the wrap raised for the lone surrogate.
             */
            const thrown = caughtFrom({
              body: () =>
                wrapWithLintBinary({
                  binary: program,
                  text: 'It naps \uD83D and wakes.',
                },),
            },);

            expect(thrown,).toBeInstanceOf(LintBinaryTextError,);
            expect((thrown as Error).message,).toContain('index 8',);
            expect(await startsOf({ log, },),).toEqual([],);
          },
        },),

        it({
          name: 'RETURNS AN EMPTY PASSAGE WITHOUT STARTING THE PROGRAM, since nothing in it can break',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'wrap-binary-empty-', },);
            const { program, log, } = await standIn({
              dir: owned.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'echo',
            },);

            expect(wrapWithLintBinary({
              binary: program,
              text: '',
            },),).toBe('',);
            expect(await startsOf({ log, },),).toEqual([],);
          },
        },),

        it({
          name: 'HOLDS THE PINNED VERSION TO THE ONE toolchain.lock.json RECORDS for the cargo tool, since '
            + 'the constant and the lock are two writings of one fact; the tool plan asks for the latest '
            + 'release of a tool this repository publishes, so the lock names the version installed. '
            + 'The lock was mise.lock until the toolchain moved to MPM on 2026-10-28',
          fn: async () => {
            /**
             The repository's tool lock, four directory levels above this one, parsed.
             */
            const lock: unknown = JSON.parse(
              await readFile(
                join(
                  import.meta.dirname,
                  '..',
                  '..',
                  '..',
                  '..',
                  'toolchain.lock.json',
                ),
                'utf8',
              ),
            );
            /**
             The lock's table of tools, keyed by tool id.
             */
            const tools = isJsonRecord(lock,) ? lock.tools : undefined;
            /**
             The lint tool's entry in that table.
             */
            const entry = isJsonRecord(tools,) ? tools[`cargo:${LINT_BINARY_NAME}`] : undefined;

            expect(isJsonRecord(entry,),).toBe(true,);
            expect(isJsonRecord(entry,) ? entry.version : undefined,).toBe(PINNED_LINT_BINARY_VERSION,);
          },
        },),
      ],
    },),
  ],
},);
