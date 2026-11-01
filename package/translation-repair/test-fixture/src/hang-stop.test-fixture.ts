//region Hang stop
// The one real-clock bound a test may set over scripted work: a per-call
// deadline, a whole-run signal, a hard cap or a grace window that no case
// waits for, because every scripted client, stage and voice it bounds answers
// on its own. It is there to turn a hang into a failure, never to be reached.
// Smaller bounds over the same work failed on a loaded machine for a reason
// no case name stated: a 5,000 ms signal on a whole drive of two scripted
// slices ran out before the drive did (commit `5086b6c6f`), and 100 ms grace
// windows a 30 ms voice had to land inside could be outrun (ledger B318). A
// per-call deadline handed to a stage over a hand-scripted client is never
// armed, since the provider clients arm it; it takes this value all the same,
// so a stage or client that comes to arm it waits a minute before it cuts
// scripted work. A bound a case means to reach is not this one: it keeps its
// own value, with work that cannot finish before it on any machine
// (`mistake-prevention.md`, "Tests on the real clock"), and
// `real-clock-bounds.unit.test.ts` holds every such bound to a listed class
// and reason.

/**
 How long a scripted test's bound waits before calling the work hung: long
 enough that no machine's load outlasts work that answers at once, and short
 enough that a real hang fails the case within a minute.
 */
export const HANG_STOP_MS = 60_000;

//endregion Hang stop
