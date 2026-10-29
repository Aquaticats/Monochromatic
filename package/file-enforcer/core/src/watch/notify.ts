import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l, } from '../logger.ts';
import { lazyOnceAsync, } from '../lazy-once.ts';
import { exec, } from '../pipeline/exec.ts';
import { evaluatePredicate, } from '../platform/evaluate-predicate.ts';

//region Notification tool detection

/**
 Desktop notification backends, checked in order of preference.
 - `notify-send`: Linux with libnotify
 - `osascript`: macOS
 - `pwsh` / `powershell`: Windows
 */
type NotificationTool = 'notify-send' | 'osascript' | 'pwsh' | 'powershell';

/**
 Sentinel for "detection ran but found no notification tool".
 A unique `Symbol` keeps the absent case out of a banned `T | null` union
 while staying distinguishable from every real {@link NotificationTool} value.
 */
const NO_NOTIFICATION_TOOL = Symbol('file-enforcer/watch: detection found no desktop notification tool',);

/**
 Detects the first available desktop notification tool by probing each
 candidate with {@link evaluatePredicate}.
 
 @returns Detected tool name, or {@link NO_NOTIFICATION_TOOL} if none found.
 
 @example
 ```ts
 const tool = await detectAvailableNotificationTool();
 ```
 */
async function detectAvailableNotificationTool(): Promise<NotificationTool | typeof NO_NOTIFICATION_TOOL> {
  if (await evaluatePredicate([
    'notify-send',
    '--version',
  ],))
    return 'notify-send';
  if (await evaluatePredicate([
    'osascript',
    '-e',
    'return',
  ],))
    return 'osascript';
  if (await evaluatePredicate([
    'pwsh',
    '--version',
  ],))
    return 'pwsh';
  if (await evaluatePredicate([
    'powershell',
    '-Command',
    'exit',
  ],))
    return 'powershell';

  return NO_NOTIFICATION_TOOL;
}

/**
 Lazily-detected desktop notification backend, cached for the process lifetime:
 available tools don't change between events.
 */
const notificationToolDetection = lazyOnceAsync({ compute: detectAvailableNotificationTool, },);

//endregion Notification tool detection

/**
 Sends a write-protection warning to both terminal and desktop notification
 (the latter via {@link sendDesktopNotification}).
 Called when an external process modifies a file that file-enforcer manages.
 
 @param filePath - Absolute path of the externally modified managed file
 
 @example
 ```ts
 await notifyWriteProtection('/abs/path/to/managed-file.json');
 ```
 */
export async function notifyWriteProtection(filePath: string,): Promise<void> {
  l.warn(
    `PROTECTED: "${filePath}" was modified externally -- reverting to enforced content`,
  );

  await sendDesktopNotification(filePath,);
}

/**
 Sends a desktop notification using the tool detected by
 {@link detectAvailableNotificationTool}, invoked through {@link exec}.
 Fails silently when no tool is available; the terminal warning
 is always printed regardless.
 
 @param filePath - Absolute path shown in the notification body
 */
async function sendDesktopNotification(filePath: string,): Promise<void> {
  /**
   Function-scoped logger tagged with the call site for traceable notification logs.
   */
  const rl = tagged({
    tag: sendDesktopNotification.name,
    l,
  },);
  /**
   Detected notification backend, or NO_NOTIFICATION_TOOL when no compatible tool is installed.
   */
  const tool = await notificationToolDetection.get();
  if (tool === NO_NOTIFICATION_TOOL)
    return;

  /**
   Desktop notification body kept short for readability in notification popups
   */
  const body = `"${filePath}" was modified externally and has been reverted.`;
  /**
   Notification title consistent across platforms
   */
  const title = 'file-enforcer: write protected';

  try {
    if (tool === 'notify-send') {
      await exec({
        cmd: 'notify-send',
        args: [
          '--urgency=critical',
          title,
          body,
        ],
      },);
      return;
    }
    if (tool === 'osascript') {
      await exec({
        cmd: 'osascript',
        args: [
          '-e',
          `display notification "${body}" with title "${title}"`,
        ],
      },);
      return;
    }
    // pwsh or powershell
    /**
     Escape single quotes in body/title for PowerShell string literals
     */
    const safeBody = body.replaceAll(
      "'",
      "''",
    );
    /**
     Title with single quotes doubled, matching the PowerShell literal escape used above for `safeBody`.
     */
    const safeTitle = title.replaceAll(
      "'",
      "''",
    );
    /**
     PowerShell toast notification via WinRT. No external modules needed.
     */
    const script = [
      '[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null',
      `$xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)`,
      '$texts = $xml.GetElementsByTagName("text")',
      `$texts[0].AppendChild($xml.CreateTextNode('${safeTitle}')) > $null`,
      `$texts[1].AppendChild($xml.CreateTextNode('${safeBody}')) > $null`,
      `[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('file-enforcer').Show([Windows.UI.Notifications.ToastNotification]::new($xml))`,
    ]
      .join('; ',);
    await exec({
      cmd: tool,
      args: [
        '-Command',
        script,
      ],
    },);
  }
  catch (notifyError: unknown) {
    rl.warn(`could not send desktop notification: ${String(notifyError,)}`,);
  }
}
