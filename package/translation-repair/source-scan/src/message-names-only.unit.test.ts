/**
 Guards the claim `messageNamesOnly` makes, by reading the source that makes
 it.

 WHAT THE MARKER CLAIMS. `refusalText` repeats the message of any error whose
 class declares `messageNamesOnly`, and drops the message of every other
 class. So the marker says: every part of this message is a sentence we wrote,
 a number this process computed, a name from our own vocabulary, or a value
 the operator handed in. Never a corpus passage, a run file's contents, a
 model's answer, or a provider's response body.

 THE RULE THAT DECIDES WHO MAY CARRY IT. A class may declare the marker when
 the CLASS writes the sentence. A class whose constructor forwards a `message`
 parameter to `super` may not, however careful its throw sites are, because
 the claim would then be about thirty call sites rather than about one class,
 and nothing here could check it. `StatedRefusalError` is the deliberate
 exception, and carries its own note saying why.

 WHY A SOURCE SCAN RATHER THAN A BEHAVIOURAL TEST. Constructing each class and
 reading its message would pin today's wording, which is not the property
 worth guarding: rewording a sentence is fine, and interpolating a new value
 into it is the thing that needs a second look. This reads exactly that, and
 fails when a marked class's message gains a part the inventory does not name.

 WHAT IT CANNOT CATCH. A field the inventory already names, such as `detail`,
 can be handed different text by a new throw site. The inventory records what
 each field holds today, and a change of that kind is caught by reading, not
 by this file.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import {
  expectFindingsAsListed,
  expectNoFindings,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Marked message inventory

/**
 Key naming the byte offset a JSON read reports, or nothing when it has none.

 SPLIT ACROSS A CONCATENATION so no plain string holds a whole `${}`, which
 `no-template-curly-in-string` reads as a template literal written by mistake.
 The halves join to exactly the expression the scan finds in the source.
 */
const BYTE_OFFSET_OR_NOTHING = "(at === OFFSET_UNSTATED) ? '' : ` at byte $"
  + '{String(at,)}`';
/**
 Every class permitted to declare the marker, with nothing else allowed to.

 ADDING A NAME HERE IS THE DECISION. The list exists so that marking a class
 cannot happen quietly inside an unrelated change: this test fails until the
 name is written down, which is the moment to ask what its message carries.
 */
const MARKED_CLASSES: readonly string[] = [
  'ArchiveBlockAlignmentMissingError',
  'ArchiveBlockContextMissingError',
  'ArchiveOriginalCompletenessError',
  'ArtifactComparisonError',
  'ArtifactParseError',
  'AssemblyContractError',
  'BedrockCreditOverrideError',
  'BedrockLedgerShapeError',
  'BlockOutsideArchiveError',
  'BedrockModelNotServedError',
  'BenchReportError',
  'BlankSelectionError',
  'CacheAccountLogError',
  'CacheAccountReadError',
  'CacheFileUnreadableError',
  'CallTimeoutError',
  'CensusBaselineError',
  'CheckerIndependenceError',
  'CheckerQuorumError',
  'CitedReferenceFetchError',
  'ClaudeModelNotServedError',
  'ClaudePrefillError',
  'ClaudeSchemaKeywordError',
  'CollapsedHeadingError',
  'CompositeNotOnSlateError',
  'ConsistencyInvariantError',
  'ConsolidationLedgerGapError',
  'ContributorCompletenessError',
  'CorpusReadError',
  'CoverageFileError',
  'CoverageTallyError',
  'CreditsShapeError',
  'DecisionReplyShapeError',
  'DecisionsCardMissingError',
  'DeclinedEntriesUnreadableError',
  'DeliveryCoherenceError',
  'DeliveryInvariantError',
  'DeepSeekBalanceShapeError',
  'DeepSeekModelNotServedError',
  'DrawReconcileError',
  'DroppedDestinationError',
  'EmptyConversationError',
  'EmptyPoolError',
  'EntryInfoParseError',
  'EnvelopeOverlapError',
  'EveryProviderDryError',
  'FamilyEntryConflictError',
  'FidelityReferenceError',
  'FloorGroundDisagreementError',
  'FootnoteOverflowError',
  'FootnoteRewriteError',
  'FragmentLandingError',
  'FrontMatterCompletenessError',
  'FrontMatterParseError',
  'GenerationDriftError',
  'GradedSheetExistsError',
  'HardCapOverrideError',
  'HoldBoundError',
  'InStreamProviderError',
  'InStreamRefusalError',
  'IssueEvidenceConflictError',
  'LaneComparisonError',
  'LaneSliceCoverageError',
  'LedgerShapeError',
  'LegacyPipelineError',
  'LintBinaryMissingError',
  'LintBinaryRunError',
  'LintBinaryTextError',
  'LintBinaryVersionError',
  'LocalProgramFailedError',
  'MalformedCompletionError',
  'MalformedImageUriError',
  'MarkdownParseError',
  'MdxParseError',
  'MislabelledArtifactError',
  'MixedGenerationError',
  'ModelNotServedError',
  'NaturalnessCompletenessError',
  'NaturalnessQuorumError',
  'NaturalnessRepairInterruptedError',
  'NestingBoundError',
  'NeuralWattAllowanceError',
  'NeuralWattAllowanceOverrideError',
  'NeuralWattDayStartOverrideError',
  'NeuralWattLedgerShapeError',
  'NeuralWattModelNotServedError',
  'NeuralWattQuotaShapeError',
  'NeuralWattReservationSettledError',
  'NeuralWattUsageShapeError',
  'NoProviderForModelError',
  'NothingInFlightError',
  'OffRosterModelError',
  'OpenRouterCreditsShapeError',
  'OpenRouterModelNotServedError',
  'OverlapRefusedError',
  'OverlappingEditsError',
  'PairingEvidenceError',
  'PipelineDigestError',
  'PlacementLayoutError',
  'ProducerRosterError',
  'PromptPayloadStoreError',
  'PublishedPageDisagreesError',
  'QuotaShapeError',
  'ReferenceLineHeadError',
  'RepairUnheardError',
  'RequiredProviderError',
  'RoundsNotRecordedError',
  'RunConfigError',
  'RunJsonUnreadableError',
  'RunsDirectoryBusyError',
  'SchemaGenerationError',
  'SeedApplicationError',
  'SheetBindingError',
  'SlatePositionsError',
  'SliceBlockCountRefusalError',
  'SliceCoverageError',
  'SliceDeliveryError',
  'SliceIndexingError',
  'SliceNotOnPageError',
  'SliceRecordContradictionError',
  'SourceMapFileError',
  'SpendCeilingOverrideError',
  'StageBenchStartError',
  'StageRosterRepeatError',
  'StatedRefusalError',
  'StragglerCutError',
  'StreamCutShortError',
  'StreamBoundError',
  'StreamBoundsOutsideCallTimeoutError',
  'StreamDegenerateError',
  'StreamErrorEventError',
  'StreamOverrunError',
  'StreamStalledError',
  'SyntheticModelNotServedError',
  'SyntheticRequestTooLargeError',
  'TranslateAbsenceError',
  'TranslationRepairInterruptedError',
  'TrialLedgerLineError',
  'TrialSliceRefusalError',
  'TransportRequestFailedError',
  'UnansweredContestSliceError',
  'UnfoldedTranslationError',
  'UnknownArtifactGenerationError',
  'UnmeasurableRepairError',
  'UnnameableToolError',
  'UnplaceableArtifactError',
  'UnparseablePageError',
  'UnpositionedContainerError',
  'UnpositionedNodeError',
  'UnpreparedSliceError',
  'UnsafeSeedError',
  'UnseatedStandingError',
  'UnsendableSchemasError',
  'VisualEvidenceInterruptedError',
  'WindowEvidenceError',
  'WireModelRepeatedError',
  'WithheldSlateAbsenceError',
  'WordingCoherenceError',
  'WorkTitleLookupError',
  'WritingBenchUnreachableError',
  'XiaomiModelNotServedError',
];

