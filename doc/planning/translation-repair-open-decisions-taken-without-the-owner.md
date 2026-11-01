# Translation repair open decisions: the numbered decisions taken without the owner

Decisions 1 to 28,
each open to veto:
validation and selection rosters,
footnote integrity at assembly,
the two-lane driver and its results,
the bench cost split,
what withdrawn and repaired mean,
cache keys,
placement chunks and unfilled passages,
and the proof that a passage was never translated.

Part of [Decisions waiting on you, 2026-08-15 morning, still open on 2026-08-19](translation-repair-open-decisions.md),
which lists every file of this record.

## The numbered decisions

1.  A revision that still fails validation is NOT taken;
    the original candidate
    stands.
    Reasoning:
    the model was asked to fix those findings and did not,
    so nothing says the new text is better,
    while the original is at least what
    it produced with the whole sheet in front of it.
2.  The incumbent never passes through structural validation and can never be
    dropped by it.
    Reasoning:
    it is the fallback,
    so a check that could drop it
    could delete the archive.
3.  A slice's candidate order is rotated by a hash of the source,
    so the
    incumbent does not sit in one ballot position.
    Deterministic,
    so a resumed
    slice asks the judges the same question a fresh one did.
4.  Structural validation compares block structure,
    footnote markers,
    link and
    image destinations,
    and inline code,
    and deliberately NOT numbers or names,
    because 三只猫 becomes "three cats" and no digit survives on either side.
5.  Atoms are compared as a multiset rather than in order,
    since a translation
    reorders clauses and a link moving inside a sentence is not damage.
6.  THE ROSTER GUARD NOW MATCHES YOUR RULING.
    It required two judges with no
    stake in any candidate,
    which is the rule your self-vote discount replaced,
    and on six models that capped producers at four.
    It now refuses only
    rosters that could not decide a round however they voted:
    repeats on either
    side,
    no producer,
    or too little available weight to reach the minimum.
    The weight limb catches a case a seat count would have passed,
    one producer
    judged by itself and one other model,
    which tops out at 1.5 against a
    minimum of 2 and would have declined every round in silence.
    CORRECTED LATER THE SAME NIGHT,
    in `9e43d5afc`,
    after a second review:
    my
    first weight limb measured what a candidate would draw if EVERY producer
    had a stake in it,
    and on that reading it refused three authors judging only
    each other.
    That bench decides comfortably,
    because a candidate one of them
    wrote draws half a vote from its author and a full one from each of the
    other two.
    It now measures the most favourable candidate instead,
    which is
    the question a guard refusing rosters that could not decide HOWEVER they
    voted has to ask.
    The case above still refuses.
    ONE EXCEPTION worth your veto if you dislike it:
    four models returning
    byte-identical text collapse into one candidate,
    and four self-votes at a
    half reach the minimum with no outside judge.
    I kept it,
    because agreement
    to the byte between independent models is itself the corroboration,
    and
    pinned both it and the three-contributor case that falls short in tests.
    THE ALTERNATIVE,
    which an external reviewer proposed and I did not take:
    require at least one FULL-weight ballot on the winner,
    which would refuse
    that four-way agreement outright.
    I left it for you because it is a new rule
    on top of the weighting you chose rather than an implementation of it,
    and
    because it would also block the case where four models agree and the two
    remaining judges simply went quiet.
    Say the word and it is a three-line
    change plus its tests.
7.  IDENTICAL CANDIDATES NOW MERGE THEIR AUTHORS in the editor and naturalness
    lanes.
    They did not,
    so a model could vote at full weight for its own words
    whenever another model wrote them first,
    and the ballot split across
    identical texts.
    Found by an external review of the guard change.
8.  The translate stage now RECORDS THE SLATE the judges were shown,
    with each
    position's text,
    hash,
    origin and producer.
    Ballots name a position and the
    slate is rotated per slice,
    so a stored ballot could not be joined to any
    text afterwards.
    Judges still see anonymous positions;
    provenance is
    attached to the record after the round.
