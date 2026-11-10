import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type CommandRunner,
  undefineVmIfExists,
} from '../dist/final/node/domain-removal.mjs';

//region Fixtures

/**
 Domain name every case removes; spelled once so a case names the behaviour it checks, not the fixture.
 */
const DOMAIN_NAME = 'fixture-domain';

/**
 Connection URI every case passes; the session connection the entry scripts use.
 */
const LIBVIRT_URI = 'qemu:///session';

/**
 Domain states `virsh domstate` prints for a domain no QEMU process is left for, where `virsh destroy` fails with
 "domain is not running".

 Listed in the order of the `virshDomainState` enum of `tools/virsh-domain-monitor.c` in libvirt, so a state that moves
 there shows up here beside its neighbours.
 */
const INACTIVE_STATES = ['shut off', 'crashed', 'no state',];

/**
 Answers the fake hypervisor gives to the calls the module makes.
 */
type FakeHypervisor = {
  /**
   Whether `virsh dominfo` finds the domain.
   */
  readonly defined: boolean;
  /**
   What `virsh domstate` prints; read only when {@link FakeHypervisor.defined} is true.
   */
  readonly state?: string;
};

/**
 Hypervisor a fake runner records calls against.
 */
type FakeVirsh = {
  /**
   Arguments of every call the module made, in order.
   */
  readonly calls: readonly (readonly string[])[];
  /**
   Runner to hand {@link undefineVmIfExists} in place of the real one.
   */
  readonly runner: CommandRunner;
  /**
   Subcommands of {@link FakeVirsh.calls}, in order, read from the third argument after the connection URI.
   
   @returns One subcommand per recorded call
   */
  readonly subcommands: () => string[];
};

/**
 Builds a command runner that answers like `virsh` against a hypervisor in a scripted state and records every call.

 @param hypervisor - domain state the fake answers from

 @returns recorded calls, the runner, and the subcommands it was asked for

 @example
 ```ts
 const fake = fakeVirsh({ defined: true, state: 'paused', },);
 await undefineVmIfExists({ name: DOMAIN_NAME, uri: LIBVIRT_URI, runner: fake.runner, },);
 expect(fake.subcommands(),).toEqual(['dominfo', 'domstate', 'undefine',]);
 ```
 */
function fakeVirsh(hypervisor: FakeHypervisor,): FakeVirsh {
  /**
   Arguments of every call, appended as the module makes them.
   */
  const calls: string[][] = [];
  const runner: CommandRunner = async ({ args = [], },) => {
    /**
     Arguments of the current call, kept so a case can assert them whole.
     */
    const call = [...args,];
    calls.push(call,);
    /**
     Subcommand of the current call; `--connect <uri>` occupies the first two arguments.
     */
    const subcommand = call.at(2,);
    /**
     Whether the module asked whether the domain is defined.
     */
    const askedForDefinition = subcommand === 'dominfo';
    if (askedForDefinition && (!hypervisor.defined)) {
      throw new Error(`failed to get domain '${String(call.at(3,),)}'`,);
    }
    if (subcommand === 'domstate') {
      return `${hypervisor.state ?? ''}\n`;
    }
    return '';
  };
  return {
    calls,
    runner,
    subcommands: () => calls.map((call,) => call.at(2,) ?? '',),
  };
}

/**
 Awaits a promise and returns its rejection reason so assertions can read error properties;
 resolves to `undefined` when the promise fulfils.

 @param promise - Operation expected to reject

 @returns The rejection reason, or `undefined` when the operation succeeded
 */
async function rejectionOf(promise: Promise<unknown>,): Promise<unknown> {
  try {
    await promise;
  }
  catch (error) {
    return error;
  }
  return undefined;
}

//endregion Fixtures

/**
 Removes the fixture domain against a fake hypervisor in the given state.

 @param fake - fake hypervisor to run the removal against

 @returns Promise that fulfils when the removal finished
 */
async function removeFixtureDomain(fake: FakeVirsh,): Promise<void> {
  return await undefineVmIfExists({
    name: DOMAIN_NAME,
    uri: LIBVIRT_URI,
    runner: fake.runner,
  },);
}

await describe({
  name: 'removing an existing libvirt domain',
  children: [
    describe({
      name: undefineVmIfExists.name,
      children: [
        //region The domain itself

        it({
          name: 'removes nothing when the domain is not defined',
          fn: async () => {
            /**
             Fake hypervisor that has no domain of this name.
             */
            const fake = fakeVirsh({ defined: false, },);
            await removeFixtureDomain(fake,);
            expect(fake.subcommands(),).toEqual(['dominfo',],);
          },
        },),
        it({
          name: 'probes the domain and its state through the given connection',
          fn: async () => {
            /**
             Fake hypervisor holding a shut-off domain.
             */
            const fake = fakeVirsh({
              defined: true,
              state: 'shut off',
            },);
            await removeFixtureDomain(fake,);
            expect(fake.calls.at(0,),).toEqual([
              '--connect',
              LIBVIRT_URI,
              'dominfo',
              DOMAIN_NAME,
            ],);
            expect(fake.calls.at(1,),).toEqual([
              '--connect',
              LIBVIRT_URI,
              'domstate',
              DOMAIN_NAME,
            ],);
          },
        },),
        it({
          name: 'undefines with --nvram so an EFI domain loses its variable store too',
          fn: async () => {
            /**
             Fake hypervisor holding a shut-off domain.
             */
            const fake = fakeVirsh({
              defined: true,
              state: 'shut off',
            },);
            await removeFixtureDomain(fake,);
            expect(fake.calls.at(-1,),).toEqual([
              '--connect',
              LIBVIRT_URI,
              'undefine',
              DOMAIN_NAME,
              '--nvram',
            ],);
          },
        },),

        //endregion The domain itself

        //region Stopping the domain first

        it({
          name: 'destroys a running domain before undefining it',
          fn: async () => {
            /**
             Fake hypervisor holding a running domain.
             */
            const fake = fakeVirsh({
              defined: true,
              state: 'running',
            },);
            await removeFixtureDomain(fake,);
            expect(fake.subcommands(),).toEqual([
              'dominfo',
              'domstate',
              'destroy',
              'undefine',
            ],);
          },
        },),
        ...INACTIVE_STATES.map((state,) => it({
          name: `undefines a domain in state '${state}' without destroying it`,
          fn: async () => {
            /**
             Fake hypervisor holding a domain in the state under test.
             */
            const fake = fakeVirsh({
              defined: true,
              state,
            },);
            await removeFixtureDomain(fake,);
            expect(fake.subcommands(),).toEqual([
              'dominfo',
              'domstate',
              'undefine',
            ],);
          },
        },)),
        it({
          name: 'reads the state past the surrounding whitespace `virsh` prints',
          fn: async () => {
            /**
             Fake hypervisor whose `domstate` output is padded, as a terminal copy would be.
             */
            const fake = fakeVirsh({
              defined: true,
              state: '  running  ',
            },);
            await removeFixtureDomain(fake,);
            expect(fake.subcommands(),).toContain('destroy',);
          },
        },),

        //endregion Stopping the domain first
      ],
    },),
  ],
},);
