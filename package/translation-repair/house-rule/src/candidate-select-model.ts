import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Candidate selection model
// Types the selection stage shares with its callers. They live apart from
// `candidate-select.ts` so the stage function keeps its line budget, and because
// the producer union is the piece other stages have to reason about: it is what
// stops a stitched composite from being reported as one model's work.

/**
 Weight one disinterested judge's ballot carries.
 */
export const FULL_VOTE_WEIGHT = 1;

/**
 Weight a judge's ballot carries when it names a candidate that judge helped
 write.

 Self-judging is ALWAYS allowed, by user decision on 2026-08-15: these models
 have different blind spots, and a producer's reading of its own text is a
 weaker instrument than a disinterested reading, not a worthless one. Barring
 producers threw that reading away and, on a six-model roster with three
 producers, halved the panel to do it.

 What the discount corrects is a TILT rather than a declared preference. The
 judge sheet is anonymized and says so, so a producer does not know which
 candidate is its own and cannot set out to back it; it is only somewhat more
 likely to land there. Sizing this as though every self-vote were deliberate
 would overcorrect a bias nobody has measured on this roster yet, which is
 what the judge-quality bench is for.

 THE HALF DECIDES WHICH CANDIDATE LEADS AND NOTHING ELSE. There is no weight
 floor a winner must draw: by the owner's rulings of 2026-10-25, "floor of 3 -
 don't set a floor." and "Every weight floor", a leader that two or more
 answering judges named wins, and one judge alone never does. A self-voting
 judge is ONE ANSWERING JUDGE; its half is a share of the weight, not half a
 judge in the count of ballots. Until 2026-10-25 a winner needed a summed
 weight of two as well, which made one author's own text (a half) or three
 authors' (three halves) need a judge with no stake in it, and a leader
 drawing one full vote and one self-vote (1.5) decline on a bench at quorum.

 The collapse case follows: models returning byte-identical text merge into
 one candidate carrying every author, and two or more of them naming it ship
 it between themselves. Kept deliberately, because independent models agreeing
 to the byte is itself the corroboration an outside judge would supply, and it
 is the rule the owner gave. `runCollapsedSelection` in
 `candidate-select.unit.test.ts` pins that the half still counts for which
 candidate leads when a rival is named.

 STATED PRECISELY, because a looser version of this sentence was written into
 four documents and was wrong: the discount attaches to a JUDGE AND CANDIDATE
 PAIR, not to a judge. A producer voting for someone else's candidate carries
 full weight, which is the whole point of seating it.

 Exactly representable in binary floating point, which the tie comparison
 depends on: two halves sum to one with no residue, so a tie between a
 self-supported candidate and an equally supported rival still reads as a tie.
 */
export const SELF_VOTE_WEIGHT: number = 1 / 2;

/**
 Ballots naming the winner a selection needs, whatever their weight: the
 floor of two answering judges, and the only floor a selection round has.

 One vote deciding is one model deciding, which the ensemble exists to prevent:
 a round where every other judge abstained or was lost would otherwise hand
 the whole chunk to whichever single judge answered (the owner's standing
 rule, 2026-09-09). Two is the smallest number that makes a selection an
 agreement rather than an opinion. A self-vote counts as a ballot, since the
 judge answered; its half weight only ranks the candidates. Until 2026-10-25
 a winner also needed a summed weight of two, which the owner removed
 ("Every weight floor"), so two ballots decide when they weigh 1.5.
 */
export const MIN_SELECTION_BALLOTS = 2;

/**
 Who produced a candidate.

 A composite is text NO model wrote or read as a whole, so it names its
 contributors instead of borrowing one model's identity. Reporting stitched
 text under a single model's name would misstate who is answerable for it, and
 every contributor still has a stake in it.

 The composite variant also carries the case where several models
 independently produced the SAME text and their candidates were collapsed:
 more than one model has a stake either way, and every one of them holds it.
 What the variant means precisely is "more than one model has a stake in this
 text", with stitching the usual reason.

 The incumbent variant is text that was ALREADY THERE: a human translation the
 translate lane offers as one candidate among the fresh ones. Nobody on the
 roster wrote it, so no judge is discounted for it, and the alternative of
 giving it a stand-in model id would both discount a model that never saw it
 and count a producer the roster guard is arithmetic over.

 @example
 ```ts
 const producer: CandidateProducer = { kind: 'model', modelId, };
 ```
 */
