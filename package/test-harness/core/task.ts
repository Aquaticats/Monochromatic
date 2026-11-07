/**
 CI build tasks for @monochromatic-dev/test-harness-core.

 Mirrors the build tasks in this package's mise.toml for the current pnpr publisher.
 The repository's task-system replacement remains a separate proposal.
 */
export default {
  "build": {
    "extends": "build"
  },
  "build:js": {
    "description": "Build each artifact once in one dependency graph before bundling the browser consumer",
    "depends": [
      "//package/test-harness/core:build:js:node",
      "//package/test-harness/core:build:js:client"
    ]
  },
  "build:js:browser": {
    "extends": "build:js:browser"
  },
  "build:js:node": {
    "extends": "build:js:node"
  },
  "build:js:client": {
    "description": "Bundle the built neutral harness consumer for browser acceptance",
    "extends": "build:js:client",
    "depends": [
      "//package/test-harness/core:build:js:browser"
    ]
  }
} as const;
