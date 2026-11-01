---
name: visual-design-review
description: >-
  Visual design work. Use before building or reviewing UI design variants, design matrices,
  HTML review forms, device mockups or frames, Android screen comparisons, a UI matched to a reference screenshot,
  or any custom interactive element (click or touch target size).
  Loading it is a required step before the first visual artifact, not an option.
---

# Visual design review

## Review workflow

QVE:
 Start visual review from the accepted design;
 name consequential concerns the user didn't raise and explore them as built variants,
 never a vague approval question.

QVM:
 Design matrix:
 per-variant pros,
 cons,
 analysis,
 full ranking,
 recommended additions;
 end with separable questions whose answers select among visible new variants.

MXQ:
 Size design matrices by consequential independent dimensions and meaningful variants,
 never by a number the user gave only as an example.

HFM:
 Ask visual-design questions via one self-contained,
 verified HTML form:
 built options,
 pros/cons,
 ranking,
 final free-text field.

QVS:
 UI decisions:
 before asking,
 show built screenshots of every option,
 committed as local repository files,
 never only in a network-hosted page.

RVC:
 After a decision,
 review shows only the active design;
 rejected candidates stay in docs,
 compared only on explicit request.

## Building and matching

PFG:
 Minimum padding/spacing is a hard floor:
 test fit there;
 when it fails,
 reflow or truncate permitted content,
 never a below-minimum compact fallback.

PXF:
 Screenshot-driven UI:
 measure reference geometry,
 colors,
 spacing,
 states;
 before completion,
 render result side-by-side at matching scale.
Memory isn't evidence.

DRW:
 UI work is done only after the running app was driven like a person uses it:
 hover,
 press,
 keyboard,
 light and dark,
 every screen;
 passing tests or lint is not that.

NCV:
 Verifying or measuring a GUI app hosts it in the nested compositor and reads state through its control protocol;
 bare runs and headless harnesses only check logic.

M1T:
 Multi-row Material Design 1 tabs:
 each label one content-width line plus horizontal padding;
 wrap whole tabs across rows,
 never text inside a tab.

## Target size

ATS:
 Custom interactive elements (web,
 Android,
 desktop):
 explicit min 48px/dp layout width + height;
 never rely on touch area expanding past bounds where neighbors can overlap.

HZA:
 Target size means the hit area,
 invisible padding included,
 never the visible glyph:
 measure it in source or a running probe before quoting a size.

HZP:
 Small glyphs reach the 48px target minimum by padding inside their own layout cell;
 a toolkit control whose hit cell is fixed smaller gets rebuilt with its own cell.

HZK:
 Desktop keeps the 48px target minimum:
 touch-screen desktops exist,
 so "desktop app" alone never waives it.

TXE:
 Targets under the 48px minimum:
 per-element user decision.
Propose only when the app is desktop-only,
 the action is infrequent,
 and pointer plus keyboard already do it;
 never generalize one.

### Worked example: one exception and one refusal in the same app

The read-only IDE in `package/ide/app`,
decided by the user on 2026-10-05.

- Sidebar divider:
  exception granted.
  It is a thin line with a narrow pointer grab zone,
  because the app runs on desktops only,
  every desktop has a touchpad or mouse and a keyboard,
  dragging the divider is a very infrequent action,
  and the divider already moves by mouse and by keyboard.
  The exception exists only because all of those hold together.
- Clear button in the find box:
  no exception.
  The toolkit's control had a hit cell 16px wide,
  so the box gets its own 48 by 48 cell around the same small glyph.
- Wrong conclusion to avoid:
  "target size is not a concern on this desktop app".
  The divider's exception says nothing about any other element.

## Devices and Android

ZDV:
 Device mockups:
 capture at cited physical px,
 display at 100% of cited dp,
 show px,
 dp,
 current scale + reset;
 never upscale a dp-sized bitmap.

BZF:
 Device frames:
 measured opaque chassis,
 bezels,
 hinge + corners;
 screenshot sits inside the screen opening,
 never clipped to reveal page.

ANB:
 Android mocks:
 show status + navigation bars at current target geometry (screen dimensions include them);
 keep app controls out of cutouts and insets.

AVP:
 Android screen comparisons:
 build nonfunctional Compose prototype,
 install on target emulator,
 capture each candidate at panel px,
 then present in HTML.
