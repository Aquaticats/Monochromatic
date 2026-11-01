import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { API, } from 'typescript/sync';

import {
  findNodeAtOffset,
  memoizeApiSnapshotRequests,
} from '../dist/final/node/index.mjs';

/** Source of the disposable project the native-server tests query. */
const PROJECT_SOURCE = 'export function twice(value: number,): number { return value * 2; }\n';

/** Hot request every case below repeats. */
const LOCATION_REQUEST = 'getTypeAtLocation';

/** Second file of the project the added-request case queries, so an import has an alias to follow. */
const OTHER_SOURCE = 'export const exported = 1;\n';

/** Source whose declarations give each added request something to answer. */
const ADDED_SOURCE = [
  "import { exported as renamed, } from './other';",
  'export const list: readonly string[] = [];',
  'export const table: { [key: string]: number } = {};',
  'export const maybe: string | undefined = undefined;',
  'export function generic<T extends string,>(value: T,): T { return value; }',
  'export const used = renamed;',
  '',
].join('\n');

/** Configured one-file project served by a memoized native API, closed and removed on scope exit. */
type MemoizedProject = {
  readonly api: API;
  readonly configPath: string;
  readonly sourcePath: string;
  readonly otherPath: string;
  readonly sent: readonly string[];
  [Symbol.dispose]: () => void;
};

/**
 Records the method of every request the native server receives from an API.

 Wraps the pipe-level send that sits below the memo, so a memo hit never appears here.

 @param api - API whose channel is observed.

 @returns the live list of method names sent to the native server.
 */
function recordServerRequests(api: API,): readonly string[] {
  /** Method names in the order they were sent. */
  const sent: string[] = [];
  /** Sync client hidden behind the unstable API. */
  const client: unknown = Reflect.get(api, 'client',);
  if (((typeof client) !== 'object') || (client === null))
    throw new Error('Expected the API to expose its client.',);
  /** RPC channel the client sends through. */
  const channel: unknown = Reflect.get(client, 'channel',);
  if (((typeof channel) !== 'object') || (channel === null))
    throw new Error('Expected the API client to expose its channel.',);
  /** Original send, called with its receiver. */
  const send: unknown = Reflect.get(channel, 'requestSync',);
  if ((typeof send) !== 'function')
    throw new Error('Expected the channel to expose requestSync.',);
  Reflect.set(channel, 'requestSync', function recordedRequestSync(method: string, payload: string,): unknown {
    sent.push(method,);
    /** Answer the native server gave. */
    const answer: unknown = Reflect.apply(send, channel, [method, payload,],);
    return answer;
  },);
  return sent;
}

/**
 Starts a native API with the request memo installed over a configured one-file project on disk.

 @returns the project, with the API closed and the directory removed when the scope ends.
 */
