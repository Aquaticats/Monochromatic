/**
 `satisfiesOrThrow`: assert custom predicate satisfaction, return candidate or throw.
 
 @module
 */

import { formatUnknownValue, } from './format-unknown-value.ts';

/**
 Parameters passed to custom `satisfiesOrThrow` predicates.
 
 @typeParam Candidate - Candidate value being judged
 
 @typeParam Value - Expected value configured on assertion factory
 
 @example
 ```ts
 const parameters: SatisfiesOrThrowPredicateParameters<string, string> = {
   candidate: 'READY',
   value: 'ready',
 };
 ```
 */
export type SatisfiesOrThrowPredicateParameters<Candidate, Value,> = {
  readonly candidate: Candidate;
  readonly value: Value;
};

/**
 Synchronous custom predicate used by `satisfiesOrThrow`.
 
 Predicates receive `unknown` candidates by default because unannotated
 checkers accept any later candidate value. Add the `Candidate` type argument
 when a predicate is only valid for a narrower candidate type.
 
 @typeParam Value - Expected value configured on assertion factory
 
 @typeParam Candidate - Candidate type accepted by predicate
 
 @example
 ```ts
 const predicate: SatisfiesOrThrowPredicate<string, string> = ({
   candidate,
   value,
 }) => candidate.toLowerCase() === value;
 ```
 */
export type SatisfiesOrThrowPredicate<Value, Candidate = unknown,> = (
  parameters: SatisfiesOrThrowPredicateParameters<Candidate, Value>,
) => boolean;

/**
 Async-capable custom predicate used by `satisfiesOrThrowAsync`.
 
 @typeParam Value - Expected value configured on assertion factory
 
 @typeParam Candidate - Candidate type accepted by predicate
 
 @example
 ```ts
 const predicate: SatisfiesOrThrowAsyncPredicate<string, string> = async ({
   candidate,
   value,
 }) => candidate.toLowerCase() === value;
 ```
 */
export type SatisfiesOrThrowAsyncPredicate<Value, Candidate = unknown,> = (
  parameters: SatisfiesOrThrowPredicateParameters<Candidate, Value>,
) => boolean | Promise<boolean>;

/**
 Options for default `Object.is(candidate, value)` satisfaction checks.
 
 @typeParam Value - Expected value compared through `Object.is`
 
 @example
 ```ts
 const options: SatisfiesOrThrowEqualityOptions<'ready'> = {
   value: 'ready',
 };
 ```
 */
export type SatisfiesOrThrowEqualityOptions<Value,> = {
  readonly value: Value;
  readonly predicate?: undefined;
};

/**
 Options for synchronous custom-predicate satisfaction checks.
 
 @typeParam Value - Expected value passed into custom predicate
 
 @typeParam Candidate - Candidate type accepted by predicate
 
 @example
 ```ts
 const options: SatisfiesOrThrowPredicateOptions<string, string> = {
   value: 'ready',
   predicate: ({ candidate, value, }) => candidate.toLowerCase() === value,
 };
 ```
 */
export type SatisfiesOrThrowPredicateOptions<Value, Candidate = unknown,> = {
  readonly value: Value;
  readonly predicate: SatisfiesOrThrowPredicate<Value, Candidate>;
};

/**
 Options accepted by `satisfiesOrThrow`.
 
 @typeParam Value - Expected value configured on assertion factory
 
 @typeParam Candidate - Candidate type accepted by predicate
 
 @example
 ```ts
 const options: SatisfiesOrThrowOptions<string, string> = {
   value: 'ready',
   predicate: ({ candidate, value, }) => candidate === value,
 };
 ```
 */
export type SatisfiesOrThrowOptions<Value, Candidate = unknown,> =
  | SatisfiesOrThrowEqualityOptions<Value>
    | SatisfiesOrThrowPredicateOptions<Value, Candidate>;

/**
 Options for async-capable custom-predicate satisfaction checks.
 
 @typeParam Value - Expected value passed into custom predicate
 
 @typeParam Candidate - Candidate type accepted by predicate
 
 @example
 ```ts
 const options: SatisfiesOrThrowAsyncPredicateOptions<string, string> = {
   value: 'ready',
   predicate: async ({ candidate, value, }) => candidate === value,
 };
 ```
 */
export type SatisfiesOrThrowAsyncPredicateOptions<Value, Candidate = unknown,> = {
  readonly value: Value;
  readonly predicate: SatisfiesOrThrowAsyncPredicate<Value, Candidate>;
};

