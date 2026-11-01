# Translation repair handover 2026-09-06: costs and the six levers of 2026-09-09

Where the 200 USD of 2026-09-09 went,
what a pass cost before and after the levers,
the catalog prices,
and the six levers that cut the spend.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## Costs

Measured at 18:25 UTC on 2026-09-09 with the scratch `costs-today.mjs` over every log of the day under
`~/temp/agent` (the `cost=` field of each `SPEND` line summed per log,
the first and last `METERS` balances beside it),
the catalogs' prices,
and the seat line of the seventh `Mio` launch.
A reckoned `estimated=abandoned` line is a ceiling (completion at the cap times the price),
so a log's sum can read above what the balance moved.

Prelaunch meter at 21:55 UTC on 2026-09-09:
OpenRouter 270.03 USD,
Bedrock 186.24 USD,
Synthetic zero of its rolling week with 2750/2750 in its five-hour window,
Hyper zero.
`~/temp/agent/class25-prelaunch-meters-20260909.log` is the live reading;
`costs-today.mjs 20260909` output is `~/temp/agent/class25-prelaunch-costs-20260909.out`.
Its `TOTAL logged OpenRouter cost=110.2945` is not an OpenRouter-only spend total:
the helper sums every provider and includes old `unit-*.log` fixtures (`cat/whiskers` alone contributes 1.25).
Use the named provider rows of actual pass logs and live balance movements,
not that label.
New test logs are `.out` or in a subdirectory so they do not enter the daily meter scan.

Historical balances at 20:29 UTC:

- OpenRouter 270.03 USD,
    the owner's final top-up ("I have topped up OpenRouter one final time";
    273.99 at the 16:45 UTC launch).
    The 3.96 spent since is the whole evening:
    the third `noname` 1.87 (1.14 of it the Kimi leak),
    the Mercury calibration 1.82 over 176 minutes,
    the seventh `Mio` 0.59,
    the three class-twenty-four probes 0.14,
    `hulicaijia` 0.06 before it stopped.
- Bedrock 186.24 USD of the 200 the owner will never top up;
    195.26 at 01:34 UTC today,
    so 9.02 spent today (gpt-oss-120b 5.00,
    gemma-4-26b-a4b 3.15,
    gemma-4-e2b 0.85,
    gemma-4-31b 0.02).
- A PASS NOW COSTS about 0.6 to 0.8 USD on OpenRouter and 0.2 on Bedrock with Synthetic dry,
    against the 4 to 9 USD of the morning's passes.
    At that rate the OpenRouter balance holds roughly 300 more passes,
    and Bedrock's credit is the scarcer one.
- Synthetic at zero of its rolling week since 17:25 UTC;
    it returns as the oldest usage ages out.
- Hyper at zero since 12:45 UTC on 2026-09-08 and never to be recharged.

WHAT THE DAY COST BEFORE THE LEVERS
(01:34 to 13:04 UTC,
the build before `037d1f650`,
Synthetic and Hyper dry throughout,
so every non-Bedrock call went to OpenRouter):
21 passes logged 104.31 USD on OpenRouter while the balance moved 130.99 to 0.09,
the 26.6 USD between them the abandoned streams the endpoints billed to the end before
`4f87555fc` wrote a line for them;
with the 69 USD spent between the 200.01 top-up of 2026-09-08 11:26 UTC and 01:34 UTC,
that is the 200 USD the owner asked about.
Per pass,
in launch order:
`hakureico19` 8.88 (SETTLED),
`yuki7` 4.64 (SETTLED),
`hakureico20` 6.15 (ERROR),
`gqt1` 5.72,
`mio1` 4.04,
`gqt2` 5.83,
`hakureico21` 6.84 (INCOMPLETE),
`mio2` 4.71,
`gqt3` 6.61,
`hakureico22` 3.87 (INCOMPLETE),
`mio3` 3.66,
`gqt4` 7.05 (SETTLED),
`hakureico23` 6.80 (SETTLED),
`mio4` 6.41,
`hulicaijia1` 0.26,
`hulicaijia2` 7.15,
`mio5` 5.32,
`noname1` 5.04 (SETTLED),
`hulicaijia3` 2.57 (ERROR),
`mio6` 1.05 (INCOMPLETE),
`noname2` 1.72 (SETTLED);
a pass with no status was killed for a fix and relaunched.
By model on OpenRouter today:
deepseek-v4-pro 39.77,
Qwen3.8-27B 24.10 (off the OpenRouter catalog since `037d1f650`),
glm-5.3 16.73 (off since `037d1f650`),
Kimi-K3 10.80 (the reach honours the withholding since `19b6043a3`),
deepseek-v4-flash 3.87,
minimax-m3 2.40,
glm-5.3-flash 0.77.