9.  The source-side slice budget is now derived from the whole document pair
    rather than from one section,
    and capped at the target budget.
    Reasoning in
    `#90`;
    the cap encodes that Chinese runs shorter than its English rendering,
    so a ratio above one is missing translation rather than density.
10. A SELECTION ROUND NOW REFUSES A ROSTER NAMING ONE MODEL TWICE,
    before it
    spends a call.
    The stage guard already refused that,
    but two of the
    selection entry points are reachable without the stage,
    and a repeat there
    bought two exchanges from one model,
    which is two ballots and enough to
    reach the minimum weight alone.
    Refused rather than silently deduplicated,
    since a caller that passed a repeat believes it has more judges than it has.
11. A REPLACEMENT THAT BREAKS A FOOTNOTE IS WITHDRAWN AT ASSEMBLY,
    rather than
    reported and shipped,
    IN BOTH LANES.
    Reasoning:
    it is the same species of
    guard as the alignment refusal you already have,
    and a dangling `[^1]` is
    not a judgement call.
    The judges are not wrong here either:
    each slice
    validated on its own,
    and the marker that went missing belongs to a line in
    a slice they never saw.
    The per-slice record still says a replacement was
    chosen,
    so the record and the document disagree ON PURPOSE:
    one says what
    was chosen,
    the other says what the document could carry,
    and the shipped
    counts follow the document.
    IN THE REPAIR LANE IT ALSO REACHES THE ISSUE RECORDS,
    which is the part
    that could have gone wrong quietly:
    an issue whose slice was withdrawn is
    recorded `withdrawn` and unresolved,
    the same disposition a non-translation
    block already used,
    because both mean the repair reached no reader.
    Crediting it would overstate the precision measurement directly.
    WHAT THIS GUARD IS NOT,
    so you do not read more into it than it does:
    it
    checks INTEGRITY,
    not preservation.
    It compares the footnote graph the
    document comes out with against the graph it went in with,
    and withdraws a
    replacement that made that graph worse.
    A candidate that drops a footnote
    pair WHOLE,
    marker and definition together,
    leaves a graph with one fewer
    footnote and nothing dangling,
    so it ships with no finding.
    So does one that
    renames both halves consistently.
    Both are losses of content rather than
    breaks in structure,
    and the instrument for those is `#85`,
    which is where
    the damage question belongs.
12. WHEN NO SLICE CAN BE BLAMED,
    EVERY replacement is withdrawn and the archive
    text ships.
    Reasoning:
    that shape comes from how replacements MEET,
    a stray
    comment opener masking markers document-wide above all,
    so picking a slice
    to withdraw would be a guess and shipping a document the lane knowingly
    broke is worse.
    Rare by construction,
    loud when it happens.
    A STRUCTURAL REGRESSION ALWAYS TAKES THIS BRANCH,
    even where one replacement
    visibly contains the `<!--` that caused it.
    Attribution asks which slice
    changed its mention of an IDENTIFIER,
    and an unterminated comment or an MDX
    downgrade names none,
    so there is nothing to match it against.
    Worth knowing
    because the branch is expensive:
    one bad slice costs the document every
    other repair in it.
    HOW OFTEN THIS FIRES IS FREE TO MEASURE under Question 5's report-only
    option,
    since a pass that writes nothing still runs the guard and records
    what it withdrew.
    If you pick that option,
    the rate arrives with it.
13. NO OTHER CROSS-SLICE GRAPH WAS BUILT,
    though an external review listed
    several.
    Measured over all 184 corpus documents:
    209 GFM footnote markers in
    45 files,
    and zero reference-style link definitions,
    zero reference-style
    link uses,
    and zero heading-anchor links.
    Footnotes are the only cross-slice
    relation this corpus has.
    Structural parse regressions are read alongside
    them,
    because an unterminated comment and an MDX downgrade name no
    identifier and nothing else would notice.