/**
 Options accepted by `satisfiesOrThrowAsync`.
 
 @typeParam Value - Expected value configured on assertion factory
 
 @typeParam Candidate - Candidate type accepted by predicate
 
 @example
 ```ts
 const options: SatisfiesOrThrowAsyncOptions<string, string> = {
   value: 'ready',
   predicate: async ({ candidate, value, }) => candidate === value,
 };
 ```
 */
export type SatisfiesOrThrowAsyncOptions<Value, Candidate = unknown,> =
  | SatisfiesOrThrowEqualityOptions<Value>
    | SatisfiesOrThrowAsyncPredicateOptions<Value, Candidate>;

/**
 Formats values that appear in satisfaction failure messages.
 
 `String(-0)` returns `"0"`, which hides the exact `Object.is` failure.
 This formatter preserves `-0` while delegating other values to the
 side-effect-free diagnostic formatter.
 
 @param value - Value being rendered for diagnostics
 
 @returns Diagnostic text for failed satisfaction check
 
 @example
 ```ts
 formatSatisfactionValue(-0,); // '-0'
 formatSatisfactionValue(Number.NaN,); // 'NaN'
 ```
 */
function formatSatisfactionValue(value: unknown,): string {
  if (((typeof value) === 'number') && Object.is(
    value,
    -0,
  ))
    return '-0';
  return formatUnknownValue(value,);
}

/**
 Builds a synchronous assertion function from an expected value and optional predicate.
 
 Without a predicate, the returned checker uses `Object.is(candidate, value)`
 and narrows successful candidates to the intersection of candidate and value
 types. With a predicate, the returned checker passes `{ candidate, value }`
 into the predicate and returns the candidate unchanged when predicate returns
 `true`.
 Predicate-thrown errors intentionally propagate unchanged so predicates can
 provide domain-specific diagnostics.
 
 @param options - Expected value and optional synchronous predicate
 
 @returns Checker returning candidate when satisfaction check passes
 
 @throws Error when predicate returns non-boolean result
 
 @throws Error when predicate throws its own error
 
 @example
 Default equality:
 ```ts
 const ready = satisfiesOrThrow({ value: 'ready' as const, })(raw,);
 // ready is typed as 'ready' when raw was unknown.
 ```
 
 @example
 Custom predicate:
 ```ts
 const readyish = satisfiesOrThrow({
   value: 'ready',
   predicate: ({ candidate, value, }) =>
     ((typeof candidate) === 'string') && (candidate.toLowerCase() === value),
 })('READY',);
 // readyish is the original candidate, 'READY'.
 ```
 */
export function satisfiesOrThrow<const Value,>(
  options: SatisfiesOrThrowEqualityOptions<Value>,
): <const Candidate,>(candidate: Candidate) => Candidate & Value;
export function satisfiesOrThrow<const Value, const PredicateCandidate = unknown,>(
  options: SatisfiesOrThrowPredicateOptions<Value, PredicateCandidate>,
): <const Candidate extends PredicateCandidate,>(candidate: Candidate) => Candidate;
/**
 Implements `satisfiesOrThrow` overloads.
 
 @param options - Expected value and optional synchronous predicate
 
 @returns Checker returning candidate when satisfaction check passes
 
 @example
 ```ts
 satisfiesOrThrow({ value: 'ready', })('ready',);
 ```
 */
export function satisfiesOrThrow<const Value, const PredicateCandidate = unknown,>(
  options: SatisfiesOrThrowOptions<Value, PredicateCandidate>,
):
  | (<const Candidate,>(candidate: Candidate) => Candidate & Value)
    | (<const Candidate extends PredicateCandidate,>(candidate: Candidate) => Candidate) {
  if (options.predicate === undefined) {
    return function judgeSatisfiesEquality<const Candidate,>(
      candidate: Candidate,
    ): Candidate & Value {
      if (!Object.is(
        candidate,
        options.value,
      ))
        throw new Error(
          `Expected candidate to satisfy ${formatSatisfactionValue(
            options.value,
          )}, got ${formatSatisfactionValue(candidate,)}`,
        );
      // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- Object.is proves candidate and configured value are the same runtime value
      return candidate as Candidate & Value;
    };
  }

  return function judgeSatisfiesPredicate<const Candidate extends PredicateCandidate,>(
    candidate: Candidate,
  ): Candidate {
    /**
     Raw predicate result, checked at runtime to protect JavaScript callers
     and unsafe casts.
     */
    const result = options.predicate({
      candidate,
      value: options.value,
    },);
    if ((typeof result) !== 'boolean')
      throw new Error(
        `Expected ${satisfiesOrThrow.name} predicate to return boolean, got ${
          typeof result
        }; use ${satisfiesOrThrowAsync.name} for async predicates`,
      );
    if (!result)
      throw new Error(
        `Expected candidate to satisfy ${formatSatisfactionValue(
          options.value,
        )}, got ${formatSatisfactionValue(candidate,)}`,
      );
    return candidate;
  };
}

