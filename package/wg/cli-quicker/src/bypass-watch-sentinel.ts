#!/usr/bin/env node

/**
 Detached sentinel that reports the death of the route watcher.

 The watcher cannot report its own death when `SIGKILL`,
 the out-of-memory killer or `systemd-oomd` ends it.
 This process waits for the watcher,
 and when the watcher is gone and its identity record still names it,
 delivers the report to the journal and the desktop.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { createWatcherDelivery, } from './bypass-watch-failure.ts';
import {
  parseSentinelArguments,
  runWatcherSentinel,
} from './bypass-watch-sentinel-core.ts';

/**
 Module logger for the sentinel.
 */
const l = tagged({ tag: 'bypass-watch-sentinel', },);

/**
 Watcher to wait for,
 from the arguments of the launcher.
 */
const target = parseSentinelArguments({ argv: process.argv
  .slice(2,), },);

/**
 Identity of the user who started the tunnel through `sudo`.
 */
const {
  SUDO_UID: sudoUid,
  SUDO_GID: sudoGid,
} = process.env;

/**
 What the record said about the end of the watcher.
 */
const verdict = await runWatcherSentinel({
  target,
  deliver: createWatcherDelivery({
    environment: {
      ...(sudoUid === undefined ? {} : { SUDO_UID: sudoUid, }),
      ...(sudoGid === undefined ? {} : { SUDO_GID: sudoGid, }),
    },
  },),
},);
l.info(`route watcher ${String(target.identity
  .pid,)} ended: ${verdict.kind}`,);
