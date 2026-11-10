/**
 Unit tests running the package's OpenTofu external-data scripts as real processes.

 OpenTofu spawns each script as a `data.external` program, so the contract worth
 pinning is the whole process: the JSON that reaches stdout, the diagnostics that
 reach stderr, and the exit code. Every case runs a copy of the script inside a
 throwaway directory that stands in for `import.meta.dirname`, so the cache files
 a script reads and writes never touch the checkout, and no case reaches the
 network.

 @module
 */

import {
  copyFile,
  mkdtempDisposable,
  readFile,
  symlink,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import spawn, { SubprocessError, } from 'nano-spawn';

//region Constants

/**
 Directory holding the scripts under test.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

/**
 Package directory whose `node_modules` the staged copies resolve workspace packages through.
 */
const PACKAGE_NODE_MODULES = join(
  SOURCE_DIRECTORY,
  '..',
  'node_modules',
);

/**
 Relay cache file name, relative to the script's own directory.
 */
const TOR_RELAY_CACHE_FILE_NAME = 'cache_tor_relays.txt';
/**
 Cache age that still counts as fresh for the relay script's one-hour window.
 */
const FRESH_CACHE_AGE_MS = 0;

/**
 Cache age that puts the relay cache past the script's one-hour window.
 */
const STALE_CACHE_AGE_MS = 2 * 60 * 60 * 1_000;

/**
 Onionoo response fixture holding one port-443 IPv4 endpoint, one non-443
 endpoint, and one port-443 IPv6 endpoint; the expected CIDR list follows from it.
 */
const ONIONOO_RESPONSE_SOURCE = JSON.stringify({
  relays: [{
    or_addresses: [
      '192.0.2.1:443',
      '192.0.2.2:9001',
      '[2001:db8::1]:443',
    ],
  },],
},);

/**
 CIDR list the fixture response must produce, in source order.
 */
const EXPECTED_FRESH_CIDRS = '192.0.2.1/32,2001:db8::1/128';

/**
 Cached CIDR text the stale-cache fallback must replay unchanged.
 */
const STALE_CACHE_CIDRS = '198.51.100.1/32,2001:db8::2/128';

/**
 Fetch replacement that rejects, standing in for an Onionoo outage.
 */
const REJECTING_FETCH_SOURCE = 'async () => { throw new Error("offline fixture"); }';

/**
 Fetch replacement that throws when it is called at all, so a cache-hit case
 proves the script never reached the network.
 */
const NEVER_CALLED_FETCH_SOURCE =
  'async () => { throw new Error("fetch reached despite a fresh cache"); }';

/**
 DNS address families both resolver scripts query for every hostname.
 */
const ADDRESS_FAMILIES = [
  'A',
  'AAAA',
] as const;

/**
 DNS's 63-octet label limit.
 */
const DNS_LABEL_LIMIT = 63;

/**
 Hostname c-ares rejects locally as malformed, so the lookup fails without a
 query and the case needs no resolver and no network.
 */
const UNRESOLVABLE_HOSTNAME = `${'a'.repeat(DNS_LABEL_LIMIT + 1,)}.example.com`;

/**
 Same local rejection, carrying the Storage Box suffix that script requires.
 */
const UNRESOLVABLE_STORAGEBOX_HOSTNAME =
  `${'a'.repeat(DNS_LABEL_LIMIT + 1,)}.your-storagebox.de`;

//endregion Constants

//region Fixtures

/**
 One script copied into a throwaway directory, with the cache files it reads
 and writes beside it and nothing else.
 */
type StagedScript = {
  /**
   Throwaway directory standing in for the script's source directory.
   */
  readonly directory: string;
  /**
   Runs the staged copy and captures both streams and the exit code.
   */
  readonly run: (options?: RunOptions,) => Promise<ScriptRun>;
};

/**
 What one staged run may change about the subprocess.
 */
type RunOptions = {
  /**
   OpenTofu external query written to stdin before the stream closes.
   */
  readonly stdin?: string;
  /**
   `--import` module URL evaluated before the script, used to replace `fetch`.
   */
  readonly importUrl?: string;
};

/**
 Captured outcome of one staged run.
 */
type ScriptRun = {
  /**
   Process exit code; the numeric code for a failed run, never a signal name.
   */
  readonly exitCode: number;
  /**
   Standard output text with its trailing newline stripped.
   */
  readonly stdout: string;
  /**
   Standard error text with its trailing newline stripped.
   */
  readonly stderr: string;
};

/**
 Copies one script into a directory that stands in for its source directory,
 with a `node_modules` link so workspace imports still resolve.

 @param scriptFileName - Script file name inside this package's `src` directory.

 @returns Manifest, staged script, and its captured-output runner.

 @example
 ```ts
 await using staged = await stageScript('fetch_tor_relays.ts',);
 ```
 */
async function stageScript(scriptFileName: string,): Promise<StagedScript & AsyncDisposable> {
  /**
   Throwaway directory holding the staged copy.
   */
  const staged = await mkdtempDisposable(join(
    scratchRoot(),
    'tofu-external-data-',
  ),);
  await writeFile(
    join(
      staged.path,
      'package.json',
    ),
    JSON.stringify({ type: 'module', },),
  );
  await symlink(
    PACKAGE_NODE_MODULES,
    join(
      staged.path,
      'node_modules',
    ),
    'dir',
  );
  await copyFile(
    join(
      SOURCE_DIRECTORY,
      scriptFileName,
    ),
    join(
      staged.path,
      scriptFileName,
    ),
  );

  return {
    directory: staged.path,
    run: async function run(options = {},): Promise<ScriptRun> {
      /**
       Node arguments before the script path; the fetch stub module runs first.
       */
      const nodeArguments = options.importUrl === undefined
        ? [join(
          staged.path,
          scriptFileName,
        ),]
        : [
          '--import',
          options.importUrl,
          join(
            staged.path,
            scriptFileName,
          ),
        ];

      try {
        /**
         Settled run for a script that exits zero.
         */
        const result = await spawn(
          process.execPath,
          nodeArguments,
          {
            cwd: staged.path,
            stdin: { string: options.stdin ?? '{}', },
            env: { MONOCHROMATIC_LOG_DIRECTORY: join(
              staged.path,
              'log-destination',
            ), },
          },
        );
        return {
          exitCode: 0,
          stdout: result.stdout,
          stderr: result.stderr,
        };
      }
      catch (error: unknown) {
        if (!(error instanceof SubprocessError))
          throw error;
        return {
          exitCode: error.exitCode ?? (-1),
          stdout: error.stdout,
          stderr: error.stderr,
        };
      }
    },
    async [Symbol.asyncDispose]() {
      await staged[Symbol.asyncDispose]();
    },
  };
}

/**
 Builds the `--import` URL that replaces `globalThis.fetch` before the script runs,
 so a case pins whether the script reached the network without leaving the machine.
 
 @param implementation - JavaScript function expression assigned to `globalThis.fetch`.
 
 @returns Data URL module Node evaluates before the script.
 
 @example
 ```ts
 const importUrl = fetchStubImportUrl('async () => { throw new Error("unused"); }',);
 ```
 */
function fetchStubImportUrl(implementation: string,): string {
  return `data:text/javascript,${
    encodeURIComponent(`globalThis.fetch = ${implementation};`,)
  }`;
}

/**
 Collects the stderr lines that report a DNS lookup failure.
 
 @param stderr - Captured standard error text of one run.
 
 @returns One entry per logged lookup failure, in emission order.
 
 @example
 ```ts
 lookupDiagnosticLines('resolve_hosts: A lookup for x failed: EBADNAME\n');
 ```
 */
function lookupDiagnosticLines(stderr: string,): readonly string[] {
  return stderr
    .split('\n',)
    .filter(function isLookupLine(line,): boolean {
      return line.includes('lookup for',);
    },);
}

/**
 Asserts one stderr line is a level-warn record carrying the module tag and the
 emitting function's tag, as the tagged logger writes it.
 
 @param line - One captured standard error line.
 
 @param moduleTag - Tag naming the script that emitted the record.
 
 @param functionTag - Tag naming the function that emitted the record.
 
 @example
 ```ts
 expectTaggedWarning({ line, moduleTag: 'resolve_hosts', functionTag: 'resolveAddressFamily' });
 ```
 */
function expectTaggedWarning(
  {
    line,
    moduleTag,
    functionTag,
  }: {
    readonly line: string;
    readonly moduleTag: string;
    readonly functionTag: string;
  },
): void {
  expect(line.startsWith('[warn] ['),).toBe(true,);
  expect(line.includes(`] [${moduleTag}] [${functionTag}] `),).toBe(true,);
}

/**
 Writes the relay cache file beside a staged script and backdates it to the
 requested age.

 @param directory - Staged script directory.

 @param text - Cache text the script must read back verbatim.

 @param ageMs - Age in milliseconds applied through the file mtime.

 @example
 ```ts
 await writeRelayCache({ directory, text: '192.0.2.1/32', ageMs: 0 });
 ```
 */
async function writeRelayCache(
  {
    directory,
    text,
    ageMs,
  }: {
    readonly directory: string;
    readonly text: string;
    readonly ageMs: number;
  },
): Promise<void> {
  /**
   Cache path beside the staged script copy.
   */
  const cachePath = join(
    directory,
    TOR_RELAY_CACHE_FILE_NAME,
  );
  await writeFile(
    cachePath,
    text,
  );
  /**
   Backdating second the file mtime is set to.
   */
  const backdated = new Date(Date.now() - ageMs,);
  await utimes(
    cachePath,
    backdated,
    backdated,
  );
}

//endregion Fixtures

//region fetch_tor_relays.ts

/**
 Source text of the relay script, read once for the structural cases.
 */
const TOR_RELAY_SCRIPT_SOURCE = await readFile(
  join(
    SOURCE_DIRECTORY,
    'fetch_tor_relays.ts',
  ),
  'utf8',
);

await describe({
  name: 'fetch_tor_relays.ts',
  children: [
    //region Script shape

    it({
      name: 'declares no single-call wrapper around the whole script',
      fn: async () => {
        expect(TOR_RELAY_SCRIPT_SOURCE.includes('function run(',),).toBe(false,);
        expect(TOR_RELAY_SCRIPT_SOURCE.includes('await run(',),).toBe(false,);
      },
    },),
    //endregion Script shape

    //region Cache and fetch branches

    it({
      name: 'emits the cached CIDR list without fetching while the cache is fresh',
      fn: async () => {
        await using staged = await stageScript('fetch_tor_relays.ts',);
        await writeRelayCache({
          directory: staged.directory,
          text: '203.0.113.1/32',
          ageMs: FRESH_CACHE_AGE_MS,
        },);

        /**
         Run with a fetch that fails loudly if the fresh cache did not short-circuit it.
         */
        const run = await staged.run({
          importUrl: fetchStubImportUrl(NEVER_CALLED_FETCH_SOURCE,),
        },);

        expect(run.exitCode,).toBe(0,);
        expect(JSON.parse(run.stdout,),).toEqual({ ips: '203.0.113.1/32', },);
        expect(run.stderr,).toBe('',);
      },
    },),

    it({
      name: 'filters a fresh Onionoo response to ORPort 443 and writes the cache',
      fn: async () => {
        await using staged = await stageScript('fetch_tor_relays.ts',);

        /**
         Run against the fixture response instead of the network.
         */
        const run = await staged.run({
          importUrl: fetchStubImportUrl(
            `async () => ({ json: async () => (${ONIONOO_RESPONSE_SOURCE}) })`,
          ),
        },);

        expect(run.exitCode,).toBe(0,);
        expect(JSON.parse(run.stdout,),).toEqual({ ips: EXPECTED_FRESH_CIDRS, },);
        expect(
          await readFile(
            join(
              staged.directory,
              TOR_RELAY_CACHE_FILE_NAME,
            ),
            'utf8',
          ),
        ).toBe(EXPECTED_FRESH_CIDRS,);
      },
    },),

    it({
      name: 'replays the stale cache when the fetch fails',
      fn: async () => {
        await using staged = await stageScript('fetch_tor_relays.ts',);
        await writeRelayCache({
          directory: staged.directory,
          text: STALE_CACHE_CIDRS,
          ageMs: STALE_CACHE_AGE_MS,
        },);

        /**
         Run against a fetch that always fails, so only the fallback can answer.
         */
        const run = await staged.run({
          importUrl: fetchStubImportUrl(REJECTING_FETCH_SOURCE,),
        },);

        expect(run.exitCode,).toBe(0,);
        expect(JSON.parse(run.stdout,),).toEqual({ ips: STALE_CACHE_CIDRS, },);
      },
    },),

    it({
      name: 'reports a failed fetch as a tagged warn line and keeps stdout JSON',
      fn: async () => {
        await using staged = await stageScript('fetch_tor_relays.ts',);
        await writeRelayCache({
          directory: staged.directory,
          text: STALE_CACHE_CIDRS,
          ageMs: STALE_CACHE_AGE_MS,
        },);

        /**
         Failed-fetch run that must still answer with the stale cache.
         */
        const run = await staged.run({
          importUrl: fetchStubImportUrl(REJECTING_FETCH_SOURCE,),
        },);

        expect(run.exitCode,).toBe(0,);
        expect(JSON.parse(run.stdout,),).toEqual({ ips: STALE_CACHE_CIDRS, },);
        expect(run.stderr.includes('fetch_tor_relays: '),).toBe(false,);

        /**
         Warn lines explaining the failed fetch.
         */
        const diagnostics = run.stderr
          .split('\n',)
          .filter(function isFallbackLine(line,): boolean {
            return line.includes('attempting cache fallback',);
          },);
        expect(diagnostics,).toHaveLength(1,);
        for (const line of diagnostics) {
          expectTaggedWarning({
            functionTag: 'fetchCidrsWithCacheFallback',
            line,
            moduleTag: 'fetch_tor_relays',
          },);
          expect(line.includes('] live fetch failed, attempting cache fallback: '),).toBe(true,);
        }
      },
    },),

    it({
      name: 'fails without output when the fetch fails and no cache exists',
      fn: async () => {
        await using staged = await stageScript('fetch_tor_relays.ts',);

        /**
         Run against a fetch that always fails with no cache file to fall back to.
         */
        const run = await staged.run({
          importUrl: fetchStubImportUrl(REJECTING_FETCH_SOURCE,),
        },);

        expect(run.exitCode,).toBe(1,);
        expect(run.stdout,).toBe('',);
        expect(run.stderr,).toContain('fetch failed and no cached fallback available',);
      },
    },),
    //endregion Cache and fetch branches
  ],
},);

//endregion fetch_tor_relays.ts

//region resolve_hosts.ts

await describe({
  name: 'resolve_hosts.ts',
  children: [
    it({
      name: 'writes one tagged warn line per address family, with no name prefix',
      fn: async () => {
        await using staged = await stageScript('resolve_hosts.ts',);

        /**
         Run for a hostname whose every lookup fails locally.
         */
        const run = await staged.run({
          stdin: JSON.stringify({ hostnames: UNRESOLVABLE_HOSTNAME, },),
        },);

        expect(run.exitCode,).toBe(1,);
        expect(run.stdout,).toBe('',);
        expect(run.stderr.includes('resolve_hosts: '),).toBe(false,);

        /**
         Warn lines reporting the per-family lookup failures.
         */
        const diagnostics = lookupDiagnosticLines(run.stderr,);
        expect(diagnostics,).toHaveLength(ADDRESS_FAMILIES.length,);
        for (const line of diagnostics) {
          expectTaggedWarning({
            functionTag: 'resolveAddressFamily',
            line,
            moduleTag: 'resolve_hosts',
          },);
        }
        for (const family of ADDRESS_FAMILIES) {
          expect(diagnostics.some(function isFamilyLine(line,): boolean {
            return line.includes(`] ${family} lookup for `,);
          },),).toBe(true,);
        }
      },
    },),
  ],
},);

//endregion resolve_hosts.ts

//region resolve_storagebox_hosts.ts

await describe({
  name: 'resolve_storagebox_hosts.ts',
  children: [
    it({
      name: 'writes one tagged warn line per address family, with no name prefix',
      fn: async () => {
        await using staged = await stageScript('resolve_storagebox_hosts.ts',);

        /**
         Run for a Storage Box hostname whose every lookup fails locally.
         */
        const run = await staged.run({
          stdin: JSON.stringify({ hostnames: UNRESOLVABLE_STORAGEBOX_HOSTNAME, },),
        },);

        expect(run.exitCode,).toBe(1,);
        expect(run.stdout,).toBe('',);
        expect(run.stderr.includes('resolve_storagebox_hosts: '),).toBe(false,);

        /**
         Warn lines reporting the per-family lookup failures.
         */
        const diagnostics = lookupDiagnosticLines(run.stderr,);
        expect(diagnostics,).toHaveLength(ADDRESS_FAMILIES.length,);
        for (const line of diagnostics) {
          expectTaggedWarning({
            functionTag: 'resolveAddressFamily',
            line,
            moduleTag: 'resolve_storagebox_hosts',
          },);
        }
        for (const family of ADDRESS_FAMILIES) {
          expect(diagnostics.some(function isFamilyLine(line,): boolean {
            return line.includes(`] ${family} lookup for `,);
          },),).toBe(true,);
        }
      },
    },),
  ],
},);

//endregion resolve_storagebox_hosts.ts
