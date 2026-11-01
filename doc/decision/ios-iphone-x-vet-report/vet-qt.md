# iOS source-audit: Qt

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Verbatim structured output from the 2026-06-12 `ios-framework-desk-audit` fan-out (16 parallel
source audits against the three iOS platform walls plus the kopia/music-player functional
requirements),
 with an adversarial cite-check.
 Not lint-conformed.
 On-device gate results,
 where
run,
 live in `device-gate-results.md`;
 the cross-cutting decision is in
`../ios-iphone-x-music-player-kopia-stack.md`.

## Verdict

- iOS runtime model:
   Mixed AOT-native + bytecode-interpreter.
   C++ application and library code (and Qt itself) is AOT-compiled by clang to native ARM64 and linked statically
    into the app bundle,
   so app logic runs as ordinary native code with no executable memory.
   QML/JavaScript runs on Qt's V4 engine,
   which on iOS uses its bytecode INTERPRETER backend because JIT is disabled (App Store policy / no W^X).
   The Qt Quick Compiler (qmlcachegen for bytecode+partial-C++ units,
   qmltc for QML-type-to-C++) AOT-compiles document structure and many bindings/functions to C++ at build time,
   but functions it cannot fully resolve still execute as V4 bytecode on the interpreter.
   Engine:
   Qt V4 (interpreter backend) for QML/JS;
   clang AOT for C++.
- Minimum iOS deployment:
   Qt 6.11:
   iOS 17 (default,
   do not lower;
   lowering risks runtime crashes).
   Qt 6.5 LTS:
   iOS 14.
   Pick the Qt version by target-device floor:
   Qt 6.5 LTS for the iPhone X (iOS 16.7 max),
   Qt 6.11 for iOS 17+ devices.
- Gate expectation:
   needs-device
- Confidence:
   high
- Key finding:
   Qt clears all three walls without a fatal blocker:
   C++ core is AOT-native,
   QML/JS runs on the V4 BYTECODE INTERPRETER (JIT disabled on iOS) which needs no executable memory
    and is therefore allowed (a performance tax,
   not the DENY_EXECMEM death that killed NativeScript),
   and because Qt-for-iOS is a fully static build,
   linking a C-ABI Rust/Go staticlib for kopia and symphonia is trivial extern \"C\" linkage.
   The real gate friction is non-wall:
   Qt 6.11 requires iOS 17,
   so it cannot install on the iPhone X gate device (iOS 16.7 max);
   the probe must run on an iOS 17+ phone or pin Qt 6.5 LTS (iOS 14+).

## Wall 2: JIT / executable memory

Verdict:
 interpreter-fallback

Qt's V4 JavaScript engine has two backends,
 an interpreter (running bytecode) and a JIT (JavaScriptCore macro assembler).
 On iOS the JIT is disabled because the platform forbids it (wiki.
qt.
io/V4:
 'Trouble only occurs if the platform itself restricts usage of JIT,
 be it App Store policies as for iOS').
 A bytecode interpreter executes by dispatching over precompiled native opcodes and needs NO writable-executable memory,
 so it is permitted on iOS,
 unlike NativeScript's V8-JIT that tripped DENY_EXECMEM.
 The consequence is performance,
 not viability:
 the V4 runtime emits 'JIT is disabled for QML.
 Property bindings and animations will be very slow.
' Qt Quick Compiler AOT-compiles structure and many bindings to C++ at build time,
 reducing (not eliminating) interpreted bytecode.
 C++ application/core code is pure AOT-native and never touches execmem.
 Net:
 Qt passes wall 2;
 QML-heavy UIs pay an interpreter performance tax that AOT QML compilation and pushing logic into C++ mitigate.

Source:
 <https://wiki.qt.io/V4> (interpreter vs JIT backends;
 iOS App Store forbids JIT;
 QV4_FORCE_INTERPRETER;
 'JIT enabled is important to get fluid animations');
 <https://doc.qt.io/qt-6/qtqml-qtquick-compiler-tech.html> (pre-built compilation units:
 'Byte code for all functions and bindings' + 'C++ code for functions and bindings the compiler fully understands',
 else 'interpreted directly from the byte code,
 or compiled to machine code via a JIT');
 /tmp/agent/qt-audit-20260612/src/corelib/global/qsystemdetection.