export type CandidateProducer =
  | {
    readonly kind: 'model';

    /**
     Model that wrote this candidate by itself.
     */
    readonly modelId: RosterModelId;
  }
    | {
      readonly kind: 'composite';

    /**
     Models whose work the composite carries, in assembly order.
     */
      readonly contributors: readonly RosterModelId[];
    }
    | {
      readonly kind: 'incumbent';

    /**
     Models that independently produced text identical to the incumbent's and
     were collapsed into it, discounted when they judge it although it is not
     their work. Empty in the ordinary case, where the incumbent stands alone.
     */
      readonly matched: readonly RosterModelId[];
    }
    | {
      readonly kind: 'lane';

    /**
     Which lane's text this is: what the repair lane or the translate lane
     would ship, offered to a consolidation slate when the contest endorsed
     neither or the standing failed the deterministic rule (class forty,
     2026-09-17). Nobody on the roster owns it as a whole, exactly as with
     the incumbent.
     */
      readonly lane: 'repair' | 'translate';

    /**
     Models whose proposals reproduced this lane's text and were collapsed
     into it, discounted when they judge it. Empty in the ordinary case.
     */
      readonly matched: readonly RosterModelId[];
    };

/**
 Every model with a stake in a candidate, so callers can weigh or exclude all
 of them without knowing how the candidate was assembled.

 @param producer - candidate's provenance

 @returns Model ids whose ballots for this candidate are self-votes

 @example
 ```ts
 const barred = producerModelIds({ kind: 'composite', contributors, },);
 ```
 */
export function producerModelIds(producer: CandidateProducer,): readonly RosterModelId[] {
  if (producer.kind === 'model')
    return [producer.modelId,];
  if ((producer.kind === 'incumbent') || (producer.kind === 'lane'))
    return producer.matched;
  return producer.contributors;
}

/**
 Renders a producer for logs and scorecard findings.

 @param producer - candidate's provenance

 @returns Model id, or the contributor list for a composite

 @example
 ```ts
 const label = describeProducer(candidate.producer,);
 ```
 */
export function describeProducer(producer: CandidateProducer,): string {
  if (producer.kind === 'model')
    return producer.modelId;
  if (producer.kind === 'incumbent') {
    /**
     Models that independently reproduced this text.
     */
    const { matched, } = producer;
    if (matched.length === 0)
      return 'incumbent';
    return `incumbent(matched by ${matched.join(' + ',)})`;
  }
  if (producer.kind === 'lane') {
    /**
     Models that independently reproduced this lane's text.
     */
    const { matched, } = producer;
    if (matched.length === 0)
      return `lane(${producer.lane})`;
    return `lane(${producer.lane}, matched by ${matched.join(' + ',)})`;
  }
  return `composite(${producer.contributors
    .join(' + ',)})`;
}

/**
 Combines the stakes of two candidates whose text turned out identical, so
 collapsing a duplicate never drops a model from the discounted set.

 Without this, an editor could judge its own words at full weight: if the
 composite carries one model's operation while another model's whole-chunk text
 happens to match it exactly, keeping either candidate alone would leave the
 other voting on text it wrote as though it were a stranger's.

 @param left - producer of the candidate being kept

 @param right - producer of the duplicate being collapsed into it

 @returns Incumbent carrying every stake when either side is the incumbent,
 single-model producer when both name the same one model, else a composite
 over the union in first-seen order

 @example
 ```ts
 const producer = mergeProducers({ left: kept.producer, right: duplicate.producer, },);
 ```
 */
