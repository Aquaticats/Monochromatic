/**
 Unit tests for the ovh backend's lifecycle against a stand-in HTTP layer: a
 create never asks for monthly billing, waits for SSH as the account the image
 names, and deletes what it made when it cannot finish; a create or a clone
 refuses a name the project is still deleting; exec refuses a machine that is
 being deleted; a destroy returns only once the project's listing has let its
 machine go, and a sweep waits for every machine it deleted; and a clone deletes
 its snapshot once the new machine runs.

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

import {
  ovhClone,
  ovhCreate,
  ovhDestroy,
  ovhDestroyAll,
  ovhExec,
  ovhList,
} from '../../../dist/final/node/index.mjs';
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
  routedAnswers as routed,
  sshCalls,
  sshKeyAnswer,
} from './fixture.ovh-api-stand-in.ts';

//region Stand-in answers

/**
 Answers a test plans, keyed by `METHOD path`, each path holding its answers in
 the order they are handed out.
 */
type Routes = Readonly<Record<string, readonly Response[]>>;

/**
 The answers a successful create needs, from the key upload to the instance
 reporting itself running.
 */
function createRoutes(): Routes {
  return {
    [`GET ${cloudPath('/instance',)}`]: [jsonAnswer([],),],
    [`GET ${cloudPath('/sshkey',)}`]: [jsonAnswer([],),],
    [`POST ${cloudPath('/sshkey',)}`]: [jsonAnswer(sshKeyAnswer(),),],
    [`GET ${cloudPath('/region',)}`]: [jsonAnswer(['bhs5',],),],
    [`GET ${cloudPath('/region/bhs5',)}`]: [jsonAnswer(regionAnswer(),),],
    [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([flavorAnswer(),],),],
    [`GET ${cloudPath('/image',)}?region=bhs5`]: [jsonAnswer([imageAnswer(),],),],
    [`POST ${cloudPath('/instance',)}`]: [jsonAnswer(instanceAnswer({ status: 'BUILD', },),),],
    [`GET ${cloudPath('/instance/instance-1',)}`]: [
      jsonAnswer(instanceAnswer({ status: 'ACTIVE', },),),
    ],
    [`DELETE ${cloudPath('/instance/instance-1',)}`]: [new Response(null, { status: 204, },),],
  };
}

/**
 The answers a clone needs, from the source instance to the clone reporting
 itself running.
 */
function cloneRoutes(): Routes {
  return {
    [`GET ${cloudPath('/instance',)}`]: [
      jsonAnswer([
        instanceAnswer({
          id: 'instance-dev',
          name: 'mvm-dev',
        },),
      ],),
      jsonAnswer([
        instanceAnswer({
          id: 'instance-dev',
          name: 'mvm-dev',
        },),
      ],),
    ],
    [`POST ${cloudPath('/instance/instance-dev/snapshot',)}`]: [jsonAnswer({ snapshotId: 'snapshot-1', },),],
    [`GET ${cloudPath('/image/snapshot-1',)}`]: [jsonAnswer(imageAnswer({
      id: 'snapshot-1',
      name: 'mvm-clone-dev-dev-2',
      visibility: 'private',
    },),),],
    [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([flavorAnswer(),],),],
    [`GET ${cloudPath('/sshkey',)}`]: [jsonAnswer([],),],
    [`POST ${cloudPath('/sshkey',)}`]: [jsonAnswer(sshKeyAnswer(),),],
    [`POST ${cloudPath('/instance',)}`]: [jsonAnswer(instanceAnswer({
      id: 'instance-clone',
      name: 'mvm-dev-2',
    },),),],
    [`GET ${cloudPath('/instance/instance-clone',)}`]: [jsonAnswer(instanceAnswer({
      id: 'instance-clone',
      name: 'mvm-dev-2',
      status: 'ACTIVE',
    },),),],
    [`DELETE ${cloudPath('/snapshot/snapshot-1',)}`]: [new Response(null, { status: 204, },),],
    [`DELETE ${cloudPath('/instance/instance-clone',)}`]: [new Response(null, { status: 204, },),],
  };
}