/**
 Every expression a marked class interpolates, and what it holds.

 A COUNT, A NAME, OR SOMETHING THE OPERATOR TYPED. That is the whole
 permission. An entry whose note cannot be written in those terms is an entry
 whose class should lose the marker instead.
 */
const NAMED_PARTS: Record<string, string> = {
  [BYTE_OFFSET_OR_NOTHING]: 'byte offset, or nothing',
  'ALLOW_DRIFT_VALUE': 'value this package expects in the override variable',
  'ALLOW_DRIFT_VAR': 'environment variable name',
  'HARD_CAP_VAR': 'environment variable name',
  'SMALLEST_FIGURE_MINUTES': 'fixed decimal figure in minutes, a constant of this package',
  'NEURALWATT_DAILY_WH_VAR': 'environment variable name',
  'NEURALWATT_DAY_START_VAR': 'environment variable name',
  'energy.day': 'UTC date the NeuralWatt ledger computed from its clock',
  'energy.remainingWh .toFixed(WH_DECIMALS,)': 'watt-hours the NeuralWatt ledger computed, fixed to three places',
  'energy.reservedWh .toFixed(WH_DECIMALS,)': 'watt-hours the NeuralWatt ledger computed, fixed to three places',
  'String(energy.dailyWh,)': 'daily free energy in watt-hours: the built-in 550 or the operator\'s override',
  'estimateWh.toFixed(WH_DECIMALS,)': 'watt-hours the energy estimate computed, fixed to three places',
  'model': 'NeuralWatt model id the call was addressed to, a member of the closed served-model union',
  'SPEND_CEILING_VAR': 'environment variable name',
  'JSON.stringify(value,)': 'value the operator set in that variable',
  'String(MINIMUM_CHECKER_COUNT,)': 'count',
  'String(lines.length,)': 'count',
  "lines.join('\\n',)": 'producer files, schema names, keywords and paths, all written in this repository',
  'String(overlap,)': 'count',
  'String(spanIndex,)': 'count',
  'String(startOffset,)': 'character offset into the archive page',
  'String(endOffset,)': 'character offset into the archive page',
  'String(earlier.start,)': 'offset into the text two edits were built for',
  'String(earlier.end,)': 'offset into the text two edits were built for',
  'String(later.start,)': 'offset into the text two edits were built for',
  'String(later.end,)': 'offset into the text two edits were built for',
  'String(PASSING_BODY_BYTES,)': 'count',
  'variable': 'environment variable name',
  'String(line,)': 'one-based line number in a file',
  'BEDROCK_CREDIT_USD_VAR': 'environment variable name',
  'String(bodyBytes - PASSING_BODY_BYTES,)': 'count',
  'String(bodyBytes,)': 'count',
  'String(cap,)': 'count',
  'String(census.total,)': 'count',
  'String(charsSeen,)': 'count',
  'String(partialText.length,)': 'count',
  'program': 'name of a program this package runs, a constant of this package',
  'binary': 'name of the program this package starts, a constant of this package (a path a test hands it)',
  'INSTALL_ADVICE': 'fixed advice this package writes from the pinned version and the tool name',
  'versionClause({ found, },)': 'fixed clause, or the word version and a token of one to 32 letters, digits, dots, '
    + 'pluses and minuses read off the program\'s answer',
  'PINNED_LINT_BINARY_VERSION': 'version this package is written against, a constant of this package',
  'LINT_BINARY_NAME': 'name of the linter program, a constant of this package',
  'String(unpairedIndex,)': 'index of the first unpaired surrogate in a passage',
  'failure.code': 'filesystem or spawn code, an upper-case word of at most 32 characters (localProgramFailureOf)',
  'String(failure.exitCode,)': 'exit code of a program this package ran',
  'exchangeFailureText({ error: cause, },)': 'a caught cause through exchangeFailureText: the cause\'s own sentence '
    + 'where its class declares one safe, the class name otherwise, with the HTTP status a provider answered with',
  'wordForCount({ count: partialText.length, one: \'character\', many: \'characters\', },)':
    'noun agreeing with a count, both forms authored here',
  'String(droppedCount,)': 'count',
  'String(slice,)': 'length of the slice a fragment lands in, in UTF-16 units',
  'String(start,)': 'offset in the slice where a fragment\'s replaced region starts',
  'String(end,)': 'offset in the slice where a fragment\'s replaced region ends',
  'whereCarried({ traces, },)': 'fixed clause naming slice indices whose original carries a dropped destination',
  'String(sourceDistinct,)': 'count',
  'String(pageDistinct,)': 'count',
  'refusal': 'strict parser refusal site, positions and rule names only (MdxParseError)',
  'String(quorumOver,)': 'numeric review quorum basis',
  'String(seatCount,)': 'count of requested independent review seats',
  'String(checkerModelIds.length,)': 'count',
  'String(entryIds.length,)': 'count',
  'String(expected,)': 'count',
  'String(found,)': 'count',
  'String(generationCount,)': 'count',
  'String(holder.pid,)': 'process id',
  'String(idleMs,)': 'duration',
  'String(index,)': 'index within a parsed tree',
  'String(failure.index,)': 'numeric position within recorded pairing seat identities',
  'String(sampled,)': 'count',
  'String(status,)': 'HTTP status code',
  'String(unrecorded,)': 'count',
  'String(blocks.length,)': 'count',
  'String(version,)': 'schema version',
  'String(sliceIndex,)': 'slice index',
  'String(round,)': 'page assembly round number, from one',
  'String(unavailableCount,)': 'count of source-referenced assets without usable visual evidence',
  'String(sliceIndices.length,)': 'count',
  'sliceIndices.join(\', \',)': 'slice indexes',
  'String(writes,)': 'schema version',
  'String(pinnedStart,)': 'number a launch pinned as the start of the bench, shown as it was given',
  'String(lastSeat,)': 'index of the last seat of the bench, a count less one',
  'String(benchSize,)': 'count of the seats of the bench',
  'String(count,)': 'count of footnote markers, or of lines in a reference block',
  'convention': 'footnote marker convention name, one of two literals',
  'String(MAX_SLICE_IDENTIFIERS,)': 'bound this package sets',
  'String(timeoutMs,)': 'deadline in milliseconds',
  'String(maxHoldMs,)': 'longest hold in milliseconds a caller passed, shown as the number it was given '
    + '(NaN and Infinity print as words)',
  'String(boundMs,)': 'stream bound in milliseconds a model card declares',
  'String(perCallTimeoutMs,)': 'per-call deadline in milliseconds',
  'faults': 'the keys of the settings found wrong and their numbers, joined (StreamBoundsOutsideCallTimeoutError)',
  'places': 'the names of the constants that set the settings found wrong, joined (StreamBoundsOutsideCallTimeoutError)',
  'UNHEARD_CLAIMS[claim]': 'one of two fixed phrases, keyed by a literal',
  'placementSentence({ fault, },)': 'offsets and counts, composed from numbers alone',
  'reconcileSentence({ fault, },)': 'kind, typeof name and counts, composed here',
  'refusedKeyword': 'JSON Schema keyword a stage schema of this package wrote, never reply text',
  'schemaPath': 'dotted path of member names of a stage schema this package wrote, never reply text',
  'lane': 'lane name, one of two literals',
  'String(changed,)': 'boolean flag',
  'String(code,)': 'numeric failure class the gateway reported, or the word unnamed',
  "changed ? 'archive wording' : 'wording of a change'": 'one of two phrases written here',
  'String(fault.position,)': 'position of a slice in its list',
  'indexingSentence({ fault, },)': 'indices, composed from numbers alone',
  'String(fault.sliceIndex,)': 'slice index',
  'terminal': 'consolidation terminal name, a member of a closed union',
  'coverageSentence({ fault, },)': 'side name, counts and positional block ids, composed here',
  'coherenceSentence({ fault, },)': 'fault kind and at most an outcome kind, composed here',
  'String(position,)': 'position of a slice in its list, or of a line in a reference block',
  'REFERENCE_LINE_MARK': 'fixed mark this package writes at the head of every reference line',
  'line': 'line of this package\'s own source, or git log output over its own commits (hash, time, subject); '
    + 'never corpus text',
  'WINDOW_LABEL': 'name of a window this package defines',
  'channel': 'stream channel name, content or reasoning',
  'checkerModelIds.join(\', \',)': 'model ids from the catalog',
  'detail': 'authored phrase naming which rule was broken, at every throw site; the one value read off a body '
    + 'is an Anthropic error event\'s type, lower-case letters and underscores of at most 64 characters, '
    + 'or the word unnamed (ledger B87); SliceBlockCountRefusalError forwards the slice grammar\'s refusal as '
    + 'readSliceSkeleton reports it, positions and rule names only (MdxParseError, ledger B100)',
  'CONTRACT_NAMES[wireFormat]': 'contract name, one of two fixed phrases keyed by the closed wire-format union (ledger B90)',
  'measured': 'meter states and hold durations, composed by the caller from two booleans and two numbers',
  'xiaomiCause': 'one of two fixed sentences, chosen by whether Xiaomi was counted dry for serving nothing here',
  'xiaomiRefill': 'a fixed sentence about Xiaomi\'s recovery, or nothing',
  'openingClauseOf({ day: neuralwatt, ...named, },)': 'one of five fixed openers about NeuralWatt\'s energy, its key '
    + 'or its day start, naming no figure, or nothing for an off provider',
  'providerClausesOf({ idle, },)': 'fixed sentences naming each provider and whether it is out of budget or reads wet '
    + 'but serves no seat that writes',
  '(idle.length === 0) ? \'Nothing further can be bought\' : \'Nothing further that writes a page can be bought\'':
    'one of two fixed sentences, chosen by whether a provider reads wet but serves no seat that writes',
  'returnClauseOf({ day: neuralwatt, ...named, afterWaiting, },)': 'one fixed sentence about NeuralWatt\'s return, '
    + 'naming at most the UTC time of day its day starts at, a distance composed from a number of milliseconds, and '
    + 'for a blocked key the instant it lifts, read as an instant and spelled in UTC (keyUnblockTextOf), never the '
    + 'reply\'s own text',
  'dir': 'directory path',
  'endpoint': 'upstream display name as the gateway spelled it, the same field the SPEND line prints, or the word unnamed',
  'errorType': 'gateway failure kind read off its metadata, or the word unnamed; in StreamErrorEventError, a documented '
    + 'provider error type from a closed list this package holds',
  'distinctRatio.toFixed(RATIO_DIGITS,)': 'ratio this process computed',
  'duplicated.join(\', \',)': 'model ids from the catalog',
  'disagreementSentence({ disagreement, },)': 'slice indices and character counts, composed from numbers alone',
  'entryId': 'person entry id, which these tools report by design',
  'unreviewedLocations({ blocks, },)': 'pair indexes and parser block ids, composed from names alone',
  'wordForCount({ count: entryIds.length, one: \'artifact here records\', many: \'artifacts here record\', },)':
    'noun and verb agreeing with a count, both forms authored here',
  'wordForCount({ count: entryIds.length, one: \'artifact in this directory records\', many: \'artifacts in this directory record\', },)':
    'noun and verb agreeing with a count, both forms authored here',
  'wordForCount({ count: bodyBytes, one: \'byte\', many: \'bytes\', },)':
    'noun agreeing with a count, both forms authored here',
  'wordForCount({ count: charsSeen, one: \'character\', many: \'characters\', },)':
    'noun agreeing with a count, both forms authored here',
  'wordForCount({ count: seatCount, one: \'seat\', many: \'seats\', },)':
    'noun agreeing with a count, both forms authored here',
  'wordForCount({ count: blockIds.length, one: \'block\', many: \'blocks\', },)':
    'noun agreeing with a count, both forms authored here',
  'wordForCount({ count: blockIds.length, one: \'it\', many: \'them\', },)':
    'pronoun agreeing with a count, both forms authored here',
  'blockIds.join(\', \',)': 'positional block ids (block/N) of one section, naming places and never wording',
  'wordForCount({ count: droppedCount, one: \'destination\', many: \'destinations\', },)':
    'noun agreeing with a count, both forms authored here',
  'wordForCount({ count: slices.length, one: \'slice\', many: \'slices\', },)':
    'noun agreeing with a count, both forms authored here',
  'wordForCount({ count: sampled, one: \'issue carries\', many: \'issues carry\', },)':
    'noun and verb agreeing with a count, both forms authored here',
  'wordForCount({ count: census.total, one: \'entry\', many: \'entries\', },)':
    'noun agreeing with a count, both forms authored here',
  'wordForCount({ count: census.total, one: \'was\', many: \'were\', },)':
    'verb agreeing with a count, both forms authored here',
  'failure': 'filesystem code a file read raised where it carries one, otherwise the class name of what the read or '
    + 'a JSON parse raised, or errorName\'s fixed phrase for a thrown value with no class (failureName, errorName); '
    + 'never a message',
  'filesystemCode': 'filesystem code a directory listing raised (ENOTDIR, EACCES), or the class name where there is none',
  'fault': 'authored phrase naming which roster rule was broken',
  'field': 'field name',
  'file': 'file path',
  'from': 'file path',
  'holder.startedAt': 'timestamp',
  'fault.sheetLabel': 'which sheet this is, one of two literals the scorers write',
  'fault.sheetPath': 'graded sheet path the operator named, or the default under the runs directory',
  'fault.manifestPath': 'draw manifest path the operator named, or the default under the runs directory',
  'MISMATCH_REASONS[fault.kind]': 'authored sentence selected by a closed mismatch kind, quoting no value read '
    + 'from the sheet or the manifest',
  'UNPLACED_REASON': 'authored sentence for a sheet that declares no draw seed, quoting nothing read from it',
  'HELD_BECAUSE[judgedBy]': 'fixed authored phrase saying how a runs-lock holder was judged alive',
  'judgeModelIds.join(\', \',)': 'model ids from the catalog',
  'kind': 'mdast node type name or closed tally/reading/archive-evidence failure-kind literal',
  'relPath': 'operator-supplied repository-relative archive path, never document text',
  'referenceId': 'caller-supplied reviewed fixture identifier, never reference prose',
  'ROOT_MESSAGES[kind]': 'authored root-planning diagnostic selected by a closed failure kind',
  'COMPARISON_MESSAGES[kind]': 'authored persisted-input comparison diagnostic selected by a closed failure kind',
  'JSON.stringify(directory)': 'operator-authorized comparison evidence directory locator; the constructor does not accept failure prose or artifact contents',
  'INPUT_RUN_MESSAGES[operation]': 'authored preparation input-runner diagnostic selected by a closed operation kind',
  'JSON.stringify(locator)': 'preparation runner input path, supporting root/reference locator or fixed output path; current throw sites never pass file contents',
  [`locator === undefined ? '' : \` Input: \${JSON.stringify(locator)}.\``]: 'optional encoded input locator from the audited preparation runner throw sites, never input contents',
  [`(received === undefined) ? '' : \`; received \${received}\``]: 'the top-level key names of a refused quotas body, at most '
    + 'twenty, each printable ASCII cut to forty characters, with the JSON type of each (receivedShapeOf, boundedKeyOf): provider '
    + 'text bounded to names, never a value',
  'JSON.stringify(affectedInput,)': 'root artifact locator, corpus-relative path, entry/parent identifier or fixed registry label; audited throw sites never pass compared content',
  'claimId': 'computed claim identifier, not its quoted text or diagnostic summary',
  'label': 'model id the transport was calling',
  'leftId': 'envelope id',
  'mdxRefusalSite({ cause, },)': 'position, built to state a place and quote nothing',
  'modelId': 'model id from the catalog',
  'nestingRefusalAccount({ reading, },)': 'measure, bound and place, or the stack exhaustion, built to quote nothing',
  'operation': 'member of a closed operation union',
  'signal': 'SIGINT or SIGTERM in the audited preparation interruption error constructor',
  'promptDigest': 'SHA-256 prompt identity',
  'modelIds.join(\', \',)': 'model ids from the catalog',
  'bench': 'name of a bench, a member of the run configuration\'s closed bench list',
  'wireModel': 'wire model name a card of the roster sends',
  'missing': 'document side, original or translation',
  'name === \'\' ? \'<>\' : name': 'container tag name, or a mark for an unnamed one',
  'overlapping.join(\', \',)': 'model ids from the catalog',
  'path': 'file path',
  'phase': 'name of the stream phase this package defines',
  'returnSentence': 'one of two fixed sentences about NeuralWatt\'s return, naming at most the UTC time of day its day '
    + 'starts at (the run composes it from the day start, never from a provider\'s text)',
  'producerModelIds.join(\', \',)': 'model ids from the catalog',
  'provider': 'member of a closed provider union',
  'reason': 'authored phrase, or a member of a closed union, at every throw site; or a marked class\'s sentence '
    + 'forwarded from a catch narrowed to it, at a site FORWARDING_SITES lists; or, for a prompt payload record, '
    + 'an authored phrase naming a JSON path in the record or ending in the filesystem code or class name '
    + '`failureName` gives (ledger B69), never a stored value',
  'WORDING_FAULT_SENTENCES[fault]': 'one of five fixed phrases, keyed by a closed fault kind',
  'rightId': 'envelope id',
  'role': 'roster role name this package defines',
  'stage': 'stage label the calling stage passes to its gather: a literal, a function name or a constant this '
    + 'package defines',
  'says': 'authored phrase saying what in a coverage census input did not read, at every throw site; it names at '
    + 'most a script URL under the package\'s own build directory, a bundle name and offsets (the coverage census, '
    + 'ledger T8)',
  'runsDir': 'directory path',
  'seedId': 'seed id',
  'short({ id: digest, },)': 'abbreviated digest',
  'short({ id: nonNullishOrThrow(requiredCommit,), },)': 'abbreviated commit',
  'short({ id: recorded, },)': 'abbreviated digest',
  'unavailableCount': 'count of source-referenced assets without usable visual evidence',
  'clauses.join(\'; \',)': 'bench names, seat counts and the floor, built by the seat reading and quoting nothing',
  'yamlRefusalSite({ cause, },)': 'position, built to state a place and quote nothing',
  'kindAndRemedy': 'a corpus read failure kind in brackets and the one of four fixed remedy sentences keyed by '
    + 'that closed union (CorpusReadError, REMEDY_SENTENCES), quoting no path, revision or content',
  'String(seats,)': 'count of seats in the roster the bench was handed (BenchReportError)',
  'invariant': 'authored sentence at every throw site of the consistency pass, built from slice indexes, group '
    + 'numbers, OCCURRENCE labels, offsets and vocabulary words and quoting no page text, which each site was read '
    + 'for (ConsistencyInvariantError)',
  'entry': 'name of a build entry, a key of a package\'s own list in src/build-entries.ts (FamilyEntryConflictError)',
  'identity': 'identity of an archive block (ArchiveBlockContextMissingError): its location as indices, its parser '
    + 'block id and a digest of its wording, never the wording (archiveBlockIdentity)',
  'heldText': 'whether the map of source sections holds no block, or how many it holds and the identities of '
    + 'them, each an archive block identity, built in the constructor from a count and identities alone (ArchiveBlockContextMissingError)',
  'blockId': 'parser block id (block/N) of an archive block, naming a place and never wording '
    + '(ArchiveBlockAlignmentMissingError)',
  'String(pairIndex,)': 'index of the aligned pair an archive block names',
  'String(pairCount,)': 'count',
  'wordForCount({ count: pairCount, one: \'pair\', many: \'pairs\', },)':
    'noun agreeing with a count, both forms authored here',
  'String(slateSize,)': 'count',
  'wordForCount({ count: slateSize, one: \'candidate\', many: \'candidates\', },)':
    'noun agreeing with a count, both forms authored here',
  'firstPath': 'path a package\'s own build entry list gives an entry (FamilyEntryConflictError)',
  'firstOwner': 'directory name of a package of the family (FamilyEntryConflictError)',
  'secondPath': 'path a package\'s own build entry list gives an entry (FamilyEntryConflictError)',
  'secondOwner': 'directory name of a package of the family (FamilyEntryConflictError)',
};

