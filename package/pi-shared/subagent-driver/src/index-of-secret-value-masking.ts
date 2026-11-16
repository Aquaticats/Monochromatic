/**
 Library surface of the masking that keeps a value of this machine's secret environment out of a model's context: the values it is
 armed with, the masking itself, the extension Pi loads, and the handover between the driver and that extension.

 Kept in a file of its own, as the other surfaces are, so no one list outgrows the line budget a file is allowed.

 @module
 */

export {
  type ArmingOfSecretValueMasking,
  armSecretValueMasking,
  maskedToolResult,
  readTheSecretValuesHandedOver,
} from './secret-value-masking-extension.ts';
export {
  createSecretValueMasking,
  isMaskableValue,
  MASKING_NOTHING,
  type TextMasking,
  MASKED_VALUE_CLOSING,
  MASKED_VALUE_OPENING,

  maskedValueMarker,
  MINIMUM_MASKED_VALUE_LENGTH,
  type SecretValue,
  SecretValueMaskingError,
  type SecretValueMasking,
} from './secret-value-masking.ts';
export { SecretEnvironmentUnreadableError, } from './secret-environment-unreadable-error.ts';
export {
  reportedSessionFile,
  type SessionFileThisRunMayRewrite,
  type SessionFileToMaskAtRest,
  sessionFileToMaskAtRest,
} from './a-session-file-this-run-may-rewrite.ts';
export {
  keepsNoSession,
  refusedFreeFormWords,
  type RefusedFreeFormWord,
} from './free-form-arguments-of-pi.ts';
export {
  isDeniedOrWrongType,
  type Presence,
  presenceFromFailure,
  presenceOfPath,
} from './what-the-operating-system-answered.ts';
export { writeSummary, } from './write-summary.ts';
export {
  createStreamMatcher,
  type StreamMatcher,
} from './masking-a-stream.ts';
export {
  maskDataOf,
  type MaskedData,
  type WithheldCause,
} from './masking-of-data.ts';
export {
  maskSpareCopyOfToolOutput,
  spareCopyOfToolOutput,
} from './pi-output-file-of-a-tool-result.ts';
export {
  maskASessionIntoACopy,
  type MaskedSessionAtRest,
  type MaskedSessionToResume,
} from './session-file-at-rest.ts';
export {
  type FileIdentity,
  identityOfFile,
  readAFileWithItsIdentity,
  replaceAFileWithItsMaskedText,
  type ReplacedFile,
  sameFile,
} from './replacing-a-file-only-if-it-is-the-one-that-was-read.ts';
export {
  storeOfTheCheckout,
  valuesInTheCheckoutStore,
} from './secret-values-in-the-checkout-store.ts';
export {
  valuesInPiFile,
  isPiCredentialField,
  PI_CREDENTIAL_FIELD_NAMES,
} from './pi-credential-values.ts';
export {
  exportedCredentialValues,
  type WrittenValue,
  writtenValue,
} from './shell-assignments-of-a-start-up-file.ts';
export {
  isKeptDespiteItsName,
  plainCauseOf,
  readJSONFile,
  type SecretValueSourceReport,
  type UnreadSecretValueSource,
  type ValuesAndTheirSources,
  valuesInPiCredentialFiles,
  valuesInShellStartFiles,
  valuesUnderCredentialFields,
  withoutRepeatedValues,
} from './secret-values-in-the-files-of-this-machine.ts';
export {
  descriptorOfSecretValues,
  MASKING_ARMED_COMMAND,
  MASKING_EXTENSION_ENTRY_NAME,
  maskingArmedDescription,
  maskingExtensionArguments,
  maskingIsArmedIn,
  SECRET_VALUES_DESCRIPTOR_VARIABLE,
  SECRET_VALUES_FD,
  secretValuesHandover,
  type SecretValuesHandover,
  SecretValuesHandoverError,
  valuesOfSecretValuesHandover,
} from './secret-values-handed-to-pi.ts';
export {
  readSecretValuesOfThisMachine,
  type SecretValuesOfThisMachine,
} from './secret-values-of-this-machine.ts';
export {
  valuesInTheAgeIdentity,
  valuesInTheEnvironment,
} from './secret-values-the-driver-holds.ts';
