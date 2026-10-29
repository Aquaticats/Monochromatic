# iOS Safari ignores `touch-action: none` for certain gesture patterns, firing `pointercancel` mid-gesture and breaking canvas pointer handlers

## Symptom

Code that works on Chrome,
 Firefox,
 and desktop Safari silently
breaks on every iPhone and iPad:

- Tap-to-zoom,
   drag-to-pan,
   and other single-pointer gestures
  do nothing.
- Long-press gestures trigger the iOS context menu instead of
  the application action.
- `pointerup` never fires after `pointerdown`:
   the browser
  swallows it via `pointercancel`.
- Draw/paint strokes cut off mid-gesture when iOS decides the
  touch is a scroll.
- The failure is silent:
   no console error,
   no warning,
   no
  indication the CSS property was not honoured.

Affects interactive canvases,
 drawing surfaces,
 map controls,
custom gesture handlers;
 anything relying on
`touch-action: none` to suppress native gestures.

## Root cause

iOS Safari claims to support `touch-action: none` since iOS 13
(2019) but does not reliably honour it.
 The browser can still
intercept touch sequences for its own gesture recognition
(scroll,
 page zoom,
 context menu,
 text selection),
 firing
`pointercancel` and killing the application's pointer event
stream mid-gesture.

The W3C Pointer Events specification states that
`touch-action: none` must prevent the browser from consuming
touch input for built-in behaviours (scrolling,
 pinch-zoom,
navigation gestures).
 When the browser decides to handle a
touch natively,
 it fires `pointercancel` so the application
abandons the gesture.
 WebKit fires `pointercancel` even when
`touch-action: none` is set on the target element,
 violating
the specification.

The result:
 `touch-action: none` means "none" on Chromium and
Gecko;
 on WebKit it means "some,
 whenever we feel like it".

### Timeline of neglect

