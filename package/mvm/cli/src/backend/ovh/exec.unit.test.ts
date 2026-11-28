/**
 Unit tests for the ovh backend's one-shot run against the OVHcloud stand-in: a
 run that asks for two vCPUs and 4 GiB creates a size that meets both floors,
 prints that size before it creates, runs the command, and destroys the machine
 its own create made; a run that asks for a floor beside a source machine to
 clone is refused before anything is read, because a clone boots its source's
 size.

 Every body here is a stand-in written by hand from OVHcloud's schema of the
 cloud API, never a recording, so nothing in this file claims what OVHcloud
 really answers. No request leaves this process: `installStandIn` owns `fetch`
 for the length of each test. The `ssh` the backend runs is this package's
 stand-in executable, which records the argv it was given.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ovhRun, } from '../../../dist/final/node/index.mjs';
import {
  cloudPath,
  errorFrom,
  failureAnswer,
  flavorAnswer,
  imageAnswer,
  installEnv,
  installStandIn,
  instanceAnswer,
  jsonAnswer,
  matching,
  regionAnswer,
  sshCalls,
  sshKeyAnswer,
  type RecordedCall,
  type StandIn,
} from './fixture.ovh-api-stand-in.ts';

//region Stand-in answers

/**
 How many vCPUs the run under test asks its machine to have.
 */
const ASKED_VCPUS = 2;

/**
 How much memory the run under test asks its machine to have, in gibibytes.
 */
const ASKED_MEMORY_GIB = 4;

/**
 Size the floors select: the smallest in stock that has both, while `d2-2` is
 the smallest in stock and has neither.
 */
const FITTING_FLAVOR = 'c3-4';

/**
 Flavor identifier the create sends for {@link FITTING_FLAVOR}.
 */
const FITTING_FLAVOR_ID = 'flavor-c3';

/**
 Flavor identifier of the smallest size in stock, which the run gets when the
 floors never reach the create.
 */
const SMALLEST_FLAVOR_ID = 'flavor-d2';

/**
 Command words the run hands its guest, as one string the guest's shell reads.
 */
const GUEST_COMMAND = 'grep MemTotal /proc/meminfo; nproc';

/**
 Path of the project's price catalogue read, which this test refuses so the
 size walk falls back to the numbers and picks a size this stand-in can name.
 */
const PRICE_PATH = '/v1/order/catalog/public/cloud?ovhSubsidiary=CA';

/**
 Stand-in for one run: it holds the machine its create made, so the listing a
 lookup reads carries that machine's own name, and drops it once deleted.
 
 The name an ephemeral run picks carries a random suffix, so a table of fixed
 answers cannot answer a lookup by name. This stand-in reads the created
 machine out of the create's body and answers with it.
 
 @returns the stand-in fetch plus the machine it recorded

 @example
 ```ts
 using standIn = installStandIn(runStandIn());
 ```
 */
function runStandIn(): StandIn {
  /**
   What the stand-in's project holds: the machine this run created, while it
   exists, and whether its delete was accepted.
   */
  const held: {
    created?: ReturnType<typeof instanceAnswer>;
    deleted: boolean;
  } = { deleted: false };
  return installStandIn(function answer(call: RecordedCall,): Response {
    /**
     This request, as a method and a path the stand-in routes on.
     */
    const route = `${call.method} ${call.path}`;
    if (route === `GET ${cloudPath('/instance',)}`) {
      return jsonAnswer(
        ((held.created === undefined) || held.deleted) ? [] : [held.created,],
      );
    }
    if (route === `GET ${cloudPath('/sshkey',)}`) {
      return jsonAnswer([],);
    }
    if (route === `POST ${cloudPath('/sshkey',)}`) {
      return jsonAnswer(sshKeyAnswer(),);
    }
    if (route === `GET ${cloudPath('/region',)}`) {
      return jsonAnswer(['bhs5',],);
    }
    if (route === `GET ${cloudPath('/region/bhs5',)}`) {
      return jsonAnswer(regionAnswer(),);
    }
    if (route === `GET ${cloudPath('/flavor',)}?region=bhs5`) {
      return jsonAnswer([
        flavorAnswer({
          disk: 25,
          id: SMALLEST_FLAVOR_ID,
          name: 'd2-2',
          ram: 2,
          vcpus: 1,
        },),
        flavorAnswer({
          disk: 50,
          id: FITTING_FLAVOR_ID,
          name: FITTING_FLAVOR,
          ram: ASKED_MEMORY_GIB,
          vcpus: ASKED_VCPUS,
        },),
        flavorAnswer({
          disk: 50,
          id: 'flavor-b3',
          name: 'b3-8',
          ram: 8,
          vcpus: ASKED_VCPUS,
        },),
      ],);
    }
    if (route === `GET ${cloudPath('/image',)}?region=bhs5`) {
      return jsonAnswer([imageAnswer({},),],);
    }
    if (route === `POST ${cloudPath('/instance',)}`) {
      held.created = instanceAnswer({
        flavorId: (call.body as { readonly flavorId: string; }).flavorId,
        id: 'instance-run',
        name: (call.body as { readonly name: string; }).name,
        status: 'BUILD',
      },);
      return jsonAnswer(held.created,);
    }
    if (route === `GET ${cloudPath('/instance/instance-run',)}`) {
      return jsonAnswer(instanceAnswer({
        flavorId: (held.created?.flavorId ?? ''),
        id: 'instance-run',
        name: (held.created?.name ?? ''),
        status: 'ACTIVE',
      },),);
    }
    if (route === `DELETE ${cloudPath('/instance/instance-run',)}`) {
      held.deleted = true;
      return new Response(null, { status: 204, },);
    }
    if (route === `GET ${PRICE_PATH}`) {
      return failureAnswer({
        errorCode: 'REFUSED',
        message: 'the price catalogue is not readable with these rights',
        status: 403,
      },);
    }
    return failureAnswer({
      message: `the stand-in planned no answer for ${call.method} ${call.path}`,
      status: 500,
    },);
  },);
}

