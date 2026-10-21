// ImportDialog 生命周期回归：窗口对象重建时复用同一 ViewModel 会话。
#include "icon_provider.hpp"
#include "project_view_model.hpp"
#include <QFile>
#include <QIODevice>
#include <QMetaObject>
#include <QObject>
#include <QPointer>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QSignalSpy>
#include <QString>
#include <QTemporaryDir>
#include <QUrl>
#include <QVariantMap>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>

class ImportDialogTest final : public QObject {
    Q_OBJECT

  private slots:
    void recreation_preserves_view_model_session() {
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        const auto path = fixture.filePath(QStringLiteral("dialog-session.stl"));
        QFile source(path);
        QVERIFY(source.open(QIODevice::WriteOnly));
        QVERIFY(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n") > 0);
        source.close();

        QQmlEngine engine;
        panta::install_icon_provider(engine);
        panta::bridge::ProjectViewModel model;
        auto* interaction = model.importInteraction();
        QVERIFY(interaction != nullptr);
        QSignalSpy imported(&model, &panta::bridge::ProjectViewModel::projectImportFinished);
        QVERIFY(imported.isValid());

        const QUrl dialogUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Dialogs/ImportDialog.qml"));
        const auto createDialog = [&]() -> QObject* {
            QQmlComponent component(&engine, dialogUrl);
            if (!component.isReady()) {
                QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
                return nullptr;
            }
            QObject* dialog = component.createWithInitialProperties(
                {{QStringLiteral("projectModel"), QVariant::fromValue(&model)}});
            if (dialog == nullptr) {
                QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
            }
            return dialog;
        };

        QObject* firstDialog = createDialog();
        QVERIFY(firstDialog != nullptr);
        QVERIFY(QMetaObject::invokeMethod(firstDialog, "open"));
        QTRY_COMPARE(interaction->stateName(), QStringLiteral("editing"));
        interaction->setUnits(QStringLiteral("centimeters"));
        interaction->setShowImportLog(false);
        QVERIFY(interaction->previewFile(path));
        QTRY_COMPARE(interaction->stateName(), QStringLiteral("ready"));
        QCOMPARE(interaction->sourcePath(), path);

        QPointer<QObject> firstDialogGuard(firstDialog);
        delete firstDialog;
        QVERIFY(firstDialogGuard.isNull());
        QCOMPARE(interaction->stateName(), QStringLiteral("ready"));

        QObject* recreatedDialog = createDialog();
        QVERIFY(recreatedDialog != nullptr);
        QVERIFY(QMetaObject::invokeMethod(recreatedDialog, "open"));
        QTRY_COMPARE(interaction->stateName(), QStringLiteral("ready"));
        QCOMPARE(interaction->sourcePath(), path);
        QCOMPARE(interaction->units(), QStringLiteral("centimeters"));
        QVERIFY(!interaction->showImportLog());
        QVERIFY(interaction->canSubmit());
        QCOMPARE(imported.count(), 0);

        delete recreatedDialog;
        interaction->close();
        QTRY_COMPARE(interaction->stateName(), QStringLiteral("closed"));
    }
};

QTEST_MAIN(ImportDialogTest)
#include "import_dialog_test.moc"
