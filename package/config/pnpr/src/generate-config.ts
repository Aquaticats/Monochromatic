import { parseArgs, } from 'node:util';

import {
  checkPnprConfig,
  generatePnprConfig,
} from './generated-config-sync.ts';
import { REPOSITORY_ROOT, } from './registry-read.ts';

//region Run

/**
 Command-line options: `--check` compares instead of writing, `--root` points at another repository root
 (tests use it to run against a disposable workspace).
 */
const { values: options, } = parseArgs({
  options: {
    check: { type: 'boolean', },
    root: { type: 'string', },
  },
},);

/**
 Repository root whose manifests feed the config and whose `config.yaml` is written or compared.
 */
const rootDir = options.root ?? REPOSITORY_ROOT;

await (options.check === true
  ? checkPnprConfig({ rootDir, },)
  : generatePnprConfig({ rootDir, },));

//endregion Run
