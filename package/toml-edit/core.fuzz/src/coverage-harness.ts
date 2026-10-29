/**
 Run-and-count primitives shared by the toml-edit fuzz coverage sweeps.
 
 The accepted and rejected operation counts live in a module-scoped counter
 mutated only by the helpers here, read back through {@link tallySnapshot}.
 Keeping the counter out of every signature lets the operation functions take
 deeply readonly parameters; one sweep runs per process, so a shared counter
 has no re-entrancy concern. Operations the API rejects by design throw a
 {@link TomlEditError}, which is caught and counted; any other throw is a real defect
 and propagates so it surfaces rather than being swallowed.
 
 @module
 */


import {
  parseTomlEdit,
  TomlEditError,
  type TomlEditMode,
  type TomlEditState,
} from '@monochromatic-dev/toml-edit-core/ts';

/**
 Operation snapshot returned by {@link tallySnapshot}.
 */
export type TallySnapshot = {
  readonly ok: number;
  readonly rejected: number;
};

/**
 Module-scoped accepted and rejected operation counts. A mutated `const` object
 is allowed where `no-module-root-let` would reject a `let`, and one sweep runs
 per process so no reset or re-entrancy guard is needed.
 */
const tallyCounts = {
  ok: 0,
  rejected: 0,
};

/**
 Read the current accepted and rejected operation counts.
 
 @returns Snapshot of the counts after the sweep.
 
 @example
 ```ts
 const { ok, rejected, } = tallySnapshot();
 ```
 */
export function tallySnapshot(): TallySnapshot {
  return {
    ok: tallyCounts.ok,
    rejected: tallyCounts.rejected,
  };
}

/**
 Count one accepted operation.
 */
function recordAccepted(): void {
  tallyCounts.ok += 1;
}

/**
 Count one by-design rejection.
 */
function recordRejected(): void {
  tallyCounts.rejected += 1;
}

/**
 Run `thunk`, counting a by-design {@link TomlEditError} rejection and re-throwing any
 other error so an unexpected failure class still surfaces.
 
 @throws Error for any non-{@link TomlEditError} thrown by `thunk`.
 
 @mutates thunk - Invoking caller-supplied operation can change captured or otherwise reachable state.
 
 @example
 ```ts
 attempt({ thunk: function once() { tomlStringify({ edit, },); }, },);
 ```
 */
export function attempt({ thunk, }: { readonly thunk: () => void; },): void {
  try {
    thunk();
    recordAccepted();
  }
  catch (caught: unknown) {
    if (caught instanceof TomlEditError) {
      recordRejected();
      return;
    }
    throw caught;
  }
}

/**
 Run `thunk`, returning its value on success or `fallback` on a by-design
 {@link TomlEditError}, counting either outcome. Any other throw propagates.
 
 @returns The thunk result, or `fallback` when the operation was rejected.
 
 @throws Error for any non-{@link TomlEditError} thrown by `thunk`.
 
 @mutates thunk - Invoking caller-supplied operation can change captured or otherwise reachable state.
 
 @example
 ```ts
 const next = attemptValue({ thunk: function step() { return tomlSet(args,); }, fallback: edit, },);
 ```
 */
export function attemptValue<T,>(
  {
    thunk,
    fallback,
  }: {
    readonly thunk: () => T;
    readonly fallback: T;
  },
): T {
  try {
    /**
     Result of the guarded operation.
     */
    const result = thunk();
    recordAccepted();
    return result;
  }
  catch (caught: unknown) {
    if (caught instanceof TomlEditError) {
      recordRejected();
      return fallback;
    }
    throw caught;
  }
}

/**
 Result of a guarded parse: the state, or a rejection marker.
 */
export type ParseResult =
  | {
    readonly ok: true;
    readonly edit: TomlEditState
  }
    | { readonly ok: false; };

/**
 Parse `source` in `mode`, counting a by-design rejection rather than throwing.
 
 @returns Parsed state, or a rejection marker for invalid input.
 
 @example
 ```ts
 const parsed = tryParse({ source: 'a = 1\n', mode: 'splice', },);
 ```
 */
export function tryParse(
  {
    source,
    mode,
  }: {
    readonly source: string;
    readonly mode: TomlEditMode;
  },
): ParseResult {
  return attemptValue({
    thunk: function parse(): ParseResult {
      return {
        ok: true,
        edit: parseTomlEdit({
          source,
          mode,
        },),
      };
    },
    fallback: { ok: false, },
  },);
}
