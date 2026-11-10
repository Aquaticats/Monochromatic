/**
 CI build tasks for @monochromatic-dev/type-guard-core.

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
  "build:js:browser": {
    "extends": "build:js:browser"
  }
} as const;
