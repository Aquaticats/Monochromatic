import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  detectHypervisor,
  stripJsoncComments,
  validateName,
  vmConfigPath,
  vmDir,
} from './config.ts';

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
      ],
    },),

    //endregion vmDir / vmConfigPath

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
