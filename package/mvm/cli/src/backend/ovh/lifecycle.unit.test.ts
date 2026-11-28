/**
 Unit tests for the ovh backend's lifecycle against a stand-in HTTP layer: a
 create never asks for monthly billing, waits for SSH as the account the image
 names, and deletes what it made when it cannot finish; a sweep touches only the
 machines whose name carries the tool's prefix; and a clone deletes its snapshot
 once the new machine runs.

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
  sshCalls,
  sshKeyAnswer,
  type RecordedCall,
} from './fixture.ovh-api-stand-in.ts';

//region Routes

/**
 Stand-in routes: the answer each `METHOD path` gets, in order, with a refusal
 for anything the test did not plan.
 */
type Routes = Readonly<Record<string, readonly Response[]>>;

/**
 Answers requests from a table of planned answers, one queue per method and path.
 
 @param routes - queued answers keyed by `METHOD path`
 
 @returns a handler that fails loudly on an unplanned request
 
 @example
 ```ts
 installStandIn(routed({ 'GET /v1/cloud/project/x/instance': [jsonAnswer([])] }));
 ```
 */
function routed(routes: Routes,): (call: RecordedCall,) => Response {
  /**
   Answers not yet handed out, per method and path.
   */
  const queues = new Map<string, Response[]>();
  for (const [key, answers,] of Object.entries(routes,)) {
    queues.set(
      key,
      [...answers,],
    );
  }
  return function answer(call,) {
    /**
     Answers still queued for this request.
     */
    const queue = queues.get(`${call.method} ${call.path}`,);
    /**
     Next planned answer, if the test planned one.
     */
    const next = queue?.shift();
    return next ?? failureAnswer({
      message: `the stand-in planned no answer for ${call.method} ${call.path}`,
      status: 500,
    },);
  };
}

/**
 The answers a successful create needs, from the key upload to the instance
 reporting itself running.
 */
function createRoutes(): Routes {
  return {
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
    [`GET ${cloudPath('/instance',)}`]: [jsonAnswer([
      instanceAnswer({
        id: 'instance-dev',
        name: 'mvm-dev',
      },),
    ],),],
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
        const order = standIn.calls.map(function describe(call,) {
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
        const order = standIn.calls.map(function describe(call,) {
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
  ],
},);