//endregion Routes

//region Priced catalogue stand-ins

/**
 The address line every machine carries, as the live CA answer of 2026-11-28
 states it: one metered entry whose `intervalUnit` is `none`, and its hour in
 the commercial blob.
 */
const ADDRESS_ADDON = {
  blobs: { commercial: { price: { interval: 'PT1H', }, }, },
  planCode: 'publicip.ip.hour.consumption',
  pricings: [{ interval: 0, intervalUnit: 'none', price: 432_000, type: 'consumption', },],
  product: 'publiccloud-publicip-ip',
};

/**
 The Gen 3 storage line, as that answer states it: 23330 per GB-hour, whose GB
 the instance plan states separately.
 */
const GEN3_STORAGE_ADDON = {
  planCode: 'instance.local-storage-gen3-gb.hour.consumption',
  pricings: [{ interval: 1, intervalUnit: 'hour', price: 23_330, type: 'consumption', },],
  product: 'publiccloud-local-storage-gen3-gb',
};

/**
 The storage line the earlier ranges name, which that answer prices at zero
 because their local storage is included in the row.
 */
const FREE_STORAGE_ADDON = {
  planCode: 'instance.local-storage-free-gb.hour.consumption',
  pricings: [{ interval: 1, intervalUnit: 'hour', price: 0, type: 'consumption', },],
  product: 'publiccloud-local-storage-free-gb',
};

/**
 One instance plan as that answer states an instance plan: its own row, the
 family naming the addon that bills its storage, and the disks its technical
 blob lists.
 
 This is a stand-in: its numbers are the ones each call passes, not live ones.
 */
function planAddon({
  diskGb,
  planCode,
  rowPrice,
  storagePlanCode,
}: {
  readonly diskGb: number;
  readonly planCode: string;
  readonly rowPrice: number;
  readonly storagePlanCode: string;
},) {
  return {
    addonFamilies: [{ addons: [storagePlanCode,], name: 'local-storage', },],
    blobs: { technical: { storage: { disks: [{ capacity: diskGb, number: 1, },], }, }, },
    planCode,
    pricings: [{ interval: 1, intervalUnit: 'hour', price: rowPrice, type: 'consumption', },],
    product: 'publiccloud-instance',
  };
}

/**
 A catalogue answer holding the answer's own three side lines and the plans a
 test prices.
 
 @param addons - the plans and side lines this answer carries
 
 @returns the body the price read parses
 
 @example
 ```ts
 priceCatalog([ADDRESS_ADDON, FREE_STORAGE_ADDON, planAddon({ ... })]);
 ```
 */
function priceCatalog(addons: readonly unknown[],) {
  return {
    addons,
    locale: { currencyCode: 'CAD', },
  };
}

//endregion Priced catalogue stand-ins