/**
 Classes that write their own sentence and still may not carry the marker.

 RECORDED RATHER THAN LEFT SILENT, because an absent marker looks identical to
 an oversight, and the next reader would have to re-derive each of these.
 */
const WITHHELD: Record<string, string> = {
  ArtifactProvenanceError: 'expected and observed carry whatever field disagreed, which may be text',
  EmptyImageAssetError: 'names the picture by the file name a corpus chose, which is not this package\'s vocabulary; '
    + 'the stage reports the picture by its reason and never forwards the message',
  PreparationIdentityError: 'quotes the string it refused, which a caller may have read from any file; the '
    + 'artifact reader refuses a stored identity by shape in its own words instead (ledger B34)',
  RenderingAuditInvariantError: 'forwards each site\'s own sentence, which names indexes and vocabulary '
    + 'words and never text; unreachable by construction, so the boundary never has to print it',
  ProviderHttpError: 'the message carries an excerpt of the provider response body, on purpose',
  TimingLineError: 'quotes the whole timing line it could not read, so an operator can find it; the line was '
    + 'chosen by the round or completion marker the run\'s own logger writes, but nothing proves a line '
    + 'carrying that marker holds only labels, counts and durations (T8, nineteenth batch)',
  UnhandledMemberInvariantError: 'names the discriminant of a value the types say cannot exist, which is '
    + 'whatever text that value carries under `kind`; unreachable while every chain names every member of its '
    + 'union, so the boundary never has to print it',
  ReasonOrderError: 'a test fixture\'s refusal (`reason-order.test-fixture.ts`) that quotes the field names and the '
    + 'instruction phrases the case handed it, which are the case\'s own strings; no boundary prints it, so the marker '
    + 'would mean nothing',
  ReturnTimeNotUtcError: 'quotes the return time text a NeuralWatt failure carried when its only writer was expected '
    + 'to give a time ending in UTC; the text is that of a broken invariant, so nothing proves it holds only a time, '
    + 'and the marker would claim a safety the message cannot show',
  SweepOptionRefusalError: 'the rotation sweep\'s own refusal of a command line, printed by `rotationSweepMain` '
    + 'and never through the boundary; its sentence names the option and value the operator typed, which is all it '
    + 'carries, so the marker would mean nothing where no boundary reads it',
};

