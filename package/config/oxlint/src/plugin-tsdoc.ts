/**
 Sidecar entry for the bundled TSDoc oxlint plugin.
 
 Re-exports the plugin's TypeScript source via its `/ts` subpath so `rolldown`
 inlines it (and its workspace deps) into `dist/final/node/plugin-tsdoc.mjs`,
 which `index.node.ts` references by relative `file://` URL.
 */

export { default, } from '@monochromatic-dev/oxlint-plugin-tsdoc/ts';
