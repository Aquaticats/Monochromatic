import { dirname, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertIsManagedVmDir,
  bootVm,
  detectHypervisor,
  isVmNameChar,
  showStatus,
  stripJsoncComments,
  syncVm,
  updateConfig,
  validateName,
  vmConfigPath,
  vmDir,
} from '../dist/final/node/index.mjs';

/**
 Names that must never reach a path derivation: separators, parent components,
 absolute paths, whitespace, and the empty string.
 */
const VM_NAMES_THAT_ESCAPE: readonly string[] = [
  '../evil',
  '..',
  '.',
  'a/b',
  '/etc/passwd',
  'my vm',
  '',
];

/**
 Every command that derives a per-VM path, paired with the call a user makes.
 A name check that lives in one handler instead of in the shared helpers shows up
 here as a command that reads or writes outside the managed root.
 */
const COMMANDS_THAT_DERIVE_A_PATH = [
  {
    name: bootVm.name,
    run: async (name: string,) => bootVm(name,),
  },
  {
    name: syncVm.name,
    run: async (name: string,) => syncVm(name,),
  },
  {
    name: showStatus.name,
    run: async (name: string,) => showStatus(name,),
  },
  {
    name: updateConfig.name,
    run: async (name: string,) => updateConfig({ name, },),
  },
] as const;

//region validateName: rejects unsafe VM names, accepts safe ones