h:
60-71 (Q_OS_IOS via TARGET_OS_IPHONE)

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 Qt for iOS is a fully STATIC build:
 'everything is compiled statically and placed into the application bundle.
 The applications are sandboxed inside their bundles and cannot make use of shared object files',
 so Qt libs and Qt plugins are static-linked via QTPLUGIN/Q_IMPORT_PLUGIN.
 Adding one more static library is the native path:
 compile kopia as a Go gomobile c-archive (.
a/.
xcframework) or Rust staticlib exposing a C ABI,
 declare it extern "C" in C++ (or a thin .
mm shim),
 and link it into the qt_add_executable target.
 No FFI bridge layer is needed because the app is already C/C++;
 the Rust/Go .
a is just another object in the static link.
 The qios*.
mm files in the clone (Objective-C++ calling UIKit/AVFoundation) prove Qt-on-iOS freely mixes C,
 C++,
 ObjC and native frameworks in one binary,
 which is the same interop used to call a C-ABI staticlib.

Source:
 <https://doc.qt.io/qt-6/ios-platform-notes.html> (static-only:
 everything compiled statically,
 sandboxed,
 no shared objects,
 plugins via QTPLUGIN/Q_IMPORT_PLUGIN);
 /tmp/agent/qt-audit-20260612/mkspecs/macx-ios-clang/qmake.
conf (uikit clang static toolchain);
 /tmp/agent/qt-audit-20260612/src/plugins/platforms/ios/qiosintegration.
mm and qiosclipboard.
mm (.
mm native C++/ObjC interop demonstrated)

## Wall 3: background execution

Qt provides NO abstraction over iOS background execution.
 Qt only reports lifecycle:
 UIApplicationDidEnterBackground / NSExtensionHostDidEnterBackground are mapped to Qt:
:
ApplicationSuspended,
 i.e. once iOS backgrounds the app Qt's event loop is suspended.
 There is no Qt equivalent of background URLSession or BGTaskScheduler.
 To get any background transfer you drop to Objective-C++ (.
mm) and call NSURLSession (background configuration) and BGProcessingTaskRequest/BGTaskScheduler directly,
 using the same C++/ObjC interop the qios*.
mm files use.
 A multi-hour kopia snapshot cannot run as a foreground service:
 it must be restructured so kopia streams chunks into a background NSURLSession upload (idle/charging,
 OS-scheduled),
 with the in-app HTTP endpoint and snapshot loop chunked to survive suspension.
 This is identical in spirit to the Android wall-3 restructuring,
 just expressed against URLSession/BGTask instead of a foreground service.

Source:
 /tmp/agent/qt-audit-20260612/src/plugins/platforms/ios/qiosapplicationstate.
mm:
33-92 (UIApplicationDidEnterBackgroundNotification -> UIApplicationStateBackground -> Qt:
:
ApplicationSuspended);
 Apple URLSession/BGTaskScheduler are native APIs with no Qt binding (Qt docs expose no background-task class)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 Use QtNetwork QTcpServer (BSD-socket based,
 works on iOS) to bind a localhost listener and implement the S3/HTTP endpoint kopia targets in C++;
 kopia,
 linked as a static lib in the same process (wall 1),
 points its repository URL at 127.0.0.1:
<port>.
 No bundled-binary exec is involved;
 it is an in-process C++ server.
 Alternatively run a small C/Rust HTTP server from the linked staticlib directly.
 QTcpServer/QTcpSocket are part of QtNetwork which builds and links statically on iOS.

Source:
 <https://doc.qt.io/qt-6/qtcpserver.html> (QTcpServer listen/socket API);
 Qt for iOS static link of QtNetwork per <https://doc.qt.io/qt-6/ios-platform-notes.html>

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 QtNetwork QNetworkAccessManager / QSslSocket provide a streaming HTTPS client for the pCloud target.
 On iOS the TLS backend is Apple SecureTransport (the securetransport TLS plugin is DEFAULT_IF APPLE),
 not bundled OpenSSL,
 so HTTPS works against the system trust store without shipping OpenSSL.
 QNetworkReply streams response/request bodies (readyRead / sequential QIODevice)
  so large transfers can be streamed rather than buffered.
 For background,
 the streaming must be re-expressed on native NSURLSession (see wall 3);
 in-foreground QNetworkAccessManager suffices.

Source:
 /tmp/agent/qt-audit-20260612/src/plugins/tls/securetransport/CMakeLists.
txt:
8 (QSecureTransportBackendPlugin DEFAULT_IF APPLE);
 /tmp/agent/qt-audit-20260612/src/plugins/tls/CMakeLists.
