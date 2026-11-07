#!/usr/bin/env node

/**
 Command-line entry of `mise run toolchain:update:daily`: updates the repository toolchain, the machine-global toolchain
 and the pnpm and Cargo dependency locks, and commits what moved.

 The systemd user service `toolchain-daily-update` runs it once a day (`mise run toolchain:timer:install` installs the
 units). A step that fails lands nothing from its group and the owner is told, on the desktop and in the journal.
 The sequencing rules are in {@link runDailyUpdate}; the steps are in {@link dailyUpdateGroups}.

 Options: `--repository <path>` (default: the current directory) and `--global-lock <path>` (default: the machine-global
 `toolchain.lock.json` under `$XDG_CONFIG_HOME` or `~/.config`).

 @example
 ```bash
 mise run toolchain:update:daily
 ```
 */

import { main, } from './daily-update-process.ts';

process.exitCode = await main(process.argv
  .slice(2,),);
