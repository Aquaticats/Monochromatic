# OpenRouter and the corpus passes: XingZ6016 on build 3637592bd4 (2026-10-21)

What was read on 2026-10-21 for the third pass of the restarted rotation,
which stopped before it wrote a page.
Entry:
XingZ6016.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the three passes of the restart share are in
[the rotation restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md).

## XingZ6016 picture stop, 2026-10-21: INCOMPLETE after 6.7 min of pass; `visual evidence incomplete for 1 referenced assets`

`XingZ6016` ran on tip `3637592bd4` and stopped INCOMPLETE after 6.7 min of pass with
`visual evidence incomplete for 1 referenced assets`.
The read of the stop was written at a coordinator halt (a machine reboot) and is partial.
Nothing was run against a provider,
and no code or worktree was changed.
The build was read with `git show` and `git grep` only.
Every inference in this file that the read marked as a guess is marked as a guess.
The read carries no clock time for the pass,
so none is given here.

### Short verdict

- The entry stopped because the picture bench had only two live readers,
  `minimax-m3` and `hf:zai-org/GLM-5.3-Flash`,
  and corroboration needs two agreeing readings.
- `GLM-5.3-Flash` did not fail for lack of vision.
  It was cut by the completion cap while still reasoning:
  `text: ""`,
  `finishReason: "length"`,
  18316 completion tokens,
  40652 reasoning characters and 0 content characters.
- The other three seats were dead for budget or credential reasons,
  two of them permanently.
- The same shape was recorded before:
  `Xu_Yushu1`,
  2026-09-26,
  `readers=5`,
  Synthetic and Hyper dry,
  and the note concluded
  "the stop is the bench's reach while Synthetic is dry,
  not a code defect".

### Who served what

In `XingZ6016`:

- `minimax-m3`:
  OpenRouter,
  endpoint `Together`,
  5107 prompt and 3472 completion tokens,
  cost 0.00566778 USD,
  read 2259 characters (log lines 529 to 531).
- `hf:zai-org/GLM-5.3-Flash`:
  NeuralWatt `glm-5.3-flash`
  (log line 533 `SPEND provider=neuralwatt`;
  the stored reply carries `servedBy: "neuralwatt"`).
- `hf:Qwen/Qwen3.8-27B`:
  no provider.
  Synthetic and Hyper were dry,
  and the OpenRouter side is absent by the `openrouter-dropped` hold (`model-cards.ts:118`).
- `gemma-4-26b-a4b-it`:
  no provider.
  For a picture only Bedrock may show it
  (`model-cards.ts:270` OpenRouter `readsImages: false`,
  `:282` Bedrock `true`),
  and Bedrock was held out for the run.
- `google.gemma-4-31b`:
  no provider.
  Bedrock-only (`model-cards.ts:482`),
  under the same hold.
- The Bedrock refusal is an expired credential,
  not an empty balance:
  log line 245 reads HTTP 401 "The security token included in the request has expired",
  while the meters read `bedrockUsd=5.02`.
- Meters at the start (log line 233):
  `neuralwatt=wet synthetic=dry bedrock=wet hyper=dry openrouter=wet`,
  `hyperBalance=0`,
  `syntheticWeekly=0%`,
  `openrouterUsd=13.64`.
- Seat line (log line 525):
  `readers=5 withheld=hf:moonshotai/Kimi-K3 unserved=hf:Qwen/Qwen3.8-27B,google.gemma-4-e2b,glm-5.3,google.gemma-4-31b`.

In `XingZ6015` (log lines 501 to 524),
the pass that read this same picture:

- Seat line:
  `synthetic=wet bedrock=wet hyper=wet openrouter=wet readers=6`.
- `minimax-m3`:
  Hyper,
  1568 characters.
- `hf:Qwen/Qwen3.8-27B`:
  Synthetic,
  2115 characters.
- `hf:moonshotai/Kimi-K3`:
  Synthetic (a first attempt hit HTTP 500),
  2134 characters.
- `hf:zai-org/GLM-5.3-Flash`:
  Hyper (`glm-5.3-flash`),
  8189 prompt and 6523 completion tokens,
  10470 reasoning characters,
  77 s,
  2125 characters.
- `gemma-4-26b-a4b-it`:
  withheld after throwing on 4 of 4 asks.
- `google.gemma-4-31b`:
  Bedrock HTTP 401,
  expired token.
- Result:
  "corroborated by 4 of 4 readers at overlap 0.898".

### Where the roster and routing are defined

All at `3637592bd4`,
with the line numbers re-checked against that commit for this record:

