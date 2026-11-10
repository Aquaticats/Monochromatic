import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  detectHypervisor,
  isVmNameChar,
  parseVmsyncConfig,
  validateName,
  vmConfigPath,
  vmDir,
} from '../dist/final/node/index.mjs';

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

    //region parseVmsyncConfig: the shared JSONC parser reads the config file

    describe({
      name: parseVmsyncConfig.name,
      children: [
        it({
          name: 'parses a config holding no comments',
          fn: async () => {
            const config = parseVmsyncConfig({
              source: '{ "name": "alpine", "boot": { "memory": "4G", "cpus": 4 } }',
            },);
            expect(config.name,).toBe('alpine',);
            expect(config.boot,).toEqual({
              memory: '4G',
              cpus: 4,
            },);
          },
        },),

        it({
          name: 'parses a config holding line and block comments',
          fn: async () => {
            const source = `{
  // VM identity
  "name": "alpine",
  /* boot settings */
  "boot": { "memory": "4G", "cpus": 4 }
}`;
            const config = parseVmsyncConfig({ source, },);
            expect(config.name,).toBe('alpine',);
            expect(config.boot
              .cpus,).toBe(4,);
          },
        },),

        it({
          name: 'accepts the trailing comma the README example carries',
          fn: async () => {
            const source = '{\n  "name": "alpine",\n  "boot": { "memory": "4G", "cpus": 4 },\n}';
            const config = parseVmsyncConfig({ source, },);
            expect(config.boot
              .memory,).toBe('4G',);
          },
        },),

        it({
          name: 'keeps a value whose text looks like a comment',
          fn: async () => {
            const config = parseVmsyncConfig({
              source: '{ "importedFrom": "https://example.com/a.qcow2", "name": "a /* b */ c" }',
            },);
            expect(config.importedFrom,).toBe('https://example.com/a.qcow2',);
            expect(config.name,).toBe('a /* b */ c',);
          },
        },),

        it({
          name: 'refuses text that is not a JSONC object, naming the fault offset',
          fn: async () => {
            let caught: unknown;
            try {
              parseVmsyncConfig({ source: '{ "name": "alpine" "boot": {} }', },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(Error,);
            expect((caught as Error).name,).toBe('JsoncParseError',);
            expect((caught as Error).message,).toContain('at offset',);
          },
        },),

        it({
          name: 'refuses a top-level scalar',
          fn: async () => {
            let caught: unknown;
            try {
              parseVmsyncConfig({ source: '"alpine"', },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).name,).toBe('JsoncParseError',);
          },
        },),
      ],
    },),


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
