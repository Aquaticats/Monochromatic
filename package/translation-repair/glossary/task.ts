/**
 CI build tasks for @monochromatic-dev/translation-repair-glossary.

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
    "depends": [
      "//package/translation-repair/document-structure:build:js:node",
      "//package/translation-repair/markup:build:js:node",
      "//package/translation-repair/shared-contract:build:js:node",
      "//package/translation-repair/typography:build:js:node"
    ]
  }
} as const;
