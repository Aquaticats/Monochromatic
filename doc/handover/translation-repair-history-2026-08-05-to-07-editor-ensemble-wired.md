# Translation repair history 2026-08-05 to 2026-08-07: the editor ensemble wired

The editor ensemble from its start to its wiring,
the ways one model could still decide,
and the shortcuts kept.

Part of [Translation repair history: 2026-08-05 to 2026-08-07](translation-repair-history-2026-08-05-to-07.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The editor ensemble: started, wired, and what one model could still decide

- ENSEMBLE WORK STARTED.
  `candidate-select.ts` is the shared propose-and-select
  component,
  needed because free-text candidates cannot be voted on the way
  claims are:
  two editors fixing one defect phrase it differently,
  so there is
  nothing to match.
  Judges compare ANONYMIZED candidates.
  Two invariants live in
  the component rather than in callers:
  a model never judges a set containing
  its own candidate,
  and a tie or an empty judge roster DECLINES to the caller's
  fallback,
  so the conservative outcome is the default whenever the ensemble
  cannot agree.
  `editor-ensemble.ts` judges at BOTH granularities (user decision):
  per
  envelope so the best fix for one issue can win even when its author botched
  the rest,
  and per chunk because coherence across envelopes is only visible
  whole.
  Per-envelope winners are assembled into a COMPOSITE that must then WIN
  at chunk level rather than being adopted by construction,
  since a composite is
  text no model wrote or read as a whole.
  A chunk-level decline falls back to a REPAIRED patch,
  never to the original:
  discarding fixes the panel already ruled real would turn a wording
  disagreement into a recall loss.
- ENSEMBLE WIRED (task 45 COMPLETE,
  commits `7cce752d4`,
  `1527e4929`,
  `688b96122`).
  `RepairModels.editorModelId` is gone;
  the roster now carries
  `editorModelIds` plus an explicit `judgeModelIds`,
  and `runEditorStage` lives
  in `repair-editor-stage.ts` with the bookkeeping split into
  `editor-candidates.ts`.
  `run-config.ts` runs TWO editors,
  Kimi-K3 and GLM-5.2,
  with the whole roster judging and the checker set reduced to the three models
  that never edit.
  Two editors rather than three deliberately:
  every editor is barred from
  judging its own chunk,
  so each added editor costs a judge as well as its own
  calls.
  At two,
  four judges remain;
  at three,
  only three.
  A composite is text no model wrote as a whole,
  so candidates carry a
  `CandidateProducer` union (`model`,
  or `composite` with contributors) rather
  than one model id,
  and collapsing duplicate candidates UNIONS their stakes.
  Without that union a real self-judging leak existed:
  if the composite carried
  model B's operation while model A's whole-chunk text matched it exactly,
  keeping either candidate alone left the other free to judge text it wrote.
- FOUR WAYS ONE MODEL COULD STILL DECIDE,
  all closed in `1527e4929`,
  three of
  them found by sol and missed by both the advisor and me:
  A plurality of ONE used to win.
  With judges lost or abstaining a single ballot
  named the winner,
  which is one model controlling the stage.
  Winners now need
  `MIN_SELECTION_VOTES` (2),
  and `assertJudgeableEditorRoster` refuses a roster
  that cannot seat that many disinterested judges,
  plus repeated or empty editor
  rosters.
  Nothing stopped an editor from also CHECKING.
  The judge roster is filtered at
  runtime so that overlap is caught,
  but `checkerModelIds` was used as given.
  `assertCheckerIndependence` refuses it at chunk entry.
  Per-envelope judges were asked whether a replacement fits its surroundings in
  register and tense while being shown only the replacement and the Chinese
  source.
  Ballots now carry the passage being replaced and a bounded window of
  the translation around it (`ENVELOPE_CONTEXT_CHARS`).
  The candidate fence was a fixed `=====`,
  which ordinary prose contains:
  a
  setext heading underline lets a candidate close its own block and have the
  rest read as instructions (AGENTS.md SYB/STB).
  The fence is now chosen against
  everything it encloses,
  always longer than any run inside.
- DECLINE IS TWO DIFFERENT VERDICTS (`688b96122`),
  a refinement neither reviewer
  proposed.
  `SelectionDisposition` splits `indecision` (tie,
  or leader short of
  the vote minimum:
  judges failed to RANK,
  and nothing was said against any
  candidate) from `rejection` (every judge answered "none of these",
  or no
  disinterested judge could be seated).
  Chunk selection ships the strongest
  repair on indecision and nothing on rejection.
  Sol argued decline should ALWAYS fall back to unchanged.
  Rejected on evidence:
  `selectRepairCandidate` in `repair-chunk.ts` already makes any repair beat the
  untouched text on checker measurements before it ships,
  so the conservative
  gate exists one stage later and implementing it twice only costs recall.
  Sol's
  specific objection,
  that the composite could ship precisely when it failed to
  win,
  was aimed at a plan where the fallback WAS the composite;
  the built
  fallback is the strongest EDITOR patch and the composite is never it.
- TWO SHORTCUTS KEPT over sol's objection,
  with reasons recorded so a future
  session does not "restore" the expensive behavior:
  a sole chunk candidate
  ships unjudged because after dedupe it means every editor AND the composite
  wrote identical text,
  and `pickFallbackPatch` over identical patches returns
  that same text,
  so the ballot cannot change the output.
  A sole per-envelope
  proposal is adopted unjudged because that operation also sits inside its
  author's whole-chunk candidate,
  which IS judged at chunk level.