- `package/translation-repair/run/src/corpus-run/run-config.ts:813` `RUN_READER_MODELS`:
  every roster model with any provider showing a picture,
  minus `READER_UNMEASURED` (`:784`,
  hold `reader-unmeasured`) and `OWNER_CULLED` (`:86`).
- `package/translation-repair/run/src/corpus-run/run-seats.ts:343` `readers = RUN_READER_MODELS.filter(seated)`,
  per entry off the meters.
- `package/translation-repair/corpus-pass/src/corpus-run/pass-seated-pictures.ts:118` passes `seats.readers` to the reader stage;
  `package/translation-repair/glossary/src/picture-reader-seating.ts` re-seats per picture.
- `package/translation-repair/provider/src/roster-reach.ts:492` `visionReachOf` and `:516` `readsImages`,
  read from each model card side.
- `package/translation-repair/provider/src/reach-past-holds.ts:45` `reachOfRequest` switches to `visionReachOf`
  when a call carries a picture (`chat-contract.ts:155` `carriesPicture`).
  The read cited `:40` for the function;
  the definition is at `:45`.
- `package/translation-repair/shared-contract/src/provider-name.ts:57` `PROVIDER_ORDER`:
  `neuralwatt`,
  `synthetic`,
  `bedrock`,
  `hyper`,
  `openrouter`.
- `package/translation-repair/provider/src/budget-routing.ts:293` `providerServing` and `:329` `routeProviderFor`:
  first provider in order that serves the model and has budget.
- The cards are in `package/translation-repair/shared-contract/src/model-cards.ts`
  (GLM `:35`,
  Qwen `:98`,
  Kimi `:120`,
  minimax `:217`,
  gemma 26b `:262`,
  deepseek `:339` with its hold at `:442`,
  gemma 31b `:479`) and in `neuralwatt-card-sides.ts`.
- Pinned by a test:
  `provider/src/roster-reach.unit.test.ts:400` says gemma pictures go through Bedrock alone,
  because "OpenRouter's listing field alone cannot widen the reader roster".

### Why GLM-5.3-Flash returned nothing

The read answered this with one guess about the cause of the overrun.

- The request is `READING_INSTRUCTION` plus one `image_url` data URI of `image0.webp` (1271156 bytes),
  built in `slice-reading/src/image-reading-stage.ts`.
  The stored prompt payloads hold only replies,
  not the image.
- The stored reply is `text: ""`,
  usage 8188 prompt and 18316 completion tokens,
  `finishReason: "length"`,
  `servedBy: "neuralwatt"`.
- 18316 equals the card cap `completionCap: 18_316` at `model-cards.ts:84`,
  sent as `max_tokens` at `provider/src/neuralwatt-client.ts:394`.
- Log line 532:
  1922093 raw characters,
  0 content characters,
  40652 reasoning characters.
  So the cap cut a reasoning stream before any answer.
- `readImageAsset` turns empty text into `empty-reply` (`image-reading-stage.ts:340`),
  which is transient (`:203`) and is not re-asked,
  because `image-reading-past-refusal.ts` re-asks only `reads-as-refusal`.
- Not a text-only model:
  the public NeuralWatt listing `GET https://api.neuralwatt.com/v1/models` lists `glm-5.3-flash` with
  `capabilities.vision: true`,
  `limits.max_images: 20`
  and the description "native vision,
  always-on reasoning".
- Not a provider dropping the image:
  prompt tokens were 8188 on NeuralWatt against 8189 on Hyper for the same file,
  which is what an image costs.
  The same seat on NeuralWatt had read three smaller pictures in the `zheermao21` pass
  (that pass's log lines 303 to 322,
  1156 prompt and 2297 completion tokens for `photo1.webp`).
- NeuralWatt lists GLM reasoning as `mandatory: true`,
  `default_effort: "max"`.
  The client sends no effort,
  by the owner rule of 2026-08-25 (`neuralwatt-client.ts:382`,
  `neuralwatt-catalog.ts:35`).
- Guess:
  mandatory `max` effort on a dense 2400 by 2616 score screenshot with about 2100 characters of Japanese
  overran a cap that was measured on text traffic.
- Known shape:
  `completion-cap.ts` records `GLM-5.3-Flash` cut at its cap on 4.85 percent of Synthetic calls
  (97 of 100 with no content) and 1.15 percent on OpenRouter.
- Only one failed run and one Hyper success exist for this picture,
  so no rate can be stated (spread not measured).
