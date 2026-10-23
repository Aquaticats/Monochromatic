/**
 Guards the bounds tests set on the real clock (`mistake-prevention.md`,
 "Tests on the real clock"). A test that drives scripted work under a
 wall-clock bound fails when a loaded machine outlasts the bound, for a reason
 the case's name never states: a 5,000 ms signal on a whole drive of two
 scripted slices ran out before the drive did, twice under load (commit
 `5086b6c6f`), and two 100 ms grace windows a 30 ms voice had to land inside
 could be outrun (ledger B318). A
 bound that only stops a hang is `HANG_STOP_MS` (`hang-stop.test-fixture.ts`),
 60,000 ms, which no case waits for; any other bound a test writes is a choice
 a reader must see the reason for.

 WHAT THE SCAN READS, in test files and test fixtures only: the argument of
 `AbortSignal.timeout`; the delay of a global `setTimeout` or `setInterval`;
 the first argument of `setTimeout` imported from `node:timers/promises` and
 of `wait` imported from `@monochromatic-dev/module-async-time/ts`, under any
 local name; and the value of an object literal's key, or the default of a
 destructured parameter, named in `BOUND_KEYS`: the per-call deadlines, grace
 windows (the two of a `WriterGrace` among them), stream bounds (a model
 card's among them), caps, budgets, holds and the runner's own `timeout` that
 production arms a timer with or compares elapsed time against; and a
 `createBedrockClient` imported from the built package under any local name
 whose options name no `streamBoundMsOverride`, since the client then arms
 each card's own bound around every attempt (ledger B416), keyed with the
 value `(none)`. A read
 whose value is the shared constant, imported from its fixture under any
 local name, is left alone, and so is a stream bound of `'unbounded'`, which
 arms nothing; every other value, a number, another constant,
 a parameter handed on or an expression, is a finding, keyed
 `path#site: form value`, where the site is the nearest enclosing named
 function, `<module>` outside one.

 OUT OF ITS REACH: a timer reached through `globalThis` or another object,
 a key outside `BOUND_KEYS` (a retry backoff, a poll interval, a field a
 report reads), a Bedrock client a production function builds for a test
 (`configureProviders` handed a Bedrock key, which takes no override; none
 does so, by the census of B416), a duration handed through an environment
 variable as text,
 an elapsed time a case measures with `performance.now` and compares, and a
 sleep inside a child process's script text. Waits a case's work takes, such
 as `baseMs` or `pollMs`, are left out on purpose: load only lengthens them,
 and a race against one shows up at the bound or the sleep on its other side.

 THE HOLDS A TEST REGISTERS FOR A PROVIDER, which reach a sleep through a client's reader of holds and not through
 a timer call (ledger B817): a hold of 1000 ms or more written onto a member named `holds` (`rig.holds.synthetic = N`),
 and a property of an object literal inside a `providerHolds` or `holds` property or a constant named for a hold, such
 as `HYPER_HELD`. Its value is a number, a top-level constant bound to one, the greater branch of a conditional, or a
 sum, difference or product of such; an assignment whose value is none of these is a finding too, keyed `holds.<name> =
 value`, and a property that is none is left. The scan flags by size and cannot tell a hold the seat reading sleeps from
 one it does not, since that depends on whether a bench is short; a listed hold carries the reason it sleeps nothing.

 THE LISTED BOUNDS name every finding that stands, with how many times its
 key occurs, its class (`reached`: a bound a case means to reach, whose work
 cannot end before it on any machine; `read back`: a value a case asserts or
 reads back, such as a production default or a field of a record; `other`:
 what it is, said in the reason) and why. A listed key the source no longer
 holds as often fails the scan, so the list cannot outlive its reasons. The
 files in `DEFERRED` are another change's to edit first: their findings are
 not listed one by one, and a deferred file that no longer holds a finding
 fails the scan until its line is removed.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own tests.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import {
  declaringModule,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  literalText,
  memberName,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectFindingsAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Real-clock bounds
// The scan itself: which nodes set a bound on the real clock, which value is
// the shared hang stop, and the walk that keys each other bound by its file,
// its enclosing named function, its form and its value.

/**
 Locations of the fixture that declares the shared hang stop: where the family keeps it, and the bare
 file name a fixture case in this file gives it, since such a case's files belong to no package.
 */
const HANG_STOP_PATHS: ReadonlySet<string> = new Set([
  'test-fixture/hang-stop.test-fixture.ts',
  'hang-stop.test-fixture.ts',
],);

/**
 Location of the module that declares the Bedrock client's builder, which a test reaches through the
 built entry of its package or the package's name.
 */
const BEDROCK_CLIENT_MODULE = 'provider/bedrock-client.ts';

/**
 Name the fixture exports the shared hang stop under.
 */
const HANG_STOP_NAME = 'HANG_STOP_MS';

/**
 End of the specifier every test imports the built package by, from `src`
 or a directory under it.
 */
const BUILT_INDEX_SUFFIX = 'dist/final/node/index.mjs';

/**
 Name the built package exports the Bedrock client's builder under, which
 arms each model card's own stream bound (`streamBoundMs`) around every
 attempt unless it is handed `streamBoundMsOverride` (`bedrock-client.ts`).
 */
const BEDROCK_CLIENT_NAME = 'createBedrockClient';

/**
 Forms of the stream bound keys, as a value and as a destructured default,
 whose value `'unbounded'` arms no timer: the Bedrock client hands an
 unbounded model's attempts to the transport as they are.
 */
const UNBOUNDED_FORMS: ReadonlySet<string> = new Set([
  'streamBoundMs:',
  'streamBoundMs =',
  'streamBoundMsOverride:',
  'streamBoundMsOverride =',
],);

/**
 The value that sets no stream bound.
 */
const UNBOUNDED = 'unbounded';

/**
 Option keys production arms a timer with, or compares elapsed time against.

 THE STRAGGLER WINDOWS UNDER EVERY NAME THEY TRAVEL BY: `graceMs` where a
 round takes one, and `roundMs` and `writerMs` where a `WriterGrace`
 (`writer-grace-override.ts`) carries both inside a `StageSettings`
 (`stage-settings.ts`), from which every stage hands one to its round as
 `graceMs` (ledger B398). A calibration grace's `effectiveMs` and a note's
 `builtInMs` are left out: production only prints them, and the calibration's
 rounds take their window from a `WriterGrace`.

 THE STREAM BOUND UNDER BOTH ITS NAMES: `streamBoundMsOverride` where a test
 hands one to the Bedrock client (`bedrock-client.ts`), and `streamBoundMs`
 where a model card carries it (`model-card.ts`), which the client arms around
 every attempt to that model when no override is given (ledger B401, B416).
 */
const BOUND_KEYS: ReadonlySet<string> = new Set([
  'boundMs',
  'cooldownMs',
  'exchangeTimeoutMs',
  'firstByteMs',
  'firstContentMs',
  'freshForMs',
  'graceMs',
  'hardCapMs',
  'idleMs',
  'modelHoldMs',
  'perCallTimeoutMs',
  'rateLimitBackoffMs',
  'roundMs',
  'runBudgetMs',
  'softBudgetMs',
  'statedWaitMs',
  'streamBoundMs',
  'streamBoundMsOverride',
  'streamFirstContentMs',
  'streamIdleMs',
  'timeout',
  'timeoutMs',
  'windowMs',
  'writerMs',
],);

/**
 Node kinds that open a named function, which names the site of the bounds
 inside.
 */