14. A GRADING SHEET NO LONGER SHOWS A REWRITE THE DOCUMENT DID NOT CARRY as the
    wording that shipped.
    The record's final-wording field was written whenever
    the naturalness lane rewrote a slice,
    which was right until decision 11 let
    assembly take a rewritten slice back.
    It is now written only where the
    document carries the rewrite,
    so a withdrawn slice states no final wording
    and the sheet says the rewrite was taken back rather than fencing an empty
    block under "the slice as actually returned".
    Found by an external review of
    tonight's guard change;
    the artifact reader was taught the same rule in the
    same commit,
    because it required that field of every rewritten slice and
    would otherwise have refused to read the run.
15. FOOTNOTE LABELS ARE FOLDED THE WAY THE PARSER FOLDS THEM before anything
    compares them.
    Markdown reads `[^Note]` and `[^note]` as one footnote and
    mdast hands back one spelling,
    while the raw scans this guard attributes
    with see what was written.
    Measured on a fixture before the fix:
    the guard
    looked for a finding about `note` in mentions keyed `Note`,
    could blame no
    slice,
    and withdrew BOTH replacements including one that touched no footnote.
    Nothing settled is affected,
    because all 209 corpus markers are numeric and
    folding a digit changes nothing;
    a model writing a word-labelled footnote
    into a replacement triggers it at once.
16. BOTH LANES NOW RUN FROM ONE DRIVER THAT ARBITRATES NOTHING.
    `runDocumentLanes` takes one prepared pair,
    runs repair and then translate
    over it,
    and returns both documents with no winner,
    no preferred lane and no
    merged text,
    because choosing between them is Question 5 and a driver that
    chose would answer it invisibly for every later count.
    Three sub-choices are
    open to veto on their own:
    the lanes run SEQUENTIALLY,
    since the quota spent
    is the same and both already serialize their own slices;
    REPAIR RUNS FIRST,
    because its naturalness phase settles after the slice loop and nothing
    persists what that phase produced,
    while the translate lane caches every
    slice as it finishes,
    so a deadline cutting the entry loses less of what was
    bought;
    and there is NO ABORT CHECK BETWEEN THE LANES,
    since both drivers
    deliberately let a fully cached lane finish after an abort and a gate there
    would refuse that.
    Alignment findings are reported once at the top level
    rather than per lane,
    because they belong to the preparation both lanes
    shared and counting them per lane would count one defect in the archive
    twice.
17. EACH LANE RESULT NOW NAMES THE SLICES IT SHIPPED AND WITHDREW,
    read off the
    assembly guard rather than off the per-slice records.
    A record says what its
    own slice CHOSE,
    and a slice can be withdrawn while carrying no issue of its
    own,
    so a comparison built from records would credit a lane with slices the
    returned document does not carry.
    Recorded into the settled artifact too,
    so
    the withdraw rate is countable over a directory.
    Artifacts settled before
    2026-08-15 lack both fields,
    and a reader must treat their absence as
    unknown rather than as empty.
20. BOTH LANES NOW REFUSE A CHANGE THE DOCUMENT DOES NOT CARRY,
    rather than
    reporting it,
    AND THE PLACE THEY REFUSE IT MOVED ON 2026-08-15.
    The way in
    was the slice cache:
    a resumed record was trusted on its index alone,
    so one
    claiming a change while holding the archive's own wording reached assembly,
    survived the footnote guard untouched,
    and landed in the shipped set beside
    a document nobody changed.
    Both lanes now check that a resumed record's
    changed flag agrees with its own text WHERE THEY ACCEPT IT,
    in both
    directions,
    and discard a record that disagrees so that slice is simply
    bought again.
    One bad cache file costs one slice rather than the entry,
    and
    each discard is named in the findings,
    because a recomputed slice is
    otherwise indistinguishable from one that was never cached.
    The quieter
    direction is the one nothing caught:
    only changed records become
    replacements,
    so a record DENYING a change it made had its wording dropped
    at assembly with nothing said.
    THE ASSEMBLY CHECKS REMAIN,
    as a backstop rather than the first line:
    one
    refuses a replacement that repeats its slice's incumbent or names a slice
    the preparation never produced,
    and one derives the shipped set from the
    surviving replacements,
    re-splices them,
    and refuses any returned document
    they do not reconstruct.
    Both index sets are also checked against each other
    and put in document order,
    which the withdrawn one never was.
    THEY STILL THROW,
    and that is now the only place the throw-versus-finding
    question is live.
    Dropping a suspect replacement silently would leave a run
    reporting counts nobody can reproduce,
    but throwing loses an entry's
    unpersisted work.
    The shape the review recommends is a typed error caught at
    the corpus-entry boundary,
    so the pass continues and the entry stays
    unsettled with its cached slices reusable,
    and that boundary is part of the
    wiring Question 5 shapes.
    `#95` records the rest.
