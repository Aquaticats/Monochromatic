/**
 Managed SSH keypair for the ovh backend.

 Generates one ed25519 keypair on first use and makes sure the project holds it,
 matched by public-key material rather than by name so a key an operator already
 named something else is reused and a foreign key that happens to be named like
 mvm's is never injected. Returns the key identifier instance creation takes.

 The keypair lives in {@link ovhDataDir}, which {@link DATA_DIR_ENV} can move, so
 two OVHcloud accounts never share one private key.

 @module
 */

import {
  access,
  mkdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { splitOnWhitespace, } from '../../list.ts';
import { spawn, } from '../../spawn.ts';
import {
  createSshKey,
  listSshKeys,
} from './api-resources.ts';
import {
  OPENSSH_REMEDY,
  ovhDataDir,
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
 Name of the managed private key inside {@link ovhDataDir}.
 */
const KEY_FILE_NAME = 'id_ed25519';

/**
 Number of trailing key-material characters used to name an uploaded key.
 */
const NAME_SUFFIX_LENGTH = 12;

/**
 Path of the managed private key.
 
 @returns absolute path of the private key file
 
 @example
 ```ts
 privateKeyPath(); // '<data dir>/id_ed25519'
 ```
 */
export function privateKeyPath(): string {
  return join(
    ovhDataDir(),
    KEY_FILE_NAME,
  );
}

//endregion Key paths

//region Keypair generation

/**
 Makes sure the managed keypair exists on disk, generating it when it does not.
 
 @throws {@link ExecutableNotFoundError} when `ssh-keygen` does not exist
 
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
  /**
   Path of the private key this backend uses.
   */
  const keyPath = privateKeyPath();
  try {
    await access(keyPath,);
    return;
  }
  catch (error: unknown) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    rl.info(`generating managed ed25519 keypair in ${ovhDataDir()} for the ovh backend`,);
  }
  await mkdir(
    ovhDataDir(),
    { recursive: true, },
  );
  await spawn({
    command: 'ssh-keygen',
    args: [
      '-t',
      'ed25519',
      '-f',
      keyPath,
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
 comment so two keys with the same material but different comments match.
 
 @param publicKeyLine - full public key line
 
 @returns `"<type> <base64>"`, or the trimmed line when it has fewer fields
 
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
 alphanumeric characters so the project key name is always valid.
 
 @param material - the `"<type> <base64>"` material
 
 @returns key name like `mvm-ed25519AAAAC3`
 
 @example
 ```ts
 uniqueKeyName('ssh-ed25519 AAAAC3NzaC1lZDI1NTE5'); // 'mvm-zaC1lZDI1NTE5'
 ```
 */
function uniqueKeyName(material: string,): string {
  /**
   Alphanumeric characters of the material, in order.
   */
  const alnum: string[] = [];
  for (const c of material) {
    if (isAlphaNum(c,)) {
      alnum.push(c,);
    }
  }
  /**
   Trailing slice used as the suffix, or a constant when the material has none.
   */
  const suffix = alnum.slice(-NAME_SUFFIX_LENGTH,)
    .join('',);
  return `mvm-${suffix === '' ? 'key' : suffix}`;
}

//endregion Material matching

//region Key id resolution

/**
 Makes sure the managed keypair exists locally and in the project, then returns
 the key identifier to inject at creation. Matches an existing project key by
 public-key material; uploads a uniquely named key otherwise.
 
 @returns OVHcloud SSH key identifier
 
 @example
 ```ts
 const sshKeyId = await ensureSshKeyId();
 ```
 */
export async function ensureSshKeyId(): Promise<string> {
  await ensureKeypair();
  /**
   Managed public key line read from disk.
   */
  const publicKey = (await readFile(
    `${privateKeyPath()}.pub`,
    'utf8',
  )).trim();
  /**
   Material of this key, used to match a project key regardless of its name.
   */
  const ourMaterial = keyMaterial(publicKey,);
  /**
   Project key whose material matches, when one already exists.
   */
  const existing = (await listSshKeys()).find(function sameMaterial(key,) {
    return keyMaterial(key.publicKey,) === ourMaterial;
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