txt:
4 (securetransport gated on QT_FEATURE_securetransport);
 <https://doc.qt.io/qt-6/qnetworkaccessmanager.html>

## Audio (music-player port)

Decode:
 symphonia 0.6 (all features) + libopus stays in the Rust core,
 reused as-is.
 It is pure Rust with no platform audio dependency,
 so it cross-compiles to arm64-apple-ios and is called over the C ABI exactly like wall-1 kopia.
 No re-decode on AVFoundation/AudioToolbox is needed;
 Qt's own codec support is irrelevant because decoding is done in Rust.

Output:
 Two viable native paths,
 neither requiring an AVAudioEngine rewrite.
 (1) Keep cpal 0.18:
 it has an iOS/CoreAudio backend,
 so the existing Rust output path can run unchanged.
 (2) Hand decoded PCM to Qt's QAudioSink,
 whose low-latency callback API is CoreAudio-backed on iOS ('only available on platforms that support the callback API:
 Apple's CoreAudio (macOS,
 iOS,
 etc)');
 the callback runs on a soft-realtime audio thread and must not block/allocate.
 True-peak normalization and the peak cache stay in Rust.
 Note the qtaudio_coreaudio plugin must be Q_IMPORT_PLUGIN'd for low-level audio on iOS.

Rust core reuse:
 reused-via-ffi:
 symphonia + true-peak + peak-cache + queue/session logic remain Rust behind a C ABI;
 cpal's iOS CoreAudio backend can drive output directly,
 or PCM is bridged into QAudioSink's CoreAudio callback.
 AVAudioEngine rewrite is NOT required.
 Caveat:
 the Slint UI is NOT reusable under Qt and must be rebuilt in Qt Quick (QML) or Qt Widgets;
 only the audio/data core is portable.

Source:
 <https://doc.qt.io/qt-6/qaudiosink.html> (QIODevice pull mode + callback low-latency mode;
 iOS CoreAudio);
 <https://wiki.qt.io/QtMultimedia_iOS> (QTPLUGIN += qtaudio_coreaudio for low-level QAudioOutput/QAudioInput on iOS);
 cpal iOS CoreAudio backend per task brief / cpal docs

## Gate probe and toolchain

Minimal app:
 a Qt Quick (or Qt Widgets) iOS app built with qt_add_executable that (a) shows one QML screen
  to exercise the V4 interpreter,
 (b) links a tiny Rust C-ABI staticlib and calls one extern \"C\" function from C++ to print a string (proves wall 1),
 and (c) opens a QTcpServer on localhost and round-trips one request (proves in-app HTTP).
 On-device signal:
 the app launches and renders on a physical iPhone;
 the QML screen animates (confirming V4 interpreter runs without an execmem kill,
 even if labeled slow);
 console shows the Rust function's return value;
 the localhost request returns 200.
 DEVICE CONSTRAINT:
 the iPhone X gate device maxes at iOS 16.7.
x (A11 chip,
 never gets iOS 17),
 but Qt 6.11 sets minimum deployment to iOS 17 and warns lowering it crashes at runtime,
 so a Qt 6.11 binary will NOT install on the iPhone X.
 Resolution:
 either gate on an iOS 17+ device,
 or pin Qt 6.5 LTS (supports iOS 14+) to test on the iPhone X.
 Without that adjustment the gate cannot run on the assigned device.

Toolchain:
 macOS with Xcode + iOS SDK and a valid signing identity/provisioning profile.
 Qt for iOS prebuilt static libraries installed via the Qt online installer (qt-unified).
 For the iPhone X gate device pin Qt 6.5 LTS (iOS 14+);
 for current/iOS-17 devices use Qt 6.11 (iOS 17+).
 CMake (qt_add_executable / qt-cmake) or qmake to generate the .
xcodeproj.
 Rust:
 rustup target add aarch64-apple-ios (and x86_64/arm64 sim) to build the kopia/symphonia staticlib;
 or Go with gomobile for a kopia c-archive xcframework.
 cargo-lipo/xcframework tooling to package the Rust lib.

## Supporting-stack vets this framework drags in

- Wall-1 deep vet:
   build kopia as a Go gomobile c-archive xcframework (cgo,
   arm64-apple-ios) OR a Rust staticlib,
   define the C-ABI surface,
   and confirm extern "C" link + call from a static Qt iOS binary;
   verify Go runtime + GC behave inside a backgrounded iOS app
- QML AOT performance vet:
   measure V4-interpreter UI smoothness vs Qt Quick Compiler (qmlcachegen/qmltc) C++ output on device;
   decide how much logic must move from QML/JS into C++ to hit acceptable frame times