19. THE TWO LANES CAN NOW BE COMPARED SLICE BY SLICE,
    and the comparison reads
    what each DOCUMENT carries rather than what each lane chose.
    Every lane
    result reports,
    for every prepared slice,
    the archive's own wording beside
    the wording that lane decided on.
    A separate pure function joins two results
    on the slice index and names each slice:
    the archive stands in both,
    one
    lane moved,
    both moved to the same wording,
    or both moved apart.
    That last
    one is the case a human has to read,
    and it is the reason the two-lane shape
    exists.
    Deliberately NOT on the per-slice records:
    whether a slice shipped
    is decided by an assembly guard reading the whole document and can differ
    between two runs of the same slice,
    so it stays on the document-level index
    sets and the comparison derives the rest.
    Nothing calls the comparison yet;
    wiring it into a pass is shaped by Question 5.
18. THE BENCH NOW PRICES SENDING AND ANSWERING SEPARATELY.
    One total could not
    answer the width question the bench exists for:
    seating one more producer
    resends the SAME prompt and adds one more answer,
    while a judge pays a
    prompt carrying every candidate,
    so those three costs scale with width
    differently and a single number prices them the same way.
    The server's own
    total is kept beside both halves rather than derived from them,
    so the
    headline cost stays what the provider stated;
    the derived sum is the
    fallback for servers that report no total at all.
    Whether any provider here
    ever states a total that differs from its two halves is unmeasured,
    and the
    first bench run under the split answers it.
21. AN ASSEMBLY THAT CHANGES NO BYTE NOW SHIPS NOTHING,
    whatever its slices
    decided.
    Two adjacent slices whose replacements each differ from their own
    incumbent can reassemble to the archive text:
    moving a paragraph across the
    join does it,
    and subdivision groups small paragraphs into slices where that
    join exists,
    so this is reachable rather than hypothetical.
    The lane used to
    return a non-empty shipped set beside a byte-identical document,
    while both
    contracts said those indices name slices the document CARRIES a change for.
    The guard now withdraws that whole set under its own reason and returns no
    survivors.
    CANONICALIZATION RATHER THAN REFUSAL,
    because nobody did anything
    wrong:
    each lane still holds every wording it decided,
    and only the
    document-level claim changes,
    to the true one.
    It also bought something
    back,
    which is why it is worth the code:
    with the guard guaranteeing it,
    the
    check that a document equal to the archive names no changed slice became
    enforceable again,
    and it had been dropped as unenforceable a day earlier.
    THIS BROADENS WHAT `withdrawn` MEANS.
    It is no longer only an integrity veto:
    a withdrawal now says the document does not carry that slice's change,
    for
    any of three reasons,
    and only the findings say which.
    A reader counting
    footnote damage from `withdrawnSliceCount` alone would now over-count.
