/**
 CI build tasks for @monochromatic-dev/typeface-aquaticat.

 Mirrors the build tasks in this package's mise.toml for the current pnpr publisher.
 The repository's task-system replacement remains a separate proposal.
 */
export default {
  "build": {
    "description": "Build the published font files; the pnpr publish job runs this task (#535)",
    "depends": [
      "build:font"
    ]
  },
  "build:font": {
    "description": "Build Aquaticat font from SVG glyphs",
    "run": "node src/build-font.ts"
  }
} as const;
