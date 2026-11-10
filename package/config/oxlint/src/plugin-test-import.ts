/**
 Sidecar entry for the bundled test-import oxlint plugin.
 
 Re-exports the plugin's TypeScript source via its `/ts` subpath so `rolldown`
 inlines it (and its workspace deps) into `dist/final/node/plugin-test-import.mjs`,
 which `index.node.ts` references by relative `file://` URL.
 */

export { default, } from '@monochromatic-dev/oxlint-plugin-test-import/ts';
