/**
 What:
  A control on the overlap's answer: two adds on one small source, one with the size read started beside the check and one
  without, must make the same fence decision and write the same worktree.

 Why:
  the overlap changes where the sizes are read, not which they are; the fault it could introduce is a map of another
  commit, and a decision that differs between the two arms is what that would look like.

 How it runs:
  builds a small fixture whose `.gitattributes` gives a text rule, so the size proof decides paths (`-source`), then adds it
  twice with the same binary, the switch set and unset (`-add`), and prints each add's fence line plus the two destinations'
  status and a byte comparison against the source (`-report`).

 Usage:
  node fence-size-read-beside-the-check-2026-11-21-control-identical-fence.ts
 */
import {
  addWith,
  tree,
} from './fence-size-read-beside-the-check-2026-11-21-identical-fence-add.ts';
import { identicalFenceReport, } from '../bin/fence-size-read-beside-the-check-2026-11-21-identical-fence-report.ts';
import { buildFixture, } from './fence-size-read-beside-the-check-2026-11-21-identical-fence-source.ts';

await buildFixture();
/**
 The arm whose size read runs before it reaches the fence.
 */
const serial = await addWith({ overlap: false, },);
/**
 The arm whose size read starts beside the source check.
 */
const overlap = await addWith({ overlap: true, },);
/**
 The files the serial arm's destination holds.
 */
const serialTree = await tree({
  directory: serial.destination,
  prefix: '',
},);
/**
 The files the overlap arm's destination holds.
 */
const overlapTree = await tree({
  directory: overlap.destination,
  prefix: '',
},);
console.log(JSON.stringify(
  identicalFenceReport({
    serial,
    overlap,
    serialTree,
    overlapTree,
  },),
  null,
  1,
),);
