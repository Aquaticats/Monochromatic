import { CommandError, } from './errors.ts';
import {
  run,
  runAllowingFailure,
} from './runner.ts';
import {
  isAbsentTableDiagnostic,
  type TableProto,
} from './tunnel-table-diagnostic.ts';

/**
 First table tried for WireGuard default policy,
 matching wg-quick.
 */
const BASE_TUNNEL_TABLE = 51_820;

/**
 First table tried for application bypass routes.
 */
const BASE_BYPASS_TABLE = 52_000;

/**
 First preference tried for application bypass rules.
 */
const BASE_BYPASS_PREFERENCE = 50;

/**
 Address families probed for route and rule ownership.
 */
const PROTOS: readonly TableProto[] = [
  '-4',
  '-6',
];

/**
 Probes one family for routes and rule references to table.
 
 `ip -6 route show table <n>` exits 2 when only IPv4 has created table.
 Exact `FIB table does not exist` diagnostic represents empty IPv6 table,
 while every other command failure remains fatal.
 
 @param proto - Address family.
 
 @param table - Candidate routing table.
 
 @returns Route and rule output for occupancy decision.
 
 @example
 ```ts
 await probeTable({ proto: '-6', table: 52000 });
 ```
 */
async function probeTable(
  {
    proto,
    table,
  }: {
    readonly proto: TableProto;
    readonly table: number;
  },
): Promise<readonly string[]> {
  /**
   Exact route-show arguments retained for error translation.
   */
  const routeArgs = [
    proto,
    'route',
    'show',
    'table',
    String(table,),
  ] as const;
  /**
   Route listing that may report absent family table.
   */
  const routes = await runAllowingFailure({
    command: 'ip',
    args: routeArgs,
  },);
  /**
   Whether result is exact absent-family-table response from `ip`.
   */
  const familyTableAbsent = isAbsentTableDiagnostic({
    proto,
    exitCode: routes.exitCode,
    stderr: routes.stderr,
  },);
  if ((routes.exitCode !== 0) && (!familyTableAbsent)) {
    throw new CommandError({
      command: 'ip',
      args: routeArgs,
      exitCode: routes.exitCode,
      stderr: routes.stderr,
    },);
  }
  /**
   Rules referencing candidate table.
   */
  const rules = await run({
    command: 'ip',
    args: [
      proto,
      'rule',
      'show',
      'table',
      String(table,),
    ],
  },);
  return [
    routes.exitCode === 0 ? routes.stdout : '',
    rules.stdout,
  ];
}

/**
 Reports whether table has no routes or policy-rule references in either family.
 
 @param table - Candidate routing table.
 
 @returns Whether table is unused.
 
 @example
 ```ts
 await tableIsFree({ table: 52000 });
 ```
 */
export async function tableIsFree(
  { table, }: { readonly table: number; },
): Promise<boolean> {
  /**
   Route and rule outputs for both families.
   */
  const probes = await Promise.all(PROTOS.map(function familyProbe(
    proto: TableProto,
  ): Promise<readonly string[]> {
    return probeTable({
      proto,
      table,
    },);
  },),);
  return probes.flat()
    .every(function probeIsEmpty(stdout: string,): boolean {
      return stdout.trim() === '';
    },);
}

/**
 Scans upward for unused table without recursive promise chain.
 
 @param minimum - First candidate.
 
 @returns First table free in both families.
 
 @example
 ```ts
 await findFreeTableAtOrAbove({ minimum: 52000 });
 ```
 */
async function findFreeTableAtOrAbove(
  { minimum, }: { readonly minimum: number; },
): Promise<number> {
  /**
   Scan cursor over numeric table namespace.
   */
  let table = minimum;
  // oxlint-disable-next-line eslint/no-await-in-loop -- Candidate probes are sequential because each result decides whether next table is needed.
  while (!(await tableIsFree({ table, })))
    table += 1;
  return table;
}

/**
 Finds free WireGuard policy table.
 
 @returns First free table from wg-quick base.
 
 @example
 ```ts
 await findFreeTable();
 ```
 */
export async function findFreeTable(): Promise<number> {
  return await findFreeTableAtOrAbove({ minimum: BASE_TUNNEL_TABLE, },);
}