WebKit bug
[133112](https://bugs.webkit.org/show_bug.cgi?id=133112) tracks
`touch-action` support:

- 2014:
   WebKit bug filed requesting `touch-action` support.
- 2014:
   Chrome 36 shipped complete `touch-action` support.
- 2015:
   WebKit implements `touch-action: manipulation` (the
  easy one;
   only disables double-tap zoom).
- 2017:
   Firefox 52 shipped complete `touch-action` support.
- 2019:
   WebKit adds `touch-action: pan-y` (iOS 13);
   `none`
  partially implemented.
- 2022:
   Safari 15.5 fixes the `setPointerCapture` release bug
  (an earlier WebKit bug that prevented pointer capture from
  working at all on iOS).
- 2025:
   `touch-action: none` still fires `pointercancel` for
  certain gesture sequences on real devices.
- 2026:
   workaround still required.
   The original bug was
  resolved as "CONFIGURATION CHANGED" rather than "FIXED".
  Twelve years after filing.

## Verification

Versions under test:

- iOS Safari on iOS 15,
   17,
   18 (latest at time of writing).
- Chrome,
   Firefox,
   and desktop Safari for comparison;
  workarounds verified to not break the working browsers.

Reproduce:
 add a canvas with `touch-action: none` and a
`pointerdown`/`pointermove`/`pointerup` handler.
 On any iPhone or
iPad,
 observe `pointercancel` firing during certain gesture
sequences (especially drag-after-long-press,
 fast pan,
 or
multi-touch start).
 On every other browser,
 no `pointercancel`
fires.

Verified fix applied in this project in
`package/doodle-widget/app/src/client/pointer-handlers-zoom.ts`
and `package/doodle-widget/app/src/styles.ts`.
Tested on a physical iPhone running iOS Safari;
 all three
workaround changes (below) are required;
 removing any one
causes the zoom tool to fail on iOS.

## Verified workaround (three changes; all required)

CSS alone is not sufficient.

### 1. `touchstart` and `touchmove` listeners with `preventDefault()`

The actual fix:
 prevent default on the native touch events
before iOS enters gesture recognition,
 which is before pointer
events are generated.
 Listeners must be registered with
`{ passive: false }` because iOS Safari defaults touch listeners
to passive.

```ts
canvas.addEventListener('touchstart',
  function handleTouchStart(event: TouchEvent,): void {
    if (shouldSuppressNativeGestures())
      event.preventDefault();
  }, { passive: false, },);

canvas.addEventListener('touchmove',
  function handleTouchMove(event: TouchEvent,): void {
    if (shouldSuppressNativeGestures())
      event.preventDefault();
  }, { passive: false, },);
```

Guard `preventDefault()` behind a condition (e.g. active tool
mode) so scrolling and native gestures still work when the
application does not need to suppress them.

Tradeoff:
 every interactive surface must register both
listeners;
 missing one re-creates the failure.
 The
`{ passive: false }` form blocks the main thread if the handler
is slow,
 so the body must stay short.

### 2. `event.preventDefault()` on `pointerdown`

Secondary defence:
 prevents iOS from cancelling the pointer
event sequence after `pointerdown` fires.

```ts
canvas.addEventListener('pointerdown',
  function handlePointerDown(event: PointerEvent,): void {
    event.preventDefault();
    canvas.setPointerCapture(event.pointerId,);
    // ... gesture logic
  },);
```

Tradeoff:
 prevents the default action (text selection,
 link
follow) even on desktop.
 Acceptable on a canvas;
 not on
ordinary text elements.

### 3. `-webkit-touch-callout: none` CSS

Suppresses the iOS long-press context menu (link preview,
copy/paste popover) that fires after ~500 ms of holding.
Without this,
 any long-press gesture (e.g. long-press-to-zoom-
out) triggers the system callout instead.

```css
#my-canvas {
  touch-action: none; /* still set; works on every other browser */
  -webkit-touch-callout: none; /* iOS long-press menu suppression */
}
```

Tradeoff:
 WebKit-only prefix;
 harmless on other browsers but
adds a prefix to the maintenance surface.

## What does not work

- **`touch-action: none` alone**:
   iOS ignores it for certain
  gesture patterns.
- **`event.preventDefault()` on `pointerdown` alone**:
  insufficient;
   iOS intercepts at the touch-event level before
  pointer events are generated.
- **`user-scalable=no` in the viewport meta tag**:
   suppresses
  page zoom but does not prevent `pointercancel` from firing on
  canvas gestures.
- **`setPointerCapture` alone**:
   Safari had a separate bug
  (fixed in 15.5) where captured pointers did not receive
  `pointermove`/`pointerup` if the contact point moved
  off-element.
   Even after that fix,
   capture alone does not
  prevent `pointercancel`.

## Why this keeps catching people

Every tutorial and MDN page says "set `touch-action: none` to
prevent the browser from handling touch input.
" Correct on
Chromium and Gecko;
 a half-truth on WebKit.
 Developers write
`touch-action: none`,
 test on desktop,
 ship,
 and then get bug
reports from every iPhone user.
 The failure is silent.

## Why we do not file this upstream

Already filed (multiple times across twelve years).

1. **Is it really upstream's fault?
   ** Yes;
    WebKit fires
   `pointercancel` in violation of the W3C Pointer Events spec.
2. **Can upstream fix it?
   ** Yes.
    The change would land in
   WebKit's gesture-recognition layer.
3. **Are they supporting this use case?
   ** Documented support
   exists since iOS 13;
    the gap is enforcement,
    not API
   surface.
4. **Will they likely fix it?
   ** History suggests no. The
   tracking bug has been open for twelve years;
    closed as
   "CONFIGURATION CHANGED" rather than "FIXED".
5. **Have we prototyped a minimal fix?
   ** No (closed source).

Decision:
 no new upstream report from us.
 The three-change
workaround is well-known and ships in this project;
 see the
verified-fix paths cited above.
