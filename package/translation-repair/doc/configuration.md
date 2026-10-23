# Configuration

Part of [the family README](../README.md).

Every setting the package reads from the environment is listed here,
beside the two command-line flags in `Choosing what a run attempts`;
none of them had been written down before 2026-08-24.
Most values are read once per invocation;
an entry says so where its value is read per entry or per round instead.
Git's repository-routing variables are not settings:
every corpus `git` call drops them rather than reading them
(`package/translation-repair/corpus-source/src/corpus-git-context.ts`).

An exported-but-empty variable counts as unset,
which is deliberate rather than incidental:
an empty export is an ordinary shell accident,
and reading one as an instruction has cost this package a defect before.
The two variables under `Probe tools` are the exception:
each uses an empty export as given.

## Credentials

-   `TRANSLATION_REPAIR_SYNTHETIC_API_KEY`.
    Bearer token for the first provider,
    Synthetic.
    Optional like every other provider key:
    a run starts on whichever provider keys are present,
    and only a run with no provider key at all is refused,
    the stated refusal (exit 6) described under `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY`
    (`configureProviders` in `package/translation-repair/run/src/corpus-run/run-providers.ts`).

-   `TRANSLATION_REPAIR_EXA_API_KEY`.
    Key for the Exa search endpoint,
    which the pass asks once per work the original names in 《…》 marks for its official English title
    (the owner's rule of 2026-09-02,
    `doc/decision/translation-repair-work-titles-established-vocabulary.md`).
    The top results reach every sheet as `web lookup` lines in the identity context,
    beside the `note` and `editor comment` lines carrying both pages' footnotes and editors' comments,
    and the house policy says what each kind licenses.
    Optional:
    unset means one warning per entry naming how many titles went unlooked-up,
    and nothing else changes.
    Every lookup is cached durably under `TRANSLATION_REPAIR_LOOKUP_CACHE_DIR`,
    else `$XDG_CACHE_HOME/translation-repair/lookup`,
    else `~/.cache/translation-repair/lookup`,
    keyed by the query's digest,
    so a title is bought once across runs and a resumed run keeps its preparation identity.
    Measured 2026-09-02:
    about 1.5 s and $0.007 a query;
    the pinned corpus's 118 spans cost about a dollar once.
    The same key buys the pages the original cites through the Exa contents endpoint
    (class thirty-five,
    2026-09-16),
    cached semi-permanently under the `reference` subdirectory of that cache,
    one JSON record per url named by the url's digest,
    a failure cached too so a dead link is not hit on every pass;
    delete a record's file to refresh that page.
    59 of the 92 pinned originals link somewhere,
    116 links in all,
    profiles and the corpus's own pages excluded before any is bought.
    Unset,
    one warning per citing entry says how many references went unread.

