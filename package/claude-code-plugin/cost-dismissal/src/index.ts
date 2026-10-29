#!/usr/bin/env node

/**
 Claude Code Stop hook that shadow-records size dismissals found in the
 session's thinking blocks. Scanning is local and the record carries hashes
 and indexes only, so no reasoning text leaves the machine. The hook never
 blocks a stop.
 
 Thin shim; handler logic, parser, and writer live in
 `@monochromatic-dev/claude-code-plugin-source/handler/cost-dismissal`.
 This file exists so the standard tsdown build produces an installable plugin
 entry at `bundle/node/index.mjs` for Claude Code's marketplace install.
 
 @example
 ```jsonc
 // In .claude/settings.json hooks config:
 "Stop": [
   {
     "hooks": [{ "type": "command", "command": "cccd" }]
   }
 ]
 ```
 
 @module
 */

import {
  costDismissalHandler,
  costDismissalParser,
  costDismissalWriter,
} from '@monochromatic-dev/claude-code-plugin-source/handler/cost-dismissal';
import { runHookPlugin, } from '@monochromatic-dev/claude-code-plugin-source/runtime';

await runHookPlugin({
  parser: costDismissalParser,
  handler: costDismissalHandler,
  writer: costDismissalWriter,
},);
