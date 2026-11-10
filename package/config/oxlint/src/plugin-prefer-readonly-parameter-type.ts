/**
 Sidecar entry for semantic readonly parameter plugin.
 
 Re-exports plugin TypeScript source through `/ts` so `rolldown` inlines it into
 co-located built configuration sidecar.
 */

export { default, } from '@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type/ts';
