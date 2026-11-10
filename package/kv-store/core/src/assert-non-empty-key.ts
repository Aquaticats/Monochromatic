import { EmptyKeyError, } from './errors.ts';

/**
 Refuses an empty storage key before a store reaches its backends.

 Lives outside both store modules because the two variants are hand-written
 twins (`doc/decision/memoize-sync-async-parallel-impls.md`), and a key rule
 agreed in one of them and not the other is the exact divergence they exist to
 avoid. Sharing the rule here keeps the decision, its wording, and its timing
 identical on both sides.

 @param operation - store method that received the key, named in the thrown error

 @param key - key the caller passed

 @throws EmptyKeyError when key is the empty string

 @example
 ```ts
 assertNonEmptyKey({ operation: 'set', key: 'user-1' });
 ```

 @example
 Rejected:
 ```ts
 assertNonEmptyKey({ operation: 'get', key: '' }); // throws EmptyKeyError
 ```
 */
export function assertNonEmptyKey({
  operation,
  key,
}: Readonly<{
  operation: string;
  key: string;
}>,): void {
  if (key.length > 0)
    return;
  throw new EmptyKeyError(operation,);
}