/**
 Finds free application-bypass table at or above requested floor.
 
 @param minimum - Optional retry floor after cooperative lock collision.
 
 @returns Free table in both families.
 
 @example
 ```ts
 await findFreeBypassTable({ minimum: 52000 });
 ```
 */
export async function findFreeBypassTable(
  { minimum, }: { readonly minimum: number; },
): Promise<number> {
  return await findFreeTableAtOrAbove({
    minimum: Math.max(
      BASE_BYPASS_TABLE,
      minimum,
    ),
  },);
}

/**
 Extracts numeric preference from one `ip rule show` line.
 
 @param line - Rule line beginning with `<preference>:`.
 
 @returns Positive preference or zero for unrecognized line.
 
 @example
 ```ts
 rulePreference({ line: '50: from all lookup main' });
 ```
 */
function rulePreference(
  { line, }: { readonly line: string; },
): number {
  /**
   Delimiter after numeric preference.
   */
  const colon = line.indexOf(':',);
  if (colon <= 0)
    return 0;
  /**
   Parsed leading preference.
   */
  const preference = Math.trunc(
    Number(line.slice(
      0,
      colon,
    ),),
  );
  return Number.isSafeInteger(preference,) && (preference > 0)
    ? preference
    : 0;
}

/**
 Reads preferences currently occupied in either address family.
 
 @returns Fresh set of positive rule preferences.
 
 @example
 ```ts
 await readUsedPreferences();
 ```
 */
async function readUsedPreferences(): Promise<ReadonlySet<number>> {
  /**
   Complete rule listings for both families.
   */
  const listings = await Promise.all(PROTOS.map(function readRules(
    proto: TableProto,
  ): Promise<{ readonly stdout: string; }> {
    return run({
      command: 'ip',
      args: [
        proto,
        'rule',
        'show',
      ],
    },);
  },),);
  /**
   Preferences occupied in either family.
   */
  const used = new Set<number>();
  for (const { stdout, } of listings) {
    for (const line of stdout.split('\n',)) {
      /**
       Numeric preference parsed from current line.
       */
      const preference = rulePreference({ line, },);
      if (preference > 0)
        used.add(preference,);
    }
  }
  return used;
}

/**
 Reports whether preference has no rule in either address family.
 
 @param preference - Candidate preference.
 
 @returns Whether candidate remains unused at probe time.
 
 @example
 ```ts
 await preferenceIsFree({ preference: 50 });
 ```
 */
export async function preferenceIsFree(
  { preference, }: { readonly preference: number; },
): Promise<boolean> {
  return !(await readUsedPreferences()).has(preference,);
}

/**
 Finds preference unused by either address family.
 
 @param maximum - Highest preferred priority number before existing catch-all rules.
 
 @returns Free positive preference ordered before occupied candidate.
 
 @throws When no safe preference remains.
 
 @example
 ```ts
 await findFreeBypassPreference({ maximum: 50 });
 ```
 */
export async function findFreeBypassPreference(
  { maximum, }: { readonly maximum: number; },
): Promise<number> {
  /**
   Preferences occupied at allocation probe.
   */
  const used = await readUsedPreferences();
  /**
   Mutable scan cursor over safe preference range.
   */
  /**
   Earliest existing positive rule priority that bypass must precede.
   */
  const earliestUsed = [...used,].reduce(
    function lowerPreference(
      current,
      candidate,
    ): number {
      return Math.min(
        current,
        candidate,
      );
    },
    Number.MAX_SAFE_INTEGER,
  );
  /**
   Highest candidate guaranteed to run before every existing positive rule.
   */
  const existingRuleBound = earliestUsed === Number.MAX_SAFE_INTEGER
    ? BASE_BYPASS_PREFERENCE
    : earliestUsed - 1;
  /**
   Mutable descending cursor toward highest-priority free slot.
   */
  const cursor = {
    preference: maximum > 0
      ? Math.min(
        BASE_BYPASS_PREFERENCE,
        maximum,
        existingRuleBound,
      )
      : Math.min(
        BASE_BYPASS_PREFERENCE,
        existingRuleBound,
      ),
  };
  while ((cursor.preference > 0) && used.has(cursor.preference,))
    cursor.preference -= 1;
  if (cursor.preference <= 0)
    throw new Error('No free application-bypass rule preference remains before occupied priorities.',);
  return cursor.preference;
}