- Side finding,
  a guess:
  the NeuralWatt energy estimate counted the base64 picture as text,
  0.9089 Wh estimated for 423985 prompt tokens against 0.2311 Wh measured (`neuralwatt-estimate.ts:210`),
  so picture calls over-reserve the free day by about four times.

### The corroboration rule and why

- Rule:
  at least two readings must exist,
  and some pair must have trigram overlap of at least 0.3,
  measured as the share of the smaller reading's distinct character trigrams that the larger reading carries
  (whitespace collapsed,
  graphemes).
- Code:
  `markup/src/reading-corroboration.ts:47` `CORROBORATION_TRIGRAM_SHARE = 0.3`,
  `:227` `trigramOverlap`,
  `:282` `readingsCorroborate`;
  `markup/src/reading-cluster.ts:112` `clusterReadings` takes every pair,
  and any reading vouched for by any other counts.
- Pair stage:
  `slice-reading/src/image-reading-pair.ts:626` fewer than two readings is `one-reader-only` or `no-reader-available`;
  `:677` no agreeing pair is `readers-disagree`.
- Gate:
  `slice-cache/src/corpus-run/visual-evidence-completeness.ts:55` throws unless every referenced asset is corroborated
  or confirmed no-text;
  that is the `TALLY` text.
- Why:
  `doc/planning/when-an-image-reading-makes-no-sense.md` says trusting a bad reading is expensive and falling back is cheap,
  so the rule is deliberately eager to fall back.
  The same record says models are not judging each other:
  they are asked blind and compared mechanically.
- Measured separation:
  the same picture 0.643 to 1.000,
  different pictures 0.000 to 0.129
  (five pictures,
  so the boundary is not placed precisely).
- Stated limit in that record:
  the rule assumes independent failures,
  and models of one lineage could agree on one misreading;
  the code does not enforce different families.
- Precedent:
  `doc/planning/translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md` (`Xu_Yushu1`)
  keeps corroboration as it is:
  "one model's reading is not evidence".
- Same picture history:
  `XingZ601`,
  2026-09-17,
  `image0.webp` (2400 by 2616),
  first two readers at 0.297,
  fixed by taking every pair.
- Reader-seat rule for new readers:
  `doc/decision/translation-repair-roster-seating-2026-09-01-addenda-2026-09-02-to-2026-09-09.md`,
  addendum 2026-09-08,
  nine pictures scored by the production overlap.
  That addendum also says a 1.27 MB picture came back as an empty stream from every Gemma size on Bedrock
  (the read's guess:
  the same asset class as `image0.webp`).
- Decision record on NeuralWatt pictures:
  `doc/decision/translation-repair-neuralwatt-provider.md` says picture handling there was
  "not verifiable without a key",
  the `deepseek-v4.1-flash` hold stays,
  and "a failed picture call is not rerouted,
  because only budget refusals reroute".

### Vision readers reachable on wet providers

Status in the read:
partly answered
(prices from public listings fetched on the day;
no quality measurement exists for the new names).

- Method:
  `GET https://openrouter.ai/api/v1/models` and `GET https://openrouter.ai/api/v1/endpoints/zdr` with no credential,
  and `GET https://api.neuralwatt.com/v1/models`.
- No row carries a per-image price for these models;
  images bill as prompt tokens.
- The cost per reading in this section is a guess at 8200 prompt and 4000 completion tokens
  at the cheapest zero-retention endpoint.

NeuralWatt (free energy covers only two models):

- `glm-5.3-flash`:
  vision true,
  reasoning mandatory,
  default effort `max`;
  0.15 in and 0.50 out per million;
  in the repo already.
- `deepseek-v4.1-flash`:
  vision true,
  default effort `high`,
  `none` allowed;
  0.15 and 0.60;
  in the repo with hold `reader-unmeasured`.
- The other NeuralWatt vision models
  (`gemma-4-31b`,
  `qwen-3.8-27b`,
  `kimi-k3`,
  `mimo-v2.6-pro`)
  bill the owner's normal plan,
  so they are not wet for this purpose.

OpenRouter with `zdr: true` (the count of zero-retention endpoints in brackets):

- `google/gemma-4-26b-a4b-it` [12]:
  image listed;
  cheapest 0.07 and 0.24;
  about 0.0015 USD;
  Google family;
  in the repo;
  the card says `readsImages: false` on OpenRouter by design;
  measured as a reader only through Bedrock (mean overlap 0.668,
  8 of 9 pictures).
- `google/gemma-4-31b-it` [13]:
  image listed;
  0.10 and 0.34;
  about 0.002 USD;
  not named in the repo (the roster seat `google.gemma-4-31b` is Bedrock-only).
