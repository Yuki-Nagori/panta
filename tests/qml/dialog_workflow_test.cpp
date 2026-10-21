// QML 对话框表单、校验、确认和取消流程回归。
#include "../support/qml/analysis_sequence_helpers.hpp"
#include "../support/qml/quick_item_helpers.hpp"
#if defined(PANTA_TEST_WITH_BRIDGE)
#include "project_view_model.hpp"
using panta::bridge::ProjectViewModel;
#endif
#include <QColor>
#include <QCoreApplication>
#include <QDir>
#include <QDirIterator>
#include <QFile>
#include <QFileInfo>
#include <QFont>
#include <QGuiApplication>
#include <QImage>
#include <QImageReader>
#include <QJSValue>
#include <QMetaType>
#include <QObject>
#include <QPointer>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickImageProvider>
#include <QQuickItem>
#include <QQuickItemGrabResult>
#include <QQuickWindow>
#include <QRect>
#include <QRegularExpression>
#include <QSignalSpy>
#include <QString>
#include <QStringList>
#include <QTemporaryDir>
#include <QTranslator>
#include <QUrl>
#include <QVariantMap>
#include <QVector>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qnamespace.h>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtQml/qqmlextensionplugin.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <icon_provider.hpp>
#include <qaccessible.h>
#include <qtestkeyboard.h>

Q_IMPORT_QML_PLUGIN(Panta_ShellPlugin)
#ifdef PANTA_TEST_WITH_BRIDGE
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
#endif

#include "qml_component_test_helpers.hpp"
using namespace panta::test::qml;

class DialogWorkflowTest final : public QObject {
    Q_OBJECT

