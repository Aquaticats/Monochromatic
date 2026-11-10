/**
 Sidecar entry for the bundled no-restricted-syntax oxlint plugin.
 
 Re-exports the plugin's TypeScript source via its `/ts` subpath so `rolldown`
 inlines it into `dist/final/node/plugin-no-restricted-syntax.mjs`, which
 `index.node.ts` references by relative `file://` URL.
 */

export { default, } from '@monochromatic-dev/oxlint-plugin-no-restricted-syntax/ts';