const NAMING_KINDS: ReadonlySet<string> = new Set([
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 What a listed bound is, as the census of 2026-10-06 (UTC) sorted them.
 */
type BoundClass = 'reached' | 'read back' | 'other';

/**
 A bound that stands with a value of its own.
 */
type ListedBound = {
  /**
   How many reads the key stands for in its file.
   */
  readonly count: number;

  /**
   What the bound is.
   */
  readonly bound: BoundClass;

  /**
   Why it keeps its own value.
   */
  readonly reason: string;
};

/**
 Files another change edits first, each with why its bounds wait. Empty since
 the seven files deferred on 2026-10-06 (UTC) took the census's rule once the
 changes in flight on them merged.
 */
const DEFERRED: Readonly<Record<string, string>> = {};

/**
 Milliseconds from which a hold a test registers for a provider is read as a real wait: the seat reading
 sleeps the shortest hold it is told of (`waitOutHold`, `budget-hold-wait.ts`), and the router registers 300000 on a
 refusal from a meter that reads dry, so a case that hands its client such a hold and drives a reading that
 waits pays it in real time (the padded-checkers case of `pass-reseat.unit.test.ts` ran 300 s in every gate
 from 2026-10-21 (UTC), ledger B817). The poll interval of that wait is the same second (`HOLD_POLL_MS`).
 */
const HOLD_READ_MS = 1_000;

/**
 Names that mark a rig's holds: a member `holds` that a case writes a provider's hold onto, and a property or
 constant that a holds reader returns or a case names for a hold.
 */
const HOLDS_MEMBER = 'holds';

/**
 Words that name a hold in a constant's name.
 */
const HOLD_WORDS: ReadonlySet<string> = new Set(['hold', 'holds', 'held',],);

/**
 Key a scripted run client answers its holds reader under.
 */
const HOLDS_READER_KEY = 'providerHolds';

/**
 Why a field of a record keeps its value: the case builds an error, a stream
 progress reading or a log line with it and reads it back, and nothing arms a
 timer with it.
 */
const RECORD_FIELD = 'a field of a record or a stream progress reading the case builds and reads back; '
  + 'nothing arms a timer with it';

/**
 Why a call config keeps its value: it is the configuration an artifact
 records, production's per-call deadline written down, which nothing arms.
 */
const CALL_CONFIG = 'the call configuration an artifact records, production\'s per-call deadline as written into '
  + 'the record; nothing arms a timer with it';

/**
 Why a window on an injected clock keeps its value: the case moves that clock
 by hand and reads the window back, and no timer is armed with it.
 */
const SCRIPTED_CLOCK = 'a window on the case\'s own injected clock, which the case moves by hand and reads '
  + 'back; no timer is armed with it';

/**
 Why a hold the reseat file names keeps its value: the production budgets cut it to the file's short window, on a
 clock that stands still, and the seat reading waits that window out once on the real clock.
 */
const CUT_TO_SHORT_WINDOW = 'a hold the production budgets cut to the file\'s short window (40 ms), placed on a clock '
  + 'that stands still; the seat reading waits that window out once, so no case waits for the named value';

/**
 Why a wait stated to the pure cut keeps its value: the cut returns the figure it cuts to and sleeps nothing, so no
 case waits for the stated value.
 */
const PURE_WAIT_CUT = 'a wait handed to the pure cut of a stated wait, which returns the figure it cuts to and sleeps '
  + 'nothing; no case waits for the stated value';

/**
 Why a hold the reseat rig registers keeps its value: the rig reports every hold cut to the file's short window
 (`endingHolds`), except in the one case that sets the cap to the hold and waits it out on a fake clock.
 */
const RIG_REPORTS_SHORT_WINDOW = 'a hold the file\'s rig reports cut to its short window (40 ms), or, in the one case '
  + 'that sets the cap to the hold, waited out on a fake clock; no case waits for the named value on the real clock';

/**
 Why a deadline handed to a client keeps its value: the client arms a timer with it, the case reads the delay back
 from the timer and clears it, and the scripted transport answers at once, so no case waits for it.
 */
const ARMED_DEADLINE_READ_BACK = 'a deadline the client arms a timer with, whose delay the case reads back; the '
  + 'scripted transport answers at once and the client clears the timer, so nothing waits for it';

/**
 Why a launch value keeps its value: the case reads it back in a printed line.
 */
const PRINTED = 'a value the case reads back in a printed line, as given; no timer is armed with it';

/**
 Why a timer turn of length 0 stands: it orders continuations by the event
 loop's turns, not by time.
 */
const TIMER_TURN = 'a timer turn of 0 that lets settled continuations run before the case reads; '
  + 'nothing races it, so its order is the event loop\'s, not the clock\'s';

/**
 Why a meter's hold stands: the overlap it holds for is in place within one
 turn of the event loop.
 */
const SAME_TURN_METER = 'a meter hold whose overlap is in place within one turn of the event loop: '
  + 'the case passes with a hold of microtask turns only and fails with none (census of 2026-10-06), '
  + 'so no timer can fire between the arrivals it waits for';

/**
 Why a serial run's head start stands: at overlap 1 nothing runs beside it.
 */
const SERIAL_HEAD_START = 'a head start in the runs where the driver admits one call at a time, so it only '
  + 'delays; the run at overlap 2 waits on a gate instead';

/**
 Why a parameter handed on is not a value of its own.
 */
const HANDED_ON = 'hands on its caller\'s value, which is read where the caller gives it';

/**
 Why the recall run's call configuration keeps its values: the case reads
 production's configuration back whole in the scorecard the run keeps.
 */
const RECALL_CALL_CONFIG = 'production\'s call configuration for the recall run, read back whole in the scorecard '
  + 'the run keeps; the case arms no timer with it, and the scripted critic client none';

/**
 Why the sentinel probe's deadline keeps its value: a number no default
 shares, so the case shows the walk handed the repair this one.
 */
const SENTINEL_DEADLINE = 'the deadline the walk hands each scripted repair, read back in what the repair was asked; '
  + 'the scripted repair arms no timer with it';

/**
 Why a window a reader returns keeps its value: the case asserts the reader's
 whole output, and no round runs under it.
 */
const READER_OUTPUT = 'a window the reader under test returns, asserted whole in its output; the case runs no round '
  + 'and arms no timer with it';

/**
 Why a writer grace's windows keep their values where both readers and the
 launch note are tested: each is the output of `readWriterGrace` asserted
 whole, or a grace handed to `writerGraceOverrideNote` and read back in the
 line it prints or the silence it keeps.
 */
const WRITER_GRACE_READ_BACK = 'a window of a writer grace the case reads back: the output of readWriterGrace '
  + 'asserted whole, or a grace handed to writerGraceOverrideNote and read back in its line; no round runs under it';

/**
 Why the values handed to the pure check of stream bounds keep their numbers: each case reads the
 check's refusal or its silence, and the check arms no timer.
 */
const PURE_BOUND_CHECK = 'a duration handed to the pure check of stream bounds, whose refusal or silence the case reads; '
  + 'the check arms no timer';

/**
 Why a stream window of the content-progress cases keeps its value: the window is the thing the case
 reaches or stays inside.
 */
const STREAM_WINDOW_REACHED = 'the window of a guard the case means to reach with a stream that sends no text, or to stay '
  + 'inside with a stream that sends text every few milliseconds; it is 31 times the pause between two chunks';

/**
 Why the pause between two chunks of a timed body keeps its value.
 */
const CHUNK_PACING = 'the pause between two chunks of a scripted body, so a stream lasts longer than the window under test; '
  + 'load only lengthens it';

/**
 The two-minute nap of the fake clock walk's own cases, which a walk on the fake clock ticks past: no real time passes,
 and a nap that were on the real clock would fail the case by not ending.
 */
const FAKE_NAP = 'a nap of two fake minutes that the walk ticks past on the fake clock; no real time passes, and a nap '
  + 'left on the real clock is the failure the case reads';

/**
 Why a hold of the reason cases keeps its value: the pure reason reader is handed a record of holds and names the
 shortest, and it sleeps nothing.
 */
const HOLD_REASON_INPUT = 'a hold handed to the pure reader of the reason a wait is taken for, which names the shortest and '
  + 'sleeps nothing';

/**
 Why the cache window of the NeuralWatt return budget cases keeps its value: the window is on the case\'s own scripted
 clock, which the case moves by hand.
 */
const RETURN_CACHE_WINDOW = 'the cache window of budgets read on the case\'s own scripted wall clock and monotonic clock, '
  + 'which the case moves by hand';

/**
 Bounds that keep a value of their own, keyed `path#site: form value`.
 */
const LISTED: Readonly<Record<string, ListedBound>> = {
  'corpus-pass/stop-landing-hook.test-fixture.ts#<module>: hold hyper: 60_000': {
    count: 1,
    bound: 'other',
    reason: 'a minute\'s hold the reading is told of after the stop has already landed: the stop aborts inside the '
      + 'dryness read that comes first, and the wait that follows throws the abort before it sleeps',
  },
  'provider/budget-hold-wait.unit.test.ts#<module>: hold hyper: 4_000': {
    count: 2,
    bound: 'read back',
    reason: 'a record `shortestHold` is asked the shortest of, which only reads the numbers and sleeps nothing',
  },
  'provider/budget-hold-wait.unit.test.ts#<module>: hold hyper: 1_000_000': {
    count: 1,
    bound: 'other',
    reason: 'a hold that outlasts the case: the wait polls every millisecond and the case\'s own abort ends it, so no '
      + 'bound on the clock decides the verdict',
  },
  'provider/budget-hold-wait.unit.test.ts#<module>: setTimeout 30': {
    count: 1,
    bound: 'reached',
    reason: 'the caller abort, armed before a wait on a hold of a million milliseconds, so it fires first on any '
      + 'machine and the case asserts its own reason',
  },
  'provider/hold-reason.unit.test.ts#<module>: hold hyper: 5_000': { count: 1, bound: 'read back', reason: HOLD_REASON_INPUT, },
  'provider/hold-reason.unit.test.ts#<module>: hold hyper: 900_000': { count: 1, bound: 'read back', reason: HOLD_REASON_INPUT, },
  'provider/hold-reason.unit.test.ts#<module>: hold neuralwatt: 60_000': {
    count: 1,
    bound: 'read back',
    reason: HOLD_REASON_INPUT,
  },
  'provider/hold-reason.unit.test.ts#<module>: hold neuralwatt: 48_000': { count: 1, bound: 'read back', reason: HOLD_REASON_INPUT, },
  'provider/hold-reason.unit.test.ts#<module>: hold openrouter: 9_000': { count: 1, bound: 'read back', reason: HOLD_REASON_INPUT, },
  'provider/hold-reason.unit.test.ts#<module>: hold synthetic: 5_000': { count: 1, bound: 'read back', reason: HOLD_REASON_INPUT, },
  'provider/provider-budget-neuralwatt-return.unit.test.ts#budgetsBefore: freshForMs: FRESH_MS': { count: 1, bound: 'other', reason: RETURN_CACHE_WINDOW, },
  'provider/provider-budget-neuralwatt-return.unit.test.ts#<module>: freshForMs: FRESH_MS': {
    count: 1,
    bound: 'other',
    reason: RETURN_CACHE_WINDOW,
  },
  'provider/provider-budget-neuralwatt-return.unit.test.ts#budgetsBefore: cooldownMs: hyperCooldownMs': {
    count: 1,
    bound: 'reached',
    reason: 'how long a refusal holds a dry Hyper meter out for a case that waits that hold out for real, handed in by '
      + 'the case (40 ms), so the wait cannot end before it on any machine',
  },
  'provider/neuralwatt-return-fit.unit.test.ts#<module>: boundMs: 600_000': {
    count: 2,
    bound: 'other',
    reason: 'a bound handed to a pure comparison of two numbers; no timer is armed and nothing sleeps',
  },
  'provider/neuralwatt-return-fit.unit.test.ts#<module>: boundMs: 0': {
    count: 1,
    bound: 'other',
    reason: 'a bound handed to a pure comparison of two numbers; no timer is armed and nothing sleeps',
  },
  'provider/neuralwatt-return-fit.unit.test.ts#<module>: boundMs: -5': {
    count: 1,
    bound: 'other',
    reason: 'a bound handed to a pure comparison of two numbers; no timer is armed and nothing sleeps',
  },
  'corpus-pass/corpus-run/pass-entry-bound.unit.test.ts#<module>: hardCapMs: 7 * HOUR_MS': {
    count: 2,
    bound: 'other',
    reason: 'an entry\'s hard cap read against a monotonic clock the case moves by hand; nothing sleeps and '
      + 'no timer is armed',
  },
  'corpus-pass/corpus-run/pass-entry-bound.unit.test.ts#<module>: hardCapMs: 1_000': {
    count: 1,
    bound: 'other',
    reason: 'an entry\'s hard cap read against a monotonic clock the case moves by hand; nothing sleeps and '
      + 'no timer is armed',
  },
  'corpus-pass/corpus-run/pass-dry-stop.unit.test.ts#<module>: cooldownMs: 40': {
    count: 1,
    bound: 'reached',
    reason: 'how long a refusal holds a dry Hyper meter out, which the router\'s wait sleeps out for real before it '
      + 'refuses with the NeuralWatt distance, so the refusal cannot come before it on any machine',
  },
  'corpus-pass/corpus-run/pass-dry-stop.unit.test.ts#<module>: freshForMs: 600_000': {
    count: 1,
    bound: 'other',
    reason: 'the cache window of a budget reading, longer than the case runs so the one reading stands; the '
      + 'case moves no clock',
  },
  'provider/provider-budget-neuralwatt-return.unit.test.ts#<module>: statedWaitMs: 120_000': {
    count: 1,
    bound: 'read back',
    reason: 'a refusal\'s stated wait handed to the budget layer on the case\'s own scripted clock, '
      + 'which sleeps nothing',
  },
  'run/corpus-run/run-seats-read.unit.test.ts#<module>: hold openrouter: 60_000': {
    count: 1,
    bound: 'other',
    reason: 'a hold under a view where every provider is wet, so no bench is short and the reading takes its one '
      + 'dryness read without a wait; the case asserts that one read',
  },
  'provider-stream/cut-stream.test-fixture.ts#cutStreamError: firstByteMs: (firstByte === \'never\') ? -1 : FIRST_BYTE_MS': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'provider-stream/cut-stream.test-fixture.ts#cutStreamError: firstContentMs: generatedAny ? FIRST_BYTE_MS : -1': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'provider/claude-call-deadline.unit.test.ts#<module>: exchangeTimeoutMs: LONGER_DEADLINE_MS': {
    count: 1,
    bound: 'read back',
    reason: ARMED_DEADLINE_READ_BACK,
  },
  'provider/claude-call-deadline.unit.test.ts#<module>: exchangeTimeoutMs: RUN_DEADLINE_MS': {
    count: 3,
    bound: 'read back',
    reason: ARMED_DEADLINE_READ_BACK,
  },
  'provider/claude-call-deadline.unit.test.ts#armedForClaude: exchangeTimeoutMs: exchangeTimeoutMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'provider/claude-client.unit.test.ts#<module>: freshForMs: 1_000': { count: 2, bound: 'other', reason: SCRIPTED_CLOCK, },
  'provider/claude-client.unit.test.ts#<module>: statedWaitMs: statedWaitMsOf({ error: failure, },)': {
    count: 2,
    bound: 'read back',
    reason: 'the wait a refusal states, read back in what the budgets hold',
  },
  'provider/deepseek-client.unit.test.ts#<module>: freshForMs: FRESH_MS': { count: 1, bound: 'other', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget-xiaomi-hold.unit.test.ts#xiaomiBudgets: freshForMs: FRESH_MS': {
    count: 1,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-budget-held-meters.unit.test.ts#budgetsOnClock: freshForMs: FRESH_MS': {
    count: 1,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-budget-held-meters.unit.test.ts#budgetsOnClock: rateLimitBackoffMs: FRESH_MS': {
    count: 1,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-budget-held-meters.unit.test.ts#holdThenLapse: statedWaitMs: 10 * FRESH_MS': {
    count: 1,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-budget-defaults.unit.test.ts#cut: statedWaitMs: STATED_MS': {
    count: 1,
    bound: 'other',
    reason: PURE_WAIT_CUT,
  },
  'provider/provider-budget-defaults.unit.test.ts#cut: statedWaitMs: statedWaitMs': {
    count: 1,
    bound: 'other',
    reason: PURE_WAIT_CUT,
  },
  'provider/provider-budget-hold-cap.unit.test.ts#cappedBudgets: rateLimitBackoffMs: 1_000': {
    count: 1,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-budget-hold-cap.unit.test.ts#refuseForPayment: statedWaitMs: STATED_MS': {
    count: 1,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-budget-hold-cap.unit.test.ts#refuseShort: statedWaitMs: 3_000': {
    count: 1,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-budget-hold-cap.unit.test.ts#refuseTwice: statedWaitMs: STATED_MS': {
    count: 2,
    bound: 'other',
    reason: SCRIPTED_CLOCK,
  },
  'stage/seat-tally-silent.unit.test.ts#<module>: graceMs: 30': {
    count: 1,
    bound: 'reached',
    reason: 'the grace window the round closes on the call that waits for the cut: that call ends only by the cut, '
      + 'so the window is reached on any machine',
  },
  'provider-stream/abandon-kind-answer-reasoning.unit.test.ts#<module>: firstByteMs: 40': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'provider-stream/abandon-kind-answer-reasoning.unit.test.ts#<module>: firstByteMs: 5': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'provider-stream/abandon-kind-answer-reasoning.unit.test.ts#<module>: firstByteMs: 812': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'provider-stream/abandon-kind-answer-reasoning.unit.test.ts#<module>: firstContentMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/abandon-kind-answer-reasoning.unit.test.ts#cutAfter: firstByteMs: firstByteMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'provider-stream/abandon-kind-answer-reasoning.unit.test.ts#cutAfter: firstContentMs: firstByteMs': { count: 1, bound: 'other', reason: HANDED_ON, },
  'provider-stream/abandon-kind.unit.test.ts#<module>: firstByteMs: -1': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/abandon-kind.unit.test.ts#<module>: firstByteMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/abandon-kind.unit.test.ts#cutWith: firstByteMs: firstByteMs': { count: 1, bound: 'other', reason: HANDED_ON, },
  'provider-stream/abandon-kind.unit.test.ts#cutWith: firstContentMs: firstByteMs': { count: 1, bound: 'other', reason: HANDED_ON, },
  'archive/archive-block-review-windows.unit.test.ts#<module>: roundMs: 0': {
    count: 1,
    bound: 'reached',
    reason: 'the readers\' window of 0 the selection case reaches: its straggler answers only after the reply delay of '
      + 'the scripted client, a second, by which the stage has tallied the other seats, so the window closes on it; '
      + 'the writers\' window beside it is the hang stop',
  },
  'archive/scripted-review-client.test-fixture.ts#chatJson: wait of module-async-time HELD_REPLY_MS': {
    count: 1,
    bound: 'reached',
    reason: 'makes the held seat late: a second gives the stage time to tally its peers, which answer at once, so it '
      + 'lands after quorum, inside a window that is the hang stop and outside one of 0 (at 25 ms it landed before '
      + 'quorum under the rotation sweep\'s load)',
  },
  'archive/absolute-naturalness-review-stage.unit.test.ts#<module>: wait of module-async-time 0': {
    count: 1,
    bound: 'other',
    reason: TIMER_TURN,
  },
  'archive/absolute-naturalness-review-stage.unit.test.ts#reviewClient: wait of module-async-time 30': {
    count: 1,
    bound: 'other',
    reason: 'makes the rejecting seat late: its peers answer in microtasks, so it lands after quorum on any '
      + 'machine, inside a window that is the hang stop',
  },
  'archive/absolute-naturalness-review-stage.unit.test.ts#runReview: graceMs = 0': {
    count: 1,
    bound: 'reached',
    reason: 'the window of 0 the held-seat case reaches: that seat answers only when the case opens its gate after '
      + 'the review settles, and every other seat answers in microtasks, before any timer can close it',
  },
  'archive/absolute-naturalness-review-stage.unit.test.ts#runReview: graceMs: graceMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'document-structure/all-in-input-order.unit.test.ts#settle: wait of module-async-time 0': { count: 1, bound: 'other', reason: TIMER_TURN, },
  'provider-stream/stream-bound-config.unit.test.ts#<module>: perCallTimeoutMs: DEADLINE_MS': { count: 2, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#<module>: streamFirstContentMs: 180_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#<module>: streamFirstContentMs: STREAM_FIRST_CONTENT_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#<module>: streamIdleMs: 120_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#<module>: streamIdleMs: STREAM_IDLE_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badDeadline: perCallTimeoutMs: bad': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badDeadline: streamFirstContentMs: 120_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badDeadline: streamIdleMs: 120_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badFirstContent: perCallTimeoutMs: DEADLINE_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badFirstContent: streamFirstContentMs: bad': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badFirstContent: streamIdleMs: 120_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badIdle: perCallTimeoutMs: DEADLINE_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badIdle: streamFirstContentMs: 120_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#badIdle: streamIdleMs: bad': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#equal: perCallTimeoutMs: DEADLINE_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#equal: streamFirstContentMs: DEADLINE_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#equal: streamIdleMs: 120_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#idleAbove: perCallTimeoutMs: DEADLINE_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#idleAbove: streamFirstContentMs: 120_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#idleAbove: streamIdleMs: DEADLINE_MS + 1': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#shipped: perCallTimeoutMs: DEADLINE_MS': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#shipped: streamFirstContentMs: 600_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-bound-config.unit.test.ts#shipped: streamIdleMs: 600_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'provider-stream/stream-content-progress.unit.test.ts#<module>: firstContentMs: WINDOW_MS': { count: 5, bound: 'reached', reason: STREAM_WINDOW_REACHED, },
  'provider-stream/stream-content-progress.unit.test.ts#<module>: idleMs: WINDOW_MS': { count: 4, bound: 'reached', reason: STREAM_WINDOW_REACHED, },
  'provider-stream/stream-content-progress.unit.test.ts#drainTimed: firstContentMs: firstContentMs': { count: 1, bound: 'other', reason: HANDED_ON, },
  'provider-stream/stream-content-progress.unit.test.ts#drainTimed: idleMs: idleMs': { count: 1, bound: 'other', reason: HANDED_ON, },
  'provider-stream/stream-content-progress.unit.test.ts#timedResponse: wait of module-async-time GAP_MS': { count: 1, bound: 'other', reason: CHUNK_PACING, },
  'provider/bedrock-client.unit.test.ts#<module>: createBedrockClient of the built package (none)': {
    count: 1,
    bound: 'other',
    reason: 'a client that reads the catalog\'s own bound and is asked only for the gpt-oss card, which carries '
      + 'none (unbounded), so no timer is armed; the case shows the transport handed the caller\'s own signal',
  },
  'provider/bedrock-client.unit.test.ts#<module>: exchangeTimeoutMs: 50': {
    count: 1,
    bound: 'reached',
    reason: 'the deadline the case reaches and reads back in the refusal: the transport answers only once its '
      + 'exchange is aborted',
  },
  'provider/bedrock-client.unit.test.ts#<module>: streamBoundMsOverride: TEST_BOUND_MS': {
    count: 2,
    bound: 'reached',
    reason: 'the stream bound two cases cross: one cuts a sleep armed after it and twice as long, which ends after '
      + 'the bound on any machine; in the other both attempts answer at once and the backoff between them outlasts it',
  },
  'provider/bedrock-client.unit.test.ts#queuedPastTheBound: sleepFor of node:timers/promises TEST_BOUND_MS * 2': {
    count: 1,
    bound: 'reached',
    reason: 'the work the stream bound cuts: armed after the bound and twice as long, so the bound ends first on any '
      + 'machine (measured 2026-10-06: timers armed in that order fire in it however long the loop stalls)',
  },
  'provider/call-deadline.unit.test.ts#<module>: timeoutMs: SHORT_DEADLINE_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the deadline the case reaches: armed before the case\'s wait of three times its length, so it fires '
      + 'first on any machine',
  },
  'provider/call-deadline.unit.test.ts#<module>: wait of module-async-time PAST_DEADLINE_MS': {
    count: 2,
    bound: 'reached',
    reason: 'the wait that must outlast a deadline armed before it, three times its length, so the deadline fires '
      + 'first on any machine, and one disposal failed to defuse would fire too',
  },
  'provider/call-deadline.unit.test.ts#armAndDispose: timeoutMs: SHORT_DEADLINE_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the deadline disposal must defuse: armed before the case\'s longer wait, so one disposal left armed '
      + 'fires inside it on any machine',
  },
  'provider/client-log-context.unit.test.ts#note: wait of module-async-time 0': { count: 1, bound: 'other', reason: TIMER_TURN, },
  'provider/client-log-context.unit.test.ts#transport: wait of module-async-time 0': {
    count: 1,
    bound: 'other',
    reason: 'a timer turn that keeps a call in its one slot while the queued calls wait behind it; the order is the '
      + 'slot\'s, not the clock\'s',
  },
  'corpus-pass/consolidate-driver.unit.test.ts#driveWith: wait of module-async-time startPosition === 0 ? 20 : 5': {
    count: 1,
    bound: 'other',
    reason: SERIAL_HEAD_START,
  },
  'artifact/corpus-run/artifact-two-lane-build.unit.test.ts#catArtifact: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'artifact/corpus-run/artifact-two-lane-corpus-verify.unit.test.ts#writeAndRead: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'provider-stream/corpus-run/cap-census-log.test-fixture.ts#streamLine: firstByteMs: 3_644': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'provider-stream/corpus-run/cap-census-log.test-fixture.ts#streamLine: firstContentMs: 3_644': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: hardCapMs: 120_000': { count: 1, bound: 'read back', reason: PRINTED, },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: hardCapMs: 25_200_000': {
    count: 2,
    bound: 'read back',
    reason: 'production\'s built-in cap of 420 minutes, read back in the launch lines',
  },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: hardCapMs: 400_000': {
    count: 1,
    bound: 'read back',
    reason: PRINTED,
  },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: hardCapMs: 510_000': {
    count: 1,
    bound: 'read back',
    reason: PRINTED,
  },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: hardCapMs: 600_000': { count: 1, bound: 'read back', reason: PRINTED, },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: hardCapMs: 60_000': { count: 1, bound: 'read back', reason: PRINTED, },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: hardCapMs: 90_000': { count: 1, bound: 'read back', reason: PRINTED, },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: softBudgetMs: 259_200_000': {
    count: 1,
    bound: 'read back',
    reason: 'production\'s soft budget of three days, read back in the start line',
  },
  'corpus-pass/corpus-run/corpus-pass-lines.unit.test.ts#<module>: softBudgetMs: 60_000': { count: 1, bound: 'read back', reason: PRINTED, },
  'corpus-pass/corpus-run/corpus-pass-queue.unit.test.ts#<module>: softBudgetMs: 1_000': {
    count: 5,
    bound: 'read back',
    reason: 'a soft budget judged against the case\'s scripted clock readings, which the case lists; no timer is '
      + 'armed with it',
  },
  'corpus-pass/corpus-run/corpus-pass-run.unit.test.ts#runPass: hardCapMs: 25_200_000': {
    count: 1,
    bound: 'read back',
    reason: 'production\'s built-in cap of 420 minutes, read back in the start line every case reads',
  },
  'coverage-census/corpus-run/coverage-control-probe-run.unit.test.ts#<module>: exchangeTimeoutMs: EXCHANGE_TIMEOUT_MS': {
    count: 1,
    bound: 'read back',
    reason: 'the deadline the scripted control records, read back in what it was asked; the inert client arms none',
  },
  'coverage-census/corpus-run/coverage-control-probe-run.unit.test.ts#scripted: exchangeTimeoutMs: input.exchangeTimeoutMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'coverage-census/corpus-run/coverage-control-probe-run.unit.test.ts#walk: exchangeTimeoutMs: EXCHANGE_TIMEOUT_MS': {
    count: 1,
    bound: 'read back',
    reason: 'the deadline handed to the walk, which the scripted control records and the case reads back',
  },
  'probe/corpus-run/damage-region-entry-id.unit.test.ts#censusOverOneEntry: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'bench/corpus-run/model-health-probe.unit.test.ts#askThrowingModel: timeoutMs: TIMEOUT_MS': {
    count: 1,
    bound: 'read back',
    reason: 'the deadline the probe hands its call, read back off the request the client saw',
  },
  'bench/corpus-run/model-health-probe.unit.test.ts#askWith: timeoutMs: TIMEOUT_MS': {
    count: 1,
    bound: 'read back',
    reason: 'the deadline the probe hands its call, read back off the request the client saw',
  },
  'bench/corpus-run/model-health-probe.unit.test.ts#askOnce: timeoutMs: RUN_PER_CALL_MS': {
    count: 1,
    bound: 'other',
    reason: 'the per-call figure of a run, handed to the probe over a fake clock that the case ticks past; no cut at '
      + 'it is the finding',
  },
  'bench/corpus-run/translate-probe-run.unit.test.ts#probeClaudeLandingAfter: perCallTimeoutMs: RUN_PER_CALL_MS': {
    count: 1,
    bound: 'other',
    reason: 'the per-call figure of a run, handed to the probe over a fake clock that the case ticks past; no cut at '
      + 'it is the finding',
  },
  'bench/corpus-run/translate-probe-run.unit.test.ts#walkOnce: perCallTimeoutMs: perCallTimeoutMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'provider/delayed-transport.test-fixture.ts#landing: setTimeout landMs': {
    count: 1,
    bound: 'other',
    reason: 'the delay a scripted reply lands after, on whichever timer function is global; a case that sets it fakes '
      + 'the timers and ticks the clock, so no real time passes',
  },
  'test-fixture/fake-clock-walk.test-fixture.ts#timeoutOnFakeClock: setTimeout ms': {
    count: 1,
    bound: 'other',
    reason: 'the figure a signal factory call was handed, put on the fake clock so the signal aborts at it in fake '
      + 'time',
  },
  'test-fixture/fake-clock-walk.test-fixture.ts#waitOnFakeClock: setTimeout ms': {
    count: 1,
    bound: 'other',
    reason: 'the figure a wait on node:timers/promises was handed, put on the fake clock so the wait ends at it '
      + 'in fake time',
  },
  'test-fixture/fake-clock-walk.unit.test.ts#napping: setTimeout NAP_MS': {
    count: 1,
    bound: 'other',
    reason: FAKE_NAP,
  },
  'test-fixture/fake-clock-walk.unit.test.ts#<module>: sleepFor of node:timers/promises NAP_MS': {
    count: 3,
    bound: 'other',
    reason: FAKE_NAP,
  },
  'test-fixture/fake-clock-walk.unit.test.ts#sleepsAfterTheRelease: sleepFor of node:timers/promises NAP_MS': {
    count: 1,
    bound: 'other',
    reason: FAKE_NAP,
  },
  'corpus-pass/corpus-run/pass-entry.unit.test.ts#entryClient: wait of module-async-time 10': {
    count: 1,
    bound: 'other',
    reason: SAME_TURN_METER,
  },
  'corpus-pass/corpus-run/pass-reseat.unit.test.ts#<module>: holds.synthetic = DRY_HOLD_MS': {
    count: 6,
    bound: 'other',
    reason: RIG_REPORTS_SHORT_WINDOW,
  },
  'corpus-pass/corpus-run/pass-reseat.unit.test.ts#<module>: statedWaitMs: DRY_HOLD_MS': {
    count: 2,
    bound: 'other',
    reason: CUT_TO_SHORT_WINDOW,
  },
  'corpus-pass/corpus-run/pass-reseat.unit.test.ts#budgetsClient: cooldownMs: SHORT_HOLD_MS': {
    count: 1,
    bound: 'other',
    reason: CUT_TO_SHORT_WINDOW,
  },
  'corpus-pass/corpus-run/pass-reseat.unit.test.ts#budgetsClient: rateLimitBackoffMs: SHORT_HOLD_MS': {
    count: 1,
    bound: 'other',
    reason: CUT_TO_SHORT_WINDOW,
  },
  'run/corpus-run/run-call-config-builder.unit.test.ts#<module>: perCallTimeoutMs: 360_000': { count: 2, bound: 'other', reason: PURE_BOUND_CHECK, },
  'run/corpus-run/run-call-config-builder.unit.test.ts#<module>: perCallTimeoutMs: 900_000': {
    count: 1,
    bound: 'other',
    reason: PURE_BOUND_CHECK,
  },
  'run/corpus-run/run-call-config-builder.unit.test.ts#<module>: streamFirstContentMs: 120_000': {
    count: 3,
    bound: 'other',
    reason: PURE_BOUND_CHECK,
  },
  'run/corpus-run/run-call-config-builder.unit.test.ts#<module>: streamIdleMs: 90_000': {
    count: 3,
    bound: 'other',
    reason: PURE_BOUND_CHECK,
  },
  'run/corpus-run/run-call-config-builder.unit.test.ts#shipped: perCallTimeoutMs: 360_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'run/corpus-run/run-call-config-builder.unit.test.ts#shipped: streamFirstContentMs: 600_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'run/corpus-run/run-call-config-builder.unit.test.ts#shipped: streamIdleMs: 600_000': { count: 1, bound: 'other', reason: PURE_BOUND_CHECK, },
  'slice-cache/corpus-run/pass-schema-guard.unit.test.ts#emptyVersionTwoArtifact: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'slice-cache/corpus-run/pass-stop-before-next.unit.test.ts#<module>: softBudgetMs: ROOMY_BUDGET_MS': {
    count: 9,
    bound: 'read back',
    reason: 'a budget handed to a pure decision beside the elapsed time the case gives it; no clock is read',
  },
  'probe/corpus-run/recall-benchmark-run.unit.test.ts#<module>: perCallTimeoutMs: 360_000': {
    count: 1,
    bound: 'read back',
    reason: RECALL_CALL_CONFIG,
  },
  'probe/corpus-run/recall-benchmark-run.unit.test.ts#<module>: streamFirstContentMs: 180_000': {
    count: 1,
    bound: 'read back',
    reason: RECALL_CALL_CONFIG,
  },
  'probe/corpus-run/recall-benchmark-run.unit.test.ts#<module>: streamIdleMs: 120_000': {
    count: 1,
    bound: 'read back',
    reason: RECALL_CALL_CONFIG,
  },
  'probe/corpus-run/recall-scorecard-store.unit.test.ts#<module>: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'run/corpus-run/required-providers.unit.test.ts#refusesHyperFirst: wait of module-async-time 0': {
    count: 1,
    bound: 'other',
    reason: TIMER_TURN,
  },
  'run/corpus-run/run-timing-report.test-fixture.ts#timedCallLine: firstByteMs: 40': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'run/corpus-run/run-timing-report.test-fixture.ts#timedCallLine: firstContentMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'run/corpus-run/run-timing.unit.test.ts#<module>: firstByteMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'run/corpus-run/run-timing.unit.test.ts#<module>: firstContentMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'bench/corpus-run/sentinel-probe-run.unit.test.ts#<module>: perCallTimeoutMs: CALL_TIMEOUT_MS': {
    count: 1,
    bound: 'read back',
    reason: SENTINEL_DEADLINE,
  },
  'bench/corpus-run/sentinel-probe-run.unit.test.ts#scripted: perCallTimeoutMs: input.perCallTimeoutMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'bench/corpus-run/sentinel-probe-run.unit.test.ts#walk: perCallTimeoutMs: CALL_TIMEOUT_MS': {
    count: 1,
    bound: 'read back',
    reason: SENTINEL_DEADLINE,
  },
  'artifact/corpus-run/settled-artifact.test-fixture.ts#settledArtifactOver: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'artifact/corpus-run/settled-artifact.test-fixture.ts#settledArtifactOverRows: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'slice-cache/corpus-run/settled-carve.unit.test.ts#writeArtifact: perCallTimeoutMs: 600_000': {
    count: 1,
    bound: 'read back',
    reason: CALL_CONFIG,
  },
  'run/corpus-run/run-timing.unit.test.ts#<module>: graceMs: 30_001': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'translate-lane/document-lanes.unit.test.ts#lanesClient: wait of module-async-time 10': {
    count: 1,
    bound: 'other',
    reason: SAME_TURN_METER,
  },
  'provider/hyper-client.unit.test.ts#<module>: wait of module-async-time SLOW_TRANSPORT_MS': {
    count: 1,
    bound: 'other',
    reason: 'a settle wait for calls to reach a gated transport, which they do within one turn of the event loop: '
      + 'the case passes with microtask turns only and fails with none (census of 2026-10-06)',
  },
  'provider/hyper-client.unit.test.ts#transport: wait of module-async-time SLOW_TRANSPORT_MS': {
    count: 1,
    bound: 'other',
    reason: SAME_TURN_METER,
  },
  'corpus-pass/lane-contest-driver.unit.test.ts#drive: wait of module-async-time startPosition < ROSTER.length ? 20 : 5': {
    count: 1,
    bound: 'other',
    reason: SERIAL_HEAD_START,
  },
  'lane-contest/lane-contest-stage.unit.test.ts#<module>: graceMs: 0': {
    count: 1,
    bound: 'reached',
    reason: 'the window of 0 that cuts the seats scripted to answer only past twice the hang stop, abandoned at '
      + 'quorum on any machine',
  },
  'lane-contest/lane-contest-stage.unit.test.ts#cannedTransport: abortableWait of node:timers/promises delayByModel[at] ?? 0': {
    count: 1,
    bound: 'other',
    reason: 'the delay each case scripts per seat: none, 30 ms for voices that land inside a window that is the hang '
      + 'stop, or twice the hang stop for seats a window of 0 cuts',
  },
  'typography/log-context.unit.test.ts#settleOne: wait of module-async-time 0': { count: 1, bound: 'other', reason: TIMER_TURN, },
  'markup/monotonic-clock.unit.test.ts#<module>: wait of module-async-time 5': {
    count: 1,
    bound: 'other',
    reason: 'a real wait the case crosses, after which it asserts only that the reading has not gone back, which load '
      + 'cannot break',
  },
  'provider/neuralwatt-routing.unit.test.ts#<module>: freshForMs: 0': { count: 1, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/openrouter-abandoned-spend.unit.test.ts#<module>: firstByteMs: 500': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider/openrouter-abandoned-spend.unit.test.ts#<module>: firstContentMs: 500': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'stage/overlapped-map.unit.test.ts#settle: wait of module-async-time 0': { count: 1, bound: 'other', reason: TIMER_TURN, },
  'provider/provider-budget.unit.test.ts#<module>: cooldownMs: 10_000': { count: 7, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget.unit.test.ts#<module>: cooldownMs: 300': { count: 2, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget.unit.test.ts#<module>: freshForMs: 100': { count: 1, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget.unit.test.ts#<module>: freshForMs: 500': { count: 2, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget.unit.test.ts#<module>: rateLimitBackoffMs: 300': { count: 7, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget.unit.test.ts#<module>: rateLimitBackoffMs: 50': { count: 2, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget.unit.test.ts#<module>: statedWaitMs: 5_000': { count: 2, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/provider-budget.unit.test.ts#credits: wait of module-async-time 20': {
    count: 4,
    bound: 'other',
    reason: 'a meter read held while reads launched in the same turn arrive; the cases pass with no hold at all '
      + '(census of 2026-10-06), so it decides nothing',
  },
  'provider/provider-budget.unit.test.ts#quotas: wait of module-async-time 20': {
    count: 2,
    bound: 'other',
    reason: 'a meter read held while reads launched in the same turn arrive; the cases pass with no hold at all '
      + '(census of 2026-10-06), so it decides nothing',
  },
  'provider/provider-router-decide.unit.test.ts#expectMarkedAndThrown: statedWaitMs: statedWaitMsOf({ error: refusal, },)': {
    count: 1,
    bound: 'read back',
    reason: 'the wait a refusal states, read back in what the router marked',
  },
  'provider/provider-router-decide.unit.test.ts#markRefused: statedWaitMs: statedWaitMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'provider/provider-router-model-hold.unit.test.ts#<module>: modelHoldMs: HOLD_MS': {
    count: 1,
    bound: 'read back',
    reason: SCRIPTED_CLOCK,
  },
  'provider/provider-router-stream-bound.unit.test.ts#chatText: boundMs: BOUND_MS': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider/provider-router-stream-bound.unit.test.ts#chatText: firstByteMs: 15_000': {
    count: 1,
    bound: 'read back',
    reason: RECORD_FIELD,
  },
  'provider/provider-router-stream-bound.unit.test.ts#chatText: firstContentMs: 15_000': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider/provider-router-stream-bound.unit.test.ts#routerOver: modelHoldMs: HOLD_MS': {
    count: 1,
    bound: 'read back',
    reason: 'a hold on the case\'s own injected clock, which the case moves by hand and reads back in the refusal',
  },
  'provider/provider-router.unit.test.ts#chatText: wait of module-async-time SLOT_HOLD_MS': {
    count: 1,
    bound: 'other',
    reason: 'a slot hold the cases pass without (census of 2026-10-06), so it decides nothing',
  },
  'provider/provider-router.unit.test.ts#markRefused: statedWaitMs = 0': {
    count: 1,
    bound: 'read back',
    reason: 'the default of a stub\'s parameter, recording the stated wait the router hands it',
  },
  'repair/refine-phase.unit.test.ts#measuringRefiners: wait of module-async-time startPosition === 0 ? 20 : 5': {
    count: 1,
    bound: 'other',
    reason: SERIAL_HEAD_START,
  },
  'repair/repair-benchmark.unit.test.ts#<module>: runBudgetMs: 0': {
    count: 1,
    bound: 'reached',
    reason: 'a budget of 0 that every entry finds spent, on any machine',
  },
  'repair/repair-benchmark.unit.test.ts#<module>: runBudgetMs: HOUR_MS': {
    count: 1,
    bound: 'read back',
    reason: 'the budget of an hour the case\'s name states, against a two-hour step of the stubbed wall clock; the '
      + 'run takes milliseconds of the monotonic clock it is judged on',
  },
  'corpus-pass/repair-translation.unit.test.ts#steeringClient: wait of module-async-time 10': {
    count: 2,
    bound: 'other',
    reason: SAME_TURN_METER,
  },
  'provider/request-pace.unit.test.ts#<module>: windowMs: 1_000': { count: 1, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/request-pace.unit.test.ts#<module>: windowMs: REAL_WINDOW_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the window the case sleeps through on the real clock, with the pacer\'s clock held still until the '
      + 'second take sleeps, and only a lower bound asserted',
  },
  'provider/request-pace.unit.test.ts#<module>: windowMs: WINDOW_MS': { count: 6, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/request-pace.unit.test.ts#builds: windowMs: 1_000': {
    count: 2,
    bound: 'read back',
    reason: 'a window of a pacer whose construction is refused, so it is never armed',
  },
  'provider/request-pace.unit.test.ts#patienceRunsOut: pause of module-async-time RELEASE_PATIENCE_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the patience that fails the case when a release is stuck; the release settles before any timer (the '
      + 'case passes with a patience of 0, census of 2026-10-06)',
  },
  'provider/request-pace.unit.test.ts#scriptedPace: windowMs: WINDOW_MS': { count: 1, bound: 'read back', reason: SCRIPTED_CLOCK, },
  'provider/retry-abort-after-transport-failure.unit.test.ts#<module>: setTimeout 0': {
    count: 1,
    bound: 'reached',
    reason: 'the caller abort, armed before the ladder\'s backoff of at least 10 s, so it fires first on any machine',
  },
  'stage/stage-call.unit.test.ts#callWith: exchangeTimeoutMs: 1_000': {
    count: 1,
    bound: 'read back',
    reason: 'the deadline the pass-through case reads back off the request the client saw; the scripted client arms '
      + 'none',
  },
  'stage/stage-quorum.unit.test.ts#<module>: graceMs: 50': {
    count: 1,
    bound: 'reached',
    reason: 'the window the abandoning case reaches: the hung seat answers only once it is aborted',
  },
  'stage/stage-quorum.unit.test.ts#<module>: graceMs: RECOVERY_GRACE_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the window the bounded recovery case reaches: the re-asked seat answers only once it is aborted',
  },
  'stage/stage-quorum.unit.test.ts#hangingClient: wait of module-async-time lateMs': {
    count: 1,
    bound: 'other',
    reason: 'the delay of a late voice, 20 ms, which lands inside a window that is the hang stop',
  },
  'stage/stage-round-refill.unit.test.ts#<module>: graceMs: GRACE_MS': {
    count: 6,
    bound: 'reached',
    reason: 'the window that cuts the slow seat or, in the cases where the grace opens on one voice fewer (ledger B816), '
      + 'the stuck seat, either of which answers only once it is aborted',
  },
  'stage/stage-round-refill.unit.test.ts#scriptedClient: setTimeout STUCK_MS': {
    count: 1,
    bound: 'other',
    reason: 'the stuck seat giving up in transport, cleared when the grace cuts it first; reached only by a round that '
      + 'waits for every live seat, where it ends the case in a failed assertion rather than a hang',
  },
  'stage/stage-round.unit.test.ts#<module>: graceMs: GRACE_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the window the case that loses a voice reaches: its hanging seat answers only once the round cuts it, '
      + 'and its slow voice, armed before the window and shorter, lands inside it on any machine',
  },
  'stage/stage-round.unit.test.ts#<module>: wait of module-async-time SLOW_MS': {
    count: 1,
    bound: 'other',
    reason: 'the turn at which the whole-roster case reads the round\'s line: armed after the slow voice\'s own wait '
      + 'and as long, so it fires after that answer and its microtasks however long the loop stalls (measured '
      + '2026-10-10); a round still waiting on a window has written no line by then',
  },
  'stage/stage-round.unit.test.ts#scheduledClient: wait of module-async-time SLOW_MS': {
    count: 1,
    bound: 'other',
    reason: 'the delay of the slow voice: inside the window in the case that loses a voice, and before quorum, as a '
      + 'floor only, in the whole-roster case',
  },
  'stage/stage-settings.test-fixture.ts#scriptedStageSettingsWaiting: roundMs: graceMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'stage/stage-settings.test-fixture.ts#scriptedStageSettingsWaiting: writerMs: graceMs': {
    count: 1,
    bound: 'other',
    reason: HANDED_ON,
  },
  'stage/stage-settings.unit.test.ts#<module>: roundMs: 90_000': { count: 1, bound: 'read back', reason: READER_OUTPUT, },
  'stage/stage-settings.unit.test.ts#<module>: roundMs: STRAGGLER_GRACE_MS': {
    count: 1,
    bound: 'read back',
    reason: READER_OUTPUT,
  },
  'stage/stage-settings.unit.test.ts#<module>: writerMs: 240_000': { count: 1, bound: 'read back', reason: READER_OUTPUT, },
  'stage/stage-settings.unit.test.ts#<module>: writerMs: WRITER_GRACE_MS': {
    count: 1,
    bound: 'read back',
    reason: READER_OUTPUT,
  },
  'stage/stage-windowed-rounds.unit.test.ts#runBench: graceMs: graceMs': { count: 1, bound: 'other', reason: HANDED_ON, },
  'provider/stream-bound.unit.test.ts#<module>: boundMs: SHORT_BOUND_MS': {
    count: 2,
    bound: 'reached',
    reason: 'the bound the cutting case reaches, armed before its wait of twice the length, and the same value as the '
      + 'bound field of the error the reading case builds',
  },
  'provider/stream-bound.unit.test.ts#<module>: firstByteMs: 15_000': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider/stream-bound.unit.test.ts#<module>: firstContentMs: 15_000': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider/stream-bound.unit.test.ts#<module>: wait of module-async-time SHORT_BOUND_MS * 2': {
    count: 1,
    bound: 'reached',
    reason: 'the wait that must outlast a bound armed before it and half its length, so the bound fires first on any '
      + 'machine',
  },
  'provider-stream/stream-cut.unit.test.ts#<module>: firstByteMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-cut.unit.test.ts#<module>: firstContentMs: -1': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-cut.unit.test.ts#<module>: firstContentMs: 55': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-drain.unit.test.ts#<module>: firstByteMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-drain.unit.test.ts#<module>: firstContentMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-drain.unit.test.ts#<module>: idleMs: 60_000': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-idle-guard.unit.test.ts#<module>: firstContentMs: TINY_MS': {
    count: 3,
    bound: 'reached',
    reason: 'the window the tripping cases reach, waited out by polling the guard\'s own signal, and the one the '
      + 'disposal case defuses',
  },
  'provider-stream/stream-idle-guard.unit.test.ts#<module>: idleMs: 1_500': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-idle-guard.unit.test.ts#<module>: idleMs: TINY_MS': {
    count: 3,
    bound: 'reached',
    reason: 'the window the tripping cases reach, waited out by polling the guard\'s own signal, and the one the '
      + 'disposal case defuses',
  },
  'provider-stream/stream-idle-guard.unit.test.ts#<module>: wait of module-async-time TINY_MS': {
    count: 3,
    bound: 'other',
    reason: 'a real span so the first byte lands past zero, and two more so the first text lands past the first byte '
      + 'and a later piece of text past the first text; the cases assert only lower bounds, which load widens',
  },
  'provider-stream/stream-idle-guard.unit.test.ts#settle: setTimeout TINY_MS * 5': {
    count: 1,
    bound: 'reached',
    reason: 'the wait past windows a disposal defused, armed after them and five times as long, so a window left '
      + 'armed fires inside it on any machine',
  },
  'provider-stream/stream-idle-guard.unit.test.ts#tick: setTimeout TINY_MS': {
    count: 1,
    bound: 'other',
    reason: 'the poll interval of a wait that ends on the guard\'s own signal, not on time',
  },
  'provider-stream/stream-overrun.unit.test.ts#<module>: firstByteMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/stream-overrun.unit.test.ts#<module>: firstContentMs: 40': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider-stream/transport-first-content-window.unit.test.ts#<module>: firstContentMs: WINDOW_MS': { count: 1, bound: 'reached', reason: STREAM_WINDOW_REACHED, },
  'provider-stream/transport-first-content-window.unit.test.ts#keepAliveResponse: wait of module-async-time GAP_MS': { count: 1, bound: 'other', reason: CHUNK_PACING, },
  'provider/synthetic-client.unit.test.ts#<module>: exchangeTimeoutMs: 50': {
    count: 1,
    bound: 'reached',
    reason: 'the deadline the hung-exchange case reaches: the transport answers only once it is aborted',
  },
  'provider/synthetic-client.unit.test.ts#<module>: exchangeTimeoutMs: QUEUED_DEADLINE_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the deadline the queued call must not reach: shorter than its wait in the queue, while its exchange '
      + 'answers at once when its slot opens',
  },
  'provider/synthetic-client.unit.test.ts#<module>: wait of module-async-time SETTLE_MS': {
    count: 2,
    bound: 'other',
    reason: 'a settle wait for calls to reach a gated transport, which they do within one turn of the event loop: '
      + 'the case passes with microtask turns only and fails with none (census of 2026-10-06)',
  },
  'provider/synthetic-client.unit.test.ts#holdFirstReply: wait of module-async-time QUEUE_HOLD_MS': {
    count: 1,
    bound: 'reached',
    reason: 'the hold that keeps the queued call waiting past its own deadline',
  },
  'provider/transient-retry.unit.test.ts#<module>: firstByteMs: 10': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider/transient-retry.unit.test.ts#<module>: firstContentMs: 10': { count: 1, bound: 'read back', reason: RECORD_FIELD, },
  'provider/transient-retry.unit.test.ts#<module>: setTimeout 0': {
    count: 1,
    bound: 'reached',
    reason: 'the caller abort, armed before the ladder\'s backoff of at least 10 s, so it fires first on any machine',
  },
  'corpus-pass/translate-document.unit.test.ts#laneClient: wait of module-async-time 10': {
    count: 1,
    bound: 'other',
    reason: SAME_TURN_METER,
  },
  'stage/twin-memo.unit.test.ts#settle: wait of module-async-time 0': { count: 1, bound: 'other', reason: TIMER_TURN, },
  'test-fixture/wall-clock-stub.unit.test.ts#<module>: sleepFor of node:timers/promises 1': {
    count: 1,
    bound: 'other',
    reason: 'a timer turn the case crosses to show the stub holds across it; its length decides nothing',
  },
  'stage/writer-grace-override.unit.test.ts#<module>: roundMs: CALIBRATION_STRAGGLER_GRACE_MS': {
    count: 1,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: roundMs: LONG_ROUND': {
    count: 2,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: roundMs: ROUND': {
    count: 6,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: roundMs: STRAGGLER_GRACE_MS': {
    count: 3,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: writerMs: CALIBRATION_STRAGGLER_GRACE_MS': {
    count: 1,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: writerMs: LONG_ROUND': {
    count: 2,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: writerMs: ROUND': {
    count: 1,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: writerMs: WRITER': {
    count: 4,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
  'stage/writer-grace-override.unit.test.ts#<module>: writerMs: WRITER_GRACE_MS': {
    count: 4,
    bound: 'read back',
    reason: WRITER_GRACE_READ_BACK,
  },
};

