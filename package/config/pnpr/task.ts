/**
 Package tasks for `@monochromatic-dev/config-pnpr`, replacing the
 `mise.toml` tasks CI ran through `mise run` with node-native definitions for
 `@monochromatic-dev/monorepo-task-cli`.
 */

import type { PackageTaskFile, } from '../../monorepo-task/cli/src/types.ts';

export default {
  'build': { extends: 'build', },
  'build:js': { extends: 'build:js', },
  'build:js:node': { extends: 'build:js:node', },
  'lint': { extends: 'lint', },
  'lint:oxlint': { extends: 'lint:oxlint', },
  'format:oxlint': { extends: 'format:oxlint', },
  'lint:types': { extends: 'lint:types', },
  'test:unit': {
    extends: 'test:unit',
    depends: ['build',],
  },
  'generate:config': {
    description: 'Write the pnpr config.yaml from the workspace manifests',
    run: 'node src/generate-config.ts',
  },
  'check:config': {
    description: 'Fail when the pnpr config.yaml differs from the generator',
    run: 'node src/generate-config.ts --check',
  },
  'publish': {
    description: 'Publish missing pnpr versions',
    run: 'node src/publish-missing-versions.ts',
  },
  'publish:dry-run': {
    description: 'Show which pnpr versions a publish would send',
    run: 'node src/publish-missing-versions.ts --dry-run',
  },
  'build:image': {
    description: 'Build the pnpr container image with podman under bounded resources',
    run: 'podman build --memory=2g --cpu-quota=200000 --tag localhost/config-pnpr:dev --file Containerfile .',
  },
  'start:image': {
    description: 'Run the pnpr container image under bounded resources',
    depends: ['build:image',],
    run: 'podman run --rm --init --name config-pnpr --memory=2g --cpus=2 --publish 127.0.0.1:17677:7677 --env PNPR_SECRET=local-throwaway-secret-0123456789abcdef --env PNPR_PUBLIC_URL=http://127.0.0.1:17677 localhost/config-pnpr:dev',
  },
} satisfies PackageTaskFile;
