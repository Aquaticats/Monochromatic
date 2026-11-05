import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

import {
  createTaskInvocation,
  type ResolvedTaskRequest,
  type RunTask,
  TaskDependencyCycleError,
  TaskInvocationClosedError,
} from '../dist/final/node/index.mjs';

//region Observable completion and input identity

/**
 Failure created by the test executor, never by the invocation cache.
 */
class FixtureFailureError extends Error {
  constructor(message: string) {
    super(message);
    this.name = 'FixtureFailureError';
  }
}

await describe({
  name: createTaskInvocation.name,
  children: [
    it({
      name: 'shares concurrent and completed work with omitted or empty inputs',
      fn: async () => {
        const executed: ResolvedTaskRequest[] = [];
        const run = createTaskInvocation({ execute: async ({ request, }) => {
          executed.push(request,);
        }, },);
        const name = 'shared';
        await Promise.all([run({ name, },), run({ name, argv: [], environment: {}, },),],);
        await run({ name, },);
        expect(executed,).toEqual([{ name, argv: [], environment: {}, },],);
      },
    },),
    it({
      name: 'distinguishes names, arguments and environments containing syntax delimiters',
      fn: async () => {
        const requests = [
          { name: 'compile|a', argv: ['b',], environment: {}, },
          { name: 'compile', argv: ['a|b',], environment: {}, },
          { name: 'compile', argv: ['a', 'b',], environment: {}, },
          { name: 'compile', argv: ['../path; $(false)', '"quoted"\nline\\end',], environment: { VALUE: String.raw`\u0027`, }, },
          { name: 'compile', argv: ['../path; $(false)', '"quoted"\nline\\end',], environment: { VALUE: "'", }, },
        ];
        const executed: ResolvedTaskRequest[] = [];
        const run = createTaskInvocation({ execute: async ({ request, }) => {
          executed.push(request,);
        }, },);
        await Promise.all(requests.map(async request => {
          await run(request,);
        }),);
        expect(executed,).toEqual(requests,);
      },
    },),
    it({
      name: 'shares work when environment keys arrive in a different order',
      fn: async () => {
        const environment = { MODE: 'debug', NOTE: 'literal value', };
        const executed: ResolvedTaskRequest[] = [];
        const run = createTaskInvocation({ execute: async ({ request, }) => {
          executed.push(request,);
        }, },);
        const request = { name: 'compile', argv: [], environment, };
        await run(request,);
        await run({ ...request, environment: Object.fromEntries(Object.entries(environment,).toReversed(),), },);
        expect(executed,).toEqual([request,],);
      },
    },),
    it({
      name: 'snapshots caller-owned arguments and environment before starting work',
      fn: async () => {
        const before = 'debug';
        const after = 'release';
        const argv = ['--mode', before,];
        const environment = { MODE: before, };
        const executed: ResolvedTaskRequest[] = [];
        const run = createTaskInvocation({ execute: async ({ request, }) => {
          executed.push(request,);
        }, },);
        const pending = run({ name: 'compile', argv, environment, },);
        argv[argv.length - 1] = after;
        environment.MODE = after;
        await pending;
        expect(executed,).toEqual([{ name: 'compile', argv: ['--mode', before,], environment: { MODE: before, }, },],);
      },
    },),
    it({
      name: 'shares the original failure without retrying the failed task',
      fn: async () => {
        const failure = new FixtureFailureError('executor failure',);
        const calls: string[] = [];
        const run = createTaskInvocation({ execute: async ({ request, }) => {
          calls.push(request.name,);
          throw failure;
        }, },);
        const request = { name: 'broken', };
        const results = await Promise.allSettled([run(request,), run(request,),],);
        for (const result of results) {
          expect(result.status,).toBe('rejected',);
          if (result.status === 'rejected')
            expect(result.reason,).toBe(failure,);
        }
        expect(calls,).toEqual([request.name,],);
      },
    },),
    it({
      name: 'starts fresh work in each independent invocation',
      fn: async () => {
        const calls: string[] = [];
        async function execute({ request, }: { readonly request: ResolvedTaskRequest; },): Promise<void> {
          calls.push(request.name,);
        }
        const invocations = [createTaskInvocation({ execute, },), createTaskInvocation({ execute, },),];
        const request = { name: 'compile', };
        await Promise.all(invocations.map(async run => {
          await run(request,);
        }),);
        expect(calls,).toEqual(invocations.map(() => request.name),);
      },
    },),
    it({
      name: 'rejects a direct dependency cycle with task identities only',
      fn: async () => {
        const secret = 'private-fixture-value';
        const request = { name: 'loop', environment: { SECRET: secret, }, };
        const run = createTaskInvocation({ execute: async ({ runTask, }) => {
          await runTask(request,);
        }, },);
        let caught: unknown;
        try {
          await run(request,);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TaskDependencyCycleError,);
        if (!(caught instanceof TaskDependencyCycleError))
          throw new FixtureFailureError('Expected a dependency-cycle error',);
        expect(caught.message,).toContain(request.name,);
        expect(caught.message,).not.toContain(secret,);
      },
    },),
    it({
      name: 'rejects a cycle between concurrently started branches',
      fn: async () => {
        const dependencies: Readonly<Record<string, string>> = { left: 'right', right: 'left', };
        const run = createTaskInvocation({ execute: async ({ request, runTask, }) => {
          const dependency = dependencies[request.name];
          if (dependency === undefined)
            throw new FixtureFailureError(`Unexpected fixture task ${request.name}`,);
          await runTask({ name: dependency, },);
        }, },);
        const results = await Promise.allSettled(Object.keys(dependencies,).map(async name => {
          await run({ name, },);
        }),);
        for (const result of results) {
          expect(result.status,).toBe('rejected',);
          if (result.status === 'rejected')
            expect(result.reason,).toBeInstanceOf(TaskDependencyCycleError,);
        }
      },
    },),
    it({
      name: 'rejects dependent work requested after its owner completed',
      fn: async () => {
        const retained: { run?: RunTask } = {};
        const executed: string[] = [];
        const owner = { name: 'owner', };
        const run = createTaskInvocation({ execute: async ({ request, runTask, }) => {
          executed.push(request.name,);
          retained.run = runTask;
        }, },);
        await run(owner,);
        if (retained.run === undefined)
          throw new FixtureFailureError('The fixture did not receive a dependency dispatcher',);
        let caught: unknown;
        try {
          await retained.run({ name: 'late-child', },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(TaskInvocationClosedError,);
        expect(executed,).toEqual([owner.name,],);
      },
    },),
  ],
},);

//endregion Observable completion and input identity