22. A REPAIRED SLICE IS ONE WHOSE TEXT MOVED,
    not one whose patch won.
    The
    repair lane derived `changed` from which candidate selection chose,
    which is
    a different question:
    the patch gate refuses an operation that rewrites its
    region to itself,
    but two operations in adjacent envelopes can each change
    their own region and concatenate back to the archive text,
    the same shape as
    decision 21 one level down.
    Such a patch could win and write no byte,
    and
    the per-slice assertion would then throw on a run nobody did anything wrong
    in.
    `changed` now reads the text;
    `accuracyPatchSelected` keeps the
    selection fact,
    which stays true in that case and is a different thing.
    The
    translate lane already worked this way,
    so the invariant now holds by
    construction on both.
23. A CACHE KEY NOW NAMES THE QUESTION AND NOT THE PLACE IT WAS ASKED.
    You
    authorized the invalidation;
    the shape is mine and is open to veto.
    A key is
    the run's shape,
    both slice texts and the governance flag,
    and the slice
    index was in it for no reason a reader could defend:
    the same source and the
    same incumbent produce the same answer wherever they sit,
    and one-sided
    slicing moves indices around by design,
    so keeping it would have made every
    slice after an inserted section a miss.
    A resumed record is RE-STAMPED with
    the index it is resumed under,
    rather than trusted to carry its own,
    since
    the stored one is now meaningless.
    What made this safe to do rather than
    merely right was that nothing had settled under either lane's current
    version,
    measured before the change:
    the translate cache was empty and the
    repair one held no record under its current version.
    THE ONE THING THAT DEPENDED ON POSITION was removed rather than kept:
    a
    refusal sentence naming `slice 3` was being stored inside the record,
    so
    after the key change it would have been resumed at another position and
    reported the wrong slice number.
    The stored record now carries the finding
    without the sentence,
    and the sentence is composed at the document level
    where the positions are known.
24. A CHUNK IS EITHER CONTENT OR A PLACE,
    and the discriminant is a FIELD rather
    than emptiness.
    This is the first landing of `#100` and it changes nothing
    that runs today,
    because nothing produces a place yet.
    Reasoning:
    a
    `nodes.length === 0` test would work today and would silently promote any
    fabricated empty chunk to an insertion tomorrow,
    while a field says what a
    value is.
    An external review argued me out of my first shape,
    an optional
    `placement` marker on one broad type,
    which buys the word without the
    protection:
    an insertion would stay assignable everywhere content is
    expected.
    ONE RULE REFUSES EVERY BAD PLACEMENT,
    checked at assembly:
    each target span
    starts at or after the previous one ends,
    walked in slice order.
    Overlaps,
    an
    anchor sitting inside a span,
    two spans at one offset,
    backwards ranges and
    stale text all fall out of it,
    and the review agreed there is no
    offset-only counterexample once the per-slice shape checks pass.
    It also
    found something I had not:
    array order is only slice order if the indices ARE
    positions,
    which assembly never asserted,
    so two anchors at one boundary with
    unique but shuffled indices would have been written in reverse.
    WHAT IT LEFT OPEN,
    and I did not decide:
    whether a MISSING replacement for an
    anchor should be refused the way a BLANK one now is.
    A blank one is refused
    because an anchor is where a rendering belongs,
    so blank text there leaves
    the passage missing while the run reports it delivered.
    A missing one cannot
    be answered until the absent-incumbent work says whether assembly may ever
    withdraw an anchor's replacement,
    since withdrawing one restores nothing.
25. THE SEPARATOR BETWEEN TWO BLOCKS IS ASSEMBLY'S,
    NOT THE PROMPT'S.
    Every
    replacement until now went into a span that already sat between the right
    blank lines,
    so writing model text verbatim preserved them;
    an anchor has no
    span,
    and verbatim text written before a heading produces `...afternoon.##
    Habits`,
    which still parses as Markdown and says something else.
    A prompt
    asking for correct leading and trailing blank lines is a hope that fails
    silently,
    and it cannot be right anyway,
    since several fragments landing at
    one boundary would each carry their own and put two between every pair.
    THE RULES,
    all add-only:
    strip only outer blank-line material from a
    fragment and keep its indentation,
    because a rendering that begins with
    spaces is inside a list or a quote;
    join same-boundary fragments with one
    blank line;
    preserve existing whitespace byte for byte and only top it up;
    write the document's OWN line ending,
    which a Windows translation needs and a
    diff would otherwise report as changes to lines nobody touched;
    and treat the
    end of a file as termination rather than as separation from nothing.
    Four
    existing test expectations changed,
    each of which had pinned the verbatim
    write this replaces.