-   `TRANSLATION_REPAIR_NEURALWATT_API_KEY`.
    Bearer token for the fifth provider,
    NeuralWatt,
    which is first in `PROVIDER_ORDER` since 2026-10-20,
    the owner having said "Use the neuralwatt free energy front and center."
    Optional and loud like the others.
    The owner's account carries 550 Wh of free energy a day on DeepSeek V4.1 Flash and GLM-5.3-Flash,
    not carried over;
    nothing reports what is left of it and overage is charged to the owner's plan,
    so the package meters it itself from a durable ledger
    (`doc/decision/translation-repair-neuralwatt-provider.md`).
    The owner,
    2026-10-20:
    "The stop logic should obviously count total free energy used per day.
    Every neuralwatt day starts at `15:15`"
    (15:15 EDT for the owner in America/New_York).
    A NeuralWatt day is therefore the 24 hours from the latest instant,
    at or before now,
    whose UTC time of day is that of the quota's `subscription.current_period_start`
    (`GET /v1/quota` answered `2026-10-20T19:15:08Z`,
    which is 15:15 EDT for the owner in America/New_York),
    never midnight UTC and never a local time,
    so daylight saving does not move it.
    What the day has spent is the larger of two figures,
    never their sum:
    the covered energy the whole account used since the day began
    (`GET /v1/usage/by-model`,
    the rows of the served models and their `-flex` lanes,
    read at run start and at each meter reading,
    the reply's echoed period checked against the window asked),
    and the ledger's own settled energy since the day began,
    plus the holds of calls in flight.
    Calls made under a tracking alias such as `nw-flash` count too:
    a usage row for one carries the alias as its `requested_model`,
    so `GET /v1/models` is read at each account reading for the aliases (and their `-flex` twins)
    whose `metadata.huggingface_id` is that of a covered model,
    since aliases move.
    Where that listing cannot be read,
    every `nw-` row that is not a speed lane and was served by a covered model counts,
    with one warning per outage.
    Speed lanes never count.
    Flex energy is counted as consumed rather than as charged,
    which over-counts and so errs on the safe side.
    When the by-model route cannot be read the last figure for the same day stands,
    else the ledger counts alone,
    with one warning per outage naming the endpoint and status.
    The `METERS` line carries `neuralwatt=`,
    and `neuralwattWh=` (left in the day),
    `neuralwattReckonedWh=` (the share resting on estimates),
    `neuralwattAccountWh=` (the account's covered energy,
    or `none`)
    and `neuralwattDayStart=` (the day's first instant,
    ISO UTC,
    or `none`) among its numbers.
    The run logs at start where the day starts and which source said so.
    The owner also caps this key at 1 kWh a day;
    the quota reply carries it as `key.allowance`,
    which still read `null` on 2026-10-20,
    so what a live cap reports is unconfirmed.
    When present it is read at each meter reading:
    `neuralwattKeyAllowance=none` for an uncapped key,
    else `neuralwattKeyRemainingUsd=` and `neuralwattKeyBlocked=yes|no`,
    and a blocked cap holds NeuralWatt dry until its `reset_at` (the next day start where it names none)
    with a warning.
    A cap that will not read is warned about once and ignored.
    The free-energy stop stays the primary one;
    the 550 Wh is under the 1 kWh cap,
    so the cap is a backstop.
    A cost a request reports does not stop the provider;
    it is a nominal figure that free-energy requests carry too,
    and it appears on the `SPEND` line as `cost=`.

-   `TRANSLATION_REPAIR_NEURALWATT_LEDGER`.
    Where that ledger lives;
    `~/.local/state/translation-repair/neuralwatt-energy.jsonl` under the running user's home when unset.
    Append-only JSON lines,
    one per settled call;
    a line that will not read stops the meter with the line's number rather than reading the energy as unspent.

-   `TRANSLATION_REPAIR_NEURALWATT_DAILY_WH`.
    The day's free energy the ledger is read against;
    550 when unset.
    Set it to 0 on a day without a promotion,
    when the account has no free energy and the first request would be billed to the plan;
    anything that is not a non-negative number is refused at once.

-   `TRANSLATION_REPAIR_NEURALWATT_DAY_START`.
    The UTC time of day a NeuralWatt day starts at,
    written `HH:MM` with two digits each,
    such as `19:15`.
    Read only when the quota reply carries no usable `subscription.current_period_start`;
    the quota's own period start wins when both exist.
    When neither gives a time of day the day is unknown:
    NeuralWatt is marked dry and refused until it is learned,
    with one warning naming the missing field and both remedies.
    The quota is then read again,
    no sooner than 60 seconds after each failed attempt,
    so a transient failure at the start of a run does not refuse NeuralWatt for the whole run;
    the day finally learned is logged with its source.
    There is no midnight fallback,
    since an assumed 00:00 UTC would count the wrong day's energy.
    A value that is not `HH:MM` is refused at once.

-   `TRANSLATION_REPAIR_CHARM_HYPER_API_KEY`.
    Bearer token for the second provider,
    Charm Hyper.
    Optional and loud:
    a run starts on whichever provider keys are present,
    an absent provider is marked dry before routing so its seats are unavailable,
    and every key absent is a stated refusal naming the variables (exit 6).
    A calibration once settled clean with half its roster dark because a missing key was silent;
    it is not silent now,
    and the seat lines at the end of every command name what was dark.
    Note the `CHARM` in the middle;
    a name missing it is read by nothing and reported by nothing.
    The owner will not top this balance up again (2026-09-03),
    so it serves until it runs dry.

-   `TRANSLATION_REPAIR_OPENROUTER_API_KEY`.
    Bearer token for the third provider,
    OpenRouter,
    the paid per-token fallback the owner chose on
    2026-09-03 (`doc/decision/translation-repair-openrouter-fallback.md`).
    Optional and loud like the second.
    The bench task `reader-measure` reads this key too,
    and spends it:
    one ask per matched picture over the zero-data-retention route (ledger B723).
    Every request carries `provider: { zdr: true, require_parameters: true, sort: 'price', ignore: [...] }`
    (`OPENROUTER_PROVIDER_PREFERENCES` and `openRouterProviderPreferencesFor` in
    `package/translation-repair/provider/src/openrouter-catalog.ts`),
    so only zero-data-retention endpoints that support `response_format` may serve a passage,
    and among those the cheapest serves (`sort: 'price'` since 2026-09-09,
    the owner's instruction to stop bleeding).
    The card's `ignoredEndpoints` keeps a measured-broken or measured-slow upstream off the wire;
    every card that names an endpoint keeps its OpenRouter side,
    with the measurements behind it,
    in `package/translation-repair/shared-contract/src/openrouter-card-sides.ts`
    (moved there from `model-cards.ts` on 2026-10-22):
    Parasail and ModelRun for `minimax-m3`
    (Parasail since 2026-09-03,
    when it answered into the reasoning channel and left content empty;
    ModelRun since 2026-09-04,
    when it timed out 119 of 300 streams in-stream
    while CoreWeave answered the same schema request 4 of 4),
    and DeepInfra,
    Wafer,
    OpenInference,
    DekaLLM and Sail Research for `deepseek-v4.1-flash`
    (the first two after XingZ607 on 2026-09-18,
    the last three after XingZ624 to XingZ626 on 2026-09-23).
    The card has sent the 1017 id,
    `deepseek/deepseek-v4.1-flash-1017`,
    since 2026-10-20.
    Its endpoint listing of that day carries DeepInfra and Wafer and none of the other three,
    so those three ignores are inert until one lists it.
    A card whose `preferredEndpoints` is not empty also sends `order`,
    which names those endpoints ahead of the price sort and leaves `allow_fallbacks` unset:
    Wafer for `hf:zai-org/GLM-5.3-Flash` and Morph for `deepseek-v4.1-flash`,
    both since 2026-09-23,
    because the price sort had landed those seats on endpoints that reason at length by default;
    Together then CoreWeave for `minimax-m3` since 2026-09-28 (ledger H12),
    the only structured-output endpoints left once the two ignores apply,
    both FP4,
    with Together the faster over the week before.
    Such a card carries its preferred endpoint's listed price,
    not the model's cheapest listing,
    since that is the endpoint the seat buys;
    only the abandoned-spend estimate reads the card's OpenRouter price
    (ledger P13,
    2026-09-28:
    both cards had carried a price no call of theirs paid).
    The ignores once kept for DeepSeek V4 Flash,
    `hf:Qwen/Qwen3.8-27B` and `glm-5.3` left with their OpenRouter rows:
    the last two have been off the OpenRouter catalog since 2026-09-09 (`openrouter-dropped` on their cards),
    and DeepSeek V4 Flash left the roster on 2026-09-16.
    Slugs are the ones `GET /api/v1/providers` lists,
    not display names lower-cased:
    `open-inference`,
    not `openinference`,
    which is how the 2026-09-03 ignore stayed off the wire for a day.
    Credits are read from `GET /api/v1/credits` (purchased less used,
    in USD) and printed on the `METERS` line as `openrouterUsd=`;
    every call's cost is read off the final stream chunk onto its `SPEND` line as `cost=`,
    so a run's OpenRouter bill is summed from the wire rather than from a price table.
    The upstream that served each call is read off the same chunks:
    `endpoint=` on the `SPEND` line (percent-encoded) and `served by "..."` on the stream progress line,
    cut streams included,
    so a slow or broken upstream is named by a grep of the run log.
    An upstream that fails after the gateway has answered 200 writes one chunk carrying an `error` object
    and closes without `[DONE]`;
    the client reads that chunk first and retries under `InStreamProviderError`,
    whose message names the code,
    the gateway's error type and the endpoint (`openrouter-stream-error.ts`;
    ModelRun's 504 timeouts on MiniMax M3,
    2026-09-04,
    were the case).
    A chunk whose code is a 4xx status other than 408 and 429 is `InStreamRefusalError`,
    which the retry ladder returns unretried,
    as it returns the same refusal over plain HTTP (ledger B251).

-   `TRANSLATION_REPAIR_AMAZON_BEDROCK_API_KEY`.
    Bearer token for the fourth provider,
    Amazon Bedrock,
    the owner's 200 USD of prepaid credits of 2026-09-07,
    expiring early next year and never to be topped up,
    to be spent freely until then.
    Optional and loud like the second and third.
    Under the account's zero-data-retention mode the served models are the three Gemma 4 sizes
    and `openai.gpt-oss-120b`,
    on the `bedrock-mantle` host in us-east-1 over raw fetch and chat completions:
    the Gemma sizes under `/openai/v1` ending on `[DONE]`,
    `openai.gpt-oss-120b` under `/v1` ending on its usage chunk
    (`bedrock-catalog.ts` records the probes).
    The provider exposes no balance to a bearer key,
    so every priced call appends one line to a durable ledger and the `METERS` line's `bedrockUsd=`
    is the credit less that file's sum;
    `SPEND` lines carry `cost=` computed from the usage and the catalog's prices.

-   `TRANSLATION_REPAIR_BEDROCK_LEDGER`.
    Where that ledger lives;
    `~/.local/state/translation-repair/bedrock-spend.jsonl` under the running user's home when unset.
    Append-only JSON lines,
    one per priced call;
    a line that will not read stops the meter with the line's number rather than reading the money as unspent.

-   `TRANSLATION_REPAIR_BEDROCK_CREDIT_USD`.
    The credit line the ledger is read against;
    200 when unset,
    the owner's figure.
    Set it when the console reads a different balance;
    anything that is not a non-negative number is refused at once.

-   `DEEPSEEK_API_KEY` and `TRANSLATION_REPAIR_XIAOMI_MIMO_API_KEY`.
    Bearer tokens for the two direct providers,
    granted by the owner on 2026-10-22,
    each for one model:
    DeepSeek V4.1 Flash on the first and MiMo V2.6 Flash on the second.
    Both are optional and never fall back to another key:
    an unset one leaves its provider unconfigured,
    with one info line naming the variable,
    and neither is accepted by `--require-providers`.
    The `METERS` line carries `deepseek=` and `deepseekUsd=`,
    and `xiaomi=` and `xiaomiUsdSpent=`
    (`doc/decision/translation-repair-direct-providers.md`).
    A 401 from either provider reaches the log without the provider's own words,
    since a refusal may echo a fragment of the key that the transport's mask would not find.

Every key lives in the sops-encrypted,
gitignored `.env.local.json` at the repository root,
which `mise run` decrypts into the task's environment.
A worktree created with `git worktree add` starts without that file;
copy the encrypted file into the worktree root (it stays encrypted at rest) or launch from the main worktree.
Either way,
launch under `mise run`:
a bare `node dist/...` launch has no key,
and since 2026-08-25 it fails at once with the refusal instead of running half-dark.

Since 2026-10-06 a built command also refuses the opposite start:
a process that holds provider keys and does not name the command as the one it means to start
(`package/translation-repair/adjudication/src/task-runner-guard.ts`;
exit 6,
before the command line is read).
A key here is any variable whose name ends in `_API_KEY` and whose value is not blank.
The command's task names it by setting `TRANSLATION_REPAIR_STARTED_BY` to the command's name,
so every `mise run //package/translation-repair/<package>:<command>` start is unchanged.
A start that must not go through the task,
because every task builds the package first and nothing may rebuild while a pass is running,
sets the variable by hand for that one start:
`TRANSLATION_REPAIR_STARTED_BY=<command>` before the launch line
(`doc/runbook/translation-repair-corpus-pass.md` holds the two cases).
A process that holds no key runs as before,
with or without the variable.
The guard stops a start nobody meant,
a built file run by hand from a shell that happens to hold keys;
it is not access control.

Every command ends by printing one `SEAT <model> asked=N usable=N unusable=N threw=N abandoned=N unreachable=N` line per seat to stderr,
and a `SEATS DARK:` line naming every seat that was asked and never once produced a usable answer.
`asked` counts calls that settled.
`abandoned` counts calls the caller left before they settled,
chiefly the replies a round cut at its straggler deadline after quorum;
they are in no other column,
so a seat with a large `abandoned` was waited on less often than `asked` says
and is neither withheld nor named dark for them.
`unreachable` counts calls the provider router refused with `NoProviderForModelError` before any request was sent,
because no wet provider serves the seat (or the owner culled it);
since 2026-10-21 they are in no other column either
(audit ledger B711),
where before that they were counted as `asked` and `threw`.
A seat only the router refused prints `asked=0` and its `unreachable` count,
is never withheld by the tally and is not named dark,
since it was never asked;
the `unserved=` field of the `JUDGE SEATS` lines names it while it sits on a bench.
The `SEATS DARK:` line states its count over the seats asked.
A dark seat is a provider that errors on it
or a model that answers nothing readable;
the run log names which.
A seat no provider could take at all,
because its provider has no key or no wet provider serves it,
is refused by the router before any request,
counts `unreachable` and is not dark.
Do not read a run with a dark seat as a comparison of the roster.

The first time a provider rejects its credential with HTTP 401,
the run writes one `SEAT CREDENTIAL REFUSED <model> on <provider>:` warn line for that provider,
naming the seat whose call met it,
the provider that refused it
and the provider's own words.
A credential belongs to the provider and serves every model there,
and it does not renew during a run,
so the router holds that provider out of every model for the rest of the run
(`package/translation-repair/provider/src/credential-hold.ts`).
The call that met the refusal goes on to the next provider serving the model in the same call,
and every other model skips the provider without asking it.
The line names as where the seat moves only the providers that serve it
and can take a call now:
not held out for a refused credential,
not dry,
and not held on the model's stream bound.
Where providers serve it but none can take a call now,
the line names each as dry or held for the model
and says the seat is not withheld on this refusal,
since the router's rules for a dry or held provider decide what follows.
Only where no provider serves it any longer does the line say the seat is withheld.
The cross-provider re-ask holds a provider the same way a first ask does.
OpenRouter's decisions endpoint shares OpenRouter's credential,
so a 401 on either ends the other.
The line offers both ways forward:
renew that provider's credential before the next run,
or leave it.
No meter shows the hold,
so the view the seat readers and benches are derived from reads a provider whose credential was refused as dry
(`foldCredentialHolds` in `package/translation-repair/provider/src/credential-hold.ts`,
applied by `providerDryness` in `package/translation-repair/run/src/corpus-run/run-config.ts`),
and a seat served only by such providers reads as unreachable before it is asked.
Before 2026-10-20 the 401 reached the seat tally as any other failed ask,
and the run withheld a seat after its failed asks even where another provider served it
(zheermao19 withheld `gemma-4-26b-a4b-it` with OpenRouter wet).

Each `JUDGE SEATS` line names how every provider reads as `<provider>=<state>`:
`dry` when the router holds the provider out
or its credential was refused,
`wet` when its meter answered and the provider is not held out,
and `unreadable-treated-as-spendable` when its meter could not be read,
which routes as spendable by recorded policy (`package/translation-repair/provider/src/provider-budget.ts`).
`wet` says the meter reports money left;
it says nothing of whether a credential is accepted.

The line goes on to name the benches' sizes and two lists of seats.
`withheld=` names the models the slowness and cost rules took from a bench.
`unserved=<models or none>` names,
since 2026-10-21,
every model on a bench of the reading that no wet provider serves:
it stays seated and is lost at call time as unreachable,
so a bench count includes it
(`doc/decision/translation-repair-short-bench-share.md`).
`threw-withheld=` names the seats the run withheld for failing every real ask,
and never a seat the router only refused.
The repair lane's `chunk re-seated under a hold: checkers=<models>` line ends with `unserved-checkers=<models or none>`,
naming the checkers that are padding the bench with no wet provider behind them.

## Where a run writes

-   `TRANSLATION_REPAIR_RUNS_DIR`.
    Root for artifacts,
    the published tree,
    slice caches,
    and the attempt map.
    Defaults to `node_modules/.monochromatic/translation-repair-runs` under the worktree root.
    Point it at a throwaway directory for any run whose output should not join a pool later.

## Bounding one run

-   `TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD`.
    Overrides the per-run spend ceiling,
    a non-negative number of USD,
    built in at 20 (`doc/decision/translation-repair-run-spend-ceiling.md`,
    the owner's decision of 2026-09-04).
    Every call's reported or priced cost on those providers feeds a process-wide meter as its `SPEND` line is written,
    and before each entry the scheduler stops starting new ones once the run's spend on any one of OpenRouter,
    DeepSeek and Xiaomi is at or past the ceiling (each against the ceiling alone,
    never summed),
    printing `SPEND CEILING reached`;
    entries already running finish.
    Unset and blank are the built-in;
    an unreadable or negative value is refused at launch for the same reason the entry ceiling's is;
    zero is allowed and means start nothing,
    which is how the guard is shown to fire on a live run at no cost.
    A run that overrides logs `SPEND CEILING OVERRIDDEN`.
-   `TRANSLATION_REPAIR_HARD_CAP_MINUTES`.
    Overrides the per-entry ceiling,
    a positive number of minutes.
    A value that is not one is refused rather than replaced by the default,
    including `30m`,
    which `parseFloat` would have read as 30:
    a ceiling is what stops a runaway entry,
    so an operator who set it must not be left believing a run is bounded some way it is not.
    A run that overrides logs `CAP OVERRIDDEN` above its work.

    There is a floor,
    and it is one model exchange.
    A ceiling at or below `RUN_PER_CALL_TIMEOUT_MS`,
    currently 360_000,
    cuts every attempt before any exchange can return,
    so nothing caches,
    the attempt reports no progress,
    and the queue drops the entry as stalled after its first try
    (`readAttemptOutcome` in `package/translation-repair/slice-cache/src/corpus-run/entry-reattempt.ts`).
    A run in that state logs `CAP TOO TIGHT` naming both numbers.
    It is warned rather than refused,
    because cutting mid-exchange is exactly what a test of the stall path wants.

The cap ends an **attempt** rather than an entry.
An entry the cap cut goes to the back of the queue and is attempted again inside the same invocation,
against the same frozen pipeline digest,
so an entry too large for one attempt no longer needs a relaunch per attempt.

A re-attempt is **earned** rather than automatic.
The pass counts the entry's cache records before and after each attempt,
and re-queues only when that count grew.
An attempt that bought nothing logs `STALLED` and the entry is dropped for this invocation,
because no progress guarantee holds:
an abort can land before the first persistence,
and the slices a lane deliberately leaves uncached produce no record however long they took.
Without the earned rule a stuck entry would spend the whole soft budget.

A re-attempt logs `REATTEMPT <id> queued`,
naming what the attempt bought.

## Pacing a run

These move how much a run keeps in flight and how long a round waits,
each for one launch and without a rebuild.
The decisions behind the built-in values,
and how the editor calibration differs from the pass,
are in [Deciding who fills a seat](seats-and-calibration.md),
section `Historical writer and editor runners`.

-   `TRANSLATION_REPAIR_SLICE_OVERLAP`.
    How many slices a driver keeps in flight at once.
    The corpus pass defaults to 4 (`PASS_OVERLAP` in
    `package/translation-repair/corpus-pass/src/corpus-run/pass-overlap.ts`,
    `doc/decision/translation-repair-pass-overlap.md`,
    2026-09-06),
    and so does the editor calibration (`CALIBRATION_OVERLAP` in
    `package/translation-repair/slice-cache/src/corpus-run/slice-overlap.ts`);
    `1` reproduces the sequential driver.
    The pass reads it once per entry and prints `OVERLAP <id> value=N source=...`,
    where the source is `fallback` or the variable's name.
    Anything but a canonical decimal whole number of at least 1 is refused.

-   `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS`.
    How long a round keeps waiting on stragglers once quorum stands,
    in milliseconds.
    Built in at 120000 (`STRAGGLER_GRACE_MS` in `package/translation-repair/stage/src/stage-round.ts`);
    the editor calibration adopts 300000 when the variable is unset
    (`CALIBRATION_STRAGGLER_GRACE_MS` in `package/translation-repair/glossary/src/grace-override.ts`).
    Every round reads it when it gathers,
    and a pass whose window differs from the built-in prints `STRAGGLER GRACE OVERRIDDEN` at launch.
    Anything but a whole number of milliseconds from 1 to 2147483647,
    the longest delay a timer holds,
    is refused.

-   `TRANSLATION_REPAIR_WRITER_GRACE_MS`.
    The same window for the writer rounds alone
    (editor,
    refiner,
    translate and consolidation producers,
    and the archive block review round),
    since a cut writer voice is a whole candidate lost.
    The archive block review joined on 2026-10-21 (ledger B713):
    each reviewer answers with the complete corrected block,
    so its reply is a writer voice.
    The selection round after it keeps the round window,
    and the classification is open to the owner's veto.
    Built in at 180000 (`WRITER_GRACE_MS` in `package/translation-repair/glossary/src/writer-grace-override.ts`,
    the owner's decision of 2026-09-06 in `doc/decision/translation-repair-straggler-grace.md`);
    when it is unset and the round window is longer,
    the writers follow the round window instead.
    Every writer round reads it when it gathers,
    and a launch prints `WRITER GRACE built in` or `WRITER GRACE OVERRIDDEN`
    unless the writers follow the round window;
    the line names every writer round (`WRITER_STAGE_LABELS`).
    It is refused by the same rule as the round window.

-   `TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR`.
    How many Hyper requests may start in any rolling hour,
    retries and credit reads included;
    the rest queue in arrival order instead of being refused with HTTP 429.
    Defaults to 1,000,
    the account's limit as the owner stated it (`HYPER_REQUESTS_PER_HOUR` in
    `package/translation-repair/provider/src/request-pace.ts`).
    It is read once,
    when the Hyper client is built.
    A value that is not a whole number of one or more,
    written in digits,
    is refused there as every other dial is,
    naming the variable and the value as written.

## Choosing what a run attempts

These two are command-line flags rather than variables,
passed after `--`:

-   `--only Id1,Id2`.
    Restricts the invocation to the named entries,
    chosen by hand in place of every pending pair at the pin;
    those still pending run in the pass's own order,
    cached progress first and then the size bands by rank.
    Run it into a throwaway `TRANSLATION_REPAIR_RUNS_DIR`,
    so a hand-picked entry never joins a pool that later draws treat as natural accumulation.
    A flag with no value,
    or one whose value parses to no id at all,
    is refused:
    a flag that parsed to nothing would run the **whole** corpus,
    which is the opposite of what was asked and expensive to discover afterwards.
    A restricted run logs `ONLY` and the ids it took.

-   `--plan`.
    Reads the corpus,
    builds the pending list,
    constructs the client,
    prints `PLAN ok` with the tip,
    the pipeline digest and the first few pending ids,
    and returns without calling a model.
    Use it to check a run's setup,
    selection and credentials for no quota.
    Measured at 1.88 seconds with no stream opened.

## Probe tools

Two probe tasks read a variable of their own,
without the package prefix:

-   `DAMAGE_SAMPLE_SEED`.
    Seed for the `damage-sample` draw of shipped regions,
    `damage-round-one` when unset (`package/translation-repair/probe/src/corpus-run/damage-sample.ts`);
    set a new one to draw a fresh sample for a later round.

-   `VERIFY_SHEET_BASENAME`.
    Which graded sheet `score-verify` joins to its manifest,
    read as `<basename>-sheet.md` under the runs directory;
    `probe-verify` when unset (`package/translation-repair/probe/src/corpus-run/score-verify.ts`).

## A source change while a pass is in flight means kill and relaunch

Every pass and probe task declares `depends = ["build"]`,
so invoking one rewrites `dist/final/node` underneath any pass already running from the same worktree.
The running process survives that
(the bundle has no dynamic imports,
verified rather than assumed:
[`translation-repair-overlap-dial.md`](../../../doc/handover/translation-repair-overlap-dial.md),
"Do not land a driver into a live pass launch"),
but the pass is now on a superseded build:
it computed its pipeline digest once at startup and stamps that digest into every artifact it writes,
so its artifacts name files that are no longer on disk,
and its page cannot say whether the change that prompted the rebuild worked.

The rule is **always kill and relaunch**,
the owner's on 2026-09-06.
When source changes while a pass is running,
kill the pass by pid,
build,
and relaunch the same entry into a fresh `TRANSLATION_REPAIR_RUNS_DIR` on the new build.
Never let a pass finish on a build a commit has superseded,
and never read its page as readiness evidence.
The calls the killed run spent are the price of a fix landing while it ran;
a relaunch into the old runs dir would republish what the old digest bought,
which is why the runs dir is fresh.
When the fix is already known before the launch,
fix first and launch once:
a launch made ahead of a change that will be built within the hour buys nothing.

A rebuild with no source change is byte-identical and harmless,
which is exactly why this is easy to get away with and worth stating anyway:
the digest is the only thing that reveals it,
and it reveals it after the fact.
Same immutable build may back concurrent passes when no rebuild follows.
Source-distinct pass requires separate throwaway worktree built before that process launches.

Concurrent passes require separate run roots,
logs,
and publication roots.
They still share provider capacity,
so elapsed times are operational results rather than matched performance comparison.
Record each pipeline digest and corpus commit separately.

## Which corpus a run reads

Every corpus read resolves through one pin,
a clone directory and a commit (`readRunCorpusPinSetting` in
`package/translation-repair/run/src/corpus-run/run-corpus-pin.ts`,
moved out of `run-config.ts` on 2026-10-22).
Either half can be overridden without editing source,
which is what a fixture run against an unmerged corpus pull request needs.
Both are read once per command,
at the start of its run and inside its refusal boundary,
by `readRunCorpusPinSetting`,
which reads the environment it is handed through `readCorpusPinSetting` in
`package/translation-repair/run/src/corpus-run/corpus-pin-override.ts`,
and the command hands the pin on as a value.
Until 2026-10-10 both were read when `package/translation-repair/run/src/corpus-run/run-config.ts` loaded.
Unset and blank both mean the built-in half;
anything else that is not valid is refused as a stated refusal rather than replaced by the built-in,
because a run against the wrong corpus would record its conclusions as the pinned corpus's.

-   `TRANSLATION_REPAIR_CORPUS_CLONE_DIR`.
    Absolute path of the corpus Git clone every read runs `git` in;
    the clone must hold the pinned commit.
    Defaults to `one-among-us/data` under the running user's home.
    A relative path is refused.
    Git searches for a repository no higher than the clone,
    so the directory must be the top of the clone itself.
    A clone whose parent directory's real path holds a colon is refused at its first read,
    since git's list of directories it stops at is split at colons:
    move the clone,
    since a link to it keeps the colon in the real path.

-   `TRANSLATION_REPAIR_CORPUS_COMMIT`.
    Commit every read resolves against,
    written as one full 40-character lowercase hexadecimal name.
    Defaults to `CORPUS_COMMIT_SHA` in `package/translation-repair/corpus-source/src/corpus-source.ts`,
    the milestone-one benchmark pin.
    An abbreviated name is refused,
    since clones can resolve an abbreviation differently.

A settled artifact records the commit it read as `corpusSha`
(`package/translation-repair/corpus-pass/src/corpus-run/pass-entry-artifact.ts`),
so an overridden run's artifacts name the commit they came from.
When either half comes from the environment,
`corpus-pass` prints a `CORPUS PIN OVERRIDDEN` line at launch naming the clone,
the commit and where each came from
(`corpusPinOverrideNote`,
ledger D13);
record both values with the run's provenance all the same.

Production `corpus-pass` has no pull-request flag;
these two variables replace the uncommitted source fork the 2026-08-29 pull-request runs needed.
For a corpus pull request,
point `TRANSLATION_REPAIR_CORPUS_CLONE_DIR` at an isolated corpus clone or minimal Git fixture
holding the exact head commit,
set `TRANSLATION_REPAIR_CORPUS_COMMIT` to that commit,
write into a throwaway `TRANSLATION_REPAIR_RUNS_DIR`,
run `--plan --only <entry>` first,
and retain provenance mapping the fixture bytes to the pull-request head.
Supply credentials through the trusted worktree environment;
copying secret values into a command,
a log,
or a provenance file is forbidden.

## Pooling artifacts across builds

Each settled artifact records the digest of the pipeline that produced it,
and readers refuse to mix generations unless told to.

-   `TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT`.
    Set to `yes` to resume an accumulation under a build different from the one that filled it.
    The value is spelled out so a stray `0` cannot silently disable the guard.

-   `TRANSLATION_REPAIR_REQUIRED_COMMIT`.
    Restricts a pool to entries whose recorded pipeline contains that commit.

-   `TRANSLATION_REPAIR_POOL_ALL`.
    Set to `yes` to take every generation and say so above the resulting number.
    Setting this together with `TRANSLATION_REPAIR_REQUIRED_COMMIT` throws:
    that asks for a filtered pool and an unfiltered one at once,
    and preferring either would record a policy nobody chose.

## Schema generations, which the drift opt-in does not cover

The pass writes schema generation 14
(`ARTIFACT_SCHEMA_VERSION_V14`,
`package/translation-repair/artifact/src/corpus-run/artifact-two-lane-build.ts`),
and refuses to resume into a directory holding another one
(`package/translation-repair/slice-cache/src/corpus-run/pass-schema-guard.ts`).
That refusal is separate from the build guard in `Pooling artifacts across builds`
and is not waved past by `TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT`:
drift is an opinion about which build filled a pool,
and its remedy works because every file still answers the same questions.
A file of another schema generation cannot answer them at all.

Generation 1 recorded one lane at the top level.
Generations 2 to 4 record the same two-lane shape and differ only in how four keys are spelled,
and generations 5 to 14 keep generation 4's spelling while each changes what the file records
(the version notes in `package/translation-repair/artifact/src/artifact-schema-version.ts` and
`package/translation-repair/artifact/src/corpus-run/artifact-two-lane-contract.ts`).
Generations 4 to 14 spell all four keys the current way:

-   `changedSliceIndices`,
    which generations 1 and 2 spelled `shippedChunkIndices`.
-   `withdrawnSliceIndices`,
    which generations 1 and 2 spelled `withdrawnChunkIndices`.
-   `sliceCritics`,
    which generations 1 and 2 spelled `chunkCritics`.
-   `sliceIndex`,
    which generations 1 to 3 spelled `chunkIndex`.

Generation 3 is therefore a mixture:
the change-set arrays already carry their current names there,
and the index does not.
That is why the reader holds a table rather than a flag
(`package/translation-repair/artifact/src/artifact-key-vocabulary.ts`).
A reader holding a flag would read every generation 3 artifact's index as absent.

The schema reader knows every generation from 1 to 14
(`KNOWN_ARTIFACT_SCHEMA_VERSIONS` in `package/translation-repair/artifact/src/artifact-schema-version.ts`),
and the key table has a spelling for each;
a reader that needs a shape a generation lacks refuses that generation by name.
The reader takes the spelling from the version the file records,
so nothing is ever tried under two spellings,
and a stamp over another generation's keys is refused rather than read as a file missing the keys it names.

Meeting the refusal on a resume,
the ways forward are the ones the message lists:
start a fresh directory with `TRANSLATION_REPAIR_RUNS_DIR`,
restore the code those entries were settled under and resume there,
or move the older artifacts to an archive directory and pay for their re-run.
Deleting them is the one thing to avoid:
it costs the same re-run and destroys a sound result of the generation that wrote it.