/**
 Local names one file's imports give the timers and the shared hang stop.
 */
type TimerNames = {
  /**
   Local names of `setTimeout` from `node:timers/promises`.
   */
  readonly promiseSleeps: ReadonlySet<string>;

  /**
   Local names of `wait` from `@monochromatic-dev/module-async-time/ts`.
   */
  readonly waits: ReadonlySet<string>;

  /**
   Local names of the shared hang stop.
   */
  readonly hangStops: ReadonlySet<string>;

  /**
   Local names of `createBedrockClient` from the built package.
   */
  readonly bedrockClients: ReadonlySet<string>;
};

/**
 Reads the local names a file's imports bind to the timers and the shared
 hang stop.

 @param file - file read, whose path resolves its relative imports

 @param program - file's program

 @returns The local names, empty sets where the file imports none

 @example
 ```ts
 const names = timerNamesOf({ file, program, },);
 ```
 */
function timerNamesOf(
  {
    file,
    program,
  }: {
    readonly file: SourceText;
    readonly program: TreeNode;
  },
): TimerNames {
  /**
   Local names of the promise sleep.
   */
  const promiseSleeps = new Set<string>();
  /**
   Local names of the module-async-time wait.
   */
  const waits = new Set<string>();
  /**
   Local names of the shared hang stop.
   */
  const hangStops = new Set<string>();
  /**
   Local names of the Bedrock client's builder.
   */
  const bedrockClients = new Set<string>();
  for (const statement of (program.body as readonly TreeNode[])) {
    if (statement.type !== 'ImportDeclaration')
      continue;
    /**
     Module the declaration names.
     */
    const from = literalText({ node: statement.source, },);
    for (const specifier of (statement.specifiers as readonly TreeNode[])) {
      if (specifier.type !== 'ImportSpecifier')
        continue;
      /**
       Name the module exports, written as an identifier or a string.
       */
      const exported = identifierName({ node: specifier.imported, },) || literalText({ node: specifier.imported, },);
      /**
       The module declaring that name when the specifier names a file of the family, the specifier otherwise.
       */
      const landsAt = declaringModule({
        fromPath: file.path,
        specifier: from,
        imported: exported,
      },);
      /**
       Name the file uses.
       */
      const local = identifierName({ node: specifier.local, },);
      if ((from === 'node:timers/promises') && (exported === 'setTimeout'))
        promiseSleeps.add(local,);
      if ((from === '@monochromatic-dev/module-async-time/ts') && (exported === 'wait'))
        waits.add(local,);
      if (HANG_STOP_PATHS.has(landsAt,) && (exported === HANG_STOP_NAME))
        hangStops.add(local,);
      if ((from.endsWith(BUILT_INDEX_SUFFIX,) || (landsAt === BEDROCK_CLIENT_MODULE)) && (exported === BEDROCK_CLIENT_NAME))
        bedrockClients.add(local,);
    }
  }
  return {
    promiseSleeps,
    waits,
    hangStops,
    bedrockClients,
  };
}

