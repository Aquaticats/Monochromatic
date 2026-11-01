/**
 Registry lookups for a pick's publish time.

 Reads the full packument (`time` map) directly from the registry.
 `pnpm view <name>@<version> time` was rejected: on pnpm 12.4.2 it returned a
 `time` map missing versions that `pnpm view <name> versions` listed
 (see `doc/troubleshooting/pnpm-update-no-save-strict-release-age.md`).

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { PackageSpec, } from './spec.ts';

/**
 Module logger for registry lookups.
 */
const l = tagged({ tag: 'deps-update/registry', },);

//region Types

/**
 Minimal `fetch` shape used here, injectable for tests.
 */
export type FetchLike = (
  url: string,
  init: RequestInit,
) => Promise<Response>;

//endregion Types

//region Errors

/**
 Thrown when the registry does not report a publish time for a version.
 */
export class PublishTimeError extends Error {
  /**
   Builds the error naming the spec and the registry response.

   @param spec - package version whose time was requested

   @param detail - what the registry returned instead

   @example
   ```ts
   throw new PublishTimeError({ spec: { name: 'a', version: '1.0.0' }, detail: 'HTTP 404' });
   ```
   */
  constructor({
    spec,
    detail,
  }: {
    readonly spec: PackageSpec;
    readonly detail: string;
  },) {
    super(`No publish time for ${spec.name}@${spec.version}: ${detail}`,);
    this.name = 'PublishTimeError';
  }
}

//endregion Errors

//region URL

/**
 Builds the packument URL for a package on a registry.

 Scoped names keep their `@` and encode the scope slash as `%2f`, matching
 how npm clients request scoped packuments.

 @param registry - registry base URL, with or without trailing slash

 @param name - npm package name

 @returns absolute packument URL

 @example
 ```ts
 packumentUrl({ registry: 'https://registry.npmjs.org/', name: '@a/b' });
 // 'https://registry.npmjs.org/@a%2fb'
 ```
 */
export function packumentUrl({
  registry,
  name,
}: {
  readonly registry: string;
  readonly name: string;
},): string {
  /**
   Registry base guaranteed to end in one slash.
   */
  const base = registry.endsWith('/',) ? registry : `${registry}/`;
  return `${base}${name.replace(
    '/',
    '%2f',
  )}`;
}

//endregion URL

//region Fetch

/**
 Fetches a version's publish time from its registry packument.

 @param registry - registry base URL serving this package

 @param spec - package version to look up

 @param fetchImpl - `fetch` implementation; tests pass a stub

 @returns publish instant

 @throws PublishTimeError when the response is not OK or lacks a parseable `time` entry

 @example
 ```ts
 await fetchPublishTime({ registry: 'https://registry.npmjs.org/', spec, fetchImpl: fetch });
 ```
 */
export async function fetchPublishTime({
  registry,
  spec,
  fetchImpl,
}: {
  readonly registry: string;
  readonly spec: PackageSpec;
  readonly fetchImpl: FetchLike;
},): Promise<Date> {
  /**
   Logger tagged with this function.
   */
  const fl = tagged({
    tag: fetchPublishTime.name,
    l,
  },);
  /**
   Packument URL for this package.
   */
  const url = packumentUrl({
    registry,
    name: spec.name,
  },);
  fl.debug(`GET ${url}`,);
  /**
   Registry response; full packument because abbreviated metadata omits `time`.
   */
  const response = await fetchImpl(
    url,
    { headers: { accept: 'application/json', }, },
  );
  if (!response.ok) {
    throw new PublishTimeError({
      spec,
      detail: `HTTP ${String(response.status,)} from ${url}`,
    },);
  }
  /**
   Packument body, untyped until checked.
   */
  const body: unknown = await response.json();
  /**
   `time` map when the body carries one.
   */
  const time: unknown = ((typeof body) === 'object') && (body !== null)
    && ('time' in body) ? body.time : undefined;
  /**
   Raw publish timestamp for this version.
   */
  const stamp: unknown = ((typeof time) === 'object') && (time !== null)
    ? Reflect.get(
      time,
      spec.version,
    )
    : undefined;
  if ((typeof stamp) !== 'string') {
    throw new PublishTimeError({
      spec,
      detail: `packument at ${url} has no time entry for this version`,
    },);
  }
  /**
   Parsed publish instant.
   */
  const publishedAt = new Date(stamp,);
  if (Number.isNaN(publishedAt.getTime(),)) {
    throw new PublishTimeError({
      spec,
      detail: `unparseable time ${JSON.stringify(stamp,)}`,
    },);
  }
  fl.debug(`${spec.name}@${spec.version} published ${publishedAt.toISOString()}`,);
  return publishedAt;
}

//endregion Fetch
