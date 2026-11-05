/**
 CI build tasks for @monochromatic-dev/oxlint-plugin-no-restricted-syntax.

 Mirrors the build tasks in this package's mise.toml for the current pnpr publisher.
 The repository's task-system replacement remains a separate proposal.
 */
export default {
  "build": {
    "extends": "build"
  },
  "build:js": {
    "extends": "build:js"
  },
  "build:js:node": {
    "extends": "build:js:node",
    "run": "node ../../../node_modules/rolldown/bin/cli.mjs --configLoader native --config rolldown.node.config.ts"
  }
} as const;
