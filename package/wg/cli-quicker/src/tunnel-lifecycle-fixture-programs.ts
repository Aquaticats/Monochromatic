/**
 Programs the lifecycle scenario runs, and the kernel state it reads.

 The scenario brings real interfaces up through the built CLI inside a disposable
 user and network namespace, so it calls the real `ip`, `wg` and `nft`, and reads
 back what the kernel holds for each interface.

 @module
 */

import { access, } from 'node:fs/promises';
import { join, } from 'node:path';

import { runAllowingFailure, } from '../dist/final/node/runner.mjs';

/**
 Preferences of the kernel's own policy rules, which no tunnel rule carries.
 */
/**
 Preference of the kernel's own local rule.
 */
const LOCAL_RULE_PREFERENCE = 0;

/**
 Preference of the kernel's own main rule.
 */
const MAIN_RULE_PREFERENCE = 32_766;

/**
 Preference of the kernel's own default rule.
 */
const DEFAULT_RULE_PREFERENCE = 32_767;

/**
 The kernel's own policy rules, which no tunnel rule carries.
 */
const KERNEL_RULE_PREFERENCES: ReadonlySet<number> = new Set([
  LOCAL_RULE_PREFERENCE,
  MAIN_RULE_PREFERENCE,
  DEFAULT_RULE_PREFERENCE,
]);

/**
 What one program call answered.
 */
export type ProgramResult = {
  /**
   Exit code of the program.
   */
  readonly status: number;
  /**
   Standard output of the program.
   */
  readonly stdout: string;
  /**
   Standard error of the program.
   */
  readonly stderr: string;
};

/**
 Absolute paths of the programs the scenario and its stand-ins run.
 */
export type ProgramPaths = {
  /**
   Path of `ip`.
   */
  readonly ip: string;
  /**
   Path of `wg`.
   */
  readonly wg: string;
  /**
   Path of `nft`.
   */
  readonly nft: string;
};

/**
 Runs one program and answers its exit code and output.

 @param command - Absolute path of the program.

 @param args - Arguments for it.

 @param input - Text written to its standard in, when it reads one.

 @returns Exit code, standard output and standard error.

 @example
 ```ts
 await callProgram({ command: '/usr/bin/wg', args: ['show', 'interfaces'] });
 ```
 */
