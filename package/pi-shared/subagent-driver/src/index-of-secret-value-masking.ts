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
  MASKED_VALUE_CLOSING,
  MASKED_VALUE_OPENING,
  type MaskedDeeply,
  maskedValueMarker,
  MINIMUM_MASKED_VALUE_LENGTH,
  type SecretValue,
  SecretValueMaskingError,
  type SecretValueMasking,
} from './secret-value-masking.ts';
export {
  SecretEnvironmentUnreadableError,
  storeOfTheCheckout,
  valuesInTheCheckoutStore,
} from './secret-values-in-the-checkout-store.ts';
export {
  isKeptDespiteItsName,
  isMaskableValue,
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