//endregion Marked message inventory

//region Source scan

/**
 Characters an identifier may carry, tested by membership rather than by
 pattern.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_';

/**
 What `indexOf` and `findIndex` answer when they found nothing.

 NAMED so the comparison carries no bare unary minus, which reads as a mixed
 operator beside an equality test.
 */
const NOT_FOUND = -1;

/**
 Whitespace this scan collapses, so a reflowed expression compares equal.
 */
const WHITESPACE = [
  ' ',
  '\n',
  '\t',
  '\r',
];

/**
 One class declaration as this scan read it.
 */
type ScannedClass = {
  /**
   Declared class name.
   */
  readonly name: string;

  /**
   Whether its body declares the marker.
   */
  readonly marked: boolean;

  /**
   Whether it writes its own sentence rather than forwarding a parameter.
   */
  readonly writesOwnSentence: boolean;

  /**
   Every expression interpolated into its `super()` argument, whitespace
   collapsed so a reflowed line is not read as a different expression.
   */
  readonly parts: readonly string[];
};

/**
 Collapses whitespace runs so a reformatted expression compares equal.

 @param text - expression as it appears in the source

 @returns Same expression on one line with single spaces

 @example
 ```ts
 const key = oneLine({ text: 'short({\n  id,\n},)', },);
 ```
 */