/**
 A bound one node sets: its form as the key writes it and the expression
 that gives its value.
 */
type BoundRead = {
  /**
   Form, such as `AbortSignal.timeout` or `graceMs:`.
   */
  readonly form: string;

  /**
   Value expression, absent where a timer is called without a delay.
   */
  readonly value: unknown;
};

/**
 Whether the options a Bedrock client is built with name its stream bound
 override, whose value the object walk then reads; options that are no
 object literal, or carry it only through a spread, do not.

 @param options - first argument of the call

 @returns Whether the literal names `streamBoundMsOverride` as a key of its own

 @example
 ```ts
 const named = namesStreamBoundOverride({ options: args[0], },);
 ```
 */
function namesStreamBoundOverride({ options, }: { readonly options: unknown; },): boolean {
  /**
   The options inside any parentheses and casts.
   */
  const { inner, } = unwrapped({ node: options, },);
  if ((!isTreeNode(inner,)) || (inner.type !== 'ObjectExpression'))
    return false;
  return (inner.properties as readonly TreeNode[]).some(function namesOverride(property,): boolean {
    return (property.type === 'Property')
      && (property.computed !== true)
      && ((identifierName({ node: property.key, },) || literalText({ node: property.key, },)) === 'streamBoundMsOverride');
  },);
}

