/**
 Identity resolution and curated child-environment tests.

 These tests use plain identifier files with synthetic placeholder text; no
 real age key is needed to resolve a path or build the child environment.

 @module
 */

import { mkdir, writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createTempRoot, } from './fixture.ts';
import {
  EnvLocalJsonMissingIdentityError,
} from '../dist/final/node/index.mjs';
import {
  buildSopsChildEnv,
  resolveSopsIdentity,
} from '../dist/final/node/testing.mjs';

//region Fixtures

/**
 Synthetic inline age key text; not a usable credential.
 */
const INLINE_KEY = 'AGE-SECRET-KEY-1TESTSYNTHETICINLINEVALUE';

/**
 Disposable fixture root shared by the cases.
 */
const temp = await createTempRoot({ prefix: 'env-local-json-identity-', },);

/**
 Existing identity file named by the mise variable.
 */
const miseIdentityPath = join(temp.root, 'mise-identity.txt',);
await writeFile(miseIdentityPath, INLINE_KEY, 'utf8',);

/**
 Existing identity file named by the sops variable.
 */
const sopsIdentityPath = join(temp.root, 'sops-identity.txt',);
await writeFile(sopsIdentityPath, INLINE_KEY, 'utf8',);

//endregion Fixtures

//region Helpers

/**
 Captures the rejection of an identity resolution.

 @param options - environment and home passed to the resolver

 @returns rejection value

 @throws Error when the resolver unexpectedly resolves

 @example
 ```ts
 const error = await captureIdentityError({ env: {}, home: temp.root });
 ```
 */
async function captureIdentityError({
  env,
  home,
}: {
  readonly env: Readonly<Record<string, string>>;
  readonly home: string;
},): Promise<unknown> {
  try {
    await resolveSopsIdentity({
      env,
      home,
    },);
  }
  catch (error) {
    return error;
  }
  throw new Error('expected identity resolution to fail',);
}

//endregion Helpers

//region Resolution order

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Resolved identity for an inline mise key and an existing file.
         */
        const identity = await resolveSopsIdentity({
          env: {
            MISE_SOPS_AGE_KEY: INLINE_KEY,
            MISE_SOPS_AGE_KEY_FILE: miseIdentityPath,
          },
          home: temp.root,
        },);
        expect(identity.kind,).toBe('inline',);
        expect(identity.source,).toBe('MISE_SOPS_AGE_KEY',);
        expect(identity.value,).toBe(INLINE_KEY,);
      },
      name: 'MISE_SOPS_AGE_KEY wins over every file candidate',
    },),
    it({
      fn: async () => {
        /**
         Resolved identity when both file variables name existing files.
         */
        const identity = await resolveSopsIdentity({
          env: {
            MISE_SOPS_AGE_KEY_FILE: miseIdentityPath,
            SOPS_AGE_KEY_FILE: sopsIdentityPath,
          },
          home: temp.root,
        },);
        expect(identity.kind,).toBe('file',);
        expect(identity.source,).toBe('MISE_SOPS_AGE_KEY_FILE',);
        expect(identity.value,).toBe(miseIdentityPath,);
      },
      name: 'existing MISE_SOPS_AGE_KEY_FILE wins over SOPS_AGE_KEY_FILE',
    },),
    it({
      fn: async () => {
        /**
         Missing mise file path that must be skipped.
         */
        const missingMisePath = join(temp.root, 'missing-mise.txt',);
        /**
         Resolved identity after skipping the missing file.
         */
        const identity = await resolveSopsIdentity({
          env: {
            MISE_SOPS_AGE_KEY_FILE: missingMisePath,
            SOPS_AGE_KEY_FILE: sopsIdentityPath,
          },
          home: temp.root,
        },);
        expect(identity.source,).toBe('SOPS_AGE_KEY_FILE',);
        expect(identity.value,).toBe(sopsIdentityPath,);
      },
      name: 'missing MISE_SOPS_AGE_KEY_FILE is skipped',
    },),
    it({
      fn: async () => {
        /**
         Missing sops file path that must be skipped.
         */
        const missingSopsPath = join(temp.root, 'missing-sops.txt',);
        /**
         Resolved identity after skipping the missing file.
         */
        const identity = await resolveSopsIdentity({
          env: {
            SOPS_AGE_KEY_FILE: missingSopsPath,
            SOPS_AGE_KEY: INLINE_KEY,
          },
          home: temp.root,
        },);
        expect(identity.source,).toBe('SOPS_AGE_KEY',);
        expect(identity.value,).toBe(INLINE_KEY,);
      },
      name: 'missing SOPS_AGE_KEY_FILE is skipped in favor of SOPS_AGE_KEY',
    },),
  ],
  name: resolveSopsIdentity.name,
},);

//endregion Resolution order

