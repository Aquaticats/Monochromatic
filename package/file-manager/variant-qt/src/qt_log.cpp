// SPDX-License-Identifier: LGPL-3.0-or-later
//
// Route Qt's own logging (qDebug / qCDebug / qWarning / ...) into the app's non-blocking
// Rust tracing sink. Qt's default handler is synchronous on the emitting thread, so under
// load (or with QSG_RENDER_TIMING) it can block a frame; forwarding into the off-thread
// tracing pipeline removes that risk. Paired with src/qt_log.rs.

#include <QtGlobal>
#include <QString>
#include <QByteArray>
#include <QMessageLogContext>

// Implemented in Rust (src/qt_log.rs, #[unsafe(no_mangle)] extern "C").
extern "C" void fmqt_forward_log(int kind, const char *category, const char *message);

namespace {
// Qt message handler: convert the message to UTF-8 and hand it, with its level and
// category, to the Rust side. QtMsgType is passed as its integer value
// (0 debug, 1 warning, 2 critical, 3 fatal, 4 info).
void fmqt_handler(QtMsgType type, const QMessageLogContext &context, const QString &message)
{
    const QByteArray utf8 = message.toUtf8();
    const char *category = context.category ? context.category : "qt";
    fmqt_forward_log(static_cast<int>(type), category, utf8.constData());
}
} // namespace

// Install the handler process-wide. Called once from Rust at startup.
extern "C" void fmqt_install_qt_message_handler()
{
    qInstallMessageHandler(fmqt_handler);
}
