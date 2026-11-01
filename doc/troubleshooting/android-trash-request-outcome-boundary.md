# Android SDK 37 trash requests and comparable provider results are not per-item success evidence

## Symptom and scope

The light error/Undo design audit found that a request being generated,
accepted or finished can be mistaken for a successfully trashed item.
This is a source/evidence boundary,
not an observed storage failure on the user's device.
No real file was trashed,
restored or deleted and no storage permission was requested.

A successful-trash toast and its Undo action require a known result for
the affected item and an operation owner able to attempt restoration.
Merely receiving `RESULT_OK` does not supply that whole contract in the
comparable provider path inspected here.
Completion,
failure and restoration capability remain separate facts.

## SDK deciding source

The installed source is
`${ANDROID_HOME}/sources/android-37.0/android/provider/MediaStore.java`.
Its `createTrashRequest` documentation at lines 2027 to 2071 describes
request generation and sequencing:

```java
// android/provider/MediaStore.java, API37 SDK
public static @NonNull PendingIntent createTrashRequest(@NonNull ContentResolver resolver,
        @NonNull Collection<Uri> uris, boolean value) {
    final ContentValues values = new ContentValues();
    if (value) {
        values.put(MediaColumns.IS_TRASHED, 1);
    } else {
        values.put(MediaColumns.IS_TRASHED, 0);
    }
    return createRequest(resolver, CREATE_TRASH_REQUEST_CALL, uris, values);
}
```

The comments specify that this generates the request for a prompt;
when approval applies,
the operation finishes before the activity result is delivered.
That is a sequencing guarantee,
not an explicit per-item success report.
The SDK declaration alone does not establish the exact installed
MediaProvider implementation.

The same installed source's flagged path-based `trashFile` and
`restoreFileFromTrash` APIs require declared and granted
`MANAGE_EXTERNAL_STORAGE`.
The current music-player manifest declares neither that permission nor
`MANAGE_MEDIA`.
No broader storage-access mechanism is selected by this audit.

`DocumentsContract.java:576` exposes provider deletability;
its `deleteDocument` calls provider deletion.
That is not a documented reversible trash/restore contract.
Read grants and delete capability do not manufacture an Undo owner.

## Comparable provider trace

The read-only clone of
`aosp-mirror/platform_packages_providers_mediaprovider`
was inspected at `a183e2b92c71d86ec7b104c116ef3f4dbcbb523f`.
It is comparable source,
not proven equivalent to the captured API37 image.

`src/com/android/providers/media/MediaProvider.java:8057` to `8119`
validates requested URIs and permitted columns,
then returns a one-shot activity PendingIntent:

```java
// src/com/android/providers/media/MediaProvider.java
final Intent intent = new Intent(method, null, context, PermissionActivity.class);
intent.putExtras(extras);
return PendingIntent.getActivity(context, PermissionActivity.REQUEST_CODE, intent,
        FLAG_ONE_SHOT | FLAG_CANCEL_CURRENT | FLAG_IMMUTABLE, options.toBundle());
```

`PermissionActivity.java:214` to `234` conditionally skips its action
prompt rather than universally displaying one:

```java
// src/com/android/providers/media/PermissionActivity.java
if (!shouldShowActionDialog) {
    onPositiveAction(null, 0);
    return;
}
```

The permission predicate includes read access,
`MANAGE_MEDIA` and other operation-specific checks.
That permission is not a sufficient universal dialog-skipping condition.
No source claim that Android always prompts or never permits direct
operations is made.

In the positive operation task at `PermissionActivity.java:402` to `435`,
trash updates allow per-item exceptions,
the batch results are not inspected,
and the catch logs a caught exception:

```java
// src/com/android/providers/media/PermissionActivity.java
ops.add(ContentProviderOperation.newUpdate(uri)
        .withValues(values)
        .withExtra(MediaStore.QUERY_ARG_ALLOW_MOVEMENT, true)
        .withExceptionAllowed(true)
        .build());
getContentResolver().applyBatch(MediaStore.AUTHORITY, ops);
```

The task's `onPostExecute` then sets `RESULT_OK` without a per-item result
condition:

```java
// src/com/android/providers/media/PermissionActivity.java
setResult(Activity.RESULT_OK);
mHandler.removeCallbacks(mShowProgressDialogRunnable);
```

This excerpt quotes the task's leading result statements;
the subsequent progress-dialog completion handling is not shown.
Negative/cancelled activity paths remain distinct.
The activity does not universally return `RESULT_OK`.
The supported conclusion is that this positive-result path does not itself
establish successful trash for each requested item.
It does not prove that every provider loses failures or that a particular
read-back mechanism is mandatory.

## Fixture verification and design correction

The initial authored fixture used `completed` as its success marker.
It did not implement a production `RESULT_OK` adapter,
so the finding corrects an ambiguous fixture contract rather than a
production bug.
The revised fixture uses `verified-success` for explicitly authored
per-item success and rejects the retired `completed` token.
`approved-but-unverified` represents an accepted or finished request
whose item outcome is not established;
it is neither failure nor necessarily a pending task or a human click.

Controls include:

- Authored `verified-success`,
  live handle and unexpired interval:
  permits the study's Undo-bearing state.
- The same success without a handle or after expiry:
  does not provide actionable Undo.
- Pending,
  accepted-but-unverified,
  cancelled or failed outcomes with a handle and live interval:
  do not become successful-trash Undo feedback.
- Unknown or retired completion-only tokens:
  throw rather than select a success-looking fallback.

This implements no provider query,
verification deadline,
retry,
permission acquisition or storage mutation.
Success without an Undo handle is not reclassified as failed trash.
The revised 15-test report passed with zero failures,
errors or skips.
Fresh unknown-scene,
pending-outcome and accepted-but-unverified guard-removal mutants failed
the intended targeted assertions.
Exact restoration and the complete unit task passed.
The earlier 13-test proof remains separate evidence for the retired
fixture contract,
not a replacement for these revised reports.
The native fit renderer remains a separate task.

## What does not work

- Promoting request construction directly to a successful-trash toast.
- Treating finished-request sequencing as a per-item success report.
- Treating provider deletion as reversible trash.
- Treating a restore-handle premise as proof that an actual restore will
  succeed or require no further consent.
- Using D8's app-confirmation decision as authority to bypass OS security
  or adopt broader storage access.

## Upstream filing decision

No upstream issue,
comment or patch is prepared or sent.
The `.out-of-scope/` check and duplicate search are not a bypass here:
this audit establishes an evidence contract,
not an upstream defect requiring a change.

- Fault:
  no defect is proved in the installed provider;
  documented sequencing and comparable implementation are narrower than
  the UI success inference.
- Ability:
  providers can expose result information,
  but no architectural impossibility or required redesign is claimed.
- Supported input:
  the MediaStore request API and provider document operations are
  documented supported operations with distinct ownership requirements.
- Contribution policy:
  no external contribution path was assessed because no patch is
  proposed.
- Likely action:
  no maintainer decision about this inference was established.
- Prototype:
  only the repository-owned authored fixture contract is corrected;
  no third-party source edit or upstream fix is proposed.

The verified design study must retain its authored-state and implementation
boundary rather than turn these source findings into a new product policy.