WHAT A PASS COSTS ON THE LEVERS
(since the 16:45 UTC launch of the third `noname` on `33a023445`):

- The third `noname`,
    83 minutes,
    SETTLED,
    Synthetic wet for its first 40 minutes:
    1.87 USD on OpenRouter,
    of which 1.14 was 22 Kimi-K3 calls the router sent there after Synthetic dried,
    so 0.73 for the other 382 OpenRouter calls;
    0.19 on Bedrock;
    28 abandoned streams reckoned at 0.44 (a ceiling;
    23 of them DeepSeek-flash at 0.05 in all).
- The Mercury producer calibration,
    0.98 USD logged on OpenRouter by round 21 of 40 (0.17 of it reckoned abandoned);
    every `inception/mercury-2.5` call so far billed `cost=0` on the Inception endpoint,
    against the listing of 0.04 and 0.15 USD per million.
- The seated-roster fidelity probe,
    0.02 USD.
- Since the top-up the balance moved 273.99 to 271.65 (2.34 USD),
    with 2.87 logged,
    the difference the reckoned ceilings.
- The seventh `Mio` (18:20 UTC,
    frozen `19b6043a3`) is the first pass with the leak closed;
    read its `SPEND` sum for what a page costs with Synthetic dry,
    expected near 0.7 to 1.0 USD on OpenRouter and 0.2 on Bedrock,
    which puts the balances at some 270 pages on OpenRouter and 600 on Bedrock.

PRICES THE CATALOGS CARRY,
USD per million prompt and completion tokens,
which `SPEND cost=` is read off the wire against:

- OpenRouter:
    deepseek-v4-pro 0.58 and 1.74 (paid 1.87 times that before the price sort of `1fe7ca2fe`),
    deepseek-v4-flash 0.065 and 0.18,
    minimax-m3 0.3 and 1.2,
    glm-5.3-flash 0.075 and 0.25,
    gpt-oss-120b 0.037 and 0.17,
    gemma-4-26b-a4b 0.07 and 0.34,
    mercury-2.5 0.04 and 0.15,
    kimi-k3 3 and 15 (served,
    not bought).
- Bedrock:
    gemma-4-e2b 0.04 and 0.08,
    gemma-4-26b-a4b 0.13 and 0.40,
    gemma-4-31b 0.14 and 0.40,
    gpt-oss-120b 0.1545 and 0.618.
- Synthetic and Hyper are prepaid quotas,
    not per token;
    what reaches them costs nothing per call,
    which is why the levers that cut tokens make them last longer and send less to OpenRouter.

## The bleed and the six levers

The owner asked on 2026-09-09 where 200 USD went in a day and said to do everything to stop bleeding,
to always fix and relaunch,
and not to ignore that whatever reaches Hyper and Synthetic more efficiently makes those last longer.
The reckoning and the six levers are in the planning log under
"The owner asks where 200 USD went,
and six levers land".
In one breath:
every call on every provider carries a measured `max_tokens` (`completion-cap.ts`);
every round asks quorum plus one seat from a bench rotated by the prompt and the rest only when a voice is lost
(`stage-fanout-window.ts` for the sixteen `gatherStageVoices` stages,
`stage-windowed-rounds.ts` for the six that read their own round);
OpenRouter calls are price-sorted,
carry `cached=` on the `SPEND` line,
write a reckoned `SPEND ... estimated=abandoned` line for every abandoned stream,
and no longer serve Qwen3.8-27B or glm-5.3.
Three rules came with the window:
the coverage verdict's majority is over the seats asked;
a judge bench of four asks the whole bench,
since three halves cannot carry a unanimous self-written slate;
and the naturalness confirmation challenges exactly the seats the discovery asked,
at the discovery's quorum.
A fixture that scripts every seat says `fanOut: 'whole-bench'`;
a fixture that counts calls counts `firstRoundWindow({ benchSize, })`.
Mercury 2.5 judges since `fcc8ca197` (14 of 14 on the fidelity probe of 17:00 UTC),
wrote in both lanes from its calibration of 2026-09-09,
and writes the consolidation alone since the calibration of 2026-09-19 read it below the pooled null.
