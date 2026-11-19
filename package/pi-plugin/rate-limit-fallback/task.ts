/**
 Build tasks for the workspace publisher.
 */
export default {
  build: { extends: 'build', },
  'build:js': { extends: 'build:js', },
  'build:js:node': { extends: 'build:js:node', },
} as const;
