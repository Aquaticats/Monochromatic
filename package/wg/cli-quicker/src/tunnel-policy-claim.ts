//! The policy table and the suppression preference one interface's tunnel owns.

import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  readFile,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  BypassStateError,
  isErrnoException,
} from './errors.ts';
import {
  bypassRuntimeDirectory,
  bypassStateKey,
} from './tunnel-bypass-path.ts';

/**
 Sentinel standing for an interface that has no claim.
 */
export const POLICY_CLAIM_ABSENT: unique symbol = Symbol(
  'this interface has no policy claim record',
);

/**
 Address family whose rules a claim covers.
 */
export type ClaimFamily = '-4' | '-6';

/**
 What one interface's tunnel owns in the policy routing.

 The claim is written before the fwmark is set and before any rule exists,
 and it is what a later allocator reads to tell a free table from one a lifecycle
 has taken but not yet made visible in the kernel.
 Without it, a start killed between `wg set fwmark` and its first `ip rule add`
 left a table that looked free to the next start, two tunnels shared one table,
 and the `down` of the dead one deleted the live one's rules.

 The suppression preference is the exact identity of this tunnel's
 `table main suppress_prefixlength 0` rule, which carries no interface and no table
 and therefore cannot be told apart from another tunnel's by its shape.
 */
export type PolicyClaim = {
  /**
   Version of the record.
   */
  readonly version: 1;

  /**
   Interface the claim belongs to.
   */
  readonly interfaceName: string;

  /**
   Policy table carried in the interface fwmark.
   */
  readonly table: number;

  /**
   Preference of this tunnel's main-table suppression rule,
   whose `not fwmark` rule sits at the next preference.
   */
  readonly suppressionPreference: number;

  /**
   Families whose rules the tunnel installed.
   */
  readonly families: readonly ClaimFamily[];
};

/**
 Suffix of every claim record, which is what makes the runtime directory's claims findable.
 */
export const CLAIM_SUFFIX = '.policy.json';

/**
 Resolves the claim record path of one interface.

 @param interfaceName - Interface identity.

 @returns Path under the runtime directory.

 @example
 ```ts
 policyClaimPath({ interfaceName: 'wg0' });
 ```
 */
export function policyClaimPath({ interfaceName, }: { readonly interfaceName: string; },): string {
  return join(
    bypassRuntimeDirectory(),
    `tunnel-${bypassStateKey({ interfaceName, },)}${CLAIM_SUFFIX}`,
  );
}

/**
 Reports whether a parsed JSON value is an object whose fields can be read.

 @param value - Parsed JSON value.

 @returns Whether it is a record.

 @example
 ```ts
 isRecord({ table: 51820 }); // true
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Parses record text as a record with readable fields.

 @param text - Record text.

 @param path - Path of the record, for the message.

 @returns The record.

 @throws {@link BypassStateError} when the text is not a JSON object.

 @example
 ```ts
 parseRecord({ text: '{"version":1}', path: '/run/wg-quicker/tunnel-abc.policy.json' });
 ```
 */
export function parseRecord(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): Readonly<Record<string, unknown>> {
  /**
   Parsed record.
   */
  const parsed = parseJson({
    text,
    path,
  },);
  if (!isRecord(parsed,))
    throw new BypassStateError(`tunnel policy claim is not an object: ${path}`,);
  return parsed;
}

/**
 Parses claim text.

 @param text - Record text.

 @param path - Path it was read from, for the message.

 @returns The parsed value, whatever its shape.

 @throws {@link BypassStateError} when the text is not JSON.
 */
function parseJson(
  {
    text,
    path,
  }: {
    readonly text: string;
    readonly path: string;
  },
): unknown {
  try {
    return JSON.parse(text,);
  }
  catch (error: unknown) {
    throw new BypassStateError(
      `tunnel policy claim is not JSON: ${path}`,
      { cause: error, },
    );
  }
}

/**
 Reads a claim record and checks its shape.

 @param text - Record text.

 @param path - Path it was read from, for the message.

 @param interfaceName - Interface the record must name.

 @returns The claim.

 @throws {@link BypassStateError} when the record is not a claim of this interface.

 @example
 ```ts
 parseClaimEntry({ text: claimText, path: '/run/wg-quicker/tunnel-abc.policy.json', interfaceName: 'wg0' });
 ```
 */
