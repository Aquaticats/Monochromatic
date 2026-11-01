//region Roster seats
// THE SEATS UNIT TESTS SIT MODELS IN, NAMED BY THE ROLE THE TEST LEANS ON
// rather than by model, so a model leaving the roster changes one line here
// and not the hundred-odd tests that only needed "a Hyper-only model" or
// "a Synthetic model that reads pictures". The owner asked for this on
// 2026-09-16 after the DeepSeek V4 unseat touched fifty-eight test files.
//
// EACH NAME IS A CLAIM ABOUT REACH, IMAGE INPUT AND HOLDS, and
// `roster-seats.unit.test.ts` holds every claim against the cards, so a
// remapping that no longer fits its name fails there before any test that
// leaned on the claim fails somewhere else.
//
// TEST SUPPORT, NOT PACKAGE SOURCE (ledger B31). Until 2026-09-29 this was
// `roster-fixture.ts`, which the roster barrel re-exported, so the build
// shipped seats only tests read.
//
// One exported constant per seat rather than one object: declaration emit
// (`isolatedDeclarations`) infers a string literal's type and not an
// object literal's; the `as const` keeps each literal non-widening so a
// fixture object built from it keeps the id's literal type.

/**
 Hyper and OpenRouter serve it (Synthetic lists it and is withheld on measured
 latency, class one hundred eighteen); reads pictures; holds the first editor
 seat and is out of the wide and late judge seats.
 */
export const SEAT_HYPER_OPENROUTER_VISION_EDITOR = 'hf:zai-org/GLM-5.3-Flash' as const;

/**
 Synthetic and Hyper serve it, OpenRouter dropped it; reads pictures.
 */
export const SEAT_SYNTHETIC_VISION_NO_OPENROUTER = 'hf:Qwen/Qwen3.8-27B' as const;

/**
 Synthetic, Hyper and OpenRouter serve it, OpenRouter withheld on cost;
 reads pictures; too slow in the select seats when Hyper serves it.
 */
export const SEAT_SYNTHETIC_VISION_WITHHELD = 'hf:moonshotai/Kimi-K3' as const;

/**
 Every provider serves it; text only; measured out of the translator seat.
 */
export const SEAT_SYNTHETIC_TEXT_EVERYWHERE = 'hf:openai/gpt-oss-120b' as const;

/**
 Hyper and OpenRouter serve it; reads pictures; holds a refiner seat.
 */
export const SEAT_HYPER_VISION = 'minimax-m3' as const;

/**
 Hyper, OpenRouter and Bedrock serve it; Bedrock and OpenRouter show it pictures (OpenRouter
 since 2026-10-21, when `reader-measure` lifted the hold `openrouter-pictures-unmeasured`), Hyper does not.
 */
export const SEAT_HYPER_TEXT_BEDROCK = 'gemma-4-26b-a4b-it' as const;

/**
 Only Hyper serves it (OpenRouter dropped it); text only; out of the wide
 seats; holds an editor seat.
 */
export const SEAT_HYPER_ONLY = 'glm-5.3' as const;

/**
 Hyper and OpenRouter serve it; reads pictures; its picture reading is
 unmeasured (the name dates from before the producer calibration of
 2026-09-19 measured its writing in).
 */
export const SEAT_HYPER_OPENROUTER_UNMEASURED = 'deepseek-v4.1-flash' as const;

/**
 Spelling Charm Hyper takes on the wire for that seat since 2026-10-20: the
 1017 weights (`deepseek-ai/DeepSeek-V4.1-Flash-1017`), beside the seat's
 roster id, which is unchanged.
 */
export const SEAT_HYPER_OPENROUTER_UNMEASURED_HYPER_ID = 'deepseek-v4.1-flash-1017' as const;

/**
 Spelling OpenRouter takes on the wire for that seat since 2026-10-20: the
 1017 weights, beside the seat's roster id, which is unchanged.
 */
export const SEAT_HYPER_OPENROUTER_UNMEASURED_OPENROUTER_ID = 'deepseek/deepseek-v4.1-flash-1017' as const;

/**
 Only Bedrock serves it; text only; a seated judge and measured writer.
 */
export const SEAT_BEDROCK_ONLY_TEXT = 'google.gemma-4-e2b' as const;

/**
 Only Bedrock serves it; reads pictures; no judge seat yet.
 */
export const SEAT_BEDROCK_ONLY_VISION_UNSEATED = 'google.gemma-4-31b' as const;

/**
 Only OpenRouter serves it; text only; a seated judge and consolidation
 writer, out of the translator seat since the calibration of 2026-09-19.
 */
export const SEAT_OPENROUTER_ONLY = 'inception/mercury-2.5' as const;

/**
 Only OpenRouter serves it; reads pictures, unmeasured as a reader; a
 measured checker and held out of every other seat until measured there.
 */
export const SEAT_OPENROUTER_ONLY_CHECKER = 'xiaomi/mimo-v2.6-flash' as const;

/**
 Only OpenRouter's decisions endpoint serves it; no chat reach, no
 pictures; held out of every seat until measured.
 */
export const SEAT_OPENROUTER_DECISIONS = 'typesafe/jev-1.13' as const;

//endregion Roster seats
