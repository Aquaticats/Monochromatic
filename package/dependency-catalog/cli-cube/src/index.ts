/**
 Public surface of `@monochromatic-dev/dependency-catalog-cli-cube`.
 
 Re-exports the library-level entry points that tests and other
 tooling import directly. The CLI in `./cli.ts` is run via the
 `deps-cube` bin entry and is not re-exported here.
 
 @example
 ```ts
 import { readCatalog, createCache, probeAll, renderHtml } from '@monochromatic-dev/dependency-catalog-cli-cube';
 const entries = await readCatalog();
 const cache = createCache();
 const probes = await probeAll({ entries, cache });
 const html = await renderHtml({ probes });
 ```
 */

export {
  decodeAlias,
  readCatalog,
} from './catalog.ts';
export type { CatalogEntry, } from './catalog.ts';

export { createCache, } from './cache.ts';
export type { Cache, } from './cache.ts';

export { probeAll, } from './probe.ts';
export type {
  LicenseClass,
  PackageProbe,
  UnknownReason,
} from './probe.ts';

export { renderHtml, } from './render-html.ts';

export { renderControls, } from './render-controls.ts';
