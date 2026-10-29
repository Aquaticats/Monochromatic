import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  run,
  runAllowingFailure,
} from './runner.ts';
import { makeTempDir, } from './tempdir.ts';
import { findFreeTable, } from './tunnel-table.ts';
import {
  isV6,
  protoFlag,
} from './tunnel-util.ts';

/**
 Address-family flag accepted by the `ip` tool.
 */
type Proto = '-4' | '-6';

/**
 Ensures the interface fwmark is set and returns its table number.
 
 The interface fwmark is the policy table shared by both address families, so
 it is allocated and set exactly once per interface, matching wg-quick.
 
 @param interfaceName - Interface whose fwmark is configured.
 
 @returns The interface's policy table number.
 
 @example
 ```ts
 await ensureFwmark({ interfaceName: 'wg0' });
 ```
 */
export async function ensureFwmark(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<number> {
  /**
   Existing interface fwmark reported by `wg show fwmark`, `0`/off means unset.
   */
  const { stdout, } = await run({
    command: 'wg',
    args: [
      'show',
      interfaceName,
      'fwmark',
    ],
  },);
  /**
   Parsed fwmark; `off` or `0` means no table is allocated yet.
   */
  const existing = stdout.trim();
  if ((existing !== 'off') && (existing !== '0')
    && (existing !== '')) {
    /**
     Already-allocated table number carried in the fwmark.
     */
    const parsed = Math.trunc(Number(existing,),);
    if (Number.isSafeInteger(parsed,) && (parsed > 0))
      return parsed;
  }
  /**
   Freshly allocated policy table shared by both families.
   */
  const table = await findFreeTable();
  await run({
    command: 'wg',
    args: [
      'set',
      interfaceName,
      'fwmark',
      String(table,),
    ],
  },);
  return table;
}

/**
 Builds one `route add` batch command for a prefix on the interface.
 
 @param prefix - Allowed prefix to route.
 
 @param interfaceName - Interface carrying the route.
 
 @param table - Optional policy table for default prefixes.
 
 @returns The batch command line, or an empty line when the route already exists.
 
 @example
 ```ts
 await routeCommand({ prefix: '10.0.0.0/8', interfaceName: 'wg0' });
 ```
 */
export async function routeCommand(
  {
    prefix,
    interfaceName,
    table,
  }: {
    readonly prefix: string;
    readonly interfaceName: string;
    readonly table?: number;
  },
): Promise<string> {
  /**
   Routes already on this interface covering the prefix, used to skip a
   duplicate that would otherwise fail with `File exists`.
   */
  const existing = await runAllowingFailure({
    command: 'ip',
    args: [
      protoFlag({ prefix, },),
      'route',
      'show',
      ...(table === undefined
        ? []
        : [
          'table',
          String(table,),
        ]),
      'dev',
      interfaceName,
      'match',
      prefix,
    ],
  },);
  if (existing.stdout
    .split('\n',)
    .some(function isDuplicate(line,): boolean {
      return line.trim()
        .startsWith(`${prefix} `,);
    },))
    return '';
  /**
   Table suffix, present only for a policy-table default route.
   */
  const suffix = table === undefined ? '' : ` table ${String(table,)}`;
  return `route add ${prefix} dev ${interfaceName}${suffix}`;
}

/**
 Adds routes for the given prefixes via a single `ip -batch` run per family.
 
 Batching keeps startup near-constant even with thousands of expanded prefixes,
 and `ip -batch` accepts plain `route add` lines (unlike `route restore`, which
 expects `ip route save` netlink binary format).
 
 @param interfaceName - Interface carrying the routes.
 
 @param prefixes - Prefixes to add.
 
 @param table - Optional policy table applied to every prefix.
 
 @example
 ```ts
 await addRoutes({ interfaceName: 'wg0', prefixes: ['10.0.0.0/8'] });
 ```
 */
export async function addRoutes(
  {
    interfaceName,
    prefixes,
    table,
  }: {
    readonly interfaceName: string;
    readonly prefixes: readonly string[];
    readonly table?: number;
  },
): Promise<void> {
  /**
   IPv4 prefixes among the set.
   */
  const v4 = prefixes.filter(function isV4(prefix,): boolean {
    return !isV6({ prefix, },);
  },);
  /**
   IPv6 prefixes among the set.
   */
  const v6 = prefixes.filter(function isV6Prefix(prefix,): boolean {
    return isV6({ prefix, },);
  },);
  /**
   Per-family batches handed to `ip -batch`.
   */
  const families: readonly (readonly [
    Proto,
    readonly string[],
  ])[] = [
    [
      '-4',
      v4,
    ],
    [
      '-6',
      v6,
    ],
  ];
  /* oxlint-disable eslint/no-await-in-loop -- Per-family batches run sequentially to surface the failing family. */
  for (const [proto, list,] of families) {
    if (list.length === 0)
      continue;
    /**
     Route-add commands, skipping routes already present.
     */
    const commands: string[] = [];
    for (const prefix of list) {
      /**
       Batch command for this prefix, empty when the route already exists.
       */
      const line = await routeCommand({
        prefix,
        interfaceName,
        ...(table === undefined ? {} : { table, }),
      },);
      if (line !== '')
        commands.push(line,);
    }
    if (commands.length === 0)
      continue;
    /**
     Temp file holding the batch commands; `ip -batch` reads a seekable file
     as `/dev/stdin`, which a pipe from a spawned child is not.
     */
    await using dir = await makeTempDir();
    /**
     Path of the batch file.
     */
    const batchPath = join(
      dir.path,
      'routes.batch',
    );
    await writeFile(
      batchPath,
      `${commands.join('\n',)}\n`,
    );
    await run({
      command: 'ip',
      args: [
        proto,
        '-batch',
        batchPath,
      ],
    },);
  }
  /* oxlint-enable eslint/no-await-in-loop */
}