/**
 The bounds a call node sets.

 @param node - call expression read

 @param names - local names the file gives the timers

 @returns One read where the call arms a timer, none otherwise

 @example
 ```ts
 const reads = callBounds({ node, names, },);
 ```
 */
function callBounds(
  {
    node,
    names,
  }: {
    readonly node: TreeNode;
    readonly names: TimerNames;
  },
): readonly BoundRead[] {
  /**
   The callee inside any parentheses and casts.
   */
  const { inner: callee, } = unwrapped({ node: node.callee, },);
  /**
   Arguments the call passes.
   */
  const args = node.arguments as readonly unknown[];
  if (isTreeNode(callee,) && (callee.type === 'MemberExpression')) {
    return ((memberName({ node: callee, },) === 'timeout') && (identifierName({ node: callee.object, },) === 'AbortSignal'))
      ? [{
        form: 'AbortSignal.timeout',
        value: args[0],
      },]
      : [];
  }
  /**
   The callee's name, empty for anything but a plain identifier.
   */
  const name = identifierName({ node: callee, },);
  if (names.bedrockClients.has(name,)) {
    return namesStreamBoundOverride({ options: args[0], },)
      ? []
      : [{
        form: `${name} of the built package`,
        value: undefined,
      },];
  }
  if (names.promiseSleeps.has(name,)) {
    return [{
      form: `${name} of node:timers/promises`,
      value: args[0],
    },];
  }
  if (names.waits.has(name,)) {
    return [{
      form: `${name} of module-async-time`,
      value: args[0],
    },];
  }
  return ((name === 'setTimeout') || (name === 'setInterval'))
    ? [{
      form: name,
      value: args[1],
    },]
    : [];
}