async function callProgram(
  {
    command,
    args,
    input,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly input?: string;
  },
): Promise<ProgramResult> {
  /**
   What the program ended with.
   */
  const result = await runAllowingFailure({
    command,
    args,
    ...(input === undefined ? {} : { input, }),
  },);
  return {
    status: result.exitCode,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

/**
 Runs one `ip` call.

 @param programs - Programs the scenario runs, whose `ip` is used.

 @param args - Arguments for `ip`.

 @returns Exit code and output.

 @example
 ```ts
 await ipCall({ programs, args: ['-4', 'rule', 'show'] });
 ```
 */
export async function ipCall(
  {
    programs,
    args,
  }: {
    readonly programs: ProgramPaths;
    readonly args: readonly string[];
  },
): Promise<ProgramResult> {
  return await callProgram({
    command: programs.ip,
    args,
  },);
}

/**
 Runs one `wg` call.

 @param programs - Programs the scenario runs, whose `wg` is used.

 @param args - Arguments for `wg`.

 @returns Exit code and output.

 @example
 ```ts
 await wgCall({ programs, args: ['show', 'interfaces'] });
 ```
 */
export async function wgCall(
  {
    programs,
    args,
  }: {
    readonly programs: ProgramPaths;
    readonly args: readonly string[];
  },
): Promise<ProgramResult> {
  return await callProgram({
    command: programs.wg,
    args,
  },);
}

/**
 Runs one `nft` call.

 @param programs - Programs the scenario runs, whose `nft` is used.

 @param args - Arguments for `nft`.

 @returns Exit code and output.

 @example
 ```ts
 await nftCall({ programs, args: ['list', 'tables'] });
 ```
 */
export async function nftCall(
  {
    programs,
    args,
  }: {
    readonly programs: ProgramPaths;
    readonly args: readonly string[];
  },
): Promise<ProgramResult> {
  return await callProgram({
    command: programs.nft,
    args,
  },);
}

/**
 Finds a program on the search path, without any stand-in the scenario installs.

 The stand-ins are installed after this, so a stand-in never runs itself.

 @param name - Program to find.

 @returns Its absolute path.

 @throws {@link Error} when the search path has no such program.

 @example
 ```ts
 await findProgram({ name: 'wg' });
 ```
 */
export async function findProgram({ name, }: { readonly name: string; },): Promise<string> {
  for (const directory of (process.env
    .PATH
    ?? '').split(':',)) {
    if (directory === '')
      continue;
    /**
     Candidate path of the program in this directory.
     */
    const candidate = join(
      directory,
      name,
    );
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Each search-path entry is looked at in turn.
      await access(candidate,);
      return candidate;
    }
    catch (error) {
      // This directory has no program of that name; the search goes on.
      void error;
    }
  }
  throw new Error(`The scenario found no ${name} on its search path.`,);
}


/**
 The kernel state the scenario reports about one interface.
 */
export type InterfaceState = {
  /**
   Whether the link is present.
   */
  readonly link: boolean;
  /**
   Whether the link is a WireGuard interface.
   */
  readonly wireguard: boolean;
  /**
   Policy table carried in the fwmark, absent when none.
   */
  readonly table?: number;
  /**
   Routes of that table.
   */
  readonly routes: readonly string[];
};

/**
 Lists the routes of one policy table.

 @param programs - Programs the scenario runs.

 @param table - Table number.

 @returns The listed routes, one per line.
 */
async function routesInTable(
  {
    programs,
    table,
  }: {
    readonly programs: ProgramPaths;
    readonly table: number;
  },
): Promise<readonly string[]> {
  /**
   IPv4 routes of the table, which is where the interfaces of this scenario route.
   */
  const listed = await ipCall({
    programs,
    args: [
      '-4',
      'route',
      'show',
      'table',
      String(table,),
    ],
  },);
  return listed.stdout
    .split('\n',)
    .map(function trimmed(line,): string {
      return line.trim();
    },)
    .filter(function present(line,): boolean {
      return line !== '';
    },);
}

/**
 Reads the kernel state of one interface.

 @param programs - Programs the scenario runs.

 @param interfaceName - Interface to read.

 @returns What the kernel holds for it.

 @example
 ```ts
 await stateOf({ programs, interfaceName: 'wg0' });
 ```
 */
export async function stateOf(
  {
    programs,
    interfaceName,
  }: {
    readonly programs: ProgramPaths;
    readonly interfaceName: string;
  },
): Promise<InterfaceState> {
  /**
   Link details, whose type line says whether this is a WireGuard interface.
   */
  const detailed = await ipCall({
    programs,
    args: [
      '-d',
      'link',
      'show',
      'dev',
      interfaceName,
    ],
  },);
  if (detailed.status !== 0)
    return {
      link: false,
      wireguard: false,
      routes: [],
    };
  /**
   Output of `wg show <interface> fwmark`.
   */
  const shown = await wgCall({
    programs,
    args: [
      'show',
      interfaceName,
      'fwmark',
    ],
  },);
  /**
   Table number carried in the hexadecimal mark.
   */
  const table = Math.trunc(
    Number(shown.stdout
      .trim(),),
  );
  /**
   Whether the interface carries a policy table in its mark.
   */
  const hasTable = (shown.status === 0) && Number.isSafeInteger(table,)
    && (table > 0);
  return {
    link: true,
    wireguard: detailed.stdout
      .includes('wireguard',),
    ...(hasTable ? { table, } : {}),
    routes: hasTable ? await routesInTable({
      programs,
      table,
    },) : [],
  };
}

/**
 Deletes what earlier rounds of the scenario left, so each round starts from nothing.

 @param programs - Programs the scenario runs.

 @param interfaceNames - Interfaces the scenario owns.

 @example
 ```ts
 await cleanInterfaces({ programs, interfaceNames: ['wg0'] });
 ```
 */
export async function cleanInterfaces(
  {
    programs,
    interfaceNames,
  }: {
    readonly programs: ProgramPaths;
    readonly interfaceNames: readonly string[];
  },
): Promise<void> {
  for (const interfaceName of interfaceNames) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Each interface is deleted before the next one is looked at.
    await ipCall({
      programs,
      args: [
        'link',
        'delete',
        'dev',
        interfaceName,
      ],
    },);
  }
  for (const interfaceName of interfaceNames) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Each kill-switch table is deleted in turn.
    await nftCall({
      programs,
      args: [
        'delete',
        'table',
        'inet',
        `wg-quicker-kill-${interfaceName}`,
      ],
    },);
  }
  /**
   Every policy rule of a tunnel, which is every rule above the kernel's own three.
   */
  const rules = await ipCall({
    programs,
    args: [
      '-4',
      'rule',
      'show',
    ],
  },);
  for (const line of rules.stdout
    .split('\n',)) {
    /**
     Preference of the rule on this line.
     */
    const preference = Number(line.split(':',)[0],);
    if (Number.isInteger(preference,) && (!KERNEL_RULE_PREFERENCES.has(preference,))) {
      // oxlint-disable-next-line eslint/no-await-in-loop -- Each rule is deleted before the next is read.
      await ipCall({
        programs,
        args: [
          '-4',
          'rule',
          'delete',
          'pref',
          String(preference,),
        ],
      },);
    }
  }
}

/**
 Lists the IPv4 policy rules of the namespace.

 @param programs - Programs the scenario runs.

 @returns What `ip -4 rule show` answered.

 @example
 ```ts
 await ruleListing({ programs });
 ```
 */
export async function ruleListing(
  { programs, }: { readonly programs: ProgramPaths; },
): Promise<ProgramResult> {
  return await ipCall({
    programs,
    args: [
      '-4',
      'rule',
      'show',
    ],
  },);
}

/**
 Lists the kill-switch table of one interface.

 @param programs - Programs the scenario runs.

 @param interfaceName - Interface whose table is listed.

 @returns What `nft list table` answered.

 @example
 ```ts
 await killSwitchTable({ programs, interfaceName: 'wg0' });
 ```
 */
export async function killSwitchTable(
  {
    programs,
    interfaceName,
  }: {
    readonly programs: ProgramPaths;
    readonly interfaceName: string;
  },
): Promise<ProgramResult> {
  return await nftCall({
    programs,
    args: [
      'list',
      'table',
      'inet',
      `wg-quicker-kill-${interfaceName}`,
    ],
  },);
}
