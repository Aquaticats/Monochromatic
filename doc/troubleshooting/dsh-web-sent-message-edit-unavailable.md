# dsh 0.2.1-alpha.4 web chat: a sent user message offers no edit action; only pending queue items are editable

This file records one capability boundary of DeepSeek Harness (`dsh`),
 reported as "I can't edit previous messages in dsh".
 The boundary is product scope plus an append-only session log,
 not a broken control:
 sent transcript messages carry no edit affordance anywhere in the
 shipped verb surface,
 and the only `edit` verb reaches still-pending queue occurrences.

Surface trigger:
 the DeepSeek Harness Web GUI chat transcript,
 after a user message has been admitted into the session log.
 The same verb surface backs every Chat target,
 so the boundary is not Web-specific.

## Symptom

What the user sees in the Web GUI chat:

- Hovering a sent user message reveals its clock and one icon action:
  copy.
  No Edit control appears,
  in any work-details mode.
- The completed-turn assistant tail shows copy,
  the clock,
  and a branch icon labeled `message.branch`
  ("Branch into a new conversation").
  Everywhere else that icon is disabled and reads
  `message.branchUnavailable`
  ("Available only on the last message of a completed turn").
- Editing does exist in the queue dock,
  but only while a message is still queued
  (not yet admitted).
  Its pencil carries the label `queue.edit`
  ("Edit queued message"),
  and a queued item that carries attachments disables it with the title
  `queue.edit.unsupported`
  ("Contains non-text content;
  editing is not supported yet").

Expected by the user:
 an Edit action on a sent user message that rewrites it and re-runs the
 turn from the correction,
 as in ChatGPT-style chats.

Observed instead:
 the sent message stays as recorded forever,
 and the only first-party moves are a follow-up message,
 a steer,
 or a session fork.

## Root cause

### Step 1: the outward session verb surface has no durable-message edit verb

Feature packages may only call the verbs declared on `ISession`;
 the interface comment names that closure explicitly
 (`@deepseek-ai/dsh-api-session-controller/lib/types/client/contract/session.d.ts:2-8`):

```ts
// @deepseek-ai/dsh-api-session-controller/lib/types/client/contract/session.d.ts:2-8
 * The outward session face. Feature packages never see the concrete Session
 * class: components read lifecycle state through `useSession` (the
 * ObservableSnapshot half), and orchestration code calls the behavior verbs
 * below — nothing else. Widening this interface is the explicit act of
 * widening what features may do to a session (and what every test fixture
 * must stub); implementation-internal entry points (history staging, wire-frame
 * dispatch) stay on the class, invisible out here.
```

The complete verb list is
 `beginSubmission`,
 `prompt`,
 `readAttachment`,
 `updateQueue`,
 `cancel`,
 `rename`,
 `loadOlder`,
 `loadThrough`,
 and `command`
 (`session.d.ts:57-150`).
 None of them rewrites an admitted message.

### Step 2: the one `edit` verb is scoped to still-pending queue items, text only

`QueueAction` is defined as a mutation of a still-pending queue item
 (`@deepseek-ai/dsh-api-session-controller/lib/types/types.d.ts:149-158`):

```ts
// @deepseek-ai/dsh-api-session-controller/lib/types/types.d.ts:149-158
/** One client-requested mutation of a still-pending queue item. */
export type QueueAction = {
    readonly kind: 'edit';
    /** Non-empty text-only replacement content. */
    readonly content: readonly TextBlock[];
} | {
    readonly kind: 'remove';
} | {
    readonly kind: 'steer';
};
```

The only caller-facing verb that accepts it repeats the same scope
 (`@deepseek-ai/dsh-api-session-controller/lib/types/client/contract/session.d.ts:95-100`):

```ts
// @deepseek-ai/dsh-api-session-controller/lib/types/client/contract/session.d.ts:95-100
    /**
     * Apply one edit, remove, or Steer action to a still-pending queue occurrence.
     * @param itemId - agent-owned inbox occurrence identity.
     * @param action - requested queue operation.
     * @returns acceptance, or a business/transport error.
     */
    updateQueue(itemId: MessageId, action: QueueAction): Promise<RemoteResult<{
```

Once the agent admits a queue occurrence into the session log,
 its queue row is gone and `updateQueue` no longer addresses it.

### Step 3: Chat renders user bubbles with copy and clock only; branch is a fork on the turn tail

The user-bubble renderer passes exactly three props to the shared action
 row
 (`@deepseek-ai/dsh-client-ui-chat/lib/client.js:5485-5499`):

```js
// @deepseek-ai/dsh-client-ui-chat/lib/client.js:5485-5499
		const UserMessageNodeView = (0, react.memo)(function UserMessageNodeView({ node, renderFactorySlot, loadImage, t }) {
			const data = node.data;
			return (0, react_jsx_runtime.jsx)(UserStyleBubble, {
				content: data.content,
				interactive: true,
				renderFactorySlot,
				loadImage,
				...data.referenceLabels === void 0 ? {} : { referenceLabels: data.referenceLabels },
				...data.skillNames === void 0 ? {} : { skillNames: data.skillNames },
				actions: (text) => (0, react_jsx_runtime.jsx)(MessageIconActions, {
					text,
					time: data.time,
					clock: "start",
```

`MessageIconActions` itself documents copy and branch as its whole action
 set,
 and its `onBranch` prop is documented as a fork
 (`@deepseek-ai/dsh-client-ui-chat/lib/types/client/chat/MessageIconActions.d.ts:5-13`):

```ts
// @deepseek-ai/dsh-client-ui-chat/lib/types/client/chat/MessageIconActions.d.ts:5-13
export interface MessageIconActionsProps {
    /** Plain text the copy action writes. */
    text: string;
    /** Unix epoch ms for the clock label; omitted for transient messages. */
    time?: number | undefined;
    /** Clock before icons (user) or after (assistant). */
    clock: 'start' | 'end';
    /** Fork the session at this message; omission hides the branch action. */
    onBranch?: (() => void) | undefined;
    /** The message is not a completed transcript tail, so branch stays visible but unavailable. */
    branchUnavailable?: boolean | undefined;
```

The single `forkAt` call site sits on the assistant turn tail,
 and it forwards the turn-end seq to `fork`
 (`@deepseek-ai/dsh-client-ui-chat/lib/client.js:6997-6998` and
 `:13267-13271`):

```js
// @deepseek-ai/dsh-client-ui-chat/lib/client.js:6997-6998
					onBranch: forkAllowed ? () => {
						forkAt(data.seq);
// @deepseek-ai/dsh-client-ui-chat/lib/client.js:13267-13271
							forkAt: (seq) => {
								const messageId = [...chat.getSnapshot().timeline.turns.values()].find((turn) => turn.end?.seq === seq)?.data.get("turn-tail")?.closing?.finalNode.messageId;
								ctx.sessions.fork({
									sessionId,
									atSeq: seq,
```

Earlier reading was wrong here:
 this file first assumed the branch icon also sat on user bubbles and
 could be used to replace a message.
 It does neither.
 The action row on a user bubble receives no `onBranch` and no
 `extraActions` seat,
 and `fork({ atSeq })` keeps the prefix through `atSeq` inclusive
 (Step 4),
 so branching continues from the branched-from message instead of
 replacing it.

### Step 4: the durable log is append-only and has no user-message rewrite event type

The known session event catalog carries `user/message`
 (append),
 `developer/message`,
 and compaction events,
 but no user-message rewrite or supersede type
 (`@deepseek-ai/dsh-session/lib/types/known-event-types.js:21-85`):

```js
// @deepseek-ai/dsh-session/lib/types/known-event-types.js:36
    'developer/message',
// @deepseek-ai/dsh-session/lib/types/known-event-types.js:80
    'user/message',
```

Fork is therefore the designed redo-or-continue path,
 and its boundary is an inclusive event seq
 (`@deepseek-ai/dsh-api-session-controller/lib/types/client/contract/sessions.d.ts:119-125`):

```ts
// @deepseek-ai/dsh-api-session-controller/lib/types/client/contract/sessions.d.ts:119-125
     * @param opts - source session id, the optional exact inclusive boundary
     *   seq (a real event seq the caller already knows; a cut inside an open
     *   turn is balanced Host-side with synthetic closers, and omission selects
     *   the latest completed-turn prefix), and whether to increment an
     *   inherited durable title before resolving. `onCreated` observes the catalogued child before that optional rename.
```

Chain of cause:
 `UserMessageNodeView` renders copy and clock only
 (Step 3),
 no edit verb exists on the closed `ISession` surface
 (Step 1),
 the one `edit` verb names still-pending queue occurrences and text-only
 content
 (Step 2),
 and the durable log records `user/message` appends with no rewrite event
 type,
 so the only supported divergence point is an inclusive fork
 (Step 4).

## Verification

### Version under test

- `dsh --version` prints `0.2.1-alpha.4`.
- Install root:
  `/var/home/user/.local/share/mise/installs/npm-deepseek-ai-dsh/alpha/v11/7256-18df34f15ddc07b6-0`,
  npm package `@deepseek-ai/dsh@0.2.1-alpha.4`
  (`repository` field:
  [deepseek-ai/deepseek-harness][upstream],
  directory `apps/cli`).
- Assumption,
  not probed:
  the running Web GUI server boots from this same install,
  because the session runtime context names this install's
  `dsh-harness-dev-context` checkout as the implementation source.
  The running server itself was not introspected.

[upstream]: https://github.com/deepseek-ai/deepseek-harness

### Harness

One shell invocation,
 run as written against the installed dist,
 prints the capability catalogs below.
 `$DSH_PNPM` is the `node_modules/.pnpm` store of the install root named
 in the version list:

```bash
# harness: enumerate the edit capability surface of the installed dsh dist
D=/var/home/user/.local/share/mise/installs/npm-deepseek-ai-dsh/alpha/v11/7256-18df34f15ddc07b6-0/node_modules/.pnpm
S=$(ls -d "$D"/@deepseek-ai+dsh-api-session-controller@*/node_modules/@deepseek-ai/dsh-api-session-controller/lib/types)
echo '--- A: outward session verbs (session.d.ts) ---'
grep --only-matching --extended-regexp '^    [a-zA-Z]+\(' "$S/client/contract/session.d.ts"
echo '--- B: queue mutation kinds (types.d.ts:149-158) ---'
sed -n '149,158p' "$S/types.d.ts"
echo '--- C: chat icon-action locale keys ---'
sed -n '5171,5250p' "$D"/@deepseek-ai+dsh-client-ui-chat@*/node_modules/@deepseek-ai/dsh-client-ui-chat/lib/client.js | grep --only-matching 't("[^"]*")' | sort --unique
echo '--- D: any durable-message edit verb? ---'
grep --line-number --extended-regexp 'editMessage|edit-message|message\.edit|resendMessage|regenerateMessage' "$S"/*.d.ts "$S"/client/contract/*.d.ts; echo "grep exit: $?"
```

Output on 0.2.1-alpha.4:

```text
--- A: outward session verbs (session.d.ts) ---
    abandon(
    faceOf(
    beginSubmission(
    prompt(
    readAttachment(
    updateQueue(
    cancel(
    rename(
    loadOlder(
    loadThrough(
    command(
--- B: queue mutation kinds (types.d.ts:149-158) ---
/** One client-requested mutation of a still-pending queue item. */
export type QueueAction = {
    readonly kind: 'edit';
    /** Non-empty text-only replacement content. */
    readonly content: readonly TextBlock[];
} | {
    readonly kind: 'remove';
} | {
    readonly kind: 'steer';
};
--- C: chat icon-action locale keys ---
t("copied")
t("copy")
t("message.branch")
t("message.branchUnavailable")
--- D: any durable-message edit verb? ---
grep exit: 1
```

Positive control for catalog C:
 the scan reaches the action-row keys at all,
 and finds the ones that render
 (copy,
 branch),
 so the absent edit key is a real absence and not a blind scan.
 Catalog D exits 1 on purpose:
 no durable-message edit verb exists in the type surface.

### Patterns that work cleanly

- Editing a still-pending queue item's text:
  queue dock pencil,
  inline editor,
  save
  (`saveEdit` sends `{ kind: 'edit', content: [{ type: 'text', text }] }`,
  `@deepseek-ai/dsh-client-ui-conversation/lib/client.js:16938-16947`).
- Removing a still-pending queue item
  (`{ kind: 'remove' }`).
- Steer-promoting a still-pending queue item
  (`{ kind: 'steer' }`).
- Forking from the last message of a completed turn
  (branch icon on the assistant tail,
  `fork({ atSeq })` with that turn's end seq).
- Prompting in `queue` or `steer` mode,
  cancelling the running turn,
  renaming the session.

### Patterns that fail

- Editing a user message that is already in the transcript:
  no action,
  no verb,
  no event type
  (Root cause Steps 1 to 4).
- Editing a queued item that carries images or files:
  the pencil is disabled and reads
  "Contains non-text content;
  editing is not supported yet"
  (`@deepseek-ai/dsh-client-ui-conversation/lib/client.js:17067-17078`).
- Editing an admitted message by returning to the queue dock:
  the dock renders only queue rows and renders nothing when the queue is
  empty
  (`@deepseek-ai/dsh-client-ui-conversation/lib/types/client/queue/QueueDock.d.ts:14-20`).
- Branching at an arbitrary earlier message from the GUI:
  the branch icon is disabled outside a completed-turn tail with the
  tooltip "Available only on the last message of a completed turn"
  (`@deepseek-ai/dsh-client-ui-chat/lib/client.js:5223-5240`,
  `:6063-6064`).

### Live-GUI probe, attempted and blocked

A read-only DOM probe of the running GUI
 (`agent-browser open http://127.0.0.1:3080`)
 reaches only the launch-token fence page:

```text
dsh web authentication required; reopen the URL printed by dsh web.
```

The fence is the one-time launch-token exchange in
 `@deepseek-ai/dsh-client-connection/lib/index.js:276-477`
 (query key `token`,
 minted cookie after `tokenMatches`)
 and the rejection page at `:576`.
 Without the tokenized URL printed at `dsh web` startup,
 no browser session can render the transcript,
 so catalogs A to D above carry this verification instead of a DOM dump.

## Verified workarounds

### Edit the message while it is still queued

Steps:
 queue dock pencil on the pending row,
 edit the text,
 save.
 The saved text replaces the queued occurrence before the agent reaches
 it.

Tradeoffs:
 text only,
 because `QueueAction.edit` accepts `TextBlock[]` content;
 a queued item with attachments keeps its pencil disabled,
 so that item can only be removed or steer-promoted.
 The window closes at admission:
 once the agent starts the occurrence,
 it is a durable `user/message` and this path is gone.
 A failed save surfaces as the `queue.editFailed` notice,
 not as an inline revert.

### Steer the running turn

Steps:
 send the correction as a steer
 (the queue dock steer action on a pending row,
 or a steer-mode submission),
 which interrupts the running turn and inserts the correction.

Tradeoffs:
 the wrong message stays in the transcript and in the model context,
 so the correction costs tokens and attention every later request re-reads.
 Nothing is replaced;
 the conversation grows by a correction pair.

### Branch into a new conversation

Steps:
 hover the assistant reply at the end of a completed turn,
 click the branch icon
 ("Branch into a new conversation"),
 and continue in the opened child session.

Tradeoffs:
 the fork boundary is the turn-end seq and is inclusive,
 so the branched-from message and its reply stay in the child prefix;
 this is "continue from here",
 not "redo this message".
 The icon is disabled anywhere else,
 so an earlier cut is not reachable from the GUI even though
 `SessionForkRequest.atSeq` accepts any inclusive seq.
 The original session and its later tail remain,
 and tool side effects
 (file writes,
 commands)
 are not rolled back.

## Community options, reported upstream and not verified in this session

Two third-party routes for the actual rewrite-and-rerun behavior are
 described in [discussion 9422][disc-9422] and
 [discussion 9127][disc-9127];
 neither was installed or exercised here.

- [`dsh-rewind`][dsh-rewind],
  an unofficial community plugin announced by its author in discussion
  9422:
  a rewind action on a target message
  (also `/rewind`)
  withdraws that message and everything after it from the model context
  and the rendered view,
  then refills the composer with the original text.
  It works in place,
  appending to the log the way `/compact` does
  (a rewind replaces the span with an empty-message marker),
  and can optionally restore tool-modified files from a write-before
  backup.
  Tradeoffs:
  editing happens in the composer,
  not inline;
  the plugin is unofficial and unpinned here,
  so its compatibility with any given `dsh` build is unverified.
- The reference implementation in the `Myoontyee/deepseek-harness` fork
  announced in discussion 9127
  (commits `9d9544d6fc`,
  then `143b42096f` extending edits to earlier user messages):
  same-session edit and regenerate built on the append-only surface
  replacement mechanism,
  keeping original events in the log while superseded rows hide in Chat.
  Tradeoffs:
  it is a customized harness fork,
  not an installable plugin,
  so it lags upstream releases;
  its author states that running or queued work is never editable and
  that completed tool effects are not undone.

[disc-9422]: https://github.com/deepseek-ai/deepseek-harness/discussions/9422
[disc-9127]: https://github.com/deepseek-ai/deepseek-harness/discussions/9127
[dsh-rewind]: https://github.com/SiriLee/dsh-rewind

## What does not work

- Looking for an Edit control on a sent user message.
  `UserMessageNodeView` passes `text`,
  `time`,
  and `clock` only,
  and no `extraActions` seat reaches the user bubble,
  so the documented plugin action slot is not offered there either.
- Treating the branch icon as edit-and-resend.
  It appears only on a completed-turn assistant tail,
  and `fork({ atSeq })` keeps the prefix through `atSeq` inclusive.
- Editing a queued item in place while keeping its attachments.
  The edit content type is `TextBlock[]` and the dock says so.
- Rewriting the session log in place.
  `KNOWN_SESSION_EVENT_TYPES` has no user-message rewrite type,
  and discussion 9127 records the cold-read rejection
  (`developer/message does not match an open turn and step`)
  that a surface-replacement marker hit when written against closed
  turn and step scopes,
  so a log-level patch corrupts history on the next cold read.
- The live-GUI DOM probe without the `dsh web` launch-token URL
  (see the blocked probe in the Verification section).

## Upstream filing artifact

### Check for an existing issue

Upstream issues are disabled
 (`gh api repos/deepseek-ai/deepseek-harness --jq .has_issues`
 prints `false`),
 so `gh search issues` and `gh search prs` over the repository return
 empty lists for every term tried
 (`edit message`,
 `regenerate`,
 `resend`,
 `edit a message`,
 `edit previous`,
 `queue`,
 `fork`,
 `branch`).
 The project's channel is GitHub Discussions,
 named as such by `CONTRIBUTING.md`:

```text
- Identify and report issues or bugs in GitHub Discussions:
```

A Discussions search
 (`search(type: DISCUSSION)`,
 terms `edit message`)
 finds the same capability request already filed:

- [discussion 9422][disc-9422],
  "[Feature] Allow editing a sent user message and resending it (stop current thinking,
  regenerate after resend)",
  filed 2026-10-11:
  same symptom,
  same observation that the message bubble offers only Copy plus its
  timestamp and that queue-dock edit and remove are pending-only,
  plus a proposed edit-and-resend UX.
- [discussion 9127][disc-9127],
  "[Feature proposal] Edit the latest user message and regenerate in the same session",
  filed 2026-10-08:
  same capability,
  plus a community reference implementation and its cold-read bug
  history.
- [discussion 9692][disc-9692],
  filed later,
  bundles message editing with unrelated memory requests;
  no separate reading is needed here.

[disc-9692]: https://github.com/deepseek-ai/deepseek-harness/discussions/9692

`.out-of-scope/` was checked before this decision.
 No exemption covers `dsh` or this bug class:
 `claude-code-upstream-bugs.md` is scoped to the Anthropic CLI and
 `codex-harness.md` to Codex harness integrations,
 and a content search of `.out-of-scope/` for `dsh` and `deepseek` finds
 nothing.

### Upstream filing decision

1. **Is it really upstream's fault?**
   No.
   This is a product scope boundary backed by an architectural
   restriction
   (append-only log,
   queue-scoped edit),
   not a defect in a shipped control.
   The same-session rewrite is a feature request,
   already filed twice.
2. **Can upstream fix it?**
   Yes.
   Nothing algebraic blocks it:
   two community implementations reach the behavior from the existing
   surface-replacement machinery,
   and the read side already folds positional replacements.
3. **Are they supporting this use case?**
   Partly.
   Queue edit,
   steer,
   and fork ship and are documented in their package READMEs;
   same-session edit appears in no README,
   type contract,
   or test of the installed dist,
   so the requested behavior is not a supported use case today.
4. **Would the repo welcome our contribution?**
   A Discussions comment is in scope;
   `CONTRIBUTING.md` asks for issue and bug reports there.
   A patch is not:
   the same file states "We are sorry that we cannot accept external pull
   requests at the moment."
   No AI-assisted-filing ban was found in `CONTRIBUTING.md`,
   the repository description,
   or the discussion threads read.
5. **Will they likely fix it?**
   Unknown,
   and not leaning no.
   No maintainer decline appears on either thread,
   no documented won't-fix exists,
   and community activity on discussion 9127 continued through
   2026-10-20.
6. **Have we prototyped a minimal fix compatible with their
   architecture?**
   No,
   and the auto-prototype step is deliberately not run here.
   Constraints 1 and 3 do not hold,
   and constraint 4 rules out external pull requests,
   so a prototype would have no upstream landing path;
   an independently maintained community prototype already exists and is
   linked from discussion 9127
   (`Myoontyee/deepseek-harness` commits `9d9544d6fc` and `143b42096f`).
   Reimplementing it here would duplicate that work without a merge
   route.

Decision:
 do not open anything new
 (the issue tracker is disabled,
 and a second Discussions thread would be a duplicate).
 The kept artifact is the additive comment draft below,
 marked do not file as-is because the six constraints do not all hold.
 Posting it is an external action and stays parked until the workspace
 owner authorizes it.

### Kept artifact: additive comment draft (do not file as-is)

Only content missing from the existing threads is kept:
 the installed-0.2.1-alpha.4 scope of the pending-only edit path,
 and a smaller-step suggestion that reuses an existing verb.

~~~md
Two additive points from a source audit of the installed `0.2.1-alpha.4`
dist, against the symptom description above:

1. The supported edit path is narrower than "pending only":
   `QueueAction`'s `edit` variant takes `TextBlock[]` content
   (`@deepseek-ai/dsh-api-session-controller/lib/types/types.d.ts:149-158`),
   and the queue dock disables its pencil for any row carrying
   non-text content with
   "Contains non-text content; editing is not supported yet"
   (`@deepseek-ai/dsh-client-ui-conversation/lib/client.js:16746-16747`,
   `17067-17078`).
   So attachment-bearing prompts are never editable, even while queued.

2. A smaller step than same-session rewrite is already possible with the
   shipped protocol:
   `SessionForkRequest.atSeq` accepts any "exact inclusive boundary seq"
   (`@deepseek-ai/dsh-api-session-controller/lib/types/types.d.ts:319-325`),
   but the Chat branch action renders only on a completed-turn tail and
   calls `forkAt` with the turn-end seq
   (`@deepseek-ai/dsh-client-ui-chat/lib/client.js:6997-6998`).
   Surfacing "continue from before this message" on user bubbles would
   deliver redo-from-here through the existing `fork` verb,
   with no rewrite event type and no history corruption risk,
   while the inline same-session editor from #9127 proceeds separately.

For reference,
 the known-event-type catalog of this build
 (`@deepseek-ai/dsh-session/lib/types/known-event-types.js:21-85`)
 still has no user-message rewrite type,
 which matches the cold-read rejection recorded in #9127 and scopes any
 future contract: a rewrite is a surface replacement appended to the log,
 never an in-place edit.
~~~
