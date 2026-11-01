import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  resetApiKeyCache,
  resolveMorphApiKey,
} from './api-key.ts';

/** Set an env var and return a disposable that restores the original value. */
function setEnv({
  key,
  value,
}: {
  key: string;
  value: string;
},): Disposable {
  /** Captured pre-test value; absent (undefined) means the var was unset. */
  const original = process.env[key];
  process.env[key] = value;
  resetApiKeyCache();
  return {
    [Symbol.dispose](): void {
      if (original === undefined)
        Reflect.deleteProperty(process.env, key,);
      else
        process.env[key] = original;
      resetApiKeyCache();
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: resolveMorphApiKey.name,
      children: [
        it({
          name: 'resolves MORPH_API_KEY from environment',
          fn: async () => {
            using _restore = setEnv({ key: 'MORPH_API_KEY',
              value: 'test-key-from-env', },);
            const key = await resolveMorphApiKey();
            expect(key,).toBe('test-key-from-env',);
          },
        },),
        it({
          name: 'returns env value directly when set',
          fn: async () => {
            using _restore = setEnv({ key: 'MORPH_API_KEY', value: 'cached-key', },);
            const first = await resolveMorphApiKey();
            // Env var takes priority over cache
            process.env.MORPH_API_KEY = 'new-key';
            const second = await resolveMorphApiKey();
            // Env var always wins over cache
            expect(second,).toBe('new-key',);
          },
        },),
        it({
          name: 'resetApiKeyCache clears the cache',
          fn: async () => {
            using _restore = setEnv({ key: 'MORPH_API_KEY', value: 'before-reset', },);
            await resolveMorphApiKey();
            process.env.MORPH_API_KEY = 'after-reset';
            resetApiKeyCache();
            const key = await resolveMorphApiKey();
            expect(key,).toBe('after-reset',);
          },
        },),
      ],
    },),
    describe({
      name: resetApiKeyCache.name,
      children: [
        it({
          name: 'allows re-reading after cache reset',
          fn: async () => {
            using _restore = setEnv({ key: 'MORPH_API_KEY', value: 'initial', },);
            await resolveMorphApiKey();
            process.env.MORPH_API_KEY = 'post-reset';
            resetApiKeyCache();
            const key = await resolveMorphApiKey();
            expect(key,).toBe('post-reset',);
          },
        },),
      ],
    },),
  ],
},);
