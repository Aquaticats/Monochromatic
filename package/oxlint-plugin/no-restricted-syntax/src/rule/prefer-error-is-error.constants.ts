//region Canonical replacement

/**
 Canonical replacement callee for Error object detection.
 
 @example
 ```ts
 const replacement = `${ERROR_IS_ERROR_CALLEE}(error,)`;
 ```
 */
export const ERROR_IS_ERROR_CALLEE = 'Error.isError';

//endregion Canonical replacement

//region Global and property names

/**
 Global Error constructor name.
 */
export const ERROR_CONSTRUCTOR_NAME = 'Error';

/**
 Global object name used by `globalThis.Error`.
 */
export const GLOBAL_THIS_NAME = 'globalThis';

/**
 Built-in Object constructor name.
 */
export const OBJECT_CONSTRUCTOR_NAME = 'Object';

/**
 Prototype property name in `Object.prototype.toString.call(value)`.
 */
export const PROTOTYPE_PROPERTY_NAME = 'prototype';

/**
 toString property name in `Object.prototype.toString.call(value)`.
 */
export const TO_STRING_PROPERTY_NAME = 'toString';

/**
 call property name in `Object.prototype.toString.call(value)`.
 */
export const CALL_PROPERTY_NAME = 'call';

/**
 Constructor property name used by `value.constructor === Error`.
 */
export const CONSTRUCTOR_PROPERTY_NAME = 'constructor';

/**
 Node util namespace property holding type-check helpers.
 */
export const TYPES_PROPERTY_NAME = 'types';

/**
 Deprecated Node helper replaced by `Error.isError`.
 */
export const IS_NATIVE_ERROR_PROPERTY_NAME = 'isNativeError';

/**
 Object.prototype.toString tag for Error objects.
 */
export const ERROR_OBJECT_TAG = '[object Error]';

/**
 Object.prototype.toString tag suffix for Error objects.
 */
export const ERROR_OBJECT_TAG_SUFFIX = ' Error]';

/**
 Parsed Object.prototype.toString type name for Error objects.
 */
export const ERROR_OBJECT_TAG_TYPE_NAME = 'Error';

/**
 endsWith property name used by Object.prototype.toString suffix checks.
 */
export const ENDS_WITH_PROPERTY_NAME = 'endsWith';

/**
 slice property name used by parsed Object.prototype.toString checks.
 */
export const SLICE_PROPERTY_NAME = 'slice';

/**
 Number of characters before Object.prototype.toString's embedded type name.
 */
export const OBJECT_TAG_TYPE_PREFIX_LENGTH = 8;

/**
 slice end offset that removes Object.prototype.toString's closing bracket.
 */
export const OBJECT_TAG_TYPE_END_OFFSET = -1;

//endregion Global and property names

//region Node util sources

/**
 Plain Node util import source.
 */
export const NODE_UTIL_SOURCE = 'util';

/**
 Protocol-qualified Node util import source.
 */
export const NODE_PROTOCOL_UTIL_SOURCE = 'node:util';

/**
 Plain Node util/types import source.
 */
export const NODE_UTIL_TYPES_SOURCE = 'util/types';

/**
 Protocol-qualified Node util/types import source.
 */
export const NODE_PROTOCOL_UTIL_TYPES_SOURCE = 'node:util/types';

//endregion Node util sources

//region Detection sentinel

/**
 Sentinel returned when a syntax form is not an alternative Error detector.
 
 @example
 ```ts
 const result = NOT_ERROR_DETECTION;
 if (typeof result === 'symbol') return;
 ```
 */
export const NOT_ERROR_DETECTION: unique symbol = Symbol(
  'syntax is not alternate error detection',
);

/**
 Text for a detected Error value expression, or {@link NOT_ERROR_DETECTION}
 when no match exists.
 */
export type ErrorDetectionArgumentText = string | typeof NOT_ERROR_DETECTION;

/**
 Fix channel for a detected Error value expression.
 */
export type ErrorDetectionFixKind = 'fix' | 'suggestion';

/**
 Detected Error expression replacement metadata.
 */
export type ErrorDetectionReplacement = {
  /**
   Source text for value being tested.
   */
  readonly argumentText: string;
  /**
   Fix channel used for the replacement.
   */
  readonly fixKind: ErrorDetectionFixKind;
};

/**
 Replacement metadata, or {@link NOT_ERROR_DETECTION} when no match exists.
 */
export type ErrorDetectionReplacementResult =
  | ErrorDetectionReplacement
    | typeof NOT_ERROR_DETECTION;

//endregion Detection sentinel
