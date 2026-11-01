#!/usr/bin/env node

/**
 CLI entrypoint reporting workspace exports nothing references.
 
 @example
 ```bash
 unused-export
 unused-export --json /path/to/workspace
 unused-export --check
 ```
 */

import { parseArgs, } from 'node:util';

import { findUnusedExports, } from './find-unused.ts';

/**
 Parsed CLI options and optional workspace root positional.
 */
const {
  values,
  positionals,
} = parseArgs({
  options: {
    json: {
      type: 'boolean',
      default: false,
    },
    check: {
      type: 'boolean',
      default: false,
    },
  },
  allowPositionals: true,
},);

/**
 Workspace root under analysis, defaulting to the working directory.
 */
const workspaceRoot = positionals[0] ?? process.cwd();

/**
 Exports with zero workspace references.
 */
const findings = await findUnusedExports({ workspaceRoot, },);

if (values.json) {
  console.log(JSON.stringify(
    findings,
    null,
    2,
  ),);
}
else {
  for (const finding of findings)
    console.log(
      `${finding.file}:${String(finding.line,)}:${String(finding.column,)} ${finding.typeOnly ? 'type ' : ''}${finding.name}`,
    );
}

if (values.check && (findings.length > 0))
  process.exitCode = 1;