await describe({
  name: '',
  children: [
    describe({
      name: validateName.name,
      children: [
        it({
          name: 'accepts simple alphanumeric name',
          fn: async () => {
            expect(() => {
              validateName('alpine',);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'accepts name with hyphens and underscores',
          fn: async () => {
            expect(() => {
              validateName('my-vm_01',);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'accepts single character name',
          fn: async () => {
            expect(() => {
              validateName('a',);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'accepts name starting with digit',
          fn: async () => {
            expect(() => {
              validateName('9test',);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'rejects name starting with hyphen',
          fn: async () => {
            expect(() => {
              validateName('-bad',);
            },)
              .toThrow('invalid VM name',);
          },
        },),

        it({
          name: 'rejects name starting with underscore',
          fn: async () => {
            expect(() => {
              validateName('_bad',);
            },)
              .toThrow('invalid VM name',);
          },
        },),

        it({
          name: 'rejects path traversal',
          fn: async () => {
            expect(() => {
              validateName('../evil',);
            },)
              .toThrow('invalid VM name',);
          },
        },),

        it({
          name: 'rejects name with spaces',
          fn: async () => {
            expect(() => {
              validateName('my vm',);
            },)
              .toThrow('invalid VM name',);
          },
        },),

        it({
          name: 'rejects name with dots',
          fn: async () => {
            expect(() => {
              validateName('my.vm',);
            },)
              .toThrow('invalid VM name',);
          },
        },),

        it({
          name: 'rejects empty string',
          fn: async () => {
            expect(() => {
              validateName('',);
            },)
              .toThrow('invalid VM name',);
          },
        },),

        it({
          name: 'rejects name with slashes',
          fn: async () => {
            expect(() => {
              validateName('a/b',);
            },)
              .toThrow('invalid VM name',);
          },
        },),
      ],
    },),

    //endregion validateName

    //region isVmNameChar: the single owner of the allowed-name-character policy

    describe({
      name: isVmNameChar.name,
      children: [
        ...['a', 'Z', '0', '_', '-',].map(function mapAllowed(c,) {
          return it({
            name: `accepts ${c}`,
            fn: async () => {
              expect(isVmNameChar(c,),).toBe(true,);
            },
          },);
        },),
        ...[' ', '.', '/', '~', '!', '+', '=', 'é',].map(function mapRejected(c,) {
          return it({
            name: `rejects ${JSON.stringify(c,)}`,
            fn: async () => {
              expect(isVmNameChar(c,),).toBe(false,);
            },
          },);
        },),
      ],
    },),

    //endregion isVmNameChar

    //region stripJsoncComments: removes comments while preserving string content

    describe({
      name: stripJsoncComments.name,
      children: [
        it({
          name: 'returns plain JSON unchanged',
          fn: async () => {
            const input = '{"a": 1, "b": "hello"}';
            expect(stripJsoncComments(input,),).toBe(input,);
          },
        },),

        it({
          name: 'strips single-line comment',
          fn: async () => {
            const input = '{"a": 1 // comment\n}';
            expect(stripJsoncComments(input,),).toBe('{"a": 1 \n}',);
          },
        },),

        it({
          name: 'strips multi-line comment',
          fn: async () => {
            const input = '{"a": /* block */ 1}';
            expect(stripJsoncComments(input,),).toBe('{"a":  1}',);
          },
        },),

        it({
          name: 'preserves comment-like content inside strings',
          fn: async () => {
            const input = '{"url": "http://example.com"}';
            expect(stripJsoncComments(input,),).toBe(input,);
          },
        },),

        it({
          name: 'preserves double-slash inside strings',
          fn: async () => {
            const input = '{"msg": "see // this"}';
            expect(stripJsoncComments(input,),).toBe(input,);
          },
        },),

        it({
          name: 'preserves block comment syntax inside strings',
          fn: async () => {
            const input = '{"msg": "a /* b */ c"}';
            expect(stripJsoncComments(input,),).toBe(input,);
          },
        },),

        it({
          name: 'handles escaped quotes inside strings',
          fn: async () => {
            const input = String.raw`{"msg": "say \"hello\""}`;
            expect(stripJsoncComments(input,),).toBe(input,);
          },
        },),

        it({
          name: 'closes a block comment whose body starts with a slash',
          fn: async () => {
            const input = '{/*/*/ "a": 1}';
            const stripped = stripJsoncComments(input,);
            expect(stripped,).toBe('{ "a": 1}',);
            expect(() => {
              JSON.parse(stripped,);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'closes a block comment whose body starts with two slashes',
          fn: async () => {
            expect(stripJsoncComments('{"a": /*// x */ 1}',),).toBe('{"a":  1}',);
          },
        },),

        it({
          name: 'consumes an unterminated block comment to end of input',
          fn: async () => {
            expect(stripJsoncComments('{"a": /* never closed',),).toBe('{"a": ',);
          },
        },),

        it({
          name: 'strips multiple single-line comments',
          fn: async () => {
            const input = '{\n// first\n"a": 1,\n// second\n"b": 2\n}';
            const expected = '{\n\n"a": 1,\n\n"b": 2\n}';
            expect(stripJsoncComments(input,),).toBe(expected,);
          },
        },),

        it({
          name: 'strips trailing comment after value',
          fn: async () => {
            const input = '{"a": 1} // trailing';
            expect(stripJsoncComments(input,),).toBe('{"a": 1} ',);
          },
        },),

        it({
          name: 'handles empty input',
          fn: async () => {
            expect(stripJsoncComments('',),).toBe('',);
          },
        },),

        it({
          name: 'result is valid JSON after stripping JSONC comments',
          fn: async () => {
            const input = `{
  // VM name
  "name": "alpine",
  /* disk size in bytes */
  "size": 1024
}`;
            const stripped = stripJsoncComments(input,);
            expect(() => {
              JSON.parse(stripped,);
            },)
              .not
              .toThrow();
            const parsed = JSON.parse(stripped,) as { name: string; size: number; };
            expect(parsed.name,).toBe('alpine',);
            expect(parsed.size,).toBe(1_024,);
          },
        },),
      ],
    },),

    //endregion stripJsoncComments

    //region vmDir / vmConfigPath: path construction from VM name

    describe({
      name: vmDir.name,
      children: [
        it({
          name: 'appends name to data directory',
          fn: async () => {
            const result = vmDir('alpine',);
            expect(result.endsWith('vmsync/alpine',),).toBe(true,);
          },
        },),

        it({
          name: 'handles name with hyphens',
          fn: async () => {
            const result = vmDir('fedora-dev',);
            expect(result.endsWith('vmsync/fedora-dev',),).toBe(true,);
          },
        },),

        it({
          name: 'returns a path directly inside the managed root',
          fn: async () => {
            const result = vmDir('alpine',);
            expect(dirname(result,),).toBe(dirname(vmDir('other',),),);
          },
        },),

        ...VM_NAMES_THAT_ESCAPE.map(function mapEscapingName(name,) {
          return it({
            name: `rejects ${JSON.stringify(name,),}`,
            fn: async () => {
              let caught: unknown;
              try {
                vmDir(name,);
              }
              catch (error) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(Error,);
              expect((caught as Error).message,).toContain('invalid VM name',);
            },
          },);
        },),
      ],
    },),
    describe({
      name: vmConfigPath.name,
      children: [
        it({
          name: 'appends config filename to VM directory',
          fn: async () => {
            const result = vmConfigPath('alpine',);
            expect(result.endsWith('vmsync/alpine/vmsync.jsonc',),).toBe(true,);
          },
        },),

        ...VM_NAMES_THAT_ESCAPE.map(function mapEscapingName(name,) {
          return it({
            name: `rejects ${JSON.stringify(name,),}`,
            fn: async () => {
              let caught: unknown;
              try {
                vmConfigPath(name,);
              }
              catch (error) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(Error,);
              expect((caught as Error).message,).toContain('invalid VM name',);
            },
          },);
        },),
      ],
    },),

    //endregion vmDir / vmConfigPath

    //region assertIsManagedVmDir: the containment check behind every derived path

    describe({
      name: assertIsManagedVmDir.name,
      children: [
        it({
          name: 'accepts a direct child of the managed root',
          fn: async () => {
            expect(() => {
              assertIsManagedVmDir({ name: 'alpine', candidate: vmDir('alpine',), },);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'rejects a sibling directory outside the managed root',
          fn: async () => {
            let caught: unknown;
            try {
              assertIsManagedVmDir({
                name: 'alpine',
                candidate: join(dirname(vmDir('alpine',),), 'elsewhere',),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('resolves outside',);
          },
        },),

        it({
          name: 'rejects a nested child of the managed root',
          fn: async () => {
            let caught: unknown;
            try {
              assertIsManagedVmDir({
                name: 'alpine',
                candidate: join(vmDir('alpine',), 'images',),
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).message,).toContain('resolves outside',);
          },
        },),
      ],
    },),

    //endregion assertIsManagedVmDir

    //region command entry points: each validates before it reads or writes

    describe({
      name: 'commands deriving a per-VM path',
      children: [
        ...COMMANDS_THAT_DERIVE_A_PATH.map(function mapCommand(command,) {
          return it({
            name: `${command.name} rejects an escaping name`,
            fn: async () => {
              let caught: unknown;
              try {
                await command.run('../evil',);
              }
              catch (error) {
                caught = error;
              }
              expect(caught,).toBeInstanceOf(Error,);
              expect((caught as Error).message,).toContain('invalid VM name',);
            },
          },);
        },),
      ],
    },),

    //endregion command entry points

    //region detectHypervisor: platform-based hypervisor detection

    describe({
      name: detectHypervisor.name,
      children: [
        it({
          name: 'returns kvm on linux',
          fn: async () => {
            // The test environment is Linux, so this should return 'kvm'
            expect(detectHypervisor(),).toBe('kvm',);
          },
        },),
      ],
    },),
    //endregion detectHypervisor
  ],
},);