await describe({
  name: 'ovh lifecycle',
  concurrency: 1,
  children: [
    //region Create

    it({
      name: 'a create asks for hourly billing, never monthly, and builds the body the schema names',
      fn: async () => {
        await using env = await installEnv();
        using standIn = installStandIn(
          routed(createRoutes(),),
        );
        await ovhCreate({ name: 'dev', },);
        const created = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        expect(created.length,).toBe(1,);
        const body = created[0]?.body;
        expect(body,).toEqual({
          flavorId: 'flavor-small',
          imageId: 'image-ubuntu',
          monthlyBilling: false,
          name: 'mvm-dev',
          region: 'bhs5',
          sshKeyId: 'key-1',
        },);
        expect(JSON.stringify(body,),).not.toContain('"monthlyBilling":true',);
        expect(await sshCalls({ dir: env.dir, },),).toHaveLength(1,);
      },
    },),
    it({
      name: 'a create connects as the account the image names, at the public IPv4',
      fn: async () => {
        await using env = await installEnv();
        using _standIn = installStandIn(
          routed(createRoutes(),),
        );
        await ovhCreate({ name: 'dev', },);
        const [probe,] = await sshCalls({ dir: env.dir, },);
        expect(probe?.at(-2,),).toBe('ubuntu@192.0.2.10',);
        expect(probe?.at(-1,),).toBe('true',);
      },
    },),
    it({
      name: 'a create deletes the instance when it never becomes reachable, and says nothing is billed',
      fn: async () => {
        await using _env = await installEnv({ sshCommand: ['/no/such/ssh-program',], },);
        using standIn = installStandIn(
          routed(createRoutes(),),
        );
        const error = await errorFrom({ run: () => ovhCreate({ name: 'dev', },) },);
        expect(caughtValueText(error,),).toContain('nothing is left billed',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/instance/instance-1',),
          },).length,
        ).toBe(1,);
      },
    },),
    it({
      name: 'a create whose instance reaches a failed status deletes it before reporting',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/instance/instance-1',)}`]: [jsonAnswer(instanceAnswer({ status: 'ERROR', },),),],
        },),);
        const error = await errorFrom({ run: () => ovhCreate({ name: 'dev', },) },);
        expect(caughtValueText(error,),).toContain('ERROR',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/instance/instance-1',),
          },).length,
        ).toBe(1,);
      },
    },),
    it({
      name: 'a create whose delete fails names the one command that removes the instance',
      fn: async () => {
        await using _env = await installEnv({ sshCommand: ['/no/such/ssh-program',], },);
        using _standIn = installStandIn(routed({
          ...createRoutes(),
          [`DELETE ${cloudPath('/instance/instance-1',)}`]: [failureAnswer({
            message: 'the instance is locked by a pending operation',
            status: 409,
          },),],
        },),);
        const error = await errorFrom({ run: () => ovhCreate({ name: 'dev', },) },);
        const message = caughtValueText(error,);
        expect(message,).toContain('keeps being billed',);
        expect(message,).toContain('mvm --backend ovh destroy dev',);
        expect(message,).toContain('the instance is locked by a pending operation',);
      },
    },),
    it({
      name: 'a create refuses a name the project is still deleting',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/instance',)}`]: [jsonAnswer([
            instanceAnswer({
              id: 'instance-dev',
              name: 'mvm-dev',
              status: 'DELETING',
            },),
          ],),],
        },),);
        const error = await errorFrom({ run: () => ovhCreate({ name: 'dev', },) },);
        /**
         What the refusal says, which has to name the status, the bill and the command that finishes the delete.
         */
        const message = caughtValueText(error,);
        expect(message,).toContain('DELETING',);
        expect(message,).toContain('bills an instance until it is deleted',);
        expect(message,).toContain('mvm --backend ovh destroy dev',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'POST',
            path: cloudPath('/instance',),
          },).length,
        ).toBe(0,);
      },
    },),
    it({
      name: 'a create walks past a region with nothing in stock and creates in the next one',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/region',)}`]: [jsonAnswer(['bhs5', 'gra9',],),],
          [`GET ${cloudPath('/region/gra9',)}`]: [jsonAnswer(regionAnswer({ name: 'gra9', },),),],
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([flavorAnswer({ available: false, },),],),],
          [`GET ${cloudPath('/flavor',)}?region=gra9`]: [jsonAnswer([flavorAnswer({ region: 'gra9', },),],),],
          [`GET ${cloudPath('/image',)}?region=gra9`]: [jsonAnswer([imageAnswer({},),],),],
          [`POST ${cloudPath('/instance',)}`]: [jsonAnswer(instanceAnswer({
            region: 'gra9',
            status: 'BUILD',
          },),),],
        },),);
        await ovhCreate({ name: 'dev', },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        /**
         Body the create sent, as the schema names its fields.
         */
        const body = created.body as { readonly region: string; };
        expect(body.region,).toBe('gra9',);
      },
    },),

    //region The size a create asks for

    it({
      name: 'a create with two floors sends the smallest general-purpose size that meets both',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 25,
              id: 'd2-2',
              ram: 2,
              vcpus: 1,
            },),
            flavorAnswer({
              disk: 50,
              id: 'c3-4',
              ram: 4,
              vcpus: 2,
            },),
            flavorAnswer({
              disk: 50,
              id: 'b3-8',
              ram: 8,
              vcpus: 2,
            },),
          ],),],
        },),);
        await ovhCreate({
          memoryGiB: 4,
          name: 'dev',
          vcpus: 2,
        },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        /**
         Body the create sent, as the schema names its fields.
         */
        const body = created.body as { readonly flavorId: string; };
        expect(body.flavorId,).toBe('c3-4',);
      },
    },),
    it({
      name: 'a create whose floors only a GPU size meets refuses and creates nothing',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 25,
              id: 'd2-2',
              ram: 2,
              vcpus: 1,
            },),
            flavorAnswer({
              disk: 25,
              id: 't1-45',
              ram: 45,
              vcpus: 8,
            },),
          ],),],
        },),);
        const error = await errorFrom({
          run: () => ovhCreate({
            memoryGiB: 40,
            name: 'dev',
            vcpus: 8,
          },),
        },);
        expect((error as Error).message,).toContain('t1-45',);
        expect((error as Error).message,).toContain('left out as a family mvm does not pick without being told',);
        expect((error as Error).message,).toContain('--server-type',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'POST',
            path: cloudPath('/instance',),
          },),
        ).toHaveLength(0,);
      },
    },),
    it({
      name: 'a create that names a flavor outright and also sets a floor refuses before reading anything',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(
          routed(createRoutes(),),
        );
        const error = await errorFrom({
          run: () => ovhCreate({
            name: 'dev',
            serverType: 'b3-8',
            vcpus: 2,
          },),
        },);
        expect((error as Error).message,).toContain('b3-8',);
        expect((error as Error).message,).toContain('at least 2 vCPUs',);
        expect(standIn.calls,).toHaveLength(0,);
      },
    },),
    it({
      name: 'a create walks past a region with no size that fits and asks the next region for the same floors',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/region',)}`]: [jsonAnswer(['bhs5', 'gra9',],),],
          [`GET ${cloudPath('/region/gra9',)}`]: [jsonAnswer(regionAnswer({ name: 'gra9', },),),],
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 25,
              id: 'd2-2',
              ram: 2,
              vcpus: 1,
            },),
          ],),],
          [`GET ${cloudPath('/flavor',)}?region=gra9`]: [jsonAnswer([
            flavorAnswer({
              disk: 50,
              id: 'b3-8',
              ram: 8,
              region: 'gra9',
              vcpus: 2,
            },),
          ],),],
          [`GET ${cloudPath('/image',)}?region=gra9`]: [jsonAnswer([imageAnswer({},),],),],
          [`POST ${cloudPath('/instance',)}`]: [jsonAnswer(instanceAnswer({
            region: 'gra9',
            status: 'BUILD',
          },),),],
        },),);
        await ovhCreate({
          memoryGiB: 4,
          name: 'dev',
          vcpus: 2,
        },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        const body = created?.body as {
          readonly flavorId: string;
          readonly region: string;
        };
        expect(body.region,).toBe('gra9',);
        expect(body.flavorId,).toBe('b3-8',);
      },
    },),

    it({
      name: 'a create with the prices in hand sends the cheapest size that fits, not the smallest numbers',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 50,
              hourlyPlanCode: 'b2-7.consumption',
              id: 'b2-7',
              ram: 7,
              vcpus: 2,
            },),
            flavorAnswer({
              disk: 50,
              hourlyPlanCode: 'd2-8.consumption',
              id: 'd2-8',
              ram: 8,
              vcpus: 4,
            },),
          ],),],
          'GET /v1/order/catalog/public/cloud?ovhSubsidiary=CA': [jsonAnswer(priceCatalog([
            ADDRESS_ADDON,
            FREE_STORAGE_ADDON,
            planAddon({
              diskGb: 50,
              planCode: 'b2-7.consumption',
              rowPrice: 11_350_000,
              storagePlanCode: 'instance.local-storage-free-gb.hour.consumption',
            },),
            planAddon({
              diskGb: 50,
              planCode: 'd2-8.consumption',
              rowPrice: 5_960_000,
              storagePlanCode: 'instance.local-storage-free-gb.hour.consumption',
            },),
          ],),),],
        },),);
        await ovhCreate({
          memoryGiB: 6,
          name: 'dev',
          vcpus: 2,
        },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        /**
         Body the create sent, as the schema names its fields.
         */
        const body = created.body as { readonly flavorId: string; };
        expect(body.flavorId,).toBe('d2-8',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'GET',
            path: '/v1/order/catalog/public/cloud?ovhSubsidiary=CA',
          },).length,
        ).toBe(1,);
      },
    },),
    it({
      name: 'a create ranks and prints the whole hour, so a Gen 3 size cheaper by its row loses once its storage is counted',
      fn: async ctx => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 50,
              hourlyPlanCode: 'b2-7.consumption',
              id: 'b2-7',
              ram: 7,
              vcpus: 2,
            },),
            flavorAnswer({
              disk: 400,
              hourlyPlanCode: 'b3-8.consumption',
              id: 'b3-8',
              ram: 8,
              vcpus: 2,
            },),
          ],),],
          // The rows turn the other way: b3-8's row is 0.09 against b2-7's
          // 0.1135, and its 400 GB of local storage add 0.09332, which puts its
          // whole hour at 0.18764. The rows are stand-ins chosen to make the
          // order turn; the live grid changes no pick (issue 6666).
          'GET /v1/order/catalog/public/cloud?ovhSubsidiary=CA': [jsonAnswer(priceCatalog([
            ADDRESS_ADDON,
            GEN3_STORAGE_ADDON,
            FREE_STORAGE_ADDON,
            planAddon({
              diskGb: 50,
              planCode: 'b2-7.consumption',
              rowPrice: 11_350_000,
              storagePlanCode: 'instance.local-storage-free-gb.hour.consumption',
            },),
            planAddon({
              diskGb: 400,
              planCode: 'b3-8.consumption',
              rowPrice: 9_000_000,
              storagePlanCode: 'instance.local-storage-gen3-gb.hour.consumption',
            },),
          ],),),],
        },),);
        /**
         Everything the create printed while it chose.
         */
        const info = ctx.sinon.stub(console, 'info',);
        info.returns(undefined,);
        await ovhCreate({
          memoryGiB: 6,
          name: 'dev',
          vcpus: 2,
        },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        expect((created.body as { readonly flavorId: string; }).flavorId,).toBe('b2-7',);
        // Positive control: the capture can show a line, so a missing figure
        // below means the create did not print it, not a blind probe.
        console.info('capture-probe',);
        /**
         Every line the create printed, as one text.
         */
        const printed = info.args
          .map(function asText(args,) {
            return args.map(String,)
              .join(' ',);
          },)
          .join('\n',);
        expect(printed,).toContain('capture-probe',);
        // The figure is b2-7's whole hour, not its 0.1135 row.
        expect(printed,).toContain('0.11782 CAD per hour',);
        expect(printed,).toContain('public IPv4 address',);
        expect(printed,).toContain('local storage',);
        expect(printed,).toContain('excluding tax',);
      },
    },),
    it({
      name: 'a create never takes a Gen 3 size whose storage line the answer does not price, even when its row is cheapest',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 100,
              hourlyPlanCode: 'c3-8.consumption',
              id: 'c3-8',
              ram: 8,
              vcpus: 4,
            },),
            flavorAnswer({
              disk: 50,
              hourlyPlanCode: 'd2-8.consumption',
              id: 'd2-8',
              ram: 8,
              vcpus: 4,
            },),
          ],),],
          // c3-8's row is 0.03 and d2-8's whole hour is 0.06392, so a read that
          // priced a size by its row alone would take c3-8. Its storage family
          // names a line this answer does not carry, which is a size with no
          // published price at all.
          'GET /v1/order/catalog/public/cloud?ovhSubsidiary=CA': [jsonAnswer(priceCatalog([
            ADDRESS_ADDON,
            FREE_STORAGE_ADDON,
            planAddon({
              diskGb: 100,
              planCode: 'c3-8.consumption',
              rowPrice: 3_000_000,
              storagePlanCode: 'instance.local-storage-gen3-gb.hour.consumption',
            },),
            planAddon({
              diskGb: 50,
              planCode: 'd2-8.consumption',
              rowPrice: 5_960_000,
              storagePlanCode: 'instance.local-storage-free-gb.hour.consumption',
            },),
          ],),),],
        },),);
        await ovhCreate({
          memoryGiB: 8,
          name: 'dev',
          vcpus: 4,
        },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        expect((created.body as { readonly flavorId: string; }).flavorId,).toBe('d2-8',);
      },
    },),
    it({
      name: 'a create whose price read is refused falls back to the smallest numbers and says so',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...createRoutes(),
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 50,
              hourlyPlanCode: 'b2-7.consumption',
              id: 'b2-7',
              ram: 7,
              vcpus: 2,
            },),
            flavorAnswer({
              disk: 50,
              hourlyPlanCode: 'd2-8.consumption',
              id: 'd2-8',
              ram: 8,
              vcpus: 4,
            },),
          ],),],
          'GET /v1/order/catalog/public/cloud?ovhSubsidiary=CA': [failureAnswer({
            errorCode: 'REFUSED',
            message: 'the price catalogue is not readable with these rights',
            status: 403,
          },),],
        },),);
        await ovhCreate({
          memoryGiB: 6,
          name: 'dev',
          vcpus: 2,
        },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        expect((created.body as { readonly flavorId: string; }).flavorId,).toBe('b2-7',);
      },
    },),

    //endregion The size a create asks for

    //endregion Create

    //region Clone

    it({
      name: 'a clone boots the snapshot in the source region and deletes the snapshot once the clone runs',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(
          routed(cloneRoutes(),),
        );
        await ovhClone({
          destination: 'dev-2',
          source: 'dev',
        },);
        const [created,] = matching({
          calls: standIn.calls,
          method: 'POST',
          path: cloudPath('/instance',),
        },);
        if (created === undefined) {
          throw new Error('expected one create call',);
        }
        /**
         Body the clone's create sent, as the schema names its fields.
         */
        const body = created.body as {
          readonly imageId: string;
          readonly name: string;
          readonly region: string;
        };
        expect(body.imageId,).toBe('snapshot-1',);
        expect(body.name,).toBe('mvm-dev-2',);
        expect(body.region,).toBe('bhs5',);
        const deletes = matching({
          calls: standIn.calls,
          method: 'DELETE',
          path: cloudPath('/snapshot/snapshot-1',),
        },);
        expect(deletes.length,).toBe(1,);
        /**
         Position of the snapshot's creation and of its deletion in the call log.
         */
        const order = standIn.calls.map(function method(call,) {
          return `${call.method} ${call.path}`;
        },);
        expect(
          order.indexOf(`DELETE ${cloudPath('/snapshot/snapshot-1',)}`,),
        ).toBeGreaterThan(
          order.indexOf(`POST ${cloudPath('/instance',)}`,),
        );
      },
    },),
    it({
      name: 'a clone whose instance is refused deletes the snapshot and leaves no instance behind',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...cloneRoutes(),
          [`POST ${cloudPath('/instance',)}`]: [failureAnswer({
            message: 'the flavor is not available in this region',
            status: 400,
          },),],
        },),);
        const error = await errorFrom({
          run: () => ovhClone({
            destination: 'dev-2',
            source: 'dev',
          },),
        },);
        expect(caughtValueText(error,),).toContain(
          'the flavor is not available in this region',
        );
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/snapshot/snapshot-1',),
          },).length,
        ).toBe(1,);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/instance/instance-clone',),
          },).length,
        ).toBe(0,);
      },
    },),
    it({
      name: 'a clone that never becomes reachable deletes the clone and its snapshot',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          ...cloneRoutes(),
          [`GET ${cloudPath('/instance/instance-clone',)}`]: [jsonAnswer(instanceAnswer({
            id: 'instance-clone',
            name: 'mvm-dev-2',
            status: 'ERROR',
          },),),],
        },),);
        const error = await errorFrom({
          run: () => ovhClone({
            destination: 'dev-2',
            source: 'dev',
          },),
        },);
        expect(caughtValueText(error,),).toContain('ERROR',);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/instance/instance-clone',),
          },).length,
        ).toBe(1,);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/snapshot/snapshot-1',),
          },).length,
        ).toBe(1,);
      },
    },),

    //endregion Clone

    //region Lookups by name

    it({
      name: 'exec refuses a machine the project is listing as deleting',
      fn: async () => {
        await using env = await installEnv();
        using _standIn = installStandIn(routed({
          [`GET ${cloudPath('/instance',)}`]: [jsonAnswer([
            instanceAnswer({
              id: 'instance-dev',
              name: 'mvm-dev',
              status: 'DELETING',
            },),
          ],),],
        },),);
        const error = await errorFrom({
          run: () => ovhExec({
            command: 'true',
            name: 'dev',
          },),
        },);
        /**
         What the refusal says, which has to name the status and the state it stands for.
         */
        const message = caughtValueText(error,);
        expect(message,).toContain('DELETING',);
        expect(message,).toContain('being deleted',);
        expect(await sshCalls({ dir: env.dir, },),).toHaveLength(0,);
      },
    },),

    //endregion Lookups by name

    //region Destroy

    it({
      name: 'destroy returns only once the project stops listing the machine',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          [`GET ${cloudPath('/instance',)}`]: [
            jsonAnswer([instanceAnswer({
              id: 'instance-dev',
              name: 'mvm-dev',
            },),],),
            jsonAnswer([],),
          ],
          [`DELETE ${cloudPath('/instance/instance-dev',)}`]: [new Response(null, { status: 204, },),],
        },),);
        await ovhDestroy({ name: 'dev', },);
        /**
         Requests in the order they were made, by method and path.
         */
        const order = standIn.calls.map(function methodAndPath(call,) {
          return `${call.method} ${call.path}`;
        },);
        /**
         Where the delete sits, after which the listing has to be read again before destroy may return.
         */
        const deletedAt = order.indexOf(`DELETE ${cloudPath('/instance/instance-dev',)}`,);
        expect(deletedAt,).toBeGreaterThan(-1,);
        expect(
          order.filter(function readAfterDelete(entry, index,) {
            return (index > deletedAt) && (entry === `GET ${cloudPath('/instance',)}`);
          },).length,
        ).toBe(1,);
      },
    },),

    //endregion Destroy

    //region Destroy all

    it({
      name: 'destroy --all deletes only the instances and snapshots whose names carry the tool prefix',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          [`GET ${cloudPath('/instance',)}`]: [
            jsonAnswer([
              instanceAnswer({
                id: 'instance-dev',
                name: 'mvm-dev',
              },),
              instanceAnswer({
                id: 'instance-web',
                name: 'web-1',
              },),
            ],),
            jsonAnswer([],),
          ],
          [`DELETE ${cloudPath('/instance/instance-dev',)}`]: [new Response(null, { status: 204, },),],
          [`DELETE ${cloudPath('/instance/instance-web',)}`]: [new Response(null, { status: 204, },),],
          [`GET ${cloudPath('/snapshot',)}`]: [jsonAnswer([
            imageAnswer({
              id: 'snapshot-mvm',
              name: 'mvm-clone-dev-dev-2',
            },),
            imageAnswer({
              id: 'snapshot-foreign',
              name: 'nightly-backup',
            },),
          ],),],
          [`DELETE ${cloudPath('/snapshot/snapshot-mvm',)}`]: [new Response(null, { status: 204, },),],
          [`DELETE ${cloudPath('/snapshot/snapshot-foreign',)}`]: [new Response(null, { status: 204, },),],
        },),);
        await ovhDestroyAll();
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/instance/instance-dev',),
          },).length,
        ).toBe(1,);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/instance/instance-web',),
          },).length,
        ).toBe(0,);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/snapshot/snapshot-mvm',),
          },).length,
        ).toBe(1,);
        expect(
          matching({
            calls: standIn.calls,
            method: 'DELETE',
            path: cloudPath('/snapshot/snapshot-foreign',),
          },).length,
        ).toBe(0,);
      },
    },),
    it({
      name: 'destroy --all sends every delete before it waits, and waits for every machine',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          [`GET ${cloudPath('/instance',)}`]: [
            jsonAnswer([
              instanceAnswer({
                id: 'instance-dev',
                name: 'mvm-dev',
              },),
              instanceAnswer({
                id: 'instance-dev-2',
                name: 'mvm-dev-2',
              },),
            ],),
            jsonAnswer([],),
          ],
          [`DELETE ${cloudPath('/instance/instance-dev',)}`]: [new Response(null, { status: 204, },),],
          [`DELETE ${cloudPath('/instance/instance-dev-2',)}`]: [new Response(null, { status: 204, },),],
          [`GET ${cloudPath('/snapshot',)}`]: [jsonAnswer([],),],
        },),);
        await ovhDestroyAll();
        /**
         Requests in the order they were made, by method and path.
         */
        const order = standIn.calls.map(function methodAndPath(call,) {
          return `${call.method} ${call.path}`;
        },);
        /**
         Where each machine's delete sits.
         */
        const deletedAt = [
          order.indexOf(`DELETE ${cloudPath('/instance/instance-dev',)}`,),
          order.indexOf(`DELETE ${cloudPath('/instance/instance-dev-2',)}`,),
        ];
        expect(deletedAt.at(0,),).toBeGreaterThan(-1,);
        expect(deletedAt.at(1,),).toBeGreaterThan(-1,);
        /**
         Where the second read of the listing sits, the first one after the deletes being the wait's own.
         */
        const waitedAt = order.indexOf(
          `GET ${cloudPath('/instance',)}`,
          1,
        );
        expect(waitedAt,).toBeGreaterThan(Math.max(...deletedAt,),);
      },
    },),

    //endregion Destroy all
    //region Listing

    it({
      name: 'a listing reports each machine size from its own region flavor list, read once per region',
      fn: async () => {
        await using _env = await installEnv();
        using standIn = installStandIn(routed({
          [`GET ${cloudPath('/instance',)}`]: [jsonAnswer([
            instanceAnswer({
              flavorId: 'flavor-c3',
              id: 'instance-1',
              name: 'mvm-dev',
            },),
            instanceAnswer({
              flavorId: 'flavor-b3',
              id: 'instance-2',
              ip: '192.0.2.11',
              name: 'mvm-build',
            },),
          ],),],
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([
            flavorAnswer({
              disk: 50,
              id: 'flavor-c3',
              name: 'c3-4',
              ram: 4,
              vcpus: 2,
            },),
            flavorAnswer({
              disk: 50,
              id: 'flavor-b3',
              name: 'b3-8',
              ram: 8,
              vcpus: 2,
            },),
          ],),],
        },),);
        const vms = await ovhList();
        expect(vms,).toEqual([
          {
            memoryGiB: 4,
            name: 'dev',
            state: 'active',
            vcpus: 2,
          },
          {
            memoryGiB: 8,
            name: 'build',
            state: 'active',
            vcpus: 2,
          },
        ],);
        expect(
          matching({
            calls: standIn.calls,
            method: 'GET',
            path: `${cloudPath('/flavor',)}?region=bhs5`,
          },).length,
        ).toBe(1,);
      },
    },),
    it({
      name: 'a listing says nothing about a size whose flavor the region no longer lists',
      fn: async () => {
        await using _env = await installEnv();
        using _standIn = installStandIn(routed({
          [`GET ${cloudPath('/instance',)}`]: [jsonAnswer([
            instanceAnswer({
              flavorId: 'flavor-gone',
              id: 'instance-1',
              name: 'mvm-dev',
            },),
          ],),],
          [`GET ${cloudPath('/flavor',)}?region=bhs5`]: [jsonAnswer([flavorAnswer(),],),],
        },),);
        expect(await ovhList(),).toEqual([{
          name: 'dev',
          state: 'active',
        },],);
      },
    },),

    //endregion Listing
  ],
},);
