/**
 What:
  The host-side measurement of rung 2 (the clone rung) on this host's ZFS home dataset.

 Why:
  rung 2's read is already beside the copies, so this rung says whether starting it beside the source check too changes
  anything, and its step records say where the time went.

 How it runs:
  Builds one fixture source of this repository's tracked tree (`-fixture`), then drives one `git worktree add` per arm per
  round through a shim of that arm's binary (`-add`), reading the record line the add appends (`-add-timing-record`). The
  destination sits on the same dataset as the source, so the clone rung applies and the fence's size read runs beside the
  copies. The rows a phase writes and the medians it prints are `-host-measure-phase`.

 Usage:
  node host-measure.ts fixture
  node host-measure.ts adds <layout> <rounds>
  node host-measure.ts gc
  node host-measure.ts refused <rounds>
  node host-measure.ts probe
 */
import {
  fixture,
  gc,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-source.ts';
import {
  defaultRefusedRounds,
  defaultRounds,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-paths.ts';
import {
  adds,
  probe,
  refusedAdds,
} from './fence-size-read-beside-the-check-2026-11-21-host-measure-phases.ts';

/**
 The phase this run was asked for, which its first argument names, and the words after it.
 */
const [phase, ...rest] = process.argv
  .slice(2,);
if (phase === 'fixture') {
  await fixture();
}
else if (phase === 'gc') {
  await gc();
}
else if (phase === 'adds') {
  /**
   The layout the `adds` phase measures, which the run names.
   */
  const [layout] = rest;
  if (layout === undefined) throw new Error('the adds phase needs a layout');
  await adds({
    layout,
    rounds: Number(rest[1] ?? defaultRounds,),
  },);
}
else if (phase === 'refused') {
  await refusedAdds({ rounds: Number(rest[0] ?? defaultRefusedRounds,), },);
}
else if (phase === 'probe') {
  await probe();
}
else throw new Error(`unknown phase ${String(phase,)}`);
