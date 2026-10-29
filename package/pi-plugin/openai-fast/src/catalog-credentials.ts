/**
 Credential-free metadata bootstrap with immutable read-only capabilities. @module
 */
import type { CredentialStore, } from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { FastModelError, } from './fast-model-error.ts';

//region Read-only native credential contract

/**
 Module logger records operations, never credential values.
 */
const moduleLogger = tagged({ tag: 'pi-plugin-openai-fast.catalog-credentials', },);

/**
 Return the native missing-credential value without opening the real store.

 @returns empty native credential result
 */
function read(): ReturnType<CredentialStore['read']> {
  return Promise.resolve(undefined,);
}

/**
 Return an empty native metadata list without resolving any credentials.

 @returns empty metadata list
 */
function list(): ReturnType<CredentialStore['list']> {
  return Promise.resolve([],);
}

/**
 Reject write authority rather than invoking the supplied native mutation callback.

 @returns rejected credential operation
 */
function modify(): ReturnType<CredentialStore['modify']> {
  return Promise.reject(new FastModelError('Read-only Codex catalog bootstrap attempted to change credentials.',),);
}

/**
 Reject deletion authority without touching any credential storage.

 @returns rejected credential operation
 */
function deleteCredential(): ReturnType<CredentialStore['delete']> {
  return Promise.reject(new FastModelError('Read-only Codex catalog bootstrap attempted to delete credentials.',),);
}

/**
 Create frozen native credential capabilities for metadata-only bootstrap.

 @returns store with no real credential access or write authority

 @example
 ```ts
 const credentials = createCatalogCredentials();
 ```
 */
export function createCatalogCredentials(): CredentialStore {
  /**
   Factory logger records metadata bootstrap lifecycle only.
   */
  const l = tagged({
    tag: createCatalogCredentials.name,
    l: moduleLogger,
  },);
  l.trace('creating credential-free catalog reader',);
  return Object.freeze({
    read,
    list,
    modify,
    delete: deleteCredential,
  },);
}

//endregion