26. A PASSAGE THE RUN COULD NOT TRANSLATE COSTS ITS OWN SLICE,
    NOT THE ENTRY.
    This is the one place I departed from a review's recorded shape,
    so it is
    the decision in tonight's work most worth your veto.
    THE PROBLEM IT SOLVES either way:
    every fallback in the translate stage
    ships the wording already in the archive,
    which is right for a slice that
    HAS one,
    since leaving a passage as it stands is the state the run began in.
    For a slice with none,
    the same fallback shipped the empty string and
    reported a settled slice,
    so the run read as having delivered a translation
    it never produced.
    THE REVIEW'S SHAPE was for the lane to throw and leave the whole entry
    unsettled,
    its cached slices reusable by the next attempt.
    WHAT I BUILT INSTEAD:
    the stage raises a typed refusal,
    the driver catches
    it per slice,
    and the document settles with that passage still missing and
    named in a new `unfilledChunkIndices` field,
    with the findings saying which
    translators were heard and what the judges counted.
    Nothing is cached for
    such a slice,
    so the next run asks again.
    REASONING:
    the document keeps the gap the archive already had,
    which states
    nothing false;
    a decline depends on which judges answered,
    so it varies
    between runs,
    and throwing the entry away discards every other slice's work
    over that;
    and the missing passage is now nameable,
    which is what stops it
    being read as a slice the judges kept.
    WHAT WOULD CHANGE IF YOU PREFER THE REVIEW'S SHAPE:
    one branch in the
    driver,
    plus the entry-level catch that Question 5's wiring needs anyway.
27. A TRANSLATOR REPLY THAT SAYS NOTHING IS NO LONGER A REPLY.
    `{"translation":
    ""}` satisfies the structured-output schema,
    so it arrived as a heard voice,
    was dropped further down as an unusable candidate,
    and the model that sent
    it counted as having answered and was never asked again.
    The wire guard now
    refuses it,
    which makes it a lost voice:
    the roster re-asks that model in
    the next round,
    and if it stays blank the loss is named like any other.
    Every source slice says something,
    so no legitimate reply is blank.
    WHAT THIS CHANGES IN A COUNT you may already be reading:
    a blank reply used
    to show up as `translate-blank (model)` beside a full `heardTranslators`
    tally.
    It now shows up as `stage-voice-lost (translate model)` with one
    fewer heard translator,
    which is the same event described honestly.
    The
    blank filter downstream stays as a backstop and should now never fire.
