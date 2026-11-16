/**
 Every claim in the runtime directory, and what the claims other interfaces hold.

 A claim is what keeps a table and a suppression preference out of another tunnel's
 reach between the moment they are chosen and the moment the kernel shows them,
 so the scan must see every record, and a record it cannot read is a refusal rather
 than a guess.

 @module
 */

import {
  readFile,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  BypassStateError,
  isErrnoException,
} from './errors.ts';
import { bypassRuntimeDirectory, } from './tunnel-bypass-path.ts';
import {
  CLAIM_SUFFIX,
  parseClaimEntry,
  parseRecord,
  type PolicyClaim,
} from './tunnel-policy-claim.ts';

/**
 Reads every claim in the runtime directory.

 A claim this program cannot read is reported, not guessed around:
 allocating its table to another tunnel is how one tunnel's `down` came to delete another's rules.

 @returns The claims.

 @throws {@link BypassStateError} when a record cannot be read as a claim.

 @example
 ```ts
 await readAllPolicyClaims();
 ```
 */
export async function readAllPolicyClaims(): Promise<readonly PolicyClaim[]> {
  /**
   Directory the claims live in.
   */
  const directory = bypassRuntimeDirectory();
  /**
   Names of the directory's entries.
   */
  const names = await claimNames({ directory, },);
  return await Promise.all(names.map(function read(entry,): Promise<PolicyClaim> {
    return readClaimEntry({
      directory,
      entry,
    },);
  },),);
}

/**
 Lists the claim records in the runtime directory.

 @param directory - Directory the claims live in.

 @returns The entries that are claim records, in the order the directory lists them.

 @throws What the listing threw, other than the absence of the directory.
 */
async function claimNames(
  { directory, }: { readonly directory: string; },
): Promise<readonly string[]> {
  try {
    return (await readdir(directory,)).filter(function isClaim(entry,): boolean {
      return entry.endsWith(CLAIM_SUFFIX,);
    },);
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return [];
    throw error;
  }
}

/**
 Reads one claim record from the directory.

 @param directory - Directory the claims live in.

 @param entry - The entry to read.

 @returns The claim.

 @throws {@link BypassStateError} when the record is not a claim.
 */
async function readClaimEntry(
  {
    directory,
    entry,
  }: {
    readonly directory: string;
    readonly entry: string;
  },
): Promise<PolicyClaim> {
  /**
   Path of the record.
   */
  const path = join(
    directory,
    entry,
  );
  /**
   Record text.
   */
  const text = await readFile(
    path,
    'utf8',
  );
  /**
   Interface name the record claims to name, which the record itself checks.
   */
  const interfaceName = claimedInterfaceName({
    text,
    path,
  },);
  return parseClaimEntry({
    text,
    path,
    interfaceName,
  },);
}

/**
 Reads the interface a record names, without checking the rest of its shape.

 @param text - Record text.

 @param path - Path of the record, for the message.

 @returns The interface name.

 @throws {@link BypassStateError} when the record names none.
 */
function claimedInterfaceName(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): string {
  /**
   Parsed record.
   */
  const parsed = parseRecord({
    text,
    path,
  },);
  /**
   Interface name it carries, when it carries one.
   */
  const named = parsed.interfaceName;
  if ((typeof named) !== 'string')
    throw new BypassStateError(`tunnel policy claim names no interface: ${path}`,);
  return named;
}

/**
 The tables and suppression preferences other interfaces have claimed.

 @param interfaceName - Interface whose own claim is left out.

 @returns What the other claims hold.

 @example
 ```ts
 const taken = await policyResourcesTakenByOthers({ interfaceName: 'wg0' });
 ```
 */
export async function policyResourcesTakenByOthers(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<{
  readonly tables: ReadonlySet<number>;
  readonly preferences: ReadonlySet<number>;
}> {
  /**
   Every claim in the runtime directory.
   */
  const claims = await readAllPolicyClaims();
  /**
   Tables other interfaces hold.
   */
  const tables = new Set<number>();
  /**
   Preferences other interfaces hold, both of each pair.
   */
  const preferences = new Set<number>();
  for (const claim of claims) {
    if (claim.interfaceName === interfaceName)
      continue;
    tables.add(claim.table,);
    preferences.add(claim.suppressionPreference,);
    preferences.add(claim.suppressionPreference + 1,);
  }
  return {
    tables,
    preferences,
  };
}
