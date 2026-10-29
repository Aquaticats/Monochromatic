/**
 Managed SSH keypair for the Hetzner backend.
 
 Generates a local ed25519 keypair on first use and ensures it exists in the
 Hetzner project, matched by public-key material (not by name) so an unrelated
 key already named `mvm` is never reused. Returns the numeric key id to inject
 at server creation.
 
 @module
 */

import {
  access,
  mkdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { splitOnWhitespace, } from '../../list.ts';
import { spawn, } from '../../spawn.ts';
import {
  createSshKey,
  listSshKeys,
} from './api-resources.ts';
import {
  HETZNER_DATA_DIR,
  OPENSSH_REMEDY,
} from './config.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Key paths

/**
 Path to the managed ed25519 private key.
 */
export const PRIVATE_KEY_PATH: string = join(
  HETZNER_DATA_DIR,
  'id_ed25519',
);

/**
 Path to the managed ed25519 public key.
 */
const PUBLIC_KEY_PATH = `${PRIVATE_KEY_PATH}.pub`;

/**
 Number of trailing key-material characters used to name an uploaded key.
 */
const NAME_SUFFIX_LENGTH = 12;

//endregion Key paths

//region Keypair generation

/**
 Ensures the managed keypair exists on disk, generating it if absent.
 
 @example
 ```ts
 await ensureKeypair();
 ```
 */
async function ensureKeypair(): Promise<void> {
  /**
   Logger scoped to this helper so generation is namespaced.
   */
  const rl = tagged({
    tag: ensureKeypair.name,
    l,
  },);
  try {
    await access(PRIVATE_KEY_PATH,);
    return;
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw error;

    rl.info('generating managed ed25519 keypair for hetzner backend',);
  }
  await mkdir(
    HETZNER_DATA_DIR,
    { recursive: true, },
  );
  await spawn({
    command: 'ssh-keygen',
    args: [
      '-t',
      'ed25519',
      '-f',
      PRIVATE_KEY_PATH,
      '-N',
      '',
      '-C',
      'mvm',
      '-q',
    ],
    notFoundRemedy: OPENSSH_REMEDY,
  },);
}

//endregion Keypair generation

//region Material matching

/**
 Extracts the type-and-base64 material of a public key line, dropping the
 trailing comment so two keys with the same key but different comments match.
 
 @param publicKeyLine - full public key line
 
 @returns `"<type> <base64>"`, or the trimmed input when it has fewer fields
 
 @example
 ```ts
 keyMaterial('ssh-ed25519 AAAAC3... mvm'); // 'ssh-ed25519 AAAAC3...'
 ```
 */
function keyMaterial(publicKeyLine: string,): string {
  /**
   Whitespace-separated fields of the key line: type, base64, optional comment.
   */
  const [type, base64,] = splitOnWhitespace(publicKeyLine,);
  if ((type === undefined) || (base64 === undefined)) {
    return publicKeyLine.trim();
  }
  return `${type} ${base64}`;
}

/**
 Checks whether `c` is ASCII alphanumeric.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is `[A-Za-z0-9]`
 
 @example
 ```ts
 isAlphaNum('a'); // true
 isAlphaNum('/'); // false
 ```
 */
function isAlphaNum(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'))
    || ((c >= '0') && (c <= '9'));
}

/**
 Derives a collision-resistant key name from key material, using only
 alphanumeric characters so the Hetzner key name is always valid.
 
 @param material - the `"<type> <base64>"` material
 
 @returns key name like `mvm-<suffix>`
 
 @example
 ```ts
 uniqueKeyName('ssh-ed25519 AAAAC3NzaC1lZDI1NTE5'); // 'mvm-zaC1lZDI1NTE5'
 ```
 */
function uniqueKeyName(material: string,): string {
  /**
   Alphanumeric-only characters of the material, in order; built with a
   for...of loop to avoid spreading a string into an array.
   */
  const alnum: string[] = [];
  for (const c of material) {
    if (isAlphaNum(c,)) {
      alnum.push(c,);
    }
  }
  /**
   Trailing slice used as the suffix, or a constant fallback when empty.
   */
  const suffix = alnum.slice(-NAME_SUFFIX_LENGTH,)
    .join('',);
  return `mvm-${suffix === '' ? 'key' : suffix}`;
}

//endregion Material matching

//region Key id resolution

/**
 Ensures the managed keypair exists locally and in the Hetzner project, then
 returns the numeric key id to inject at server creation. Matches an existing
 project key by public-key material; uploads a uniquely named key otherwise.
 
 @returns numeric Hetzner SSH key id
 
 @example
 ```ts
 const sshKeyId = await ensureSshKeyId();
 ```
 */
export async function ensureSshKeyId(): Promise<number> {
  await ensureKeypair();
  /**
   Managed public key line read from disk.
   */
  const publicKey = (await readFile(
    PUBLIC_KEY_PATH,
    'utf8',
  )).trim();
  /**
   Material of our key, used to match a project key regardless of its name.
   */
  const ourMaterial = keyMaterial(publicKey,);
  /**
   Project key whose material matches ours, when one already exists.
   */
  const existing = (await listSshKeys()).find(function sameMaterial(key,) {
    return keyMaterial(key.public_key,) === ourMaterial;
  },);
  if (existing !== undefined) {
    return existing.id;
  }
  return (await createSshKey({
    name: uniqueKeyName(ourMaterial,),
    publicKey,
  },)).id;
}

//endregion Key id resolution
