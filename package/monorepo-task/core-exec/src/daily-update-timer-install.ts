#!/usr/bin/env node

/**
 Command-line entry of `mise run toolchain:timer:install`; see `daily-update-timer-install-logic.ts` for the options.

 @example
 ```bash
 mise run toolchain:timer:install
 ```
 */

import { main, } from './daily-update-timer-install-logic.ts';

process.exitCode = await main(process.argv
  .slice(2,),);