/**
 The bounds an object node sets: each bound key's value in an object
 literal, and each bound key's default in a destructuring pattern, where a
 key with no default binds a name and sets nothing.

 @param node - object literal or object pattern read

 @returns One read per bound key that sets a value, none for any other node

 @example
 ```ts
 const reads = objectBounds({ node, },);
 ```
 */
function objectBounds({ node, }: { readonly node: TreeNode; },): readonly BoundRead[] {
  if ((node.type !== 'ObjectExpression') && (node.type !== 'ObjectPattern'))
    return [];
  return (node.properties as readonly TreeNode[]).flatMap(function boundOf(property,): readonly BoundRead[] {
    /**
     The key, written as an identifier or a string; empty where computed.
     */
    const key = ((property.type !== 'Property') || (property.computed === true))
      ? ''
      : (identifierName({ node: property.key, },) || literalText({ node: property.key, },));
    if (!BOUND_KEYS.has(key,))
      return [];
    /**
     The property's value: an expression, or in a pattern a binding.
     */
    const { value, } = property;
    if (node.type === 'ObjectExpression') {
      return [{
        form: `${key}:`,
        value,
      },];
    }
    return (isTreeNode(value,) && (value.type === 'AssignmentPattern'))
      ? [{
        form: `${key} =`,
        value: value.right,
      },]
      : [];
  },);
}

/**
 Whether a name is one a case gives a record of holds.

 @param name - constant's name

 @returns Whether one word of the name, split at underscores, hyphens and case changes, is hold, holds or
 held, so that `NO_HOLDS`, `noHolds` and `HYPER_HELD` name holds and `threshold` does not

 @example
 ```ts
 const named = namesAHold({ name: 'HYPER_HELD', },); // true
 ```
 */
function namesAHold({ name, }: { readonly name: string; },): boolean {
  /**
   The name with a separator before each capital that follows a lower-case letter, so camel case reads as words.
   */
  const spaced = Array.from(name,)
    .map(function separated(char, at, chars,): string {
      /**
       Character before this one.
       */
      const before = chars.at(at - 1,) ?? '';
      return ((at > 0) && (char !== char.toLowerCase()) && (before === before.toLowerCase()))
        ? `_${char}`
        : char;
    },)
    .join('',);
  return spaced
    .toLowerCase()
    .replaceAll(
    '-',
    '_',
    )
    .replaceAll(
    '$',
    '_',
    )
    .split('_',)
    .some(function isHoldWord(word,): boolean {
      return HOLD_WORDS.has(word,);
    },);
}

/**
 Numbers the top-level constants of a file are bound to, so a hold written as a named constant reads as its
 value. A constant that spells more than one number, or none, is left out.

 @param program - file's program

 @returns Each constant's name and the number its initializer spells

 @example
 ```ts
 const numbers = numericConstantsOf({ program, },); // Map { 'LONG_MS' => 300000 }
 ```
 */
