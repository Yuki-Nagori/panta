// QML 原子组件的默认 token 与显式覆盖测试（029）。
#include <QColor>
#include <QGuiApplication>
#include <QImageReader>
#include <QObject>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QUrl>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>

namespace {

QObject* create_component(QQmlEngine& engine, const QString& path, QObject& owner) {
    QQmlComponent component(&engine, QUrl(path));
    if (!component.isReady()) {
        QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
        return nullptr;
    }
    QObject* object = component.create();
    if (object == nullptr) {
        QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
        return nullptr;
    }
    object->setParent(&owner);
    return object;
}

} // namespace

class ThemeComponentTest final : public QObject {
    Q_OBJECT

  private slots:
    void defaults_use_theme() {
        QQmlEngine engine;
        QObject owner;
        QObject* label = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedLabel.qml"),
            owner);
        QVERIFY(label != nullptr);
        QCOMPARE(label->property("textColor").value<QColor>(), QColor(QStringLiteral("#1c1c1c")));
        QCOMPARE(label->property("textSize").toInt(), 13);

        QObject* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        QCOMPARE(button->property("controlHeight").toInt(), 24);
        QCOMPARE(button->property("contentPadding").toInt(), 4);
        QCOMPARE(button->property("contentColor").value<QColor>(),
                 QColor(QStringLiteral("#4a4a4a")));
        QCOMPARE(button->property("implicitHeight").toInt(), 24);

        QObject* surface = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/PanelSurface.qml"),
            owner);
        QVERIFY(surface != nullptr);
        QCOMPARE(surface->property("surfaceColor").value<QColor>(),
                 QColor(QStringLiteral("#ffffff")));
    }

    void explicit_values_override_theme() {
        QQmlEngine engine;
        QObject owner;
        QObject* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        button->setProperty("controlHeight", 48);
        button->setProperty("contentPadding", 20);
        QCOMPARE(button->property("controlHeight").toInt(), 48);
        QCOMPARE(button->property("contentPadding").toInt(), 20);
        QCOMPARE(button->property("implicitHeight").toInt(), 48);
        QCOMPARE(button->property("leftPadding").toInt(), 20);
        QCOMPARE(button->property("rightPadding").toInt(), 20);
        // 未覆盖属性仍取 Theme 默认：显式覆盖不得破坏其余绑定。
        QCOMPARE(button->property("contentColor").value<QColor>(),
                 QColor(QStringLiteral("#4a4a4a")));
    }

    void disabled_state_is_visibly_weakened() {
        QQmlEngine engine;
        QObject owner;
        QObject* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        QCOMPARE(button->property("opacity").toDouble(), 1.0);
        button->setProperty("enabled", false);
        QCOMPARE(button->property("opacity").toDouble(), 0.4);
    }

    void icon_resolves_module_resource() {
        QQmlEngine engine;
        QObject owner;
        QObject* icon = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedIcon.qml"),
            owner);
        QVERIFY(icon != nullptr);
        icon->setProperty("name", QStringLiteral("caret-down"));
        const QUrl source = icon->property("source").value<QUrl>();
        QCOMPARE(source, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/icons/caret-down.svg")));
        // qtsvg 供给的 qsvg 图像格式插件必须能解码模块内 SVG 资源。
        QImageReader reader(QStringLiteral(":/qt/qml/Panta/Shell/icons/caret-down.svg"));
        QCOMPARE(reader.canRead(), true);
        QCOMPARE(reader.size().width(), 7);
    }
};

QTEST_MAIN(ThemeComponentTest)
#include "theme_component_test.moc"
