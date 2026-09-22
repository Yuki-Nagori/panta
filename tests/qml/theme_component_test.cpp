// QML 原子组件的默认 token 与显式覆盖测试（029）。
#include <QColor>
#include <QDir>
#include <QGuiApplication>
#include <QImage>
#include <QImageReader>
#include <QObject>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickImageProvider>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <icon_provider.hpp>

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
        panta::install_icon_provider(engine);
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
        panta::install_icon_provider(engine);
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
        panta::install_icon_provider(engine);
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
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* icon = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedIcon.qml"),
            owner);
        QVERIFY(icon != nullptr);
        icon->setProperty("name", QStringLiteral("caret-down"));
        const QUrl source = icon->property("source").value<QUrl>();
        QCOMPARE(source, QUrl(QStringLiteral("image://panta-icons/caret-down/ff4a4a4a")));
        // qtsvg 供给的 qsvg 图像格式插件必须能解码模块内 SVG 资源。
        QImageReader reader(QStringLiteral(":/qt/qml/Panta/Shell/icons/caret-down.svg"));
        QCOMPARE(reader.canRead(), true);
        QCOMPARE(reader.size(), QSize(24, 24));
        icon->setProperty("color", QColor(QStringLiteral("#ffffff")));
        QCOMPARE(icon->property("source").value<QUrl>(),
                 QUrl(QStringLiteral("image://panta-icons/caret-down/ffffffff")));
        icon->setProperty("name", QString());
        QVERIFY(icon->property("source").value<QUrl>().isEmpty());
    }
    void svg_resources_render_with_caller_color() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        auto* provider = static_cast<QQuickImageProvider*>(engine.imageProvider("panta-icons"));
        QVERIFY(provider != nullptr);
        const QStringList icons = QDir(QStringLiteral(":/qt/qml/Panta/Shell/icons"))
                                      .entryList({QStringLiteral("*.svg")}, QDir::Files);
        QVERIFY(!icons.isEmpty());
        for (const QString& file : icons) {
            for (const int pixels : {16, 18, 24, 48}) {
                QSize original;
                const QString name = file.chopped(4);
                const QImage frame =
                    provider->requestImage(name + "/ff2878b8", &original, QSize(pixels, pixels));
                QVERIFY2(!frame.isNull(), qPrintable(file));
                QCOMPARE(original, QSize(24, 24));
                QCOMPARE(frame.size(), QSize(pixels, pixels));
                int colored = 0;
                for (int y = 0; y < pixels; ++y) {
                    for (int x = 0; x < pixels; ++x) {
                        const QColor pixel = frame.pixelColor(x, y);
                        if (pixel.alpha() > 240) {
                            // 允许预乘 alpha 的整数舍入；不允许硬编码黑色漏出。
                            QVERIFY(qAbs(pixel.red() - 40) <= 1);
                            QVERIFY(qAbs(pixel.green() - 120) <= 1);
                            QVERIFY(qAbs(pixel.blue() - 184) <= 1);
                            ++colored;
                        }
                        if (x == 0 || y == 0 || x == pixels - 1 || y == pixels - 1) {
                            QCOMPARE(pixel.alpha(), 0);
                        }
                    }
                }
                QVERIFY2(colored > 0, qPrintable(file));
            }
        }
        QSize original;
        const QImage white =
            provider->requestImage("pane-close/ffffffff", &original, QSize(24, 24));
        QVERIFY(white.pixelColor(6, 6).lightness() > 240);
        const QImage transparent =
            provider->requestImage("pane-close/00ffffff", nullptr, QSize(24, 24));
        QCOMPARE(transparent.pixelColor(6, 6).alpha(), 0);
        QVERIFY(provider->requestImage("../document-new/ffffffff", nullptr, {}).isNull());
        QVERIFY(provider->requestImage("pane-close/not-a-color", nullptr, {}).isNull());
    }
};

QTEST_MAIN(ThemeComponentTest)
#include "theme_component_test.moc"