function numericConstantsOf({ program, }: { readonly program: TreeNode; },): ReadonlyMap<string, number> {
  /**
   Constants read so far, each only after the constants it reads.
   */
  const numbers = new Map<string, number>();
  for (const statement of (program.body as readonly TreeNode[])) {
    /**
     The declaration, inside an export when the file exports it.
     */
    const declaration = (statement.type === 'ExportNamedDeclaration') ? statement.declaration : statement;
    if ((!isTreeNode(declaration,)) || (declaration.type !== 'VariableDeclaration'))
      continue;
    for (const declarator of (declaration.declarations as readonly TreeNode[])) {
      /**
       Numbers the initializer spells.
       */
      const spelled = numbersOf({
        node: declarator.init,
        numbers,
      },);
      if (spelled.length === 1) {
        numbers.set(
          identifierName({ node: declarator.id, },),
          nonNullishOrThrow(spelled[0],),
        );
      }
    }
  }
  return numbers;
}

/**
 How each binary operator a hold may be written with combines its operands.
 */
const OPERATORS: Readonly<Record<string, (left: number, right: number,) => number>> = {
  '*': function times(left, right,): number {
    return left * right;
  },
  '+': function plus(left, right,): number {
    return left + right;
  },
  '-': function minus(left, right,): number {
    return left - right;
  },
};

/**
 The numbers an expression can spell: a numeric literal, a constant bound to one, both branches of a conditional,
 or a sum, difference or product of such numbers.

 @param node - expression read

 @param numbers - top-level constants and their numbers

 @returns Every number the expression can spell, none where it spells none

 @example
 ```ts
 const spelled = numbersOf({ node: init, numbers, },); // [300000, 0] for held ? LONG_MS : 0
 ```
 */
function numbersOf(
  {
    node,
    numbers,
  }: {
    readonly node: unknown;
    readonly numbers: ReadonlyMap<string, number>;
  },
): readonly number[] {
  /**
   The expression inside any parentheses and casts.
   */
  const { inner, } = unwrapped({ node, },);
  if (!isTreeNode(inner,))
    return [];
  if (inner.type === 'Literal')
    return ((typeof inner.value) === 'number') ? [inner.value as number,] : [];
  if (inner.type === 'Identifier') {
    /**
     The number the constant is bound to.
     */
    const bound = numbers.get(identifierName({ node: inner, },),);
    return (bound === undefined) ? [] : [bound,];
  }
  if (inner.type === 'ConditionalExpression') {
    return [
      ...numbersOf({
        node: inner.consequent,
        numbers,
      },),
      ...numbersOf({
        node: inner.alternate,
        numbers,
      },),
    ];
  }
  if ((inner.type !== 'BinaryExpression') || (!Object.hasOwn(
    OPERATORS,
    inner.operator as string,
  )))
    return [];
  /**
   How the operator combines two numbers.
   */
  const combine = nonNullishOrThrow(OPERATORS[inner.operator as string],);
  /**
   The numbers the right operand can spell.
   */
  const rights = numbersOf({
    node: inner.right,
    numbers,
  },);
  return numbersOf({
    node: inner.left,
    numbers,
  },)
    .flatMap(function withEachRight(left,): readonly number[] {
      return rights.map(function combined(right,): number {
        return combine(
          left,
          right,
        );
      },);
    },);
}

/**
 Whether a node opens a record of holds: a property a run client answers its holds reader under, or a
 member named `holds`, or a constant named for a hold that an object literal initializes.

 @param node - node visited

 @returns Whether the properties inside it are providers' holds

 @example
 ```ts
 const opens = opensHolds({ node, },); // true for `providerHolds: () => ({ ... })`
 ```
 */
function opensHolds({ node, }: { readonly node: TreeNode; },): boolean {
  if ((node.type === 'Property') && (node.computed !== true)) {
    /**
     The property's key.
     */
    const key = identifierName({ node: node.key, },);
    return (key === HOLDS_READER_KEY) || (key === HOLDS_MEMBER);
  }
  if (node.type !== 'VariableDeclarator')
    return false;
  /**
   The initializer inside any parentheses and casts.
   */
  const { inner, } = unwrapped({ node: node.init, },);
  return isTreeNode(inner,) && (inner.type === 'ObjectExpression')
    && namesAHold({ name: identifierName({ node: node.id, },), },);
}

/**
 Whether an assignment writes onto a rig's holds: its target is a member of something called `holds`.

 @param target - left side of the assignment

 @returns Whether the target is `holds.x` or `rig.holds.x`, written with a dot or a string key

 @example
 ```ts
 const onHolds = writesOntoHolds({ target, },); // true for rig.holds.synthetic
 ```
 */
function writesOntoHolds({ target, }: { readonly target: unknown; },): boolean {
  if ((!isTreeNode(target,)) || (target.type !== 'MemberExpression'))
    return false;
  /**
   What the target is a member of.
   */
  const { object, } = target;
  if (!isTreeNode(object,))
    return false;
  return (identifierName({ node: object, },) === HOLDS_MEMBER)
    || ((object.type === 'MemberExpression') && (memberName({ node: object, },) === HOLDS_MEMBER));
}

/**
 The holds of a second or more a node registers for a provider: an assignment onto a rig's holds, whose value
 that spells no number under that second is a finding too since the case cannot show it short, and a property
 of a record of holds whose value is a number from that second.

 @param node - node visited

 @param holding - whether the node sits inside a record of holds

 @param numbers - top-level constants of the file and their numbers

 @returns One read per hold of a second or more, none for any other node

 @example
 ```ts
 const reads = holdBounds({ node, holding, numbers, },);
 ```
 */
function holdBounds(
  {
    node,
    holding,
    numbers,
  }: {
    readonly node: TreeNode;
    readonly holding: boolean;
    readonly numbers: ReadonlyMap<string, number>;
  },
): readonly BoundRead[] {
  if (node.type === 'AssignmentExpression') {
    if ((node.operator !== '=') || (!writesOntoHolds({ target: node.left, },)))
      return [];
    /**
     The holds the assignment can write.
     */
    const written = numbersOf({
      node: node.right,
      numbers,
    },);
    return ((written.length > 0) && written.every(function isShort(hold,): boolean {
      return hold < HOLD_READ_MS;
    },))
      ? []
      : [{
        form: `${HOLDS_MEMBER}.${memberName({ node: node.left as TreeNode, },)} =`,
        value: node.right,
      },];
  }
  if ((!holding) || (node.type !== 'ObjectExpression'))
    return [];
  return (node.properties as readonly TreeNode[])
    .filter(function holdsASecond(property,): boolean {
      return (property.type === 'Property') && numbersOf({
        node: property.value,
        numbers,
      },)
        .some(function isASecond(hold,): boolean {
          return hold >= HOLD_READ_MS;
        },);
    },)
    .map(function holdOf(property,): BoundRead {
      return {
        form: `hold ${identifierName({ node: property.key, },) || literalText({ node: property.key, },)}:`,
        value: property.value,
      };
    },);
}

/**
 How a bound's value reads in its key: its source text, each line trimmed
 and the lines joined by a space, or `(none)` where the timer is called
 without a delay.

 @param file - file the value sits in

 @param value - value expression read

 @returns Text for the key

 @example
 ```ts
 const text = valueText({ file, value, },); // '5_000'
 ```
 */
function valueText(
  {
    file,
    value,
  }: {
    readonly file: SourceText;
    readonly value: unknown;
  },
): string {
  if (!isTreeNode(value,))
    return '(none)';
  return file.text
    .slice(
    value.start,
    value.end,
    )
    .split('\n',)
    .map(function trimmed(line,): string {
      return line.trim();
    },)
    .join(' ',);
}

/**
 Every bound on the real clock in the test files and fixtures given whose
 value is not the shared hang stop, keyed `path#site: form value`.

 @param files - files read; package source among them is skipped

 @returns Keys sorted, repeated once per read

 @example
 ```ts
 const found = unsharedBounds({ files, },);
 ```
 */
function unsharedBounds({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Keys found so far.
   */
  const found: string[] = [];
  for (const file of files) {
    if (!file.isTest)
      continue;
    /**
     The file's program.
     */
    const { program, } = parseSource({ file, },);
    /**
     Local names the file gives the timers and the hang stop.
     */
    const names = timerNamesOf({
      file,
      program,
    },);
    /**
     Nodes still to visit, each with its enclosing named function.
     */
    const pending: {
      readonly node: TreeNode;
      readonly site: string;
      readonly holding: boolean;
    }[] = [{
      node: program,
      site: '<module>',
      holding: false,
    },];
    /**
     Top-level constants of the file and the numbers they spell, which a hold may be written as.
     */
    const numbers = numericConstantsOf({ program, },);
    for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
      /**
       Node visited now, and the site it sits in.
       */
      const {
        node,
        site,
        holding,
      } = next;
      /**
       Enclosing named function for the node's children.
       */
      const here = (NAMING_KINDS.has(node.type,) && isTreeNode(node.id,)) ? identifierName({ node: node.id, },) : site;
      /**
       Bounds the node sets.
       */
      const reads = [
        ...((node.type === 'CallExpression')
          ? callBounds({
            node,
            names,
          },)
          : objectBounds({ node, },)),
        ...holdBounds({
          node,
          holding,
          numbers,
        },),
      ];
      for (const read of reads) {
        /**
         The value inside any parentheses and casts.
         */
        const { inner, } = unwrapped({ node: read.value, },);
        /**
         Whether the read sets a stream bound of `'unbounded'`, which arms
         nothing.
         */
        const unbounded = UNBOUNDED_FORMS.has(read.form,) && (literalText({ node: inner, },) === UNBOUNDED);
        if ((!names.hangStops.has(identifierName({ node: inner, },),)) && (!unbounded)) {
          found.push(`${file.path}#${here}: ${read.form} ${
            valueText({
              file,
              value: inner,
            },)
          }`,);
        }
      }
      /**
       Whether the children sit inside a record of holds.
       */
      const holdingBelow = holding || opensHolds({ node, });
      pending.push(...childNodes({ node, },).map(function withSite(child,) {
        return {
          node: child,
          site: here,
          holding: holdingBelow,
        };
      },),);
    }
  }
  return found.toSorted();
}

/**
 The path a finding's key names, before its site.

 @param key - finding read

 @returns Its path relative to `src`

 @example
 ```ts
 const path = pathOfKey({ key: 'litter.unit.test.ts#purr: setTimeout 20', },); // 'litter.unit.test.ts'
 ```
 */
function pathOfKey({ key, }: { readonly key: string; },): string {
  return key.slice(
    0,
    key.indexOf('#',),
  );
}

/**
 What the package-wide case compares the scan's findings with: each listed
 key as often as it is listed, and each deferred file that holds no finding,
 so a deferral that outlived its file's bounds shows as a difference.

 @param found - keys the scan found, deferred files' among them

 @param listed - bounds that keep a value of their own

 @param deferred - files whose bounds wait for another change

 @returns The scan's findings outside the deferred files, and the entries
 they must equal, both sorted

 @example
 ```ts
 const { findings, expected, } = againstListed({ found, listed: LISTED, deferred: DEFERRED, },);
 ```
 */
function againstListed(
  {
    found,
    listed,
    deferred,
  }: {
    readonly found: readonly string[];
    readonly listed: Readonly<Record<string, ListedBound>>;
    readonly deferred: Readonly<Record<string, string>>;
  },
): {
  readonly findings: readonly string[];
  readonly expected: readonly string[];
} {
  /**
   Files that hold at least one finding.
   */
  const holding = new Set(found.map(function pathOf(key,): string {
    return pathOfKey({ key, },);
  },),);
  return {
    findings: [
      ...found.filter(function outsideDeferred(key,): boolean {
        return !Object.hasOwn(
          deferred,
          pathOfKey({ key, },),
        );
      },),
      ...Object.keys(deferred,)
        .filter(function holdsNone(path,): boolean {
          return !holding.has(path,);
        },)
        .map(function staleDeferral(path,): string {
          return `${path}: deferred, and holds no bound outside the hang stop`;
        },),
    ].toSorted(),
    expected: Object.entries(listed,)
      .flatMap(function copies([key, entry,],): readonly string[] {
        return Array.from(
          { length: entry.count, },
          function copy(): string {
            return key;
          },
        );
      },)
      .toSorted(),
  };
}