  private slots:
    void fill_settings_drafts_profile_rows_and_button_geometry() {
#if !defined(PANTA_TEST_WITH_BRIDGE)
        QSKIP("Process draft validation requires the enabled Bridge module");
#else
        const auto listProperty = [](QObject* object, const char* name) {
            const auto value = object->property(name);
            return value.metaType() == QMetaType::fromType<QJSValue>()
                       ? value.value<QJSValue>().toVariant().toList()
                       : value.toList();
        };
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* dialog = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Dialogs/FillProcessSettingsDialog.qml"),
            owner);
        QVERIFY(dialog != nullptr);
        panta::bridge::ProjectViewModel validationModel;
        const QVariantList profile{
            QVariantMap{{QStringLiteral("duration"), 0.0}, {QStringLiteral("pressure"), 100.0}},
            QVariantMap{{QStringLiteral("duration"), 10.0}, {QStringLiteral("pressure"), 100.0}}};
        const QVariantMap fill{
            {QStringLiteral("moldTemperature"), 40.0},  {QStringLiteral("meltTemperature"), 230.0},
            {QStringLiteral("flowRate"), 94.7},         {QStringLiteral("switchVolume"), 99.0},
            {QStringLiteral("fiberOrientation"), true}, {QStringLiteral("crystallization"), false},
            {QStringLiteral("holdingProfile"), profile}};
        const QVariantMap target{
            {QStringLiteral("projectPath"), QStringLiteral("/test.panta")},
            {QStringLiteral("revision"), 7},
            {QStringLiteral("importId"), QStringLiteral("import-1")},
            {QStringLiteral("sequenceId"), QStringLiteral("fill")},
            {QStringLiteral("fillSettings"), fill},
            {QStringLiteral("processConstraints"),
             validationModel.planSettings().value(QStringLiteral("processConstraints"))}};
        dialog->setProperty("validationModel", QVariant::fromValue(&validationModel));
        dialog->setProperty("planSettings", target);
        QSignalSpy requested(dialog, SIGNAL(settingsRequested(QString, double, QString, QVariant)));
        QVERIFY(requested.isValid());
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QCOMPARE(dialog->property("height").toInt(), 600);
        auto* fillWindow = qobject_cast<QQuickWindow*>(dialog);
        QVERIFY(fillWindow != nullptr);
        auto* flowRate = visual_item(fillWindow->contentItem(), QStringLiteral("fillFlowRate"));
        QVERIFY(flowRate != nullptr);
        flowRate->setProperty("text", QStringLiteral("0"));
        QTRY_VERIFY(!dialog->property("canConfirm").toBool());
        flowRate->setProperty("text", QStringLiteral("94.7"));
        QTRY_VERIFY(dialog->property("canConfirm").toBool());
        auto* accept = dialog->findChild<QObject*>(QStringLiteral("fillSettingsAccept"));
        QVERIFY(accept != nullptr);
        QTRY_COMPARE(accept->property("width").toInt(), 92);
        QCOMPARE(accept->property("height").toInt(), 24);
        auto* profileDialog = dialog->findChild<QObject*>(QStringLiteral("holdingProfileDialog"));
        QVERIFY(profileDialog != nullptr);
        QVERIFY(QMetaObject::invokeMethod(profileDialog, "open", Q_ARG(QVariant, profile)));
        auto* profileWindow = qobject_cast<QQuickWindow*>(profileDialog);
        QVERIFY(profileWindow != nullptr);
        profileWindow->show();
        auto* plot =
            visual_item(profileWindow->contentItem(), QStringLiteral("holdingProfilePlot"));
        QVERIFY(plot != nullptr);
        QObject* themeViewModel =
            engine.singletonInstance<QObject*>("Panta.Shell", "ThemeViewModel");
        QVERIFY(themeViewModel != nullptr);
        bool selected = false;
        QVERIFY(QMetaObject::invokeMethod(themeViewModel, "selectTheme",
                                          Q_RETURN_ARG(bool, selected),
                                          Q_ARG(QString, QStringLiteral("light"))));
        QVERIFY(selected);
        profileDialog->setProperty("plotVisible", true);
        QTRY_VERIFY(plot->property("visible").toBool());
        QTRY_VERIFY(plot->width() > 0 && plot->height() > 0);
        const auto grabPlot = [plot]() {
            const auto grab = plot->grabToImage();
            if (!grab)
                return QImage{};
            QSignalSpy ready(grab.data(), &QQuickItemGrabResult::ready);
            if (ready.isValid() && ready.isEmpty())
                ready.wait(1000);
            return grab->image();
        };
        const QImage lightPlot = grabPlot();
        QVERIFY(!lightPlot.isNull());
        QVERIFY(QMetaObject::invokeMethod(themeViewModel, "selectTheme",
                                          Q_RETURN_ARG(bool, selected),
                                          Q_ARG(QString, QStringLiteral("dark"))));
        QVERIFY(selected);
        const QImage darkPlot = grabPlot();
        QVERIFY(!darkPlot.isNull());
        QVERIFY2(lightPlot != darkPlot,
                 "the holding profile plot should repaint when the active theme changes");
        QVERIFY(QMetaObject::invokeMethod(themeViewModel, "selectTheme",
                                          Q_RETURN_ARG(bool, selected),
                                          Q_ARG(QString, QStringLiteral("light"))));
        QVERIFY(selected);
        auto* duration =
            visual_item(profileWindow->contentItem(), QStringLiteral("profileDuration0"));
        auto* pressure =
            visual_item(profileWindow->contentItem(), QStringLiteral("profilePressure0"));
        auto* third = visual_item(profileWindow->contentItem(), QStringLiteral("profileDuration2"));
        QVERIFY(duration && pressure && third);
        QCOMPARE(third->property("text").toString(), QString{});
        QVERIFY(!third->property("invalid").toBool());

        auto* blankDuration =
            visual_item(profileWindow->contentItem(), QStringLiteral("profileDuration1"));
        auto* blankPressure =
            visual_item(profileWindow->contentItem(), QStringLiteral("profilePressure1"));
        auto* shiftedDuration =
            visual_item(profileWindow->contentItem(), QStringLiteral("profileDuration2"));
        auto* shiftedPressure =
            visual_item(profileWindow->contentItem(), QStringLiteral("profilePressure2"));
        QVERIFY(blankDuration && blankPressure && shiftedDuration && shiftedPressure);
        blankDuration->setProperty("text", QString{});
        QVERIFY(QMetaObject::invokeMethod(blankDuration, "textEdited"));
        blankPressure->setProperty("text", QString{});
        QVERIFY(QMetaObject::invokeMethod(blankPressure, "textEdited"));
        QTRY_VERIFY(!blankDuration->property("invalid").toBool());
        QVERIFY(!blankPressure->property("invalid").toBool());
        shiftedDuration->setProperty("text", QStringLiteral("20"));
        QVERIFY(QMetaObject::invokeMethod(shiftedDuration, "textEdited"));
        shiftedPressure->setProperty("text", QStringLiteral("250"));
        QVERIFY(QMetaObject::invokeMethod(shiftedPressure, "textEdited"));
        QCOMPARE(listProperty(profileDialog, "draftPoints").size(), 2);
        QVariant shiftedPressureError;
        QVERIFY(QMetaObject::invokeMethod(
            profileDialog, "hasProfileFieldError", Q_RETURN_ARG(QVariant, shiftedPressureError),
            Q_ARG(QVariant, QVariant::fromValue(2)),
            Q_ARG(QVariant, QVariant::fromValue(QStringLiteral("pressure")))));
        QVERIFY(shiftedPressureError.toBool());

