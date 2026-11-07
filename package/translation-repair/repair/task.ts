/**
 CI build tasks for @monochromatic-dev/translation-repair-repair.

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
      "//package/translation-repair/adjudication:build:js:node",
      "//package/translation-repair/archive:build:js:node",
      "//package/translation-repair/assembly:build:js:node",
      "//package/translation-repair/corpus-source:build:js:node",
      "//package/translation-repair/document-structure:build:js:node",
      "//package/translation-repair/glossary:build:js:node",
      "//package/translation-repair/house-rule:build:js:node",
      "//package/translation-repair/lane-contest:build:js:node",
      "//package/translation-repair/markup:build:js:node",
      "//package/translation-repair/page-pass:build:js:node",
      "//package/translation-repair/provider:build:js:node",
      "//package/translation-repair/provider-stream:build:js:node",
      "//package/translation-repair/reference:build:js:node",
      "//package/translation-repair/shared-contract:build:js:node",
      "//package/translation-repair/slice-reading:build:js:node",
      "//package/translation-repair/stage:build:js:node",
      "//package/translation-repair/translate-floor:build:js:node",
      "//package/translation-repair/translate-stage:build:js:node",
      "//package/translation-repair/typography:build:js:node"
    ]
  }
} as const;