export function parseClaimEntry(
  {
    text,
    path,
    interfaceName,
  }: {
    readonly text: string;
    readonly path: string;
    readonly interfaceName: string;
  },
): PolicyClaim {
  /**
   Parsed record, whose shape is checked below.
   */
  const parsed = parseRecord({
    text,
    path,
  },);
  /**
   Record fields the claim is built from.
   */
  const {
    version,
    interfaceName: named,
    table,
    suppressionPreference,
    families,
  } = parsed;
  if (version !== 1)
    throw new BypassStateError(`tunnel policy claim has an unknown version: ${path}`,);
  if (named !== interfaceName)
    throw new BypassStateError(`tunnel policy claim names another interface: ${path}`,);
  if (((typeof table) !== 'number') || (!Number.isSafeInteger(table,))
    || (table <= 0))
    throw new BypassStateError(`tunnel policy claim names no usable table: ${path}`,);
  if (((typeof suppressionPreference) !== 'number') || (!Number.isSafeInteger(suppressionPreference,))
    || (suppressionPreference <= 0))
    throw new BypassStateError(`tunnel policy claim names no usable suppression preference: ${path}`,);
  if ((!Array.isArray(families,)) || (families.length === 0))
    throw new BypassStateError(`tunnel policy claim names no address family: ${path}`,);
  /**
   Families after checking each one.
   */
  const checked: ClaimFamily[] = [];
  for (const family of families as readonly unknown[]) {
    if ((family !== '-4') && (family !== '-6'))
      throw new BypassStateError(`tunnel policy claim names an unknown address family: ${path}`,);
    checked.push(family,);
  }
  return {
    version: 1,
    interfaceName,
    table,
    suppressionPreference,
    families: checked,
  };
}

/**
 Reads a claim file's text.

 @param path - Path of the record.

 @returns The text, or {@link POLICY_CLAIM_ABSENT} when no file is there.

 @throws What the read threw, other than the absence of the file.

 @example
 ```ts
 await readClaimText({ path: '/run/wg-quicker/tunnel-abc.policy.json' });
 ```
 */
async function readClaimText(
  { path, }: { readonly path: string; },
): Promise<string | typeof POLICY_CLAIM_ABSENT> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return POLICY_CLAIM_ABSENT;
    throw error;
  }
}

/**
 Reads the claim of one interface.

 @param interfaceName - Interface identity.

 @returns The claim, or {@link POLICY_CLAIM_ABSENT}.

 @throws {@link BypassStateError} when a record is there but is not a claim of this interface.

 @example
 ```ts
 const claim = await readPolicyClaim({ interfaceName: 'wg0' });
 ```
 */
export async function readPolicyClaim(
  { interfaceName, }: { readonly interfaceName: string; },
): Promise<PolicyClaim | typeof POLICY_CLAIM_ABSENT> {
  /**
   Path of this interface's claim.
   */
  const path = policyClaimPath({ interfaceName, },);
  /**
   Record text, or the absence of one.
   */
  const text = await readClaimText({ path, },);
  if (text === POLICY_CLAIM_ABSENT)
    return POLICY_CLAIM_ABSENT;
  return parseClaimEntry({
    text,
    path,
    interfaceName,
  },);
}

/**
 Writes one claim so that a kill leaves the whole record or none of it.

 @param claim - Claim to persist.

 @example
 ```ts
 await writePolicyClaim({ claim });
 ```
 */
export async function writePolicyClaim(
  { claim, }: { readonly claim: PolicyClaim; },
): Promise<void> {
  await mkdir(
    bypassRuntimeDirectory(),
    {
      mode: 0o700,
      recursive: true,
    },
  );
  /**
   Path of the record.
   */
  const path = policyClaimPath({ interfaceName: claim.interfaceName, },);
  /**
   Temporary path beside it, renamed over it.
   */
  const temporary = `${path}.${randomUUID()}.tmp`;
  try {
    await writeFile(
      temporary,
      JSON.stringify(claim,),
      {
        flag: 'wx',
        mode: 0o600,
      },
    );
    await rename(
      temporary,
      path,
    );
  }
  catch (error) {
    await rm(
      temporary,
      { force: true, },
    );
    throw error;
  }
}

/**
 Removes the claim of one interface, and only when it still names this table.

 A rollback and a `down` both call this, and neither may remove a claim that a
 later lifecycle of the same interface wrote after a reuse of the table.

 @param claim - Claim being released.

 @example
 ```ts
 await releasePolicyClaim({ claim });
 ```
 */
export async function releasePolicyClaim(
  { claim, }: { readonly claim: PolicyClaim; },
): Promise<void> {
  /**
   The record as it stands.
   */
  const current = await readPolicyClaim({ interfaceName: claim.interfaceName, },);
  if (current === POLICY_CLAIM_ABSENT)
    return;
  if (current.table !== claim.table)
    return;
  await rm(
    policyClaimPath({ interfaceName: claim.interfaceName, },),
    { force: true, },
  );
}