        QVariant upperBoundRange;
        const QVariantMap maximumOnlyConstraint{{QStringLiteral("unit"), QStringLiteral("s")},
                                                {QStringLiteral("hasMinimum"), false},
                                                {QStringLiteral("hasMaximum"), true},
                                                {QStringLiteral("maximum"), 8.0},
                                                {QStringLiteral("maximumInclusive"), false}};
        QVERIFY(QMetaObject::invokeMethod(
            profileDialog, "rangeText", Q_RETURN_ARG(QVariant, upperBoundRange),
            Q_ARG(QVariant, QVariant::fromValue(maximumOnlyConstraint))));
        QCOMPARE(upperBoundRange.toString(), QStringLiteral("s · < 8"));

        QVariant unboundedMinimum;
        const QVariantMap unboundedConstraint{{QStringLiteral("hasMinimum"), false},
                                              {QStringLiteral("minimum"), 5.0}};
        QVERIFY(QMetaObject::invokeMethod(dialog, "minimumBound",
                                          Q_RETURN_ARG(QVariant, unboundedMinimum),
                                          Q_ARG(QVariant, QVariant::fromValue(unboundedConstraint)),
                                          Q_ARG(QVariant, QVariant::fromValue(-123.0))));
        QCOMPARE(unboundedMinimum.toDouble(), -123.0);