- Wall-3 background vet:
   native .
  mm bridge to NSURLSession background sessions + BGTaskScheduler;
   restructure the kopia snapshot into chunked,
   suspension-survivable uploads;
   verify behavior across app suspension
- In-app HTTP endpoint vet:
   QTcpServer S3/HTTP endpoint correctness and throughput feeding kopia in-process on localhost
- HTTPS streaming vet:
   QNetworkAccessManager/QSslSocket over Apple SecureTransport against pCloud,
   streaming large bodies without full buffering
- Audio core FFI vet:
   drive symphonia-decoded PCM through cpal's iOS CoreAudio backend OR QAudioSink's CoreAudio callback;
   validate true-peak normalization + on-disk peak cache + low-latency,
   no-allocation audio callback;
   confirm qtaudio_coreaudio plugin import
- UI port vet:
   rebuild the Slint music-player UI in Qt Quick/Widgets (Slint is not reusable under Qt);
   scope the two-axis pagination,
   folder-scanned queue,
   session persistence in QML/C++
- QA vets:
   in-process UI test via Qt Quick Test / Squish,
   end-to-end on-device via XCUITest driving the built app,
   fuzzing the C-ABI boundary and HTTP endpoint,
   property tests on the Rust core (proptest),
   mutation testing of the Rust audio/normalization core (cargo-mutants)

## Cited sources

- V4 has interpreter and JIT backends;
   iOS forbids JIT so the interpreter is used;
   this is a performance hit:
   <https://wiki.qt.io/V4> (App Store policies for iOS restrict JIT;
   QV4_FORCE_INTERPRETER;
   'JIT enabled is important to get fluid animations')
- Qt Quick Compiler emits pre-built compilation units (bytecode + partial C++);
   remainder is interpreted bytecode or JIT:
   <https://doc.qt.io/qt-6/qtqml-qtquick-compiler-tech.html>
- Qt for iOS is fully static:
   everything compiled statically into the bundle,
   sandboxed,
   no shared objects,
   plugins via QTPLUGIN/Q_IMPORT_PLUGIN:
   <https://doc.qt.io/qt-6/ios-platform-notes.html>
- iOS platform plugin maps background notifications to Qt:
  :
  ApplicationSuspended;
   native .
  mm interop demonstrated:
   /tmp/agent/qt-audit-20260612/src/plugins/platforms/ios/qiosapplicationstate.
  mm:
  33-92;
   src/plugins/platforms/ios/qiosintegration.
  mm
- Apple SecureTransport is the default iOS TLS backend (not bundled OpenSSL):
   /tmp/agent/qt-audit-20260612/src/plugins/tls/securetransport/CMakeLists.
  txt:
  8 (DEFAULT_IF APPLE)
- QAudioSink has QIODevice pull mode and a low-latency callback mode that is CoreAudio-backed on iOS:
   <https://doc.qt.io/qt-6/qaudiosink.html>
- Low-level audio on iOS requires the qtaudio_coreaudio plugin for QAudioOutput/QAudioInput:
   <https://wiki.qt.io/QtMultimedia_iOS>
- Qt 6.11 minimum deployment is iOS 17 and must not be lowered (runtime crashes):
   <https://doc.qt.io/qt-6/ios.html> (supported configurations:
   iOS 17 or higher)
- Qt 6.5 LTS supports iOS 14 or higher:
   <https://doc.qt.io/qt-6.5/supported-platforms.html>
- iPhone X (A11) is capped at iOS 16.7.
  x and cannot run iOS 17:
   <https://iosref.com/ios> and Apple support (A12+ required for iOS 17)