/**
 Everything a console or stream write carried while a run was in flight, in the
 order it was written.
 
 @param stubs - the recording stubs a case installed
 
 @returns one text per write, joined with newlines

 @example
 ```ts
 const printed = printedText(stubs);
 ```
 */
function printedText(
  stubs: readonly { readonly args: readonly unknown[][]; }[],
): string {
  return stubs
    .flatMap(function argLists(stub,) {
      return stub.args;
    },)
    .map(function asText(args,) {
      return args.map(String,)
        .join(' ',);
    },)
    .join('\n',);
}

//endregion Stand-in answers

await describe({
  name: 'ovh one-shot run',
  concurrency: 1,
  children: [
    it({
      name: `a run asking for ${String(ASKED_VCPUS,)} vCPUs and ${String(ASKED_MEMORY_GIB,)} GiB creates ${FITTING_FLAVOR} and prints that size before it creates`,
      fn: async ctx => {
        await using env = await installEnv();
        using standIn = runStandIn();
        /**
         Everything written to a console method or to standard error while the run
         is in flight, which is where a size line reaches a person.
         */
        const info = ctx.sinon.stub(console, 'info',);
        const log = ctx.sinon.stub(console, 'log',);
        const warn = ctx.sinon.stub(console, 'warn',);
        const error = ctx.sinon.stub(console, 'error',);
        const stderr = ctx.sinon.stub(process.stderr, 'write',);
        stderr.returns(true,);

        await ovhRun({
          command: GUEST_COMMAND,
          memoryGiB: ASKED_MEMORY_GIB,
          vcpus: ASKED_VCPUS,
        },);

        /**
         The create request the run made, which carries the size it took.
         */
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        // The floors reached the create: the machine is the smallest size that
        // meets both, not the smallest size in stock.
        expect((created.body as { readonly flavorId: string; }).flavorId,).toBe(FITTING_FLAVOR_ID,);

        /**
         Everything the run printed.
         */
        const printed = printedText([
          info,
          log,
          warn,
          error,
          stderr,
        ],);
        /**
         Index of the line naming the size the walk settled on.
         */
        const sizeLine = printed.indexOf(`fits ${FITTING_FLAVOR} (`);
        /**
         Index of the line naming the create, which names the machine as `mvm-` plus its
         ephemeral name.
         */
        const createLine = printed.indexOf('creating mvm-ephemeral-',);
        expect(sizeLine,).toBeGreaterThan(-1,);
        expect(createLine,).toBeGreaterThan(-1,);
        // The size reaches a person before the create it justifies does.
        expect(sizeLine,).toBeLessThan(createLine,);

        // The one command ran in the machine, as the single string the guest's
        // shell reads.
        const calls = await sshCalls({ dir: env.dir, },);
        const [execCall,] = calls.filter(function carriesCommand(call,) {
          return call.at(-1,) === GUEST_COMMAND;
        },);
        expect(execCall?.at(-2,),).toBe('ubuntu@192.0.2.10',);

        // The machine its own create made was destroyed, so the run leaves nothing billed.
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/instance/instance-run',),
          },).length,
        ).toBe(1,);
      },
    },),
    it({
      name: 'a run naming a source machine and a floor is refused before anything is read',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(function refuseEverything(call: RecordedCall,) {
          return failureAnswer({
            message: `a refused run read ${call.method} ${call.path}`,
            status: 500,
          },);
        },);
        const error = await errorFrom({
          run: () => ovhRun({
            command: GUEST_COMMAND,
            from: 'dev-01',
            vcpus: ASKED_VCPUS,
          },),
        },);
        /**
         What the refusal says: the clone keeps the source's size, and which floors
         could not be honoured.
         */
        const message = caughtValueText(error,);
        expect(message,).toContain('at least 2 vCPUs',);
        expect(message,).toContain('keeps');
        expect(standIn.calls,).toHaveLength(0,);
      },
    },),
  ],
},);
