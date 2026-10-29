/**
 Unit tests for the image resolver against a stubbed `fetch`: newest
 non-deprecated system image per OS flavor, exclusion of deprecated and
 snapshot images, literal passthrough, and unsupported-flavor errors.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { resolveHetznerImage, } from '@monochromatic-dev/mvm-cli/ts/backend/hetzner/images.ts';

/**
 Sets HCLOUD_TOKEN for a `using` scope so requireToken passes, restoring after.
 */
function withToken(): Disposable {
  const prior = process.env.HCLOUD_TOKEN;
  process.env.HCLOUD_TOKEN = 'tok-test';
  return {
    [Symbol.dispose]() {
      if (prior === undefined) {
        delete process.env.HCLOUD_TOKEN;
      }
      else {
        process.env.HCLOUD_TOKEN = prior;
      }
    },
  };
}

/**
 Replaces global fetch with a fixed JSON body; restores on dispose.
 */
function installImages(images: readonly unknown[],): Disposable {
  const original = globalThis.fetch;
  globalThis.fetch = (async function stubFetch() {
    return Response.json(
      { images, meta: { pagination: { next_page: null, }, }, },
      { status: 200, },
    );
  }) as unknown as typeof fetch;
  return {
    [Symbol.dispose]() {
      globalThis.fetch = original;
    },
  };
}

/**
 System and snapshot images with mixed flavors, deprecation, and null names.
 */
const IMAGES = [
  { id: 1, name: 'ubuntu-22.04', os_flavor: 'ubuntu', type: 'system', deprecated: null, created: '2024-04-01T00:00:00Z', description: '', },
  { id: 2, name: 'ubuntu-24.04', os_flavor: 'ubuntu', type: 'system', deprecated: null, created: '2024-04-25T00:00:00Z', description: '', },
  { id: 3, name: 'ubuntu-25.10', os_flavor: 'ubuntu', type: 'system', deprecated: '2026-02-05T00:00:00Z', created: '2025-10-01T00:00:00Z', description: '', },
  { id: 4, name: null, os_flavor: 'ubuntu', type: 'snapshot', deprecated: null, created: '2026-01-01T00:00:00Z', description: 'snap', },
  { id: 5, name: 'fedora-44', os_flavor: 'fedora', type: 'system', deprecated: null, created: '2026-05-18T00:00:00Z', description: '', },
];

await describe({
  name: 'hetzner image resolver',
  concurrency: 1,
  children: [
    it({
      name: 'resolves a flavor to its newest non-deprecated system image',
      fn: async () => {
        using _t = withToken();
        using _mock = installImages(IMAGES,);
        expect(await resolveHetznerImage({ shorthand: 'ubuntu', },),).toBe('ubuntu-24.04',);
        expect(await resolveHetznerImage({ shorthand: 'fedora', },),).toBe('fedora-44',);
      },
    },),
    it({
      name: 'throws for a flavor with no usable image',
      fn: async () => {
        using _t = withToken();
        using _mock = installImages([],);
        await expect(resolveHetznerImage({ shorthand: 'ubuntu', },),).rejects.toThrow(
          'no non-deprecated',
        );
      },
    },),
    it({
      name: 'passes an unrecognised value through as a literal slug',
      fn: async () => {
        using _t = withToken();
        expect(await resolveHetznerImage({ shorthand: 'ubuntu-22.04', },),).toBe('ubuntu-22.04',);
      },
    },),
    it({
      name: 'throws for alpine and windows, which Hetzner does not offer',
      fn: async () => {
        using _t = withToken();
        await expect(resolveHetznerImage({ shorthand: 'alpine', },),).rejects.toThrow('unsupported',);
        await expect(resolveHetznerImage({ shorthand: 'windows', },),).rejects.toThrow('unsupported',);
      },
    },),
  ],
},);