function oneLine({ text, }: { readonly text: string; },): string {
  /**
   Same text with every kind of whitespace turned into a plain space.
   */
  const spaced = WHITESPACE.reduce(function flatten(carried, character,): string {
    return carried
      .split(character,)
      .join(' ',);
  }, text,);

  return spaced
    .split(' ',)
    .filter(function isWord(part,): boolean {
      return part !== '';
    },)
    .join(' ',);
}

/**
 Reads forward from an opening delimiter to its match.

 INDEX SCAN RATHER THAN A PATTERN, because the thing being found is nesting,
 which no pattern expresses and which a scan expresses exactly.

 @param source - text to read

 @param from - index just past the opening delimiter

 @returns Text between the delimiters

 @example
 ```ts
 const inside = balanced({ source, from: at + 'super('.length, },);
 ```
 */
function balanced(
  {
    source,
    from,
  }: {
    readonly source: string;
    readonly from: number;
  },
): string {
  for (let at = from, depth = 1; at < source.length; at += 1) {
    /**
     Character this step is reading.
     */
    const character = source.charAt(at,);

    /**
     Depth once this character is counted.
     */
    const next = (depth + (('(['.includes(character,) || (character === '{')) ? 1 : 0))
      - ((')]'.includes(character,) || (character === '}')) ? 1 : 0);

    if (next === 0)
      return source.slice(from, at,);

    depth = next;
  }

  return source.slice(from,);
}

/**
 Collects every `${...}` body inside one expression.

 @param expression - `super()` argument as written

 @returns Each interpolated expression, whitespace collapsed

 @example
 ```ts
 const parts = interpolationsOf({ expression, },);
 ```
 */