        QVERIFY(QMetaObject::invokeMethod(profileDialog, "open", Q_ARG(QVariant, profile)));
        duration = visual_item(profileWindow->contentItem(), QStringLiteral("profileDuration0"));
        pressure = visual_item(profileWindow->contentItem(), QStringLiteral("profilePressure0"));
        third = visual_item(profileWindow->contentItem(), QStringLiteral("profileDuration2"));
        QVERIFY(duration && pressure && third);
        QTRY_VERIFY(qAbs(duration->property("width").toDouble() -
                         pressure->property("width").toDouble()) < 1.0);
        duration->setProperty("text", QStringLiteral("5"));
        QVERIFY(QMetaObject::invokeMethod(duration, "textEdited"));
        QVERIFY(QMetaObject::invokeMethod(profileDialog, "close"));
        QCOMPARE(listProperty(dialog, "profile"), profile);
        QVERIFY(QMetaObject::invokeMethod(profileDialog, "open", Q_ARG(QVariant, profile)));
        duration = visual_item(profileWindow->contentItem(), QStringLiteral("profileDuration2"));
        QVERIFY(duration != nullptr);
        duration->setProperty("text", QStringLiteral("5"));
        QVERIFY(QMetaObject::invokeMethod(duration, "textEdited"));
        QVERIFY(listProperty(profileDialog, "points").isEmpty());
        pressure = visual_item(profileWindow->contentItem(), QStringLiteral("profilePressure2"));
        QVERIFY(pressure != nullptr);
        pressure->setProperty("text", QStringLiteral("80"));
        QVERIFY(QMetaObject::invokeMethod(pressure, "textEdited"));
        QVERIFY(QMetaObject::invokeMethod(profileDialog, "acceptProfile"));
        QCOMPARE(listProperty(dialog, "profile").size(), 3);
        dialog->setProperty("planSettings", QVariantMap{});
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSettings"));
        QCOMPARE(requested.count(), 1);
        QCOMPARE(requested.at(0).at(0).toString(), QStringLiteral("/test.panta"));
        QCOMPARE(requested.at(0).at(1).toInt(), 7);
        QCOMPARE(requested.at(0).at(2).toString(), QStringLiteral("import-1"));
        dialog->setProperty("saving", true);
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSettings"));
        QCOMPARE(requested.count(), 1);
        dialog->setProperty("saving", false);
        auto* cancel = qobject_cast<QQuickItem*>(
            dialog->findChild<QObject*>(QStringLiteral("fillSettingsCancel")));
        auto* window = qobject_cast<QQuickWindow*>(dialog);
        QVERIFY(cancel && window);
        cancel->forceActiveFocus();
        QTest::keyClick(window, Qt::Key_Return);
        QTRY_VERIFY(!window->isVisible());
        QCOMPARE(requested.count(), 1);
        dialog->setProperty("planSettings", target);
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QCOMPARE(listProperty(dialog, "profile"), profile);
        QVERIFY(QMetaObject::invokeMethod(dialog, "close"));
#endif
    }
    void gate_location_confirmation_uses_rust_numeric_constraints() {
#if !defined(PANTA_TEST_WITH_BRIDGE)
        QSKIP("Process draft validation requires the enabled Bridge module");
#else
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* dialog = create_component(
            engine,
            QStringLiteral("qrc:/qt/qml/Panta/Shell/Dialogs/GateLocationSettingsDialog.qml"),
            owner);
        QVERIFY(dialog != nullptr);
        panta::bridge::ProjectViewModel validationModel;
        const auto plan = validationModel.planSettings();
        const QVariantMap gate{
            {QStringLiteral("machineId"), QStringLiteral("default-machine")},
            {QStringLiteral("algorithmId"), QStringLiteral("advanced-gate-locator")},
            {QStringLiteral("moldTemperature"), 40.0},
            {QStringLiteral("meltTemperature"), 230.0},
            {QStringLiteral("numberOfGates"), 1}};
        const QVariantMap target{{QStringLiteral("projectPath"), QStringLiteral("/test.panta")},
                                 {QStringLiteral("revision"), 7},
                                 {QStringLiteral("importId"), QStringLiteral("import-1")},
                                 {QStringLiteral("sequenceId"), QStringLiteral("gate-location")},
                                 {QStringLiteral("gateLocationSettings"), gate},
                                 {QStringLiteral("processConstraints"),
                                  plan.value(QStringLiteral("processConstraints"))}};
        dialog->setProperty("validationModel", QVariant::fromValue(&validationModel));
        dialog->setProperty("planSettings", target);
        QSignalSpy requested(dialog, SIGNAL(settingsRequested(QString, double, QString, QVariant)));
        QVERIFY(requested.isValid());
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QTRY_VERIFY(dialog->property("canConfirm").toBool());

        auto* window = qobject_cast<QQuickWindow*>(dialog);
        QVERIFY(window != nullptr);
        auto* gates = visual_item(window->contentItem(), QStringLiteral("gateNumberOfGates"));
        QVERIFY(gates != nullptr);
        gates->setProperty("text", QStringLiteral("11"));
        QTRY_VERIFY(!dialog->property("canConfirm").toBool());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSettings"));
        QCOMPARE(requested.count(), 0);

        gates->setProperty("text", QStringLiteral("10"));
        QTRY_VERIFY(dialog->property("canConfirm").toBool());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSettings"));
        QCOMPARE(requested.count(), 1);
        QVERIFY(QMetaObject::invokeMethod(dialog, "close"));
#endif
    }
    void new_project_dialog_waits_for_creation_and_preserves_failed_input() {
#if !defined(PANTA_TEST_WITH_BRIDGE)
        QSKIP("Project commands require the enabled Bridge module");
#else
        QTemporaryDir fixture;
        QVERIFY(fixture.isValid());
        panta::bridge::ProjectViewModel model;
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QQmlComponent component(
            &engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Dialogs/NewProjectDialog.qml")));
        QVERIFY2(component.isReady(), qPrintable(component.errorString()));
        QObject owner;
        auto* dialog = component.createWithInitialProperties(
            {{QStringLiteral("projectModel"), QVariant::fromValue(&model)}});
        QVERIFY2(dialog != nullptr, qPrintable(component.errorString()));
        dialog->setParent(&owner);
        auto* window = qobject_cast<QQuickWindow*>(dialog);
        QVERIFY(window != nullptr);
        auto* name = visual_item(window->contentItem(), QStringLiteral("projectNameField"));
        auto* location = visual_item(window->contentItem(), QStringLiteral("projectLocationField"));
        auto* accept = visual_item(window->contentItem(), QStringLiteral("newProjectAccept"));
        auto* cancel = visual_item(window->contentItem(), QStringLiteral("newProjectCancel"));
        QVERIFY(name && location && accept && cancel);
        QSignalSpy created(dialog, SIGNAL(projectCreated(QString)));
        QVERIFY(created.isValid());

        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        name->setProperty("text", QStringLiteral("Created"));
        location->setProperty("text", fixture.path());
        QVERIFY(QMetaObject::invokeMethod(dialog, "submit"));
        QVERIFY(window->isVisible());
        QVERIFY(dialog->property("submitting").toBool());
        QVERIFY(!name->isEnabled());
        QVERIFY(!location->isEnabled());
        QVERIFY(!accept->isEnabled());
        QVERIFY(!cancel->isEnabled());
        QVERIFY(!window->close());
        QVERIFY(QMetaObject::invokeMethod(dialog, "submit"));
        QCOMPARE(created.count(), 0);
        QTRY_COMPARE(created.count(), 1);
        QTRY_VERIFY(!window->isVisible());
        QVERIFY(!created.constFirst().constFirst().toString().isEmpty());

        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        name->setProperty("text", QStringLiteral("Created"));
        location->setProperty("text", fixture.path());
        QVERIFY(QMetaObject::invokeMethod(dialog, "submit"));
        QTRY_VERIFY(!model.projectOperationPending());
        QVERIFY(window->isVisible());
        QVERIFY(!dialog->property("submitting").toBool());
        QCOMPARE(created.count(), 1);
        QCOMPARE(name->property("text").toString(), QStringLiteral("Created"));
        QCOMPARE(location->property("text").toString(), fixture.path());
        QVERIFY(!model.error().isEmpty());
        QVERIFY(accept->isEnabled());
        QVERIFY(window->close());
#endif
    }
    void analysis_sequence_confirmation_and_cancel() {
#if !defined(PANTA_TEST_WITH_BRIDGE)
        QSKIP("Analysis sequence catalog requires the enabled Bridge module");
#endif
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* dialog = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Dialogs/AnalysisSequenceDialog.qml"),
            owner);
        QVERIFY(dialog != nullptr);
        auto* panel = dialog->findChild<QObject*>(QStringLiteral("analysisSequencePanel"));
        QVERIFY(panel != nullptr);
        const auto sequences = analysis_sequence_catalog();
        QVERIFY(!sequences.isEmpty());
        dialog->setProperty("sequences", sequences);
        const QVariantMap settings{{QStringLiteral("projectPath"), QStringLiteral("/test.panta")},
                                   {QStringLiteral("revision"), 1},
                                   {QStringLiteral("importId"), QStringLiteral("import-1")},
                                   {QStringLiteral("sequenceId"), QStringLiteral("fill")}};
        dialog->setProperty("planSettings", settings);
        QSignalSpy selection(dialog, SIGNAL(selectionRequested(QString, double, QString, QString)));
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        auto* window = qobject_cast<QQuickWindow*>(dialog);
        QVERIFY(window != nullptr);
        QTest::keyClick(window, Qt::Key_Down);
        QCOMPARE(panel->property("selectedIndex").toInt(), 1);
        QTest::keyClick(window, Qt::Key_Return);
        QCOMPARE(selection.count(), 1);
        QCOMPARE(selection.at(0).at(3).toString(), QStringLiteral("fill-pack"));
        dialog->setProperty("saving", true);
        auto* accept = dialog->findChild<QObject*>(QStringLiteral("analysisSequenceAccept"));
        auto* cancel = dialog->findChild<QObject*>(QStringLiteral("analysisSequenceCancel"));
        QVERIFY(accept && cancel);
        QVERIFY(!accept->property("enabled").toBool());
        QVERIFY(!cancel->property("enabled").toBool());
        QVERIFY(!panel->property("enabled").toBool());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSelection"));
        QTest::keyClick(window, Qt::Key_Return);
        QCOMPARE(selection.count(), 1);
        QVERIFY(!window->close());
        dialog->setProperty("errorText", QStringLiteral("Write failed"));
        dialog->setProperty("saving", false);
        QVERIFY(window->isVisible());
        QCOMPARE(panel->property("selectedIndex").toInt(), 1);
        QVERIFY(accept->property("enabled").toBool());
        QVERIFY(QMetaObject::invokeMethod(dialog, "acceptSelection"));
        QCOMPARE(selection.count(), 2);
        panel->setProperty("selectedIndex", 4);
        QVERIFY(QMetaObject::invokeMethod(cancel, "clicked"));
        QCOMPARE(selection.count(), 2);
        QVERIFY(QMetaObject::invokeMethod(dialog, "open"));
        QCOMPARE(panel->property("selectedIndex").toInt(), 0);
        dialog->setProperty("visible", false);

        auto* tasks = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/SidebarPanel.qml"), owner);
        QVERIFY(tasks != nullptr);
        QSignalSpy requested(tasks, SIGNAL(analysisSequenceRequested()));
        QVERIFY(QMetaObject::invokeMethod(
            tasks->findChild<QObject*>(QStringLiteral("projectTasksPage")), "openPlanTask",
            Q_ARG(QVariant, "analysis-sequence")));
        QCOMPARE(requested.count(), 1);
    }
};

QTEST_MAIN(DialogWorkflowTest)
#include "dialog_workflow_test.moc"