//endregion Real-clock bounds

await describe({
  name: 'real-clock bounds in tests (commit 5086b6c6f, ledger B318)',
  children: [
    it({
      name: 'FINDS a bare or local bound in AbortSignal.timeout, a global setTimeout and setInterval, the promise '
        + 'sleep and module-async-time wait under any local name, a bound key\'s value and its destructured default, '
        + 'both windows of a writer grace, and an expression over the hang stop, and leaves the imported hang stop under any local name, keys outside '
        + 'the bound keys, timers no import names, package source, and a deferral that still holds a finding',
      fn: async () => {
        /**
         Every form the scan reads, and every form it leaves.
         */
        const files: readonly SourceText[] = [
          {
            path: 'litter.unit.test.ts',
            text: [
              'import { setTimeout as nap, } from \'node:timers/promises\';',
              'import { wait as snooze, } from \'@monochromatic-dev/module-async-time/ts\';',
              'import { HANG_STOP_MS, } from \'./hang-stop.test-fixture.ts\';',
              'const TINY_MS = 20;',
              'export async function purr({ graceMs = 50, perCallTimeoutMs = HANG_STOP_MS, }: { graceMs?: number; '
                + 'perCallTimeoutMs?: number; },): Promise<void> {',
              '  AbortSignal.timeout(5_000,);',
              '  AbortSignal.timeout(HANG_STOP_MS,);',
              '  AbortSignal.timeout(HANG_STOP_MS * 8,);',
              '  setTimeout(purr, 20,);',
              '  setInterval(purr,);',
              '  await nap(30,);',
              '  await snooze(TINY_MS,);',
              '  await snooze(HANG_STOP_MS,);',
              '  void { perCallTimeoutMs: 1_000, exchangeTimeoutMs: (HANG_STOP_MS as number), graceMs, };',
              '  void { timeout: 30_000, elapsedMs: 5, baseMs: 1, \'hardCapMs\': 90_000, };',
              '  void { grace: { writerMs: TINY_MS, roundMs: 90_000, source: \'writer-dial\', }, };',
              '  void { grace: { writerMs: HANG_STOP_MS, roundMs: HANG_STOP_MS, source: \'round-window\', }, };',
              '  const { idleMs, } = { idleMs: HANG_STOP_MS, };',
              '  void idleMs;',
              '}',
              'export const steady = performance.now() + globalThis.setTimeout(purr, 9,) + timers.setTimeout(purr, 9,);',
            ].join('\n',),
            isTest: true,
          },
          {
            path: 'corpus-run/kitten.test-fixture.ts',
            text: [
              'import { HANG_STOP_MS as LONG_NAP_MS, } from \'../hang-stop.test-fixture.ts\';',
              'import { HANG_STOP_MS, } from \'./hang-stop.test-fixture.ts\';',
              'export const nap = { softBudgetMs: LONG_NAP_MS, hardCapMs: HANG_STOP_MS, };',
              'export function wait(ms: number,): number { return ms; }',
              'export const sleepy = wait(40,);',
            ].join('\n',),
            isTest: true,
          },
          {
            path: 'litter.ts',
            text: 'export const signal = AbortSignal.timeout(5_000,);',
            isTest: false,
          },
        ];
        expect(unsharedBounds({ files, },),).toEqual([
          'corpus-run/kitten.test-fixture.ts#<module>: hardCapMs: HANG_STOP_MS',
          'litter.unit.test.ts#purr: AbortSignal.timeout 5_000',
          'litter.unit.test.ts#purr: AbortSignal.timeout HANG_STOP_MS * 8',
          'litter.unit.test.ts#purr: graceMs = 50',
          'litter.unit.test.ts#purr: graceMs: graceMs',
          'litter.unit.test.ts#purr: hardCapMs: 90_000',
          'litter.unit.test.ts#purr: nap of node:timers/promises 30',
          'litter.unit.test.ts#purr: perCallTimeoutMs: 1_000',
          'litter.unit.test.ts#purr: roundMs: 90_000',
          'litter.unit.test.ts#purr: setInterval (none)',
          'litter.unit.test.ts#purr: setTimeout 20',
          'litter.unit.test.ts#purr: snooze of module-async-time TINY_MS',
          'litter.unit.test.ts#purr: timeout: 30_000',
          'litter.unit.test.ts#purr: writerMs: TINY_MS',
        ],);
        expect(againstListed({
          found: [
            'litter.unit.test.ts#purr: setTimeout 20',
            'tabby.unit.test.ts#<module>: graceMs: 0',
          ],
          listed: {
            'litter.unit.test.ts#purr: setTimeout 20': {
              count: 1,
              bound: 'other',
              reason: 'a purr the cat waits out',
            },
          },
          deferred: {
            'tabby.unit.test.ts': 'another change edits this file first',
            'calico.unit.test.ts': 'another change edits this file first',
          },
        },),).toEqual({
          findings: [
            'calico.unit.test.ts: deferred, and holds no bound outside the hang stop',
            'litter.unit.test.ts#purr: setTimeout 20',
          ],
          expected: ['litter.unit.test.ts#purr: setTimeout 20',],
        },);
      },
    },),
    it({
      name: 'FINDS A MODEL CARD\'S STREAM BOUND written in a test, which the Bedrock client arms around every attempt, '
        + 'and leaves the card\'s bound at the imported hang stop and a card that writes none',
      fn: async () => {
        /**
         A test card with a bound of its own, one at the hang stop, and one
         with no bound at all.
         */
        const files: readonly SourceText[] = [
          {
            path: 'tabby-cards.unit.test.ts',
            text: [
              'import { HANG_STOP_MS, } from \'./hang-stop.test-fixture.ts\';',
              'export const napper = { id: \'tabby.nap-1\', route: \'openai-v1\', streamBoundMs: 60_000, };',
              'export const sleeper = { id: \'tabby.sleep-1\', route: \'v1\', streamBoundMs: HANG_STOP_MS, };',
            ].join('\n',),
            isTest: true,
          },
          {
            path: 'calico-cards.unit.test.ts',
            text: 'export const purrer = { id: \'calico.purr-1\', route: \'v1\', streamEnd: \'usage-chunk\', };',
            isTest: true,
          },
        ];
        expect(unsharedBounds({ files, },),).toEqual(['tabby-cards.unit.test.ts#<module>: streamBoundMs: 60_000',],);
      },
    },),
    it({
      name: 'FINDS A BEDROCK CLIENT A TEST BUILDS WITH NO STREAM BOUND OVERRIDE under any local name, which arms each '
        + 'model card\'s own bound on the real clock, and an override of its own, and leaves an override at the '
        + 'hang stop, an override or a card at unbounded, and a function of the same name the test declares itself',
      fn: async () => {
        /**
         Clients built from the package, and one look-alike built locally.
         */
        const files: readonly SourceText[] = [
          {
            path: 'tabby-clients.unit.test.ts',
            text: [
              'import { createBedrockClient as bedrockFor, createBedrockClient, } from \'../dist/final/node/index.mjs\';',
              'import { HANG_STOP_MS, } from \'./hang-stop.test-fixture.ts\';',
              'export function napClients(transport: unknown,): unknown {',
              '  createBedrockClient({ apiKey: \'tabby\', transport, },);',
              '  bedrockFor({ apiKey: \'tabby\', transport, },);',
              '  createBedrockClient({ apiKey: \'tabby\', transport, streamBoundMsOverride: HANG_STOP_MS, },);',
              '  createBedrockClient({ apiKey: \'tabby\', transport, streamBoundMsOverride: \'unbounded\', },);',
              '  createBedrockClient({ apiKey: \'tabby\', transport, streamBoundMsOverride: 300, },);',
              '  return { id: \'tabby.nap-1\', streamBoundMs: \'unbounded\', };',
              '}',
            ].join('\n',),
            isTest: true,
          },
          {
            path: 'calico-clients.unit.test.ts',
            text: [
              'function createBedrockClient(options: unknown,): unknown { return options; }',
              'export const purrer = createBedrockClient({ apiKey: \'calico\', },);',
            ].join('\n',),
            isTest: true,
          },
        ];
        expect(unsharedBounds({ files, },),).toEqual([
          'tabby-clients.unit.test.ts#napClients: bedrockFor of the built package (none)',
          'tabby-clients.unit.test.ts#napClients: createBedrockClient of the built package (none)',
          'tabby-clients.unit.test.ts#napClients: streamBoundMsOverride: 300',
        ],);
      },
    },),
    it({
      name: 'FINDS A HOLD OF A SECOND OR MORE a case registers for a provider, written onto a rig\'s holds, in a '
        + 'record a holds reader returns or in a constant named for a hold, and leaves a short hold, a hold of 0, '
        + 'a provider-keyed number that is no hold, and package source (a pass-reseat case once waited out 300000 ms '
        + 'for real, because the seat reading sleeps the hold it is told of)',
      fn: async () => {
        /**
         A rig whose holds are written, returned and named, beside numbers that are no hold.
         */
        const files: readonly SourceText[] = [
          {
            path: 'litter.unit.test.ts',
            text: [
              'const SHORT_MS = 40;',
              'const LONG_MS = 300_000;',
              'const HYPER_HELD = { synthetic: 0, hyper: 60_000, };',
              'const budgets = { synthetic: 5_000, hyper: 10, };',
              'export function rigFor({ held, }: { held: boolean; },): unknown {',
              '  const holds = { synthetic: 0, hyper: 0, };',
              '  holds.synthetic = LONG_MS;',
              '  rig.holds.hyper = 2_000;',
              '  rig.holds[\'bedrock\'] = SHORT_MS;',
              '  rig.holds.openrouter = held;',
              '  rig.holds.xiaomi = 0;',
              '  rig.budgets.synthetic = 90_000;',
              '  return {',
              '    providerHolds: () => ({ neuralwatt: 0, synthetic: held ? LONG_MS : 0, hyper: SHORT_MS, }),',
              '    providerBudgets: () => ({ synthetic: 90_000, }),',
              '  };',
              '}',
            ].join('\n',),
            isTest: true,
          },
          {
            path: 'litter.ts',
            text: 'export const holds = { synthetic: 300_000, };',
            isTest: false,
          },
        ];
        expect(unsharedBounds({ files, },),).toEqual([
          'litter.unit.test.ts#<module>: hold hyper: 60_000',
          'litter.unit.test.ts#rigFor: hold synthetic: held ? LONG_MS : 0',
          'litter.unit.test.ts#rigFor: holds.hyper = 2_000',
          'litter.unit.test.ts#rigFor: holds.openrouter = held',
          'litter.unit.test.ts#rigFor: holds.synthetic = LONG_MS',
        ],);
      },
    },),
    it({
      name: 'SETS NO BOUND ON THE REAL CLOCK in the package\'s tests but the shared hang stop, the listed bounds as '
        + 'often as listed, and the deferred files\' own',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        /**
         The scan's findings and what they must equal.
         */
        const {
          findings,
          expected,
        } = againstListed({
          found: unsharedBounds({ files, },),
          listed: LISTED,
          deferred: DEFERRED,
        },);
        expectFindingsAsListed({
          findings,
          listed: expected,
        },);
      },
    },),
  ],
},);
