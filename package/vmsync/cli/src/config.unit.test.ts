import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  detectHypervisor,
  InvalidVmNameError,
  isVmNameChar,
  parseVmsyncConfig,
  validateName,
  type VmNameOrigin,
  vmConfigPath,
  vmDir,
} from '../dist/final/node/index.mjs';

//region validateName: accepts names that follow the rule and explains the ones it refuses

/**
 Message a refusal carries, so the assertions read the explanation a user sees.
 
 @param name - candidate VM name
 
 @param origin - where the name came from, defaulting to the user typing it
 
 @throws Error when the name was accepted, which the tests must not see
 
 @example
 ```ts
 refusalMessage('_alpine'); // 'vmsync refused the VM name "_alpine". ...'
 ```
 */
function refusalMessage(
  name: string,
  origin: VmNameOrigin = { kind: 'typed', },
): string {
  try {
    validateName({
      name,
      origin,
    },);
  }
  catch (error) {
    if (error instanceof InvalidVmNameError)
      return error.message;
    throw error;
  }
  throw new Error(`validateName accepted ${JSON.stringify(name,)}`,);
}

await describe({
  name: '',
  children: [
    describe({
      name: validateName.name,
      children: [
        //region Accepted names

        it({
          name: 'accepts simple alphanumeric name',
          fn: async () => {
            expect(() => {
              validateName({
                name: 'alpine',
                origin: { kind: 'typed', },
              },);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'accepts name with hyphens and underscores',
          fn: async () => {
            expect(() => {
              validateName({
                name: 'my-vm_01',
                origin: { kind: 'typed', },
              },);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'accepts single character name',
          fn: async () => {
            expect(() => {
              validateName({
                name: 'a',
                origin: { kind: 'typed', },
              },);
            },)
              .not
              .toThrow();
          },
        },),

        it({
          name: 'accepts name starting with digit',
          fn: async () => {
            expect(() => {
              validateName({
                name: '9test',
                origin: { kind: 'typed', },
              },);
            },)
              .not
              .toThrow();
          },
        },),

        //endregion Accepted names

        //region Refused names

        ...['-bad', '_bad', '../evil', 'my vm', 'my.vm', '', 'a/b',].map(
          function mapRefused(name,) {
            return it({
              name: `refuses ${JSON.stringify(name,)}`,
              fn: async () => {
                expect(() => {
                  validateName({
                    name,
                    origin: { kind: 'typed', },
                  },);
                },)
                  .toThrow(InvalidVmNameError,);
              },
            },);
          },
        ),

        //endregion Refused names

        //region The explanation a refusal carries

        it({
          name: 'quotes the refused name and states the rule',
          fn: async () => {
            const message = refusalMessage('_alpine',);
            expect(message,)
              .toContain('vmsync refused the VM name "_alpine"',);
            expect(message,)
              .toContain('A VM name starts with a letter or a digit',);
          },
        },),

        it({
          name: 'names the character that a leading position may not hold',
          fn: async () => {
            expect(refusalMessage('_alpine',),)
              .toContain('This name starts with "_", which may not start a name.',);
          },
        },),

        it({
          name: 'names and spells out a space that a later position may not hold',
          fn: async () => {
            const message = refusalMessage('my vm',);
            expect(message,)
              .toContain('This name holds " " (a space), which a name may not hold.',);
          },
        },),

        it({
          name: 'says an empty name is empty',
          fn: async () => {
            expect(refusalMessage('',),)
              .toContain('This name is empty.',);
          },
        },),

        it({
          name: 'says the letters and digits are ASCII, so a non-ASCII letter is no contradiction',
          fn: async () => {
            const message = refusalMessage('café',);
            expect(message,)
              .toContain('This name holds "é", which a name may not hold.',);
            expect(message,)
              .toContain('Letters and digits here are ASCII: A to Z, a to z and 0 to 9.',);
          },
        },),

        it({
          name: 'asks for a different name when the user typed it',
          fn: async () => {
            const message = refusalMessage('_alpine',);
            expect(message,)
              .toContain('Choose a different name.',);
            expect(message,)
              .not
              .toContain('--name',);
          },
        },),

        it({
          name: 'names the image file a derived name came from and the --name override',
          fn: async () => {
            const message = refusalMessage('_alpine', {
              kind: 'imagePath',
              imagePath: '/images/_alpine.qcow2',
            },);
            expect(message,)
              .toContain('vmsync took this name from the image file "/images/_alpine.qcow2"',);
            expect(message,)
              .toContain('pass --name to choose the name yourself.',);
          },
        },),

        it({
          name: 'carries the refused name on the error beside its message',
          fn: async () => {
            let caught: unknown;
            try {
              validateName({
                name: '_alpine',
                origin: { kind: 'typed', },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeInstanceOf(InvalidVmNameError,);
            expect((caught as InvalidVmNameError).vmName,).toBe('_alpine',);
          },
        },),

        //endregion The explanation a refusal carries
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
          name: 'refuses a trailing block comment that is never closed',
          fn: async () => {
            let caught: unknown;
            try {
              parseVmsyncConfig({ source: '{ "name": "alpine" } /* never closed', },);
            }
            catch (error) {
              caught = error;
            }
            expect((caught as Error).name,).toBe('JsoncParseError',);
            expect((caught as Error).message,)
              .toContain('unterminated block comment',);
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
