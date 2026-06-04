import {
  defineConfig,
  type UserConfig,
} from 'tsdown';

import base from './index.node.ts';
import { unbundledDtsPlugin, } from './unbundled-dts-plugin.ts';

/**
 * Shared tsdown configuration for Node.js builds that emit unbundled `.d.ts` assets.
 *
 * Use for application, CLI, or extension packages that expose local types but do not
 * require a single bundled declaration entry for npm library consumers.
 *
 * @example
 * ```ts
 * // tsdown.node.config.ts
 * export { default, } from '\@monochromatic-dev/config-tsdown/.node.unbundled-dts.ts';
 * ```
 */
const _default_1: UserConfig = defineConfig({
  ...base,
  dts: false,
  plugins: [
    base.plugins,
    unbundledDtsPlugin(),
  ],
},);
export default _default_1;