- Qt iOS builds via uikit clang static toolchain (arm64),
   generating an Xcode project:
   /tmp/agent/qt-audit-20260612/mkspecs/macx-ios-clang/qmake.
  conf;
   <https://doc.qt.io/qt-6/ios.html>

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   Wall-1 SOURCE MIS-ATTRIBUTION (substance correct,
   URL wrong):
   The load-bearing static-only quote -- "everything is compiled statically and placed into the application bundle.
   The applications are sandboxed inside their bundles and cannot make use
    of shared object files" plus the statically-linked-plugins + QTPLUGIN claim -- is cited
    to <https://doc.qt.io/qt-6/ios-platform-notes.html>.
   That page does NOT contain this language (I fetched it twice;
   it has no occurrence of "static",
   "shared object",
   or "sandbox";
   its sections are Deployment/`Info.plist`/Application Assets/Publishing,
   etc.).
   The exact quoted text is REAL and verbatim-correct Qt documentation,
   but it lives on a DIFFERENT page:
   <https://doc.qt.io/qt-6/porting-to-ios.html> (confirmed verbatim on qt-6.8 and across Qt 5.7 through 6.10),
   which states "In Qt for iOS,
   everything is compiled statically and placed into the application bundle.
  ",
   "The applications are 'sandboxed' inside their bundles and cannot make use of shared object files.
  ",
   "Because of this,
   also the plugins used by the Qt modules need to be statically linked.
  ",
   and "define the required plugins using the QTPLUGIN variable.
  " Correction:
   change the wall-1 citation from ios-platform-notes.
  html to porting-to-ios.
  html.
   The wall-1 verdict (feasible=yes,
   static-lib link path) stands on substance.
   Minor:
   the cited .
  mm file qiosintegration.
  mm imports AudioToolbox (#import <AudioToolbox/AudioServices.
  h>),
   not UIKit/AVFoundation as the cite wording implies;
   the C/C++/ObjC++ interop point it is offered for still holds.
   ANALYTICAL-BRIDGE NOTE (not a defect,
   no verdict impact):
   the wall-2 "bytecode interpreter needs NO writable-executable memory / DENY_EXECMEM" framing is the audit's own
    correct inference;
   the cited wiki.
  qt.
  io/V4 page contains zero mention of W^X,
   execmem,
   or DENY_EXECMEM.
   The runtime-message quote "JIT is disabled for QML.
   Property bindings and animations will be very slow.
  " is correctly attributed to the V4 RUNTIME (not the wiki) and is a genuine,
   widely-documented Qt-on-iOS warning that itself links to wiki.
  qt.
  io/V4;
   this is NOT a mis-attribution.
- Sources checked:
   /tmp/agent/qt-audit-20260612/src/corelib/global/qsystemdetection.
  h (lines 50-79:
   Q_OS_IOS defined via TARGET_OS_IPHONE block lines 60-71 -- VERIFIED accurate);
   /tmp/agent/qt-audit-20260612/mkspecs/macx-ios-clang/qmake.
  conf (includes uikit/clang.
  conf + uikit/qmake.
  conf -- VERIFIED;
   word 'static' not literally in this file);
   /tmp/agent/qt-audit-20260612/src/plugins/platforms/ios/qiosintegration.
  mm (ObjC++ mixing C++ includes with #import <AudioToolbox/AudioServices.
  h> -- interop VERIFIED);
   /tmp/agent/qt-audit-20260612/src/plugins/platforms/ios/qiosclipboard.
  mm (exists,
   ObjC++ -- VERIFIED present);
   grep of qtbase clone for 'JIT is disabled'/'very slow'/'QV4_FORCE_INTERPRETER' -- ABSENT (expected:
   V4 engine lives in qtdeclarative,
   not in this qtbase-only clone);
   <https://wiki.qt.io/V4> (interpreter+JIT backends,
   iOS App Store forbids JIT,
   QV4_FORCE_INTERPRETER,
   'Having JIT enabled is important to get fluid animations' -- ALL VERIFIED verbatim;
   QV4_* vars documented:
   QV4_SHOW_IR,
   QV4_SHOW_ASM,
   QV4_FORCE_INTERPRETER;
   page has NO mention of W^X/execmem/'very slow'/'JIT is disabled for QML' string);
   <https://doc.qt.io/qt-6/qtqml-qtquick-compiler-tech.html> ('Byte code for all functions and bindings',
   'C++ code for functions and bindings the compiler fully understands',
   interpret-or-JIT fallback sentence -- ALL VERIFIED verbatim;
   no iOS/per-platform JIT mention);
   <https://doc.qt.io/qt-6/ios-platform-notes.html> (CITED for wall-1 static-only quote -- DOES NOT CONTAIN it;
   no 'static'/'shared object'/'sandbox');
   <https://doc.qt.io/qt-6.8/porting-to-ios.html> (ACTUAL source
    of wall-1 static-only + statically-linked-plugins + QTPLUGIN quotes -- VERIFIED verbatim);
   WebSearch:
   runtime warning 'JIT is disabled for QML.
   Property bindings and animations will be very slow.
  ' confirmed real Qt-on-iOS warning across Qt forums/mailing-lists;
   QML apps demonstrably ship+run on iOS (slow but viable) -- corroborates wall-2 viability conclusion