/**
 Builds an async assertion function from an expected value and optional predicate.
 
 Without a predicate, the returned checker uses `Object.is(candidate, value)`
 and narrows successful candidates to the intersection of candidate and value
 types. With a predicate, the returned checker passes `{ candidate, value }`
 into the predicate and returns the candidate unchanged when awaited predicate
 result is `true`.
 Predicate-thrown errors and predicate promise rejections intentionally
 propagate unchanged so predicates can provide domain-specific diagnostics.
 
 @param options - Expected value and optional async-capable predicate
 
 @returns Checker returning candidate when satisfaction check passes
 
 @throws Error when awaited predicate result is non-boolean
 
 @throws Error when predicate throws or rejects with its own error
 
 @example
 Default equality:
 ```ts
 const ready = await satisfiesOrThrowAsync({ value: 'ready' as const, })(raw,);
 // ready is typed as 'ready' when raw was unknown.
 ```
 
 @example
 Custom predicate:
 ```ts
 const readyish = await satisfiesOrThrowAsync({
   value: 'ready',
   predicate: async ({ candidate, value, }) =>
     ((typeof candidate) === 'string') && (candidate.toLowerCase() === value),
 })('READY',);
 // readyish is the original candidate, 'READY'.
 ```
 */
export function satisfiesOrThrowAsync<const Value,>(
  options: SatisfiesOrThrowEqualityOptions<Value>,
): <const Candidate,>(candidate: Candidate) => Promise<Candidate & Value>;
export function satisfiesOrThrowAsync<const Value, const PredicateCandidate = unknown,>(
  options: SatisfiesOrThrowAsyncPredicateOptions<Value, PredicateCandidate>,
): <const Candidate extends PredicateCandidate,>(
  candidate: Candidate,
) => Promise<Candidate>;
/**
 Implements `satisfiesOrThrowAsync` overloads.
 
 @param options - Expected value and optional async-capable predicate
 
 @returns Checker returning candidate when satisfaction check passes
 
 @example
 ```ts
 await satisfiesOrThrowAsync({ value: 'ready', })('ready',);
 ```
 */
export function satisfiesOrThrowAsync<const Value, const PredicateCandidate = unknown,>(
  options: SatisfiesOrThrowAsyncOptions<Value, PredicateCandidate>,
):
  | (<const Candidate,>(candidate: Candidate) => Promise<Candidate & Value>)
    | (<const Candidate extends PredicateCandidate,>(candidate: Candidate) => Promise<Candidate>) {
  if (options.predicate === undefined) {
    return async function judgeSatisfiesEqualityAsync<const Candidate,>(
      candidate: Candidate,
    ): Promise<Candidate & Value> {
      /**
       Whether candidate equals configured value under default equality.
       */
      const result = await Promise.resolve(
        Object.is(
          candidate,
          options.value,
        ),
      );
      if (!result)
        throw new Error(
          `Expected candidate to satisfy ${formatSatisfactionValue(
            options.value,
          )}, got ${formatSatisfactionValue(candidate,)}`,
        );
      // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- Object.is proves candidate and configured value are the same runtime value
      return candidate as Candidate & Value;
    };
  }

  return async function judgeSatisfiesPredicateAsync<
    const Candidate extends PredicateCandidate,
    >(
    candidate: Candidate,
    ): Promise<Candidate> {
    /**
     Awaited predicate result, checked at runtime to protect JavaScript callers
     and unsafe casts.
     */
    const result = await options.predicate({
      candidate,
      value: options.value,
    },);
    if ((typeof result) !== 'boolean')
      throw new Error(
        `Expected ${satisfiesOrThrowAsync.name} predicate to return boolean, got ${
          typeof result
        }`,
      );
    if (!result)
      throw new Error(
        `Expected candidate to satisfy ${formatSatisfactionValue(
          options.value,
        )}, got ${formatSatisfactionValue(candidate,)}`,
      );
    return candidate;
  };
}
