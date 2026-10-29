// Fixture: shared formatter calls and nonformatting Error checks remain valid.

import {
  caughtValueStack,
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';

/** Uses shared message formatter. */
function message(error: unknown,): string {
  return caughtValueText(error,);
}

/** Uses shared stack formatter. */
function stack(error: unknown,): string {
  return caughtValueStack(error,);
}

/** Uses Error.isError only as a predicate. */
function isError(error: unknown,): boolean {
  return Error.isError(error,);
}

/** Provides domain-specific non-Error recovery text. */
function providerFallback(error: unknown,): string {
  return `provider:${typeof error}`;
}

/** Keeps domain fallback semantics instead of replacing them with generic coercion. */
function domainFormatter(error: unknown,): string {
  return Error.isError(error,)
    ? error.message
    : providerFallback(error,);
}

/** Keeps local Error-like detector outside global formatter pattern. */
function shadowedErrorFormatter(error: unknown,): string {
  const Error = { isError: globalThis.Error.isError, };
  return Error.isError(error,)
    ? error.message
    : String(error,);
}

/** Keeps local String fallback outside global coercion pattern. */
function shadowedStringFormatter(error: unknown,): string {
  const String = providerFallback;
  return Error.isError(error,)
    ? error.message
    : String(error,);
}

message('failure',);
stack('failure',);
isError('failure',);
domainFormatter('failure',);
shadowedErrorFormatter('failure',);
shadowedStringFormatter('failure',);
