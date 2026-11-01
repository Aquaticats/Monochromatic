/**
 In-memory presence index database for unit tests. Exported from the package entry as `\@internal` so tests import it through the built artifact.

 @module
 */

import {
  INSERT_PRESENT_SQL,
  SELECT_PRESENT_SQL,
} from './presence-index.ts';
import type {
  PresenceIndexDatabase,
  PresenceIndexStatement,
} from './store.ts';

/**
 In-memory presence index plus a view of its contents and traffic for assertions.
 */
export type MemoryPresenceIndex = PresenceIndexDatabase & {
  /**
   Oids the index currently holds.
   */
  readonly oids: Set<string>;
  /**
   SQL text of every prepared statement, in call order.
   */
  readonly statements: string[];
};

/**
 Parameters for {@link createMemoryPresenceIndex}.
 */
export type CreateMemoryPresenceIndexParams = {
  /**
   Oids the index holds before the test starts.
   */
  readonly initial?: readonly string[];
  /**
   Error every statement run throws, to exercise the fallback to the bucket.
   */
  readonly failure?: Error;
};

/**
 Parse the JSON array of oids a statement was bound to.

 @param oidsJson - JSON text of an array of strings

 @returns the strings

 @throws when the text is not a JSON array of strings
 */
function parseOidArray(oidsJson: string,): readonly string[] {
  /**
   Parsed value, still unchecked.
   */
  const parsed: unknown = JSON.parse(oidsJson,);
  if (!Array.isArray(parsed,)) {
    throw new TypeError('bound value must be a JSON array',);
  }
  return parsed.map(function requireString(value: unknown,): string {
    if ((typeof value) !== 'string') {
      throw new TypeError('bound array must hold only strings',);
    }
    return value;
  },);
}

/**
 Build an in-memory presence index that understands the two statements in `presence-index.ts`.

 @param initial - oids held before the test starts

 @param failure - error every statement run throws

 @returns database whose `oids` reflects every recorded write

 @throws when a statement other than the two index statements is prepared

 @example
 ```ts
 const index = createMemoryPresenceIndex({ initial: [oid] });
 ```
 */
export function createMemoryPresenceIndex({
  initial = [],
  failure,
}: CreateMemoryPresenceIndexParams = {},): MemoryPresenceIndex {
  /**
   Oids held, seeded from `initial`.
   */
  const oids = new Set<string>(initial,);
  /**
   Statements prepared so far.
   */
  const statements: string[] = [];
  return {
    oids,
    statements,
    prepare(sql: string,): PresenceIndexStatement {
      statements.push(sql,);
      if ((sql !== SELECT_PRESENT_SQL) && (sql !== INSERT_PRESENT_SQL)) {
        throw new Error(`unexpected statement: ${sql}`,);
      }
      /**
       Oids parsed from the bound JSON array.
       */
      let bound: readonly string[] = [];
      /**
       Statement under construction.
       */
      const statement: PresenceIndexStatement = {
        bind(oidsJson: string,): PresenceIndexStatement {
          bound = parseOidArray(oidsJson,);
          return statement;
        },
        all(): Promise<{ readonly results: readonly { readonly oid: string; }[]; }> {
          if (failure !== undefined) {
            return Promise.reject(failure,);
          }
          return Promise.resolve({
            results: bound
              .filter(function isHeld(oid: string,): boolean {
                return oids.has(oid,);
              },)
              .map(function toRow(oid: string,) {
                return { oid, };
              },),
          },);
        },
        run(): Promise<unknown> {
          if (failure !== undefined) {
            return Promise.reject(failure,);
          }
          for (const oid of bound) {
            oids.add(oid,);
          }
          return Promise.resolve(undefined,);
        },
      };
      return statement;
    },
  };
}
