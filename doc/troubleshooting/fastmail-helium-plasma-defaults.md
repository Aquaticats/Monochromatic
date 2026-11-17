# Fastmail email, calendar and phone defaults in Helium 0.21.3.1 on Plasma 6.8.3

Verification note pending owner review. The local checks read the existing associations and the installed launcher.
Delivery of a compose link into a signed-in account remains unverified.

## Symptom

Fastmail is installed in Helium and offers shortcuts for Mail, Calendar, and Contacts.
Plasma's **Default Applications** page offers **Email Client**, **Calendar**, and **Phone Numbers**.
Each row has its own association, so the Calendar and Contacts shortcuts leave the related desktop inputs unhandled.

## Root cause

Plasma uses a different input type for each row.
The following excerpts are from `kcms/componentchooser/components/` in Plasma Workspace 6.8.3.

- **Email Client** uses the `mailto:` URI scheme. [`componentchooseremail.cpp:15-28`][plasma-email]

  ```cpp
  // kcms/componentchooser/components/componentchooseremail.cpp
  static const QStringList emailMimetypes{QStringLiteral("x-scheme-handler/mailto")};
  ```

- **Calendar** uses the `text/calendar` media type, which covers `.ics` files.
  [`componentchoosercalendar.cpp:9-16`][plasma-calendar]

  ```cpp
  // kcms/componentchooser/components/componentchoosercalendar.cpp
      return {QStringLiteral("text/calendar")};
  ```

- **Phone Numbers** uses the `tel:` URI scheme. [`componentchoosertel.cpp:12-25`][plasma-phone]

  ```cpp
  // kcms/componentchooser/components/componentchoosertel.cpp
  static const QStringList telMimetypes{QStringLiteral("x-scheme-handler/tel")};
  ```

On 2026-11-17, the manifest linked from the public `https://app.fastmail.com/` page declared
`mailto` with the template `/mail/compose?mailto=%s`. The `file_handlers` field was absent.
The Calendar shortcut targets `/calendar/` directly.
The installed launcher agreed with the manifest: its `MimeType` includes
`x-scheme-handler/mailto` and `x-scheme-handler/chromium`.

## Verification

The machine had Plasma Workspace `6.8.3-1.1` and Helium `0.21.3.1` (Chromium `157.0.8103.78`).
Run the following read-only checks:

```sh
# Existing XDG desktop associations
xdg-mime query default x-scheme-handler/mailto
gio mime x-scheme-handler/mailto
gio mime text/calendar
gio mime x-scheme-handler/tel
```

- `xdg-mime` and `gio` both resolved `mailto:` to the installed Fastmail desktop ID
  `chrome-nkbljeindhmekmppbpgebpjebkjbmfaj-Default.desktop`.
- `gio` reported no recommended application for `text/calendar` and no Fastmail registration for that type.
- `gio` resolved `tel:` to `org.kde.kdeconnect.handler.desktop` and reported no Fastmail registration.
- `gio mime x-scheme-handler/webcal` reported no default. Plasma's **Calendar** row selects `text/calendar`.

## Workarounds

- For an explicit email selection, open **System Settings → Default Applications → Email Client**,
  choose **Fastmail**, and apply. Fastmail also documents a browser-level registration button in
  **Settings → Mail preferences → Compose** for handling clicked email links. [Fastmail mailto help][fastmail-mailto]
- For calendar access, use the **Calendar** action in the installed Fastmail launcher.
  To bring in an event file, open Fastmail and drag the `.ics` file into its Mail or Calendar screen.
  Fastmail asks which events and calendar to use. Alarms need to be set again after import.
  The launcher action and documented import flow were inspected. The import was left untested against account data.
  [Fastmail calendar import help][fastmail-ics]
- Fastmail Contacts uses the device's default calling app when a contact's **Call** action is clicked.
  KDE Connect already held the `tel:` association on this machine. [Fastmail contacts help][fastmail-contacts]

## What does not work

Adding a desktop association can direct a file or link to a launcher. Fastmail's manifest exposes no PWA
handler for `.ics` files, `webcal:` links, or `tel:` links. The Calendar action opens a page and accepts
no file argument. For this reason, a forced association supplies no automatic event import or calling behavior.

## Upstream filing decision

No report is proposed. The unsupported inputs follow from Fastmail's manifest, and Plasma uses the input types
defined by the respective chooser rows. No triggering Helium defect was identified. Fastmail could add new app
behavior, but its documentation describes calendar import inside Fastmail and delegates calling to the device.
There is no established upstream commitment to these desktop roles, and the browser's tracker is not the feature
owner. The `.out-of-scope/` records have no matching entry. Searches of the Helium and Helium Linux issue
and pull request trackers for `mailto OR webcal` returned no related match. No upstream patch was prototyped.

[plasma-email]: https://github.com/KDE/plasma-workspace/blob/4302f699329ebb155097beaab2d5aabbb21e33b3/kcms/componentchooser/components/componentchooseremail.cpp#L15-L28
[plasma-calendar]: https://github.com/KDE/plasma-workspace/blob/4302f699329ebb155097beaab2d5aabbb21e33b3/kcms/componentchooser/components/componentchoosercalendar.cpp#L9-L16
[plasma-phone]: https://github.com/KDE/plasma-workspace/blob/4302f699329ebb155097beaab2d5aabbb21e33b3/kcms/componentchooser/components/componentchoosertel.cpp#L12-L25
[fastmail-mailto]: https://www.fastmail.help/hc/en-us/articles/360058753694-Open-mailto-links-in-Fastmail#:~:text=In%20the%20Compose%20section
[fastmail-ics]: https://www.fastmail.help/hc/en-us/articles/360060590773-Import-export-your-calendars#:~:text=Drag%20and%20drop%20the%20.ics
[fastmail-contacts]: https://www.fastmail.help/hc/en-us/articles/1500000280101-Using-Contacts#:~:text=your%20device%27s%20default%20calling%20or%20messaging%20programs