export function mergeProducers(
  {
    left,
    right,
  }: {
    readonly left: CandidateProducer;
    readonly right: CandidateProducer;
  },
): CandidateProducer {
  /**
   Every model with a stake, in first-seen order without repeats.
   */
  const united = [
    ...new Set([
      ...producerModelIds(left,),
      ...producerModelIds(right,),
    ],),
  ];

  // Incumbency survives the collapse. A model reproducing the incumbent's text
  // exactly gains a stake in it, so its own ballot for it is discounted, but
  // the text is still the one that was already there, and reporting it as that
  // model's work would
  // turn "the human translation was kept" into "a model rewrote it identically"
  // on exactly the slices where the two are indistinguishable.
  if ((left.kind === 'incumbent') || (right.kind === 'incumbent'))
    return {
      kind: 'incumbent',
      matched: united,
    };

  // A LANE'S TEXT SURVIVES THE SAME WAY, for the same reason: a model
  // reproducing what the repair lane would ship gains a stake in it, and the
  // text is still the lane's rendering, not that model's fresh proposal.
  // Where both sides are lanes the first-seen lane keeps its name, since the
  // two lanes shipping one text is one text on the slate.
  if (left.kind === 'lane')
    return {
      kind: 'lane',
      lane: left.lane,
      matched: united,
    };
  if (right.kind === 'lane')
    return {
      kind: 'lane',
      lane: right.lane,
      matched: united,
    };

  /**
   Sole stakeholder, when both sides named the same one model.
   */
  const [only,] = united;
  if ((united.length === 1) && (only !== undefined))
    return {
      kind: 'model',
      modelId: only,
    };
  return {
    kind: 'composite',
    contributors: united,
  };
}

/**
 One proposed candidate with the provenance that bars its authors from
 judging it.

 @example
 ```ts
 const candidate: Candidate<string> = {
   producer: { kind: 'model', modelId, },
   value: patched,
   rendered: patched,
 };
 ```
 */
export type Candidate<ValueT,> = {
  /**
   Who wrote this candidate; its ballot for this candidate counts less.
   */
  readonly producer: CandidateProducer;

  /**
   Value handed back when this candidate wins.
   */
  readonly value: ValueT;

  /**
   Text judges compare; may be a rendering of `value` rather than `value`.
   */
  readonly rendered: string;
};

/**
 What a selection round counted, recorded whether or not it chose anything.

 Kept on BOTH outcomes because the decline rate is the measurement that says
 whether an ensemble does any work at all: a stage that always declines buys
 several times the tokens for a result identical to one model's.

 @example
 ```ts
 const tally: SelectionTally = { judgesAvailable: 4, ballots: 3, abstentions: 1, };
 ```
 */
export type SelectionTally = {
  /**
   Judges the round seated.
   */
  readonly judgesAvailable: number;

  /**
   Judges whose ballot arrived and validated.
   */
  readonly ballots: number;

  /**
   Ballots naming no usable candidate, whether by declining outright or by
   naming an index no candidate occupies.
   */
  readonly abstentions: number;

  /**
   Ballots naming a candidate their own judge helped write.

   Counted on every round because seating producers is a deliberate trade of
   independence for coverage, and a trade nobody measures is an assumption. A
   stage where this approaches the ballot count is one where the discount is
   carrying the whole result.
   */
  readonly selfVotes: number;
};

/**
 One judge's ballot as cast.

 Kept because the tally answers how many judges preferred something and never
 why. Ballot reasons reached a log line and nothing durable, so a run whose log
 was lost, which happened on 2026-08-13 when a pass wrote twenty minutes of
 output into a pipe whose reader had exited, left no account of any decision it
 made. That is tolerable while selection only ranks repairs of text the
 pipeline keeps either way, and not once selection decides whether a human
 translation is replaced.

 @example
 ```ts
 const ballot: SelectionBallot = { modelId, best: 2, reason: 'renders the omitted clause', };
 ```
 */
export type SelectionBallot = {
  /**
   Judge that cast it.
   */
  readonly modelId: RosterModelId;

  /**
   One-based candidate index, or `CANDIDATE_NONE` when this judge named no
   candidate. An index past the end of the set is recorded as cast rather than
   corrected, since a judge naming a candidate that does not exist is itself
   the finding.
   */
  readonly best: number;

  /**
   Judge's stated reason, verbatim.
   */
  readonly reason: string;

  /**
   Weight this ballot carried: {@link SELF_VOTE_WEIGHT} when the judge named
   a candidate it helped write, {@link FULL_VOTE_WEIGHT} otherwise, and zero
   for an abstention.
   */
  readonly weight: number;

  /**
   Whether this judge named text it produced.

   Recorded rather than derived from the weight, because deriving it would
   make every self-preference measurement depend on the two weights staying
   different from each other. Those are tuning knobs; this is a fact about
   the ballot.
   */
  readonly selfVote: boolean;
};