- `xiaomi/mimo-v2.6-flash` [6]:
  image listed;
  0.10 and 0.24;
  about 0.0018 USD;
  in the repo with hold `reader-unmeasured`.
- `deepseek/deepseek-v4.1-flash-1017` [20]:
  image listed;
  Morph 0.179 and 0.6,
  about 0.004 USD;
  in the repo with hold `reader-unmeasured`.
- `qwen/qwen3.8-27b` [17]:
  image listed;
  0.04 and 1.35;
  about 0.006 USD;
  a measured reader (it read this very picture in `XingZ6015`);
  held off OpenRouter by the owner (`openrouter-dropped`,
  24 percent of calls abandoned and billed).
- `z-ai/glm-5.3-flash` [31]:
  image listed;
  Wafer 0.074 and 0.2 (the card's preferred endpoint);
  about 0.0014 USD;
  in the repo.
- `minimax/minimax-m3` [10]:
  Together 0.3 and 1.2;
  0.0057 USD measured.
- `moonshotai/kimi-k3` [26]:
  image listed;
  about 0.05 USD a reading;
  a measured reader;
  withheld from OpenRouter on cost by the owner.
- Blocked or unusable:
  Qwen3.6 and Qwen3.7,
  `qwen3-*` and Llama by the owner blocklist (`stage/src/roster-blocklist.ts`);
  `google/gemini-3.8-flash` ("a wildly misaligned model");
  `qwen/qwen3.8-flash` and `qwen/qwen3.8-omni-flash` have no zero-retention endpoint;
  `:free` variants are never used.
- OpenRouter also keeps an account model allowlist (`doc/decision/translation-repair-openrouter-fallback.md`),
  so a model outside the roster needs the owner to add it
  (guess:
  current state unverified).
- Independent of both `minimax-m3` (MiniMax) and `GLM` (Zhipu):
  the Google,
  Xiaomi,
  DeepSeek,
  Qwen and Moonshot models listed in this section.

### Size of the exposure

Counted with a script that mirrors `shared-contract/src/photo-reference.ts` on the `page.md`,
`page.en.md` and `page.zh_hant.md` of the one-among-us data tree:

- 93 entry directories,
  92 with `page.md`.
- 50 entries reference at least one picture (49 in `page.en.md`),
  198 distinct assets,
  194 `.webp` and 4 `.jpg`,
  0 referenced files missing.
- Pictures per entry in `page.md`:
  43 with none,
  16 with one,
  6 with two,
  6 with three,
  7 with four,
  and 18 entries with five or more
  (largest `dogesir_` 17,
  `noname3031` 13,
  `Chinatsu_Suzuki` 10,
  `Mio` 10).
- Caveat:
  that tree is not a git checkout,
  so it may not be the pass's pin;
  older records quote 380 references and 191 assets.
- Only pictures with at least 16 solid characters from `tesseract` go to models;
  the figure in the doc is 119 of 191 textless at an older pin
  (a guess for today,
  not measured here).
- Guess for spend if every text-bearing picture were read by three wet readers:
  under 2 USD.

### Remedies the read ranked

These are the read's proposals,
ranked by whether each fixes the observed failure,
whether it needs an owner decision,
and cost.
None was adopted by the read.

1. Re-ask a length-cut empty reply,
   and route the second ask away from the failing route.
   - Change:
     in `image-reading-past-refusal.ts`,
     treat `finishReason === 'length'` with empty text as a runaway and ask again
     (`ChatTextReply.finishReason` already exists,
     `chat-contract.ts:282`).
   - Pros:
     smallest change;
     no new seat;
     no owner decision;
     covers every reasoning reader added later.
   - Cons:
     a same-route re-ask may repeat (guess);
     sending it to OpenRouter needs a router change,
     because today only budget refusals reroute;
     each re-ask costs a few thousand tokens.
   - Cost:
     code and tests in `slice-reading`,
     plus `provider` if rerouting.
2. Add a third and fourth wet reader from a family independent of MiniMax and Zhipu.
   - Best evidence:
     `qwen/qwen3.8-27b` on OpenRouter (it read this picture in `XingZ6015`),
     which needs the owner to reverse `openrouter-dropped` for picture reads
     (guess:
     the 24 percent abandonment was a judge-round effect that picture reads do not share).
   - Next:
     `google/gemma-4-26b-a4b-it` on OpenRouter,
     already on the roster and allowlist,
     cheapest;
     it needs the nine-picture measurement of the 2026-09-08 rule before its OpenRouter `readsImages` flips;
     risk:
     Gemma failed on a 1.27 MB picture on Bedrock.
   - Then `deepseek-v4.1-flash` (free on NeuralWatt) and `mimo-v2.6-flash`,
     each needing the same measurement to lift `reader-unmeasured`.
   - Pros:
     gives slack,
     so one reader failing no longer stops an entry.
   - Cons:
     an owner decision or a measurement run first;
     DeepSeek may reason long like GLM.
   - Cost:
     about 0.002 to 0.006 USD a reading;
     one measurement run of nine pictures per model.
3. Seat the reader bench by `visionReachOf` and retire the Bedrock-only reader sides.
   - Evidence:
     `run-seats.ts:220` `servedBy` and `run-seats-wait.ts:183` `reachableSeats` use text reach,
     so `gemma-4-26b-a4b-it` counts as served because OpenRouter serves it for text,
     and the seat line says `readers=5` when two can see.
   - Pros:
     honest bench size and quorum;
     no wasted failing calls.
   - Cons:
     adds no reading by itself.
   - Cost:
     a small code change plus test updates.
4. Raise the picture-call completion cap for reasoning readers
   (a new field,
   since `requested` only lowers the cap).
   - Pros:
     lets `max` effort finish;
     NeuralWatt energy is cheap (0.23 Wh for 18316 tokens against 550 Wh a day).
   - Cons:
     the doc notes a longer cap buys a longer runaway;
     the evidence is one failure.
   - Cost:
     a card field and tests.
5. Send a lower reasoning effort to NeuralWatt GLM.
   - Pros:
     attacks the cause directly.
   - Cons:
     breaks the owner's standing rule of 2026-08-25 ("no reasoning parameter ever");
     needs an explicit exception.
   - Cost:
     an owner decision first.
6. Route GLM pictures to OpenRouter Wafer instead of NeuralWatt.
   - Pros:
     avoids the NeuralWatt default;
     about 0.0014 USD a reading.
   - Cons:
     one failure against three small successes on NeuralWatt is not a rate;
     spends paid money where free energy exists.
7. Stopgaps with no code:
   rerun after Synthetic's weekly meter resets
   (restores `Qwen3.8-27B` and `Kimi-K3`,
   as in `XingZ6015`),
   or rerun now and hope GLM finishes.
   - The unavailable verdict is not cached (log line 540),
     so a rerun asks all readers again.
   - Pros:
     free.
   - Cons:
     leaves a two-reader bench with no slack;
     a rerun now needs GLM to finish.

Not recommended by the read:

- Dropping the GLM seat alone leaves one live reader.
- Lowering the 0.3 overlap or accepting one reading contradicts the recorded rule.
- Using `tesseract` as a second voice was measured at 0.019 to 0.111 overlap
  against models that agree at 0.643 or more.

### Findings not written elsewhere

- `reachableSeats` and `servedBy` use text reach for the readers bench (remedy 3);
  inferred from code,
  not shown to bite in this run.
- `image0.webp` is 1271156 bytes and is the `XingZ601` picture (2400 by 2616) that disagreed at 0.297 on 2026-09-17.
- `XingZ6016` text calls on OpenRouter served `google/gemma-4-26b-a4b-it` through `DeepInfra` (log lines 252 to 253),
  so the Gemma text route works and only the picture route is closed.
- OpenRouter balance was 13.64 USD at the start of the run.

### What the read left open

- Whether the OpenRouter endpoints behind `google/gemma-4-26b-a4b-it` and `z-ai/glm-5.3-flash` accept images individually
  (the listing is per model;
  not checked per endpoint).
- Whether the OpenRouter account allowlist currently holds `google/gemma-4-31b-it` (unverified).
- When Synthetic's weekly meter resets (not read).
- The cause of the overrun beyond the guess about `max` effort:
  cause not yet traced.

### What followed on 2026-10-21

The owner ruled on two points just before the machine was rebooted
(from [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-the-reboot-gemma-measured-and-the-katex-shim.md)):

- On the picture bench:
  "Yes try gemma".
  So `google/gemma-4-26b-a4b-it` on OpenRouter (zero data retention) is to be measured as a second picture reader
  from another family,
  over a sample of pictures,
  and seated if it reads them well.
- On request parameters:
  NeuralWatt is a special case,
  the ban on passing parameters is lifted for NeuralWatt only,
  so NeuralWatt requests may carry documented parameters,
  such as a lower reasoning effort for `glm-5.3-flash` on pictures.
  Every other provider still gets no such parameter.

An empty reply cut at the length cap being asked again
(remedy 1,
today only refusals are re-asked)
is queued with the picture bench fix.