function interpolationsOf(
  { expression, }: { readonly expression: string; },
): readonly string[] {
  /**
   Interpolations found so far, in source order.
   */
  const found: string[] = [];

  for (let cursor = 0; cursor < expression.length;) {
    /**
     Where the next interpolation opens.
     */
    const at = expression.indexOf('${', cursor,);

    if (at === NOT_FOUND)
      return found;

    /**
     Expression between that opening and its matching brace.
     */
    const inside = balanced({
      source: expression,
      from: at + '${'.length,
    },);

    found.push(oneLine({ text: inside, },),);
    cursor = at + '${'.length + inside.length + 1;
  }

  return found;
}

/**
 Decides whether a `super()` argument is nothing but a forwarded parameter.

 SCANNED RATHER THAN MATCHED. The rule is "every character is one an
 identifier may carry", which an index pass states directly and which a
 pattern would restate less clearly.

 @param argument - trimmed `super()` argument

 @returns Whether it is one lowercase identifier, with an optional comma

 @example
 ```ts
 const forwarded = isBareIdentifier({ argument: 'message,', },);
 ```
 */
function isBareIdentifier({ argument, }: { readonly argument: string; },): boolean {
  /**
   Same argument without the trailing comma the house style writes.
   */
  const withoutComma = argument.endsWith(',',)
    ? argument.slice(0, -1,)
    : argument;

  /**
   First character, which decides whether this could be a parameter name.
   */
  const opening = withoutComma.charAt(0,);

  if ((opening === '') || (opening !== opening.toLowerCase()))
    return false;

  for (let at = 0; at < withoutComma.length; at += 1) {
    if (!IDENTIFIER_CHARACTERS.includes(withoutComma.charAt(at,),))
      return false;
  }

  return true;
}

/**
 Reads every error class one source file declares.

 @param source - file contents

 @returns One record per class declaration

 @example
 ```ts
 const declared = classesIn({ source: text, },);
 ```
 */
function classesIn({ source, }: { readonly source: string; },): readonly ScannedClass[] {
  /**
   File split into lines, which is how a class opening is recognised.
   */
  const lines = source.split('\n',);

  return lines.flatMap(function atLine(line, index,): readonly ScannedClass[] {
    /**
     Opening this line carries, or nothing when it declares no class.
     */
    const opener = line.startsWith('export class ',)
      ? 'export class '
      : (line.startsWith('class ',) ? 'class ' : '');

    if ((opener === '') || (!line.includes(' extends ',)))
      return [];

    /**
     Declared name, which runs to the space before `extends`.
     */
    const name = line
      .slice(opener.length,)
      .split(' ',)
      .at(0,) ?? '';

    // The body runs to the next line that is a lone closing brace, which is how
    // every class in this package ends.
    /**
     Line index of that closing brace, or absent when the file ends first.
     */
    const closes = lines.findIndex(function isClose(candidate, at,): boolean {
      return (at > index) && (candidate === '}');
    },);

    /**
     Class body as text, opening line included.
     */
    const body = lines
      .slice(index, (closes === NOT_FOUND) ? lines.length : closes,)
      .join('\n',);

    /**
     Where this class calls `super`, or absent when it inherits one.
     */
    const superAt = body.indexOf('super(',);

    /**
     Whole `super()` argument, or nothing when there is no call.
     */
    const argument = (superAt === NOT_FOUND)
      ? ''
      : balanced({
        source: body,
        from: superAt + 'super('.length,
      },);

    return [{
      name,
      marked: body.includes('readonly messageNamesOnly',),
      // A bare identifier means the sentence arrived from the throw site.
      writesOwnSentence: (argument.trim() !== '')
        && (!isBareIdentifier({ argument: argument.trim(), },)),
      parts: interpolationsOf({ expression: argument, },),
    },];
  },);
}

/**
 Reads every non-test source file of every family package and returns the classes they declare.

 @returns Every scanned class across the family's source

 @example
 ```ts
 const declared = await scanSource();
 ```
 */
async function scanSource(): Promise<readonly ScannedClass[]> {
  /**
   Files this scan reads, which excludes the suites.
   */
  const sources = (await readPackageSource()).filter(function isSource({ path, },): boolean {
    return !(path.split('/',).at(-1,) ?? '').includes('.test.',);
  },);

  return sources.flatMap(function one({ text, },): readonly ScannedClass[] {
    return classesIn({ source: text, },);
  },);
}

//endregion Source scan

//region Forwarded messages
// A MARKED CLASS CAN STILL CARRY TEXT IT DID NOT WRITE, through a throw site
// that hands it a caught error's message (ledger B34): `ArtifactParseError`
// printed a pipeline digest and a preparation identity that way, quoted by the
// errors it caught. This scan finds every construction of a marked class whose
// arguments name a catch clause's binding or call `caughtValueText`, and reads
// which classes that catch narrows its binding to with `instanceof`. Only a
// catch narrowed to marked classes may forward, since their sentences are
// checked by this file; each such site is listed. The narrowing is read by
// its presence in the clause, not proven to guard the throw, so a review of
// each listed site still decides it.

/**
 A marked class's construction that carries text a caught error wrote.
 */
type Forwarding = {
  /**
   File, relative to `src`.
   */
  readonly file: string;

  /**
   Marked class constructed.
   */
  readonly className: string;

  /**
   Classes the enclosing catch narrows its binding to, sorted; none where it
   narrows nothing or the text arrives outside a catch.
   */
  readonly narrowedTo: readonly string[];
};

/**
 How a listed site uses the caught error's text.

 FORWARDS: the text becomes part of the marked message, so the catch must
 narrow to marked classes, whose sentences this file checks.
 READ-FOR-A-NUMBER: the text is read for a number the class's own inventory
 names, and none of it reaches the message. The one site that forwarded a
 library's message quoting what the operator typed, the coverage census's
 own `parseArgs` catch, went when every runner's line moved to
 `readCommandLine`, which writes its own refusals (ledger B75).
 */
type ForwardingKind = 'forwards' | 'read-for-a-number';

/**
 Every construction of a marked class whose arguments turn a caught error
 into text, with how that text is used and what it names.
 */