/**
 What one candidate drew, so a decline is as auditable as a win.

 A round tally says the leader was named by one judge alone, or that judges
 tied, without saying by how much or against what, and a leader short of two
 ballots is the commonest outcome of a judged round. This is the per-candidate
 half of that account.

 @example
 ```ts
 const drawn: CandidateWeight = { index: 2, ballots: 3, fullVotes: 2, selfVotes: 1, weight: 2.5, };
 ```
 */
export type CandidateWeight = {
  /**
   One-based candidate index, as the judges saw it.
   */
  readonly index: number;

  /**
   Ballots naming this candidate, self-votes included.
   */
  readonly ballots: number;

  /**
   Ballots from judges with no stake in it.
   */
  readonly fullVotes: number;

  /**
   Ballots from judges that helped write it.
   */
  readonly selfVotes: number;

  /**
   Summed weight, which ranks the candidates; no minimum is compared against it.
   */
  readonly weight: number;
};

/**
 Which kind of failure a decline was.

 The two are not interchangeable, and treating them alike is how a pipeline
 either overrides its judges or throws away work over a coin flip:

 -   `indecision` means judges answered but did not converge, by tying or by
     leaving the leader named by one judge alone. Every candidate may be
     perfectly good; nobody said otherwise. This is a failure to rank.
 -   `rejection` means judges affirmatively answered that NO candidate is
     acceptable, or that no judge could be seated at all. This
     is a substantive negative verdict, and shipping over it would be
     overruling the judges rather than routing around their silence.

 @example
 ```ts
 const disposition: SelectionDisposition = 'indecision';
 ```
 */
export type SelectionDisposition =
  | 'indecision'
    | 'rejection';

/**
 What a selection round decided.

 @example
 ```ts
 const outcome: SelectionOutcome<string> = { kind: 'declined', reason: 'judges tied', tally, };
 ```
 */
export type SelectionOutcome<ValueT,> =
  | {
    readonly kind: 'selected';

    /**
     Winning candidate's value.
     */
    readonly value: ValueT;

    /**
     Who produced the winner.
     */
    readonly producer: CandidateProducer;

    /**
     Summed ballot weight the winner drew, which is a COUNT only when no
     producer voted for its own work; see {@link SELF_VOTE_WEIGHT}.
     */
    readonly voteWeight: number;

    /**
     One-based index of the winner in the slate the judges were shown.

     Without it a ballot saying `best: 2` cannot be joined to any text later:
     the caller may rotate the slate before showing it, so the position is the
     only thing a ballot and a candidate have in common, and nothing recorded
     which position won.
     */
    readonly selectedIndex: number;

    /**
     What the round counted.
     */
    readonly tally: SelectionTally;

    /**
     Degradation findings from the judge fan-out, for the caller to carry
     into the artifact.

     The tally already counts how many judges answered, and that is not the
     same thing: it says how many voices were lost without saying WHICH model
     went silent, and the identity is what every voice-loss diagnosis has
     turned on. This channel existed nowhere on the selection path until
     2026-08-13, so the fan-out's findings were built and discarded.
     */
    readonly findings: readonly string[];

    /**
     Every ballot cast, in the order judges answered.
     */
    readonly ballots: readonly SelectionBallot[];

    /**
     What every candidate drew, whether or not it won.
     */
    readonly perCandidate: readonly CandidateWeight[];
  }
    | {
      readonly kind: 'declined';

    /**
     Why nothing was selected, in scorecard-stable wording.
     */
      readonly reason: string;

    /**
     Which kind of failure this was, so callers can answer the two very
     different questions separately.
     */
      readonly disposition: SelectionDisposition;

    /**
     What the round counted.
     */
      readonly tally: SelectionTally;

    /**
     {@inheritDoc SelectionOutcome.findings}
     */
      readonly findings: readonly string[];

    /**
     {@inheritDoc SelectionOutcome.ballots}
     */
      readonly ballots: readonly SelectionBallot[];

    /**
     What every candidate drew, whether or not it won.
     */
      readonly perCandidate: readonly CandidateWeight[];
    };

//endregion Candidate selection model
