/**
 Watcher that ends the process groups of fixture programs when the test process that started them is gone.

 The test process writes one line per group: `add <id>` when a program starts and `remove <id>` when it has ended. When the
 test process dies, however it dies, the kernel closes this process's standard input, and every group still listed is killed
 with everything in it. A killed test run therefore leaves no fixture program behind, not even a program that a program
 started and that no longer has a parent that could end it.
 */
import { createInterface } from 'node:readline';

import { killGroup } from './fixture-process-group.ts';

/**
 Groups that were started and not yet reported ended.
 */
const live = new Set<number>();

for await (const line of createInterface({ input: process.stdin })) {
  /**
   The word and the group ID of the line.
   */
  const [word, id] = line.split(' ');
  if (word === 'add')
    live.add(Number(id));
  else if (word === 'remove')
    live.delete(Number(id));
}

for (const group of live)
  killGroup(group);
