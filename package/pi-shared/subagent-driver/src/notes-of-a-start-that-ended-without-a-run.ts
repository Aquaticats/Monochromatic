/**
 The note a start that ended without a run leaves, composed from the answers Pi gave and the bound that ran out.

 Every builder here answers with the same shape, so the summary, the run folder's line and the launcher carry one fact: the waits
 that ended the start in the order they ended, each with its bound and how long it had waited, why no run happened, and what to do
 next. Issue 6556 is why they exist: three starts ended as a bare `prompt-rejected` that named no ask and no bound, and four ended
 before they could write a summary at all.

 @module
 */

import type {
  StartThatEndedWithoutARun,
  WaitThatEndedAStart,
} from './a-start-that-ended-without-a-run.ts';
import { START_ATTEMPTS, } from './driver-constants.ts';
import type {
  CommandAnswer,
  NoFurtherStart,
  PiExit,
} from './driver-types.ts';

/**
 Pi's answer to `get_state`, the first of the three answers a run waits for while Pi starts.
 */
const STATE_ASK = 'Pi\'s answer to `get_state`, the first of the three answers a run waits for while Pi starts';

/**
 Pi's answer to `get_commands`, the second of the three answers a run waits for while Pi starts.
 */
const COMMANDS_ASK = 'Pi\'s answer to `get_commands`, the second of the three answers a run waits for while Pi starts';

/**
 Pi's answer to the run's first prompt, the third of the three answers a run waits for while Pi starts.
 */
const PROMPT_ASK = 'Pi\'s answer to the run\'s first prompt, the third of the three answers a run waits for while Pi starts';

/**
 What Pi answered while the run started: the three answers' outcomes, and how long each wait had been waiting.
 */
export type AnswersWhilePiStarts = {
  /**
   How waiting for Pi's state ended.
   */
  readonly state: CommandAnswer;

  /**
   How waiting for Pi's command list ended.
   */
  readonly answer: CommandAnswer;

  /**
   Milliseconds the state ask had been waiting.
   */
  readonly stateMilliseconds: number;

  /**
   Milliseconds the command-list ask had been waiting.
   */
  readonly commandsMilliseconds: number;
};

/**
 Says how Pi ended, in the words a line about a start that ended without a run needs.

 @param exit - how Pi ended

 @returns the ending, in words a reader needs no log for

 @example
 ```ts
 const words = exitWordsOf({ code: 1, },);
 ```
 */
export function exitWordsOf(exit: PiExit,): string {
  if (exit.code !== undefined) {
    return `Pi exited with status ${String(exit.code,)}`;
  }
  if (exit.signal !== undefined) {
    return `Pi left on ${exit.signal}`;
  }
  return 'Pi left';
}

/**
 Says one wait of a start that ended without a run: the ask, what ended it, and how long it had waited.

 @param ask - the ask, in words a reader can follow

 @param answer - how waiting for that ask ended

 @param boundMilliseconds - the silence the ask is given, which a wait Pi went or the run's deadline ended did not reach

 @param waitedMilliseconds - how long the wait had been waiting when it ended

 @returns the wait, as the line and the summary report it

 @example
 ```ts
 const wait = waitThatEnded({ ask: 'the prompt', answer: { ending: 'silence', problem: 'no answer in 20 ms', }, boundMilliseconds: 20, waitedMilliseconds: 21, },);
 ```
 */
export function waitThatEnded({
  ask,
  answer,
  boundMilliseconds,
  waitedMilliseconds,
}: {
  readonly ask: string;
  readonly answer: CommandAnswer;
  readonly boundMilliseconds: number;
  readonly waitedMilliseconds: number;
},): WaitThatEndedAStart {
  if (answer.ending === 'pi-exited') {
    return {
      wait: `${ask}, which ended when Pi went`,
      waitedMilliseconds,
    };
  }
  if (answer.ending === 'deadline') {
    return {
      wait: ask,
      bound: `the run's own wall clock, which ended it before the ${String(
        boundMilliseconds,
      )} ms this wait is given`,
      waitedMilliseconds,
    };
  }
  return {
    wait: ask,
    bound: `${String(boundMilliseconds,)} ms`,
    waitedMilliseconds,
  };
}

/**
 Says what a start that never got Pi's command list has to say, which is why no prompt was sent.

 @param starting - what Pi answered while the run started

 @param startUpAnswerMilliseconds - the bound each of the three answers is given

 @param handedOver - how many values of this machine's secret environment the driver handed Pi

 @param exit - how Pi ended, which a start that ended because Pi went reports

 @returns the note for the summary, the run folder's line and the launcher

 @example
 ```ts
 const note = commandsNoteOf({ starting, startUpAnswerMilliseconds: 120_000, handedOver: 3, exit: {}, },);
 ```
 */