const FORWARDING_SITES: readonly (Forwarding & {
  /**
   How the caught error's text is used.
   */
  readonly kind: ForwardingKind;

  /**
   What that text names, read at the throw sites it comes from.
   */
  readonly names: string;
})[] = [
  {
    file: 'artifact/artifact-change-sets.ts',
    className: 'ArtifactParseError',
    narrowedTo: ['AssemblyContractError',],
    kind: 'forwards',
    names: 'slice indexes and index-set sizes',
  },
  {
    file: 'artifact-pool/corpus-run/artifact-two-lane-read-comparison.ts',
    className: 'ArtifactParseError',
    narrowedTo: ['ArtifactComparisonError',],
    kind: 'forwards',
    names: 'lane names, counts, positions, slice indexes, archive-wording kinds and comparison-row field names',
  },
  {
    file: 'artifact/corpus-run/artifact-two-lane-read-row-relations.ts',
    className: 'ArtifactParseError',
    narrowedTo: ['DeliveryCoherenceError', 'WordingCoherenceError',],
    kind: 'forwards',
    names: 'a slice index and fault and outcome kinds',
  },
  {
    file: 'probe/corpus-run/judge-fidelity-probe-run.ts',
    className: 'StatedRefusalError',
    narrowedTo: ['FidelityReferenceError',],
    kind: 'forwards',
    names: 'a reviewed reference id, an entry id the operator named, a fixed request label or the missing damage '
      + 'families, and a closed operation name',
  },
  {
    file: 'bench/corpus-run/producer-calibrate-run.ts',
    className: 'StatedRefusalError',
    narrowedTo: ['ProducerRosterError',],
    kind: 'forwards',
    names: 'roster roles, model ids and counts',
  },
  {
    file: 'shared-contract/run-json-read.ts',
    className: 'RunJsonUnreadableError',
    narrowedTo: [],
    kind: 'read-for-a-number',
    names: 'the byte offset JSON.parse reports, read by offsetIn as a number',
  },
];

/**
 Node kinds a scan for names under an expression does not enter.
 */
const NESTED_FUNCTION_KINDS: ReadonlySet<string> = new Set(['ArrowFunctionExpression', 'FunctionDeclaration', 'FunctionExpression',],);

/**
 Members through which a value becomes its text.
 */
const TEXT_MEMBERS: ReadonlySet<string> = new Set(['message', 'stack',],);

/**
 Every identifier some nodes turn into text, functions nested in them left out:
 one interpolated into a template, handed to `String` or `caughtValueText`, or
 read through its `message` or `stack`. A value handed on whole, as a
 `cause` a class reads a position or a name from, is not text here: the
 class's own sentence is what the inventory checks. `caughtValueText` counts
 by name too, wherever it is called.

 @param roots - nodes to read

 @returns Names found, each once

 @example
 ```ts
 const named = textNamesUnder({ roots: construction.arguments, },);
 ```
 */
function textNamesUnder({ roots, }: { readonly roots: readonly TreeNode[]; },): ReadonlySet<string> {
  /**
   Names found so far.
   */
  const found = new Set<string>();
  /**
   Nodes still to read.
   */
  const pending = [...roots,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if (NESTED_FUNCTION_KINDS.has(node.type,))
      continue;
    /**
     Name this node calls, when it is a call.
     */
    const called = (node.type === 'CallExpression') ? identifierName({ node: node.callee, },) : '';
    /**
     Arguments of that call.
     */
    const callArguments = (Array.isArray(node.arguments,) ? node.arguments : []).filter(function isNode(
      argument: unknown,
    ): argument is TreeNode {
      return isTreeNode(argument,);
    },);
    if (called === 'caughtValueText')
      found.add(called,);
    if ((called === 'caughtValueText') || (called === 'String')) {
      for (const argument of callArguments)
        found.add(identifierName({ node: argument, },),);
    }
    if ((node.type === 'MemberExpression') && TEXT_MEMBERS.has(identifierName({ node: node.property, },),))
      found.add(identifierName({ node: node.object, },),);
    if ((node.type === 'TemplateLiteral') && Array.isArray(node.expressions,)) {
      for (const expression of node.expressions)
        found.add(identifierName({ node: expression, },),);
    }
    pending.push(...childNodes({ node, },),);
  }
  found.delete('',);
  return found;
}

/**
 Classes a catch clause narrows its binding to with `instanceof`.

 @param body - the clause's block

 @param binding - name the clause binds its error to

 @returns Class names, sorted and each once

 @example
 ```ts
 const narrowedTo = narrowingsOf({ body, binding: 'error', },);
 ```
 */
function narrowingsOf(
  {
    body,
    binding,
  }: {
    readonly body: TreeNode;
    readonly binding: string;
  },
): readonly string[] {
  /**
   Classes found so far.
   */
  const found = new Set<string>();
  /**
   Nodes still to read.
   */
  const pending = [body,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    if ((node.type === 'BinaryExpression') && (node.operator === 'instanceof')
      && (identifierName({ node: node.left, },) === binding)) {
      /**
       Class tested against.
       */
      const tested = identifierName({ node: node.right, },);
      if (tested !== '')
        found.add(tested,);
    }
    pending.push(...childNodes({ node, },),);
  }
  return [...found,].toSorted();
}

/**
 One catch clause enclosing a node, as the forwarding scan carries it.
 */
type CatchScope = {
  /**
   Name the clause binds its error to.
   */
  readonly binding: string;

  /**
   The clause's block.
   */
  readonly body: TreeNode;
};

/**
 Every construction of a marked class in one file that forwards a caught
 error's text: its arguments name an enclosing catch clause's binding, or call
 `caughtValueText`.

 @param file - file to read

 @param marked - classes that declare the marker

 @returns Each such construction, in no particular order

 @example
 ```ts
 const found = forwardingIn({ file, marked: new Set(MARKED_CLASSES,), },);
 ```
 */
function forwardingIn(
  {
    file,
    marked,
  }: {
    readonly file: SourceText;
    readonly marked: ReadonlySet<string>;
  },
): readonly Forwarding[] {
  /**
   The file's syntax tree.
   */
  const { program, } = parseSource({ file, },);
  /**
   Constructions found so far.
   */
  const found: Forwarding[] = [];
  /**
   Nodes still to read, each with the catch clauses enclosing it.
   */
  const pending: { readonly node: TreeNode; readonly catches: readonly CatchScope[]; }[] = [{
    node: program,
    catches: [],
  },];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /**
     Node read now, and the clauses around it.
     */
    const {
      node,
      catches,
    } = next;
    /**
     Class constructed here, or nothing.
     */
    const className = (node.type === 'NewExpression') ? identifierName({ node: node.callee, },) : '';
    if (marked.has(className,)) {
      /**
       Names the construction's arguments carry.
       */
      const named = textNamesUnder({
        roots: Array.isArray(node.arguments,) ? node.arguments.filter(isTreeNode,) : [],
      },);
      /**
       Innermost enclosing clause whose binding the arguments name.
       */
      const scope = catches.findLast(function namesBinding(candidate,): boolean {
        return named.has(candidate.binding,);
      },);
      if ((scope !== undefined) || named.has('caughtValueText',)) {
        found.push({
          file: file.path,
          className,
          narrowedTo: (scope === undefined) ? [] : narrowingsOf(scope,),
        },);
      }
    }
    /**
     Clauses enclosing this node's children.
     */
    const inner = ((node.type === 'CatchClause') && isTreeNode(node.body,))
      ? [
        ...catches,
        {
          binding: identifierName({ node: node.param, },),
          body: node.body,
        },
      ]
      : catches;
    pending.push(...childNodes({ node, },)
      .map(function withScopes(child,) {
        return {
          node: child,
          catches: inner,
        };
      },),);
  }
  return found;
}

