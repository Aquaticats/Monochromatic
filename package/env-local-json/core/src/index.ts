/**
 Read requested keys from the repository's sops/age-encrypted
 `.env.local.json` without mutating `process.env`.

 @module
 */

export {
  EnvLocalJsonDecryptError,
  EnvLocalJsonMissingIdentityError,
  EnvLocalJsonPathError,
  EnvLocalJsonPlaintextError,
  EnvLocalJsonReadError,
  EnvLocalJsonShapeError,
  EnvLocalJsonSopsMissingError,
} from './errors.ts';
export type {
  EnvLocalJsonDecryptFailureKind,
  EnvLocalJsonPathFailureReason,
  EnvLocalJsonShapeFailureReason,
} from './errors.ts';
export { readEnvLocalJson, } from './read-env-local-json.ts';
export type {
  EnvLocalJsonEnvironment,
  ReadEnvLocalJsonOptions,
  ReadEnvLocalJsonResult,
} from './types.ts';