export function commandsNoteOf({
  starting,
  startUpAnswerMilliseconds,
  handedOver,
  exit,
}: {
  readonly starting: AnswersWhilePiStarts;
  readonly startUpAnswerMilliseconds: number;
  readonly handedOver: number;
  readonly exit: PiExit;
},): StartThatEndedWithoutARun {
  /**
   What Pi answered the state ask with.
   */
  const { state, } = starting;
  /**
   Whether that answer came, which the note names only when it did not.
   */
  const stateAnswered = state.ending === 'answered';
  /**
   Every ask that went unanswered before the start ended, in the order they were made.
   */
  const waits: WaitThatEndedAStart[] = [
    ...(stateAnswered
      ? []
      : [waitThatEnded({
        ask: STATE_ASK,
        answer: state,
        boundMilliseconds: startUpAnswerMilliseconds,
        waitedMilliseconds: starting.stateMilliseconds,
      },)]),
    waitThatEnded({
      ask: COMMANDS_ASK,
      answer: starting.answer,
      boundMilliseconds: startUpAnswerMilliseconds,
      waitedMilliseconds: starting.commandsMilliseconds,
    },),
  ];
  /**
   What ended the last of them, which says what to do about it.
   */
  const last = starting.answer;
  if (last.ending === 'pi-exited') {
    return {
      waits,
      what: `${exitWordsOf(exit,)} before it answered its command list, so no prompt was sent`,
      next: 'Read what Pi printed to its standard error, which is where an extension that fails to load, a credential store it cannot read, and a module it cannot find all say so, and start the run again: this is the failure a start is made again after. The run\'s stderr.log holds those lines, and its feed.jsonl holds each start the driver made.',
    };
  }
  return {
    waits,
    what: `Pi never answered its command list, so the driver could not tell whether the extension that masks this machine's secret values was loaded, and it sent no prompt (${String(
      handedOver,
    )} values had been handed to Pi to mask)`,
    next: `Start the run again when the host is quieter: Pi is loading its session and its extensions while these three answers wait, and a host under load can spend longer than the bound on them. If the same wait runs out on a quiet host, run \`pi --mode rpc\` by hand in the run's worktree with the provider and model the run names, and watch its standard error: an extension that fails to load, a credential store it cannot read, and a session larger than its model's window each show there. The run's feed.jsonl holds the lines this start wrote, and its stderr.log holds what Pi printed.`,
  };
}

/**
 Says what a start whose Pi answered a command list without the masking in place has to say.

 @param starting - what Pi answered while the run started

 @param handedOver - how many values the driver handed Pi, which no command reported arming

 @returns the note for the summary, the run folder's line and the launcher

 @example
 ```ts
 const note = maskingNoteOf({ starting, handedOver: 3, },);
 ```
 */
export function maskingNoteOf({
  starting,
  handedOver,
}: {
  readonly starting: AnswersWhilePiStarts;
  readonly handedOver: number;
},): StartThatEndedWithoutARun {
  return {
    waits: [{
      wait: 'the command list Pi answers `get_commands` with, which says whether the extension that masks this machine\'s secret values is loaded',
      waitedMilliseconds: starting.commandsMilliseconds,
    },],
    what: `Pi answered its command list and no command in it armed the masking with the ${String(
      handedOver,
    )} values the driver handed it, so the driver sent no prompt`,
    next: 'Read the run\'s stderr.log: an extension that did not load, a run given `--no-extensions`, and the masking extension\'s entry missing beside the driver each show there. Start the run again once the extension loads, or pass `--pi-argument` for the extension folder that holds it.',
  };
}

/**
 Says what a start ended by its prompt ask has to say, which is the third of the three answers or the bound that replaced it.

 @param answer - how waiting for the prompt ended

 @param waitedMilliseconds - how long that wait had been waiting

 @param startUpAnswerMilliseconds - the bound the prompt is given

 @param exit - how Pi ended, which a start that ended because Pi went reports

 @returns the note for the summary, the run folder's line and the launcher

 @example
 ```ts
 const note = promptNoteOf({ answer: { ending: 'silence', problem: 'no answer in 20 ms', }, waitedMilliseconds: 21, startUpAnswerMilliseconds: 20, exit: {}, },);
 ```
 */
export function promptNoteOf({
  answer,
  waitedMilliseconds,
  startUpAnswerMilliseconds,
  exit,
}: {
  readonly answer: CommandAnswer;
  readonly waitedMilliseconds: number;
  readonly startUpAnswerMilliseconds: number;
  readonly exit: PiExit;
},): StartThatEndedWithoutARun {
  /**
   The prompt's own wait, as the line reports it.
   */
  const waited = waitThatEnded({
    ask: PROMPT_ASK,
    answer,
    boundMilliseconds: startUpAnswerMilliseconds,
    waitedMilliseconds,
  },);
  if (answer.ending === 'pi-exited') {
    return {
      waits: [waited,],
      what: `${exitWordsOf(exit,)} before it took the prompt, so the run never began`,
      next: 'Read what Pi printed to its standard error, which is where a load failure, a refused provider and a missing module all say why: the run\'s stderr.log holds those lines, and its feed.jsonl holds each start the driver made.',
    };
  }
  if (answer.ending === 'deadline') {
    return {
      waits: [waited,],
      what: 'the run\'s own wall clock ended the wait for its first prompt, so the run never began',
      next: 'Give the run a longer `--wall-seconds`, which bounds Pi\'s start-up as well as the run, or start it when the host is quieter: the feed\'s last lines say what Pi was doing when the clock ran out.',
    };
  }
  return {
    waits: [waited,],
    what: 'Pi answered its state and its command list with the masking in place, and then took no prompt, so the run never began',
    next: 'Start the run again when the host is quieter: loading the session Pi was given is part of what this wait covers, and a session larger than its model\'s window makes Pi compact before it takes the prompt. If the same wait runs out on a quiet host, read the run\'s feed.jsonl for a compaction line and its stderr.log for what Pi printed.',
  };
}