/**
 Sorts forwarding records into one comparable order.

 @param records - records to sort

 @returns Each as one line, sorted

 @example
 ```ts
 const lines = forwardingLines({ records: found, },);
 ```
 */
function forwardingLines({ records, }: { readonly records: readonly Forwarding[]; },): readonly string[] {
  return records
    .map(function line({
      file,
      className,
      narrowedTo,
    },): string {
      return `${file}: ${className} narrowed to [${narrowedTo.join(', ',)}]`;
    },)
    .toSorted();
}

//endregion Forwarded messages

await describe({
  name: 'messageNamesOnly',
  children: [
    it({
      name: 'KEEPS exactly the classes the inventory records',
      fn: async () => {
        const declared = await scanSource();

        expectFindingsAsListed({
          findings: declared
            .filter(function isMarked(entry,): boolean {
              return entry.marked;
            },)
            .map(function named(entry,): string {
              return entry.name;
            },)
            .toSorted(),
          listed: MARKED_CLASSES.toSorted(),
        },);
      },
    },),

    it({
      name: 'ACCEPTS only the message parts the inventory names',
      fn: async () => {
        const declared = await scanSource();

        const unnamed = declared
          .filter(function isMarked(entry,): boolean {
            return entry.marked;
          },)
          .flatMap(function parts(entry,): readonly string[] {
            return entry
              .parts
              .filter(function isUnnamed(part,): boolean {
                return !Object.hasOwn(NAMED_PARTS, part,);
              },)
              .map(function attributed(part,): string {
                return `${entry.name}: ${part}`;
              },);
          },);

        // Naming the class and the part rather than a count, because the whole
        // point of a failure here is that a reader has to decide about one
        // specific expression.
        expectNoFindings({ findings: unnamed, },);
      },
    },),

    it({
      name: 'REFUSES to let a class that forwards a message carry the marker',
      fn: async () => {
        const declared = await scanSource();

        expectNoFindings({
          findings: declared
            .filter(function isForwarding(entry,): boolean {
              return entry.marked
                && (!entry.writesOwnSentence)
                // The documented exception, whose whole contract is the
                // sentence its caller wrote.
                && (entry.name !== 'StatedRefusalError');
            },)
            .map(function named(entry,): string {
              return entry.name;
            },),
        },);
      },
    },),

    it({
      name: 'KEEPS a reason for every class that writes its sentence and stays unmarked',
      fn: async () => {
        const declared = await scanSource();

        expectFindingsAsListed({
          findings: declared
            .filter(function isWithheld(entry,): boolean {
              return (!entry.marked) && entry.writesOwnSentence;
            },)
            .map(function named(entry,): string {
              return entry.name;
            },)
            .toSorted(),
          listed: Object.keys(WITHHELD,).toSorted(),
        },);
      },
    },),

    it({
      name: 'FINDS a marked class handed a caught error\'s text, with the classes its catch narrows to: '
        + 'unnarrowed, narrowed to a marked class, narrowed to another, through String of the binding, and '
        + 'through caughtValueText outside any catch; and leaves a catch that only logs, one handing the error '
        + 'on whole or reading its name, and an unmarked class',
      fn: async () => {
        /**
         One cat-themed file holding each shape, a function apiece. Each
         interpolation's `$` and `{` are split across a concatenation, so no
         plain string holds a whole placeholder, which lint reads as a
         template literal written by mistake.
         */
        const catFile: SourceText = {
          path: 'cat-refusals.ts',
          isTest: false,
          text: [
            'function nap() { try { purr(); } catch (error) { throw new MarkedError({ reason: `a nap: $'
              + '{caughtValueText(error,)}`, },); } }',
            'function knead() { try { purr(); } catch (error) { if (!(error instanceof PurrError)) throw error; '
              + 'throw new MarkedError({ reason: `a knead: $'
              + '{error.message}`, },); } }',
            'function stretch() { try { purr(); } catch (error) { if (!(error instanceof HissError)) throw error; '
              + 'throw new MarkedError({ reason: String(error,), },); } }',
            'function yawn(error: unknown) { return new MarkedError({ reason: caughtValueText(error,), },); }',
            'function doze() { try { purr(); } catch (error) { log(error,); throw new MarkedError({ reason: \'a doze\', },); } }',
            'function pounce() { try { purr(); } catch (error) { throw new MarkedError({ cause: error, kind: error.name, },); } }',
            'function groom() { try { purr(); } catch (error) { throw new PlainError({ reason: caughtValueText(error,), },); } }',
          ].join('\n',),
        };
        expect(forwardingLines({
          records: forwardingIn({
            file: catFile,
            marked: new Set(['MarkedError', 'PurrError',],),
          },),
        },),).toEqual([
          'cat-refusals.ts: MarkedError narrowed to [HissError]',
          'cat-refusals.ts: MarkedError narrowed to [PurrError]',
          'cat-refusals.ts: MarkedError narrowed to []',
          'cat-refusals.ts: MarkedError narrowed to []',
        ],);
      },
    },),

    it({
      name: 'FORWARDS A CAUGHT ERROR\'S TEXT into a marked class only at the listed sites, each narrowed to '
        + 'marked classes, and every listed site still forwards (ledger B34)',
      fn: async () => {
        /**
         Every non-test source file.
         */
        const files = (await readPackageSource()).filter(function isSource(file,): boolean {
          return !file.path.includes('.test.',);
        },);
        /**
         Every forwarding construction across them.
         */
        const found = files.flatMap(function inFile(file,): readonly Forwarding[] {
          return forwardingIn({
            file,
            marked: new Set(MARKED_CLASSES,),
          },);
        },);
        expectFindingsAsListed({
          findings: forwardingLines({ records: found, },),
          listed: forwardingLines({ records: FORWARDING_SITES, },),
        },);
        expectNoFindings({
          findings: FORWARDING_SITES.flatMap(function unmarkedNarrowing(site,): readonly string[] {
            if (site.kind !== 'forwards')
              return [];
            return (site.narrowedTo.length === 0)
              ? [`${site.file}: forwards from an unnarrowed catch`,]
              : site.narrowedTo
                .filter(function isUnmarked(className,): boolean {
                  return !MARKED_CLASSES.includes(className,);
                },)
                .map(function located(className,): string {
                  return `${site.file}: narrowed to unmarked ${className}`;
                },);
          },),
        },);
      },
    },),
  ],
},);
