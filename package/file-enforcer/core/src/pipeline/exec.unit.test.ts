import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  exec,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region exec (direct)

    describe({
      name: 'exec (direct)',
      children: [
        it({
          name: 'captures stdout from a successful command',
          fn: async () => {
            /** Simple echo command to verify stdout capture */
            const result = await exec({
              cmd: 'echo',
              args: ['hello',],
            },);
            expect(result.trim(),).toBe('hello',);
          },
        },),
        it({
          name: 'passes multiple arguments to the command',
          fn: async () => {
            /** Multiple args should all be forwarded */
            const result = await exec({
              cmd: 'echo',
              args: ['hello', 'world',],
            },);
            expect(result.trim(),).toBe('hello world',);
          },
        },),
        it({
          name: 'throws on non-zero exit code',
          fn: async () => {
            /** `false` always exits with code 1 */
            await expect(exec({ cmd: 'false', },),).rejects.toThrow(
              'Command failed with exit code 1: false',
            );
          },
        },),
        it({
          name: 'includes stderr in the error message',
          fn: async () => {
            /** Command that writes to stderr and fails */
            await expect(
              exec({
                cmd: 'sh',
                args: ['-c', 'echo error-msg >&2; exit 1',],
              },),
            )
              .rejects
              .toThrow('error-msg',);
          },
        },),
        it({
          name: 'handles command with no arguments',
          fn: async () => {
            /** `true` exits successfully with no output */
            const result = await exec({ cmd: 'true', },);
            expect(result,).toBe('',);
          },
        },),
        it({
          name: 'preserves newlines in stdout',
          fn: async () => {
            /** printf outputs exact bytes without a trailing newline */
            const result = await exec({
              cmd: 'printf',
              args: [String.raw`line1\nline2\nline3`,],
            },);
            expect(result,).toBe('line1\nline2\nline3',);
          },
        },),
        it({
          name: 'handles large stdout output',
          fn: async () => {
            /** Generate 1000 lines of output */
            const result = await exec({
              cmd: 'seq',
              args: ['1', '1000',],
            },);
            /** Should have 1000 lines (seq output ends with newline) */
            const lineCount = 1_000;
            expect(result.trim().split('\n',).length,).toBe(lineCount,);
          },
        },),
      ],
    },),

    //endregion exec (direct)

    //region exec (platform-aware)

    describe({
      name: 'exec (platform-aware)',
      children: [
        it({
          name: 'executes the command of the first matching predicate',
          fn: async () => {
            /** First entry predicate succeeds, so its command runs */
            const result = await exec({
              platformCommands: [
                [['true',], ['echo', 'first',],],
                [['true',], ['echo', 'second',],],
              ],
            },);
            expect(result.trim(),).toBe('first',);
          },
        },),
        it({
          name: 'skips entries whose predicate fails',
          fn: async () => {
            /** First predicate fails, second succeeds */
            const result = await exec({
              platformCommands: [
                [['false',], ['echo', 'skipped',],],
                [['true',], ['echo', 'matched',],],
              ],
            },);
            expect(result.trim(),).toBe('matched',);
          },
        },),
        it({
          name: 'supports command with no arguments',
          fn: async () => {
            /** Command as single-element array */
            const result = await exec({
              platformCommands: [
                [['true',], ['true',],],
              ],
            },);
            expect(result,).toBe('',);
          },
        },),
        it({
          name: 'supports predicate with arguments',
          fn: async () => {
            /** Predicate with multiple args */
            const result = await exec({
              platformCommands: [
                [['ls', '/dev/null',], ['echo', 'found',],],
              ],
            },);
            expect(result.trim(),).toBe('found',);
          },
        },),
        it({
          name: 'throws PlatformMatchError when no predicate matches',
          fn: async () => {
            /** All predicates fail */
            await expect(
              exec({
                platformCommands: [
                  [['false',], ['echo', 'a',],],
                  [['false',], ['echo', 'b',],],
                ],
              },),
            )
              .rejects
              .toThrow('No platform predicate matched',);
          },
        },),
        it({
          name: 'error message includes tested predicates',
          fn: async () => {
            /** Verify predicate names appear in the error for debuggability */
            await expect(
              exec({
                platformCommands: [
                  [['nonexistent-check-1',], ['echo', 'a',],],
                  [['nonexistent-check-2', '--flag',], ['echo', 'b',],],
                ],
              },),
            )
              .rejects
              .toThrow('nonexistent-check-1',);
          },
        },),
        it({
          name: 'throws when matched command fails',
          fn: async () => {
            /** Predicate matches but the command itself exits non-zero */
            await expect(
              exec({
                platformCommands: [
                  [['true',], ['false',],],
                ],
              },),
            )
              .rejects
              .toThrow('Command failed',);
          },
        },),
        it({
          name: 'supports nested PlatformCommands as command value',
          fn: async () => {
            /** Outer predicate matches, inner dispatch selects the actual command */
            const innerCommands = [
              [['false',], ['echo', 'inner-skipped',],],
              [['true',], ['echo', 'inner-matched',],],
            ] as const;
            const result = await exec({
              platformCommands: [
                [['true',], innerCommands,],
              ],
            },);
            expect(result.trim(),).toBe('inner-matched',);
          },
        },),
        it({
          name: 'nested PlatformCommands throws when no inner predicate matches',
          fn: async () => {
            /** Outer matches but all inner predicates fail */
            const innerCommands = [
              [['false',], ['echo', 'unreachable',],],
            ] as const;
            await expect(
              exec({
                platformCommands: [
                  [['true',], innerCommands,],
                ],
              },),
            )
              .rejects
              .toThrow('No platform predicate matched',);
          },
        },),
      ],
    },),
    //endregion exec (platform-aware)
  ],
},);
