/**
 Internal testing seam for identity resolution, the atomic writer, and the
 shape validators.

 This subpath is not part of the reader's public index surface. It exists so
 built-artifact tests can reach the internal modules without importing
 source.

 @module
 */

export {
  EnvLocalJsonWriteError,
} from './errors.ts';
export type {
  EnvLocalJsonWriteFailureReason,
} from './errors.ts';
export {
  buildSopsChildEnv,
  resolveSopsIdentity,
} from './identity.ts';
export type {
  BuildSopsChildEnvOptions,
  ResolveSopsIdentityOptions,
  SopsIdentity,
  SopsIdentitySource,
} from './identity.ts';
export {
  assertSopsEnvelope,
  parseDecryptedValues,
} from './store-shape.ts';
export {
  isEnvironmentVariableName,
  writeEnvLocalJsonValues,
} from './write.ts';
export type {
  WriteEnvLocalJsonValuesOptions,
} from './write.ts';