function openMemoizedProject({
  source = PROJECT_SOURCE,
}: {
  readonly source?: string;
} = {},): MemoizedProject {
  /** Unique directory holding the project. */
  const path = mkdtempSync(join(scratchRoot(), 'semantic-request-memo-',),);
  writeFileSync(
    join(path, 'tsconfig.json',),
    `${JSON.stringify({
      compilerOptions: { strict: true, module: 'esnext', moduleResolution: 'bundler', },
      include: ['input.ts', 'other.ts',],
    },)}\n`,
  );
  writeFileSync(join(path, 'input.ts',), source,);
  writeFileSync(join(path, 'other.ts',), OTHER_SOURCE,);
  /** Native API whose client the memo wraps. */
  const api = new API({ cwd: path, },);
  memoizeApiSnapshotRequests({ api, },);
  return {
    api,
    configPath: join(path, 'tsconfig.json',),
    sourcePath: join(path, 'input.ts',),
    otherPath: join(path, 'other.ts',),
    sent: recordServerRequests(api,),
    [Symbol.dispose]: function closeMemoizedProject(): void {
      api.close();
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Counts the requests of one method that reached the native server.

 @param sent - Method names sent to the native server.

 @param method - Request name to count.

 @returns how many of the sent requests had that name.
 */
function sentCount({
  sent,
  method,
}: {
  readonly sent: readonly string[];
  readonly method: string;
},): number {
  return sent.filter(function isQuery(name,): boolean {
    return name === method;
  },).length;
}

/**
 Counts the location queries that reached the native server.

 @param sent - Method names sent to the native server.

 @returns how many were the repeated query.
 */
function locationQueryCount(sent: readonly string[],): number {
  return sentCount({ sent, method: LOCATION_REQUEST, },);
}

await describe({
  name: memoizeApiSnapshotRequests.name,
  concurrency: 1,
  children: [
    it({
      name: 'sends a repeated checker query to the native server once',
      fn: async () => {
        using opened = openMemoizedProject();
        const snapshot = opened.api.createSnapshot({ openProjects: [opened.configPath,], ensurePrograms: true, },);
        const project = snapshot.getConfiguredProject(opened.configPath,);
        const sourceFile = project?.program.getSourceFile(opened.sourcePath,);
        if ((project === undefined) || (sourceFile === undefined))
          throw new Error('Expected the disposable project and its source.',);
        const node = findNodeAtOffset({
          sourceFile,
          offset: PROJECT_SOURCE.indexOf('value',),
        },);
        const first = project.checker.getTypeAtLocation(node,);
        const second = project.checker.getTypeAtLocation(node,);

        expect(locationQueryCount(opened.sent,),).toBe(1,);
        expect(second,).toBe(first,);
        if (first === undefined)
          throw new Error('Expected a type for the parameter.',);
        expect(project.checker.typeToString(first,),).toBe('number',);
      },
    },),
    it({
      name: 'sends each request added to the allow-list to the native server once and replays its answer',
      fn: async () => {
        using opened = openMemoizedProject({ source: ADDED_SOURCE, },);
        const snapshot = opened.api.createSnapshot({ openProjects: [opened.configPath,], ensurePrograms: true, },);
        const project = snapshot.getConfiguredProject(opened.configPath,);
        const sourceFile = project?.program.getSourceFile(opened.sourcePath,);
        if ((project === undefined) || (sourceFile === undefined))
          throw new Error('Expected the disposable project and its source.',);
        const { checker, } = project;
        /**
         Finds the node whose text starts at the first occurrence of a fragment.

         @param fragment - Source text the node starts with.

         @returns the node at that offset.
         */
        const nodeAt = (fragment: string,) => findNodeAtOffset({
          sourceFile,
          offset: ADDED_SOURCE.indexOf(fragment,),
        },);
        const listType = checker.getTypeAtLocation(nodeAt('list',),);
        const tableType = checker.getTypeAtLocation(nodeAt('table',),);
        const maybeType = checker.getTypeAtLocation(nodeAt('maybe',),);
        const genericType = checker.getTypeAtLocation(nodeAt('T extends',),);
        const aliasSymbol = checker.getSymbolAtLocation(nodeAt('renamed',),);
        const moduleSymbol = checker.getSymbolAtLocation(nodeAt("'./other'",),);
        if ((listType === undefined) || (tableType === undefined) || (maybeType === undefined) || (genericType === undefined)
          || (aliasSymbol === undefined) || (moduleSymbol === undefined)) {
          throw new Error('Expected a type or symbol for every declaration the case queries.',);
        }
        /** Each added request, asked twice by the same call, with the answer the real server gave. */
        const asked = [
          ['isArrayType', () => checker.isArrayType(listType,),],
          ['getIndexInfosOfType', () => checker.getIndexInfosOfType(tableType,),],
          ['getNonNullableType', () => checker.getNonNullableType(maybeType,),],
          ['getBaseConstraintOfType', () => checker.getBaseConstraintOfType(genericType,),],
          ['getAliasedSymbol', () => checker.getAliasedSymbol(aliasSymbol,),],
          ['getExportsOfModule', () => checker.getExportsOfModule(moduleSymbol,),],
        ] as const;
        for (const [method, ask,] of asked) {
          const first = ask();
          const second = ask();

          expect(sentCount({ sent: opened.sent, method, },),).toBe(1,);
          expect(second,).toEqual(first,);
        }
        expect(checker.isArrayType(listType,),).toBe(true,);
        expect(checker.getIndexInfosOfType(tableType,),).toHaveLength(1,);
        expect(checker.getExportsOfModule(moduleSymbol,).map((exported,) => exported.name),).toEqual(['exported',],);
        // A request left off the allow-list reaches the server on every repeat, so the count above can fail.
        checker.typeToString(listType,);
        checker.typeToString(listType,);
        expect(sentCount({ sent: opened.sent, method: 'typeToString', },),).toBe(2,);
      },
    },),
    it({
      name: 'sends the same checker query on a new snapshot to the native server again',
      fn: async () => {
        using opened = openMemoizedProject();
        const first = opened.api.createSnapshot({ openProjects: [opened.configPath,], ensurePrograms: true, },);
        const firstProject = first.getConfiguredProject(opened.configPath,);
        const firstFile = firstProject?.program.getSourceFile(opened.sourcePath,);
        if ((firstProject === undefined) || (firstFile === undefined))
          throw new Error('Expected the first snapshot to hold the source.',);
        firstProject.checker.getTypeAtLocation(findNodeAtOffset({
          sourceFile: firstFile,
          offset: PROJECT_SOURCE.indexOf('value',),
        },),);

        /** Same parameter, retyped, so the second snapshot differs from the first. */
        const changedSource = PROJECT_SOURCE.replace('number,): number', 'string,): string',);
        writeFileSync(opened.sourcePath, changedSource,);
        const second = first.update({ fileNotifications: { changed: [opened.sourcePath,], }, ensurePrograms: true, },);
        const secondProject = second.getConfiguredProject(opened.configPath,);
        const secondFile = secondProject?.program.getSourceFile(opened.sourcePath,);
        if ((secondProject === undefined) || (secondFile === undefined))
          throw new Error('Expected the second snapshot to hold the source.',);
        const secondType = secondProject.checker.getTypeAtLocation(findNodeAtOffset({
          sourceFile: secondFile,
          offset: changedSource.indexOf('value',),
        },),);

        expect(second.id,).not.toBe(first.id,);
        expect(locationQueryCount(opened.sent,),).toBe(2,);
        if (secondType === undefined)
          throw new Error('Expected a type for the changed parameter.',);
        expect(secondProject.checker.typeToString(secondType,),).toBe('string',);
      },
    },),
    it({
      name: 'stops answering a released snapshot from memory',
      fn: async () => {
        using opened = openMemoizedProject();
        const snapshot = opened.api.createSnapshot({ openProjects: [opened.configPath,], ensurePrograms: true, },);
        const project = snapshot.getConfiguredProject(opened.configPath,);
        const sourceFile = project?.program.getSourceFile(opened.sourcePath,);
        if ((project === undefined) || (sourceFile === undefined))
          throw new Error('Expected the disposable project and its source.',);
        const node = findNodeAtOffset({
          sourceFile,
          offset: PROJECT_SOURCE.indexOf('value',),
        },);
        project.checker.getTypeAtLocation(node,);
        snapshot.dispose();

        // The native server no longer holds the snapshot, so only a stale memo could still answer.
        expect(() => project.checker.getTypeAtLocation(node,),).toThrow();
      },
    },),
  ],
},);