//region Default mise paths

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Explicit mise configuration directory.
         */
        const configDir = join(temp.root, 'mise-config-dir',);
        await mkdir(configDir, { recursive: true, },);
        /**
         Default identity inside the explicit configuration directory.
         */
        const defaultPath = join(configDir, 'age.txt',);
        await writeFile(defaultPath, INLINE_KEY, 'utf8',);
        /**
         Resolved identity from the configuration directory.
         */
        const identity = await resolveSopsIdentity({
          env: { MISE_CONFIG_DIR: configDir, },
          home: temp.root,
        },);
        expect(identity.source,).toBe('mise-config-dir',);
        expect(identity.value,).toBe(defaultPath,);
      },
      name: 'honors MISE_CONFIG_DIR',
    },),
    it({
      fn: async () => {
        /**
         Explicit XDG configuration root.
         */
        const xdgRoot = join(temp.root, 'xdg-root',);
        /**
         Mise configuration directory under the XDG root.
         */
        const configDir = join(xdgRoot, 'mise',);
        await mkdir(configDir, { recursive: true, },);
        /**
         Default identity under the XDG root.
         */
        const defaultPath = join(configDir, 'age.txt',);
        await writeFile(defaultPath, INLINE_KEY, 'utf8',);
        /**
         Resolved identity from the XDG configuration root.
         */
        const identity = await resolveSopsIdentity({
          env: { XDG_CONFIG_HOME: xdgRoot, },
          home: temp.root,
        },);
        expect(identity.source,).toBe('xdg-config-home',);
        expect(identity.value,).toBe(defaultPath,);
      },
      name: 'honors XDG_CONFIG_HOME',
    },),
    it({
      fn: async () => {
        /**
         Fake home directory for the default configuration path.
         */
        const home = join(temp.root, 'home-default',);
        /**
         Default mise configuration directory under the fake home.
         */
        const configDir = join(home, '.config', 'mise',);
        await mkdir(configDir, { recursive: true, },);
        /**
         Default identity under the fake home.
         */
        const defaultPath = join(configDir, 'age.txt',);
        await writeFile(defaultPath, INLINE_KEY, 'utf8',);
        /**
         Resolved identity from the home-based default path.
         */
        const identity = await resolveSopsIdentity({
          env: {},
          home,
        },);
        expect(identity.source,).toBe('home-config',);
        expect(identity.value,).toBe(defaultPath,);
      },
      name: 'falls back to $HOME/.config/mise/age.txt',
    },),
  ],
  name: 'default mise identity paths',
},);

//endregion Default mise paths

//region Missing identity

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Fake home with no identity anywhere.
         */
        const home = join(temp.root, 'home-missing',);
        await mkdir(home, { recursive: true, },);
        /**
         Rejection raised when no candidate is usable.
         */
        const caught = await captureIdentityError({ env: {}, home, },);
        expect(caught,).toBeInstanceOf(EnvLocalJsonMissingIdentityError,);
        if (!(caught instanceof EnvLocalJsonMissingIdentityError))
          throw new Error('missing-identity rejection was not the named error',);
        expect(caught.candidates,).toHaveLength(5,);
        expect(caught.candidates.join('\n',),).toContain('MISE_SOPS_AGE_KEY',);
        expect(caught.candidates.join('\n',),).toContain('SOPS_AGE_KEY_FILE',);
        expect(caught.message,).not.toContain(INLINE_KEY,);
      },
      name: 'names every candidate and never echoes key material',
    },),
  ],
  name: 'missing identity',
},);

//endregion Missing identity

//region Child environment

await describe({
  children: [
    it({
      fn: async () => {
        /**
         Inline identity passed to the child environment.
         */
        const identity = await resolveSopsIdentity({
          env: { MISE_SOPS_AGE_KEY: INLINE_KEY, },
          home: temp.root,
        },);
        /**
         Curated child environment.
         */
        const childEnv = buildSopsChildEnv({
          env: {
            AWS_SECRET_ACCESS_KEY: 'not-allowed',
            EDITOR: 'vi',
            LANG: 'C.UTF-8',
            PATH: '/usr/bin',
            XDG_CONFIG_HOME: '/xdg',
          },
          home: temp.root,
          identity,
        },);
        expect(childEnv.SOPS_AGE_KEY,).toBe(INLINE_KEY,);
        expect(childEnv.SOPS_AGE_KEY_FILE,).toBeUndefined();
        expect(childEnv.PATH,).toBe('/usr/bin',);
        expect(childEnv.HOME,).toBe(temp.root,);
        expect(childEnv.LANG,).toBe('C.UTF-8',);
        expect(childEnv.XDG_CONFIG_HOME,).toBe('/xdg',);
        expect(childEnv.EDITOR,).toBeUndefined();
        expect(childEnv.AWS_SECRET_ACCESS_KEY,).toBeUndefined();
      },
      name: 'copies only curated names and the identity',
    },),
    it({
      fn: async () => {
        /**
         Interactive child environment with an explicit editor.
         */
        const childEnv = buildSopsChildEnv({
          env: { EDITOR: 'nano', VISUAL: 'nano', },
          home: temp.root,
          identity: {
            kind: 'file',
            source: 'SOPS_AGE_KEY_FILE',
            value: sopsIdentityPath,
          },
          interactive: true,
        },);
        expect(childEnv.SOPS_AGE_KEY_FILE,).toBe(sopsIdentityPath,);
        expect(childEnv.SOPS_AGE_KEY,).toBeUndefined();
        expect(childEnv.EDITOR,).toBe('nano',);
        expect(childEnv.VISUAL,).toBe('nano',);
      },
      name: 'forwards editor selection for interactive editing',
    },),
  ],
  name: buildSopsChildEnv.name,
},);

//endregion Child environment
