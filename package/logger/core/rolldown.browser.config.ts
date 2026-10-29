import neutralConfig from '@monochromatic-dev/config-rolldown/.ts';

/**
 Neutral build with the root entry plus the `./browser` subpath entry that
 ships the IndexedDB and OPFS sinks. The shared neutral config exposes no
 input-overriding factory (unlike `nodeConfig`), so its default export is
 spread and only `input` replaced. Rolldown's `neutral` platform resolves
 package `imports` with the `import` and `default` conditions only, so
 `#default-sinks` lands on the neutral list and no `node:` specifier enters
 this graph.
 */
const config: typeof neutralConfig = {
  ...neutralConfig,
  input: [
    './src/index.ts',
    './src/browser.ts',
  ],
};

export default config;