/**
 Says which bound refused another start, in words that name what a reader can change.

 @param stoppedBy - the bound the decision named

 @param startMilliseconds - how long the starts had been going

 @param startUpAnswerMilliseconds - the time the whole sequence of starts is given

 @param wallSeconds - the run's own wall clock, in seconds

 @returns the wait that ended the run without one

 @example
 ```ts
 const wait = stoppingWaitOf({ stoppedBy: 'tries-spent', startMilliseconds: 5_000, startUpAnswerMilliseconds: 120_000, wallSeconds: 600, },);
 ```
 */
export function stoppingWaitOf({
  stoppedBy,
  startMilliseconds,
  startUpAnswerMilliseconds,
  wallSeconds,
}: {
  readonly stoppedBy: NoFurtherStart;
  readonly startMilliseconds: number;
  readonly startUpAnswerMilliseconds: number;
  readonly wallSeconds: number;
},): WaitThatEndedAStart {
  if (stoppedBy === 'tries-spent') {
    return {
      wait: 'the number of starts a run is allowed, after which Pi is not started again',
      bound: `${String(START_ATTEMPTS,)} starts`,
      waitedMilliseconds: startMilliseconds,
    };
  }
  if (stoppedBy === 'start-sequence-spent') {
    return {
      wait: 'the time the whole sequence of starts is given, after which Pi is not started again',
      bound: `${String(startUpAnswerMilliseconds,)} ms`,
      waitedMilliseconds: startMilliseconds,
    };
  }
  return {
    wait: 'the run\'s own wall clock, which left no time for another start',
    bound: `${String(wallSeconds,)} s`,
    waitedMilliseconds: startMilliseconds,
  };
}

/**
 Says what a start that ended without a run has to say once a bound refused another start: the start's own reason, and the bound
 that stopped the starts, so a reader learns both without opening anything.

 @param start - what the last start said of itself, when it said anything

 @param stoppedBy - the bound that refused another start

 @param startMilliseconds - how long the starts had been going

 @param startUpAnswerMilliseconds - the time the whole sequence of starts is given

 @param wallSeconds - the run's own wall clock, in seconds

 @returns the note for the summary, the run folder's line and the launcher

 @example
 ```ts
 const note = startThatEndedWithoutARunOf({ stoppedBy: 'tries-spent', startMilliseconds: 5_000, startUpAnswerMilliseconds: 120_000, wallSeconds: 600, },);
 ```
 */
export function startThatEndedWithoutARunOf({
  start,
  stoppedBy,
  startMilliseconds,
  startUpAnswerMilliseconds,
  wallSeconds,
}: {
  readonly start?: StartThatEndedWithoutARun;
  readonly stoppedBy: NoFurtherStart;
  readonly startMilliseconds: number;
  readonly startUpAnswerMilliseconds: number;
  readonly wallSeconds: number;
},): StartThatEndedWithoutARun {
  /**
   The wait the refusing bound is, as the line reports it.
   */
  const stopping = stoppingWaitOf({
    stoppedBy,
    startMilliseconds,
    startUpAnswerMilliseconds,
    wallSeconds,
  },);
  /**
   Why no start followed, in words.
   */
  const whyNoStartFollowed = (stoppedBy === 'tries-spent')
    ? `all ${String(START_ATTEMPTS,)} starts a run is allowed were made`
    : ((stoppedBy === 'start-sequence-spent')
      ? `the ${String(startUpAnswerMilliseconds,)} ms the whole sequence of starts is given was already spent`
      : `the run's own wall clock of ${String(wallSeconds,)} s left no time for another start`);
  if (start === undefined) {
    return {
      waits: [stopping,],
      what: `No start of this run took the prompt, and ${whyNoStartFollowed}`,
      next: 'Read what Pi printed to its standard error, which is where a load failure, a refused provider and a missing module all say why: the run\'s stderr.log holds those lines, and its feed.jsonl holds each start the driver made.',
    };
  }
  return {
    waits: [
      ...start.waits,
      stopping,
    ],
    what: `${start.what}, and no start followed because ${whyNoStartFollowed}`,
    next: start.next,
  };
}