28. WHAT COUNTS AS PROOF THAT A PASSAGE WAS NEVER TRANSLATED?
    THE PIPELINE IS ABOUT TO ACT ON AN ANSWER IT DOES NOT HAVE.
    Both halves of
    the one-sided slicing work insert text where an aligner reports no
    counterpart:
    at paragraph scale (landing four) and at section scale
    (landing five).
    I measured what those reports are actually made of,
    and
    they are not evidence of a missing translation.
    AT PARAGRAPH SCALE:
    the block aligner can pair one with one,
    skip a source
    block,
    or skip a target block,
    and nothing else.
    It cannot say two source
    paragraphs were rendered as one,
    so a merged pair reports the second as
    unpaired,
    identically to an omission.
    Corpus:
    2290 paired steps,
    95
    unpaired source blocks,
    in 23 of 92 entries,
    sixty of them inside two
    entries.
    A hand sample of twelve found the strongest signals were merges
    (one entry renders four consecutive lines as a single English block) and
    the weakest were MISPAIRINGS (a footnote definition paired with the wrong
    footnote;
    a narration line paired with a translation three blocks away).
    AT SECTION SCALE the numbers are smaller and I READ THEM WRONG THE FIRST
    TIME,
    which is worth your attention because the correction is what the
    prototype below was able to make.
    Eighty-five of the 92 entries never reach
    the section matcher at all,
    because equal heading counts short circuit it
    (`#98`).
    Of the seven that do,
    two produce unpaired source sections,
    eleven
    in total.
    WHAT I FIRST CONCLUDED,
    and recorded here in an earlier draft:
    eight of the
    eleven were false,
    because one entry's sections carry English headings that
    plainly correspond,
    经历 with Experience,
    遇见 with Meeting,
    and so on
    through eight of them.
    WHAT IS ACTUALLY TRUE:
    that entry's English document is 1,218 characters
    against 7,365 Chinese,
    and every section of it except the last is a HEADING
    WITH NO BODY.
    The headings correspond and the translations do not exist.
    I
    had inferred body coverage from heading correspondence,
    which is the same
    reasoning error I have been refusing elsewhere,
    and the six-model coverage
    probe reported all eight as uncarried before I checked.
    SO THE MATCHER'S REFUSALS WERE RIGHT IN OUTCOME on that entry,
    for a reason
    it could not state,
    and the count of genuine candidates at section scale is
    higher than the earlier draft said rather than lower.
    WHY IT IS YOURS TO ANSWER:
    the paths differ in expense and in how long the
    lane stays unable to fill a real gap.
    None of them is wrong.
    A.
    ASK A MODEL WHETHER THE TARGET DOCUMENT CARRIES THIS PASSAGE AT ALL,
    scoped to the whole translation rather than to the neighbours the aligner
    picked.
    Pros:
    it is the only option immune to the pairing quality this
    measurement just impeached,
    since it never consults the pairing;
    about a
    hundred and six questions corpus-wide is a small bill;
    it caches like every
    other slice;
    and it answers both scales with one mechanism.
    Cons:
    a new
    stage with its own prompt,
    schema,
    roster and failure modes,
    and a long
    document makes a long question.
    B.
    FIX ALIGNMENT FIRST,
    then revisit insertion.
    Pros:
    the mispairings and
    the ambiguous refusals are defects rather than limits,
    `#74` and `#98`
    already own them,
    and every stage reads better pairs,
    not just this one.
    Cons:
    the largest item here,
    it does not by itself separate a merge from an
    omission,
    and both landings wait on it.
    C.
    INSERT ONLY WHERE THE EVIDENCE IS ALREADY OVERWHELMING,
    meaning a source
    section whose absence is corroborated by more than the matcher's refusal:
    no target heading resembles it,
    and the target document is shorter than the
    source by about that section's size.
    Pros:
    no new stage,
    no model call,
    and
    it would have admitted the three true ones and refused the eight false ones
    in this corpus.
    Cons:
    a threshold tuned on two entries is a threshold tuned
    on nothing,
    and it silently does nothing on the 85 entries that skip the
    matcher.
    D.
    DO NOT INSERT AT ALL for now.
    Keep both landings parked,
    ship the
    translate lane over slices that HAVE incumbent text,
    and revisit when
    alignment is trustworthy.
    Pros:
    no corpus risk whatever;
    every invariant
    built for insertion stays and stays tested.
    Cons:
    the archive's genuinely
    missing sections stay missing,
    which is the thing the second lane exists
    for.
    MY RANKING:
    A > C > B > D.
    A over C because A answers the question for both scales with one mechanism
    and does not depend on the aligner being right,
    where C still rests on
    refusals whose reasons do not distinguish the cases.
    C over B because C can ship a genuinely missing section now and B is the
    largest item here,
    and because the correction above raised how many section
    candidates are genuine rather than lowering it.
    B over D because B is work that pays off in every stage,
    where D is a
    holding position.
    WHAT I DID RATHER THAN WAIT:
    built A and measured it,
    since it was the
    stated default and the prototype touches no corpus state.
    It is four files
    and a probe task,
    nothing calls them,
    and no slicing or artifact reads them.
    WHAT IT MEASURED,
    thirty-three candidates,
    six voices each,
    about a hundred
    and ninety calls and under ten minutes:
    ELEVEN SECTIONS:
    ten absent,
    one carried.
    Every verdict agrees with what the
    documents contain,
    and NINE of my eleven hand labels were wrong.
    It found
    that one entry's English side is headings without bodies,
    and that a source
    heading meaning "blank" is rendered by an English heading spelled `__`,
    which the matcher cannot score because it holds no letters.
    TWENTY-TWO BLOCKS:
    eighteen carried,
    most unanimously,
    one absent,
    and three
    split with NO votes for absence at all.
    So at paragraph scale at most one of
    twenty-two passages the aligner refuses is one nobody translated,
    and
    landing four would insert on all twenty-two.
    WHAT THAT IS NOT:
    an accuracy number.
    The candidates come from five entries,
    most blocks carry no hand label,
    and I made the labels after seeing the
    aligner.
    What the class balance cannot explain is that pooled across both
    scales no constant answer beats nineteen of thirty-three,
    while the probe
    agrees with every label I have in both directions.
    WHAT IT COSTS:
    about six calls per candidate,
    roughly a hundred and six
    candidates corpus-wide,
    so one pass over every unpaired passage in the
    corpus is around six hundred and forty calls and a quarter of an hour.
    ONE DESIGN DEFECT IT EXPOSED,
    which matters whichever option you choose:
    a
    source section whose translation is a HEADING WITH NO BODY must have its
    body inserted UNDER that heading,
    not inserted as a whole new section,
    or
    the document gains a duplicate heading.
    Landing five as designed would have
    made that mistake on eight sections of one entry.
    RERUN AFTER THE REVIEW CHANGES LANDED,
    because the numbers above are what
    this question rests on and three things moved underneath them:
    the sheet was
    rewritten,
    the threshold became a majority of the roster rather than of the
    voices heard,
    and the quote locator learned to see through a soft line wrap.
    Nine of the eleven section verdicts are identical.
    XingZ60's section 12 reads
    `partly-carried` where it read `carried`,
    which is not the models changing
    their minds:
    the tallies are zero full and five partial,
    and the old verdict
    counted any claim of coverage as one vote,
    so those same five votes used to
    print as carried.
    XIEPT2's section 7 reads `split` where it read `absent`,
    and that one I CANNOT attribute:
    the votes themselves moved,
    and votes of
    that shape read as split under either threshold,
    so the cause is the new
    sheet or ordinary variance between two samples of six stochastic models,
    and
    one run cannot separate those.
    WHAT DID NOT MOVE IS THE ANSWER TO YOUR QUESTION.
    No candidate in either run
    reports full coverage.
    Nine of eleven are absent by a majority of the whole
    roster with every model heard,
    and both verdicts that moved moved AWAY from
    coverage rather than toward it.
    The sections I had labelled as plainly
    translated are still reported as carrying nothing.
    THE BLOCK SET WAS RERUN TOO,
    on `mikaela_khara`,
    the entry that produced the
    three splits.
    All three are now carried,
    and of the quotes still unanchored
    none is a soft line wrap where ten of eleven were before,
    so the refusals
    that remain are English a model composed rather than copied,
    plus one
    single-word quote refused for occurring twice.
    Across both runs of that entry,
    ninety-six answers from six models,
    NOT ONE VOTE FOR ABSENCE was cast on
    sixteen passages the block aligner refuses to pair.
    If anything,
    the paragraph
    scale reads worse for landing four than it did:
    at most one of twenty-two,
    and
    plausibly none.
    RUN A THIRD TIME under unchanged code,
    because the one sentence above that
    said "this might be variance" deserved testing rather than repeating.
    All
    eleven verdicts came back IDENTICAL to the second run,
    section 7's split
    included.
    So that split reproduces,
    and the eleven verdicts this question
    rests on are now two independent samples in agreement rather than one sample.
