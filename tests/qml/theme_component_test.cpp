// 主题原子组件的默认值、覆盖规则和图标资源回归。
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
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtQml/qqmlextensionplugin.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <icon_provider.hpp>
#include <qaccessible.h>
#include <qpoint.h>

Q_IMPORT_QML_PLUGIN(Panta_ShellPlugin)
#ifdef PANTA_TEST_WITH_BRIDGE
Q_IMPORT_QML_PLUGIN(Panta_BridgePlugin)
#endif

#include "qml_component_test_helpers.hpp"
using namespace panta::test::qml;

class ThemeAtomTest final : public QObject {
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
    void consolidated_controls_keep_fixed_sizes_and_stable_tabs() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        const QString atomPath = QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/");
        auto* button =
            create_component(engine, atomPath + QStringLiteral("ThemedButton.qml"), owner);
        QVERIFY(button != nullptr);
        QCOMPARE(button->property("implicitWidth").toDouble(), 92.0);
        QCOMPARE(button->property("implicitHeight").toDouble(), 24.0);
        auto* combo =
            create_component(engine, atomPath + QStringLiteral("ThemedComboBox.qml"), owner);
        QVERIFY(combo != nullptr);
        QCOMPARE(combo->property("implicitHeight").toDouble(), 30.0);
        auto* check =
            create_component(engine, atomPath + QStringLiteral("ThemedCheckBox.qml"), owner);
        QVERIFY(check != nullptr);
        QCOMPARE(check->property("implicitHeight").toDouble(), 24.0);
        QCOMPARE(check->property("padding").toDouble(), 0.0);
        auto* indicator = check->property("indicator").value<QObject*>();
        QVERIFY(indicator != nullptr);
        QCOMPARE(indicator->property("implicitWidth").toDouble(), 16.0);
        QCOMPARE(indicator->property("implicitHeight").toDouble(), 16.0);
        auto* tab = create_component(engine, atomPath + QStringLiteral("MenuTabButton.qml"), owner);
        QVERIFY(tab != nullptr);
        tab->setProperty("text", QStringLiteral("Boundary Conditions"));
        const double width = tab->property("implicitWidth").toDouble();
        QVERIFY(width > 0.0);
        tab->setProperty("highlighted", true);
        QCOMPARE(tab->property("implicitWidth").toDouble(), width);
        tab->setProperty("highlighted", false);
        QCOMPARE(tab->property("implicitWidth").toDouble(), width);
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
    void button_font_size_reaches_its_label() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        auto* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedToolButton.qml"),
            owner);
        QVERIFY(button != nullptr);
        button->setProperty("text", QStringLiteral("Font override"));
        QObject* label = nullptr;
        for (auto* child : button->findChildren<QObject*>()) {
            if (child->property("text").toString() == QStringLiteral("Font override")) {
                label = child;
                break;
            }
        }
        QVERIFY(label != nullptr);
        QFont font = button->property("font").value<QFont>();
        font.setPixelSize(19);
        button->setProperty("font", font);
        QCOMPARE(label->property("font").value<QFont>().pixelSize(), 19);
    }
    void icon_resolves_module_resource() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        QObject owner;
        QObject* icon = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedIcon.qml"),
            owner);
        QVERIFY(icon != nullptr);
        icon->setProperty("name", QStringLiteral("caret"));
        QCOMPARE(icon->property("source").value<QUrl>(),
                 QUrl(QStringLiteral("image://panta-icons/caret/ff4a4a4a")));
        // qtsvg 提供的 qsvg 插件可以读取模块打包的原始 SVG viewBox 尺寸。
        QImageReader reader(QStringLiteral(":/qt/qml/Panta/Shell/icons/caret.svg"));
        QVERIFY(reader.canRead());
        QCOMPARE(reader.size(), QSize(7, 5));
        icon->setProperty("color", QColor(QStringLiteral("#ffffff")));
        QCOMPARE(icon->property("source").value<QUrl>(),
                 QUrl(QStringLiteral("image://panta-icons/caret/ffffffff")));
        icon->setProperty("preserveSourceColors", true);
        icon->setProperty("name", QStringLiteral("ribbon-project"));
        QCOMPARE(icon->property("source").value<QUrl>(),
                 QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/icons/ribbon-project.svg")));
        icon->setProperty("name", QString());
        QVERIFY(icon->property("source").value<QUrl>().isEmpty());
    }
    void qml_icon_catalog_and_modes_are_consistent() {
        const QString icon_root = QStringLiteral(":/qt/qml/Panta/Shell/icons/");
        const QStringList files = QDir(icon_root).entryList({QStringLiteral("*.svg")}, QDir::Files);
        QVERIFY(!files.isEmpty());

        QSet<QString> resource_ids;
        const QRegularExpression source_symbol(
            QStringLiteral("shell\\.js 的 #i-([a-z][a-z0-9-]*)"));
        for (const QString& file_name : files) {
            QFile file(icon_root + file_name);
            QVERIFY2(file.open(QIODevice::ReadOnly), qPrintable(file_name));
            const QString svg = QString::fromUtf8(file.readAll());
            const auto match = source_symbol.match(svg);
            QVERIFY2(match.hasMatch(), qPrintable(file_name));
            const QString resource_id = QFileInfo(file_name).completeBaseName();
            QCOMPARE(match.captured(1), resource_id);
            QVERIFY2(!resource_ids.contains(resource_id), qPrintable(resource_id));
            resource_ids.insert(resource_id);
        }

        QFile reference_file(QStringLiteral(":/panta-test/shell.js"));
        QVERIFY2(reference_file.open(QIODevice::ReadOnly), qPrintable(reference_file.fileName()));
        const QString reference = QString::fromUtf8(reference_file.readAll());
        const QRegularExpression official_symbol(
            QStringLiteral("<symbol\\s+id=[\\\"']i-([a-z][a-z0-9-]*)[\\\"']"));
        QSet<QString> official_ids;
        auto symbols = official_symbol.globalMatch(reference);
        while (symbols.hasNext()) {
            const QString symbol_id = symbols.next().captured(1);
            QVERIFY2(!official_ids.contains(symbol_id), qPrintable(symbol_id));
            official_ids.insert(symbol_id);
        }
        QCOMPARE(official_ids, resource_ids);

        const QSet<QString>& monochrome_ids = panta::monochrome_icon_names();
        for (const QString& icon_id : monochrome_ids) {
            QVERIFY2(resource_ids.contains(icon_id), qPrintable(icon_id));
        }

        int referenced_icon_count = 0;
        int ribbon_data_icons = 0;
        int task_data_icons = 0;
        bool ribbon_model_uses_source_colors = false;
        bool ribbon_content_forwards_model_icon = false;
        bool tool_button_forwards_icon_mode = false;
        bool task_model_uses_source_colors = false;
        bool layer_model_uses_item_mode = false;
        bool qml_resources_found = false;
        QDirIterator qml_files(QStringLiteral(":/qt/qml/Panta/Shell"), {QStringLiteral("*.qml")},
                               QDir::Files, QDirIterator::Subdirectories);

        while (qml_files.hasNext()) {
            qml_resources_found = true;
            const QString path = qml_files.next();
            QFile file(path);
            QVERIFY2(file.open(QIODevice::ReadOnly), qPrintable(path));
            const QString qml = QString::fromUtf8(file.readAll());

            if (path.endsWith(QStringLiteral("/RibbonContent.qml"))) {
                for (const QString& block : qml_object_blocks(qml, "RibbonTile")) {
                    ribbon_content_forwards_model_icon =
                        qml_binding(block, "iconName") == QStringLiteral("modelData.icon");
                }
            }

            for (const QString& item : qml_simple_object_blocks(qml)) {
                const QString icon_expression = qml_binding(item, "icon");
                if (icon_expression.isEmpty()) {
                    continue;
                }
                const QString icon_id = qml_icon_literal(icon_expression);
                QVERIFY2(!icon_id.isEmpty(),
                         qPrintable(path + QStringLiteral(": model icon must be a literal: ") +
                                    icon_expression));
                ++referenced_icon_count;
                QVERIFY2(resource_ids.contains(icon_id),
                         qPrintable(path + QStringLiteral(": missing SVG for ") + icon_id));

                const QString mode = qml_binding(item, "preserveIconColors");
                const bool is_ribbon_data = path.contains(QStringLiteral("/Panels/Ribbon/"));
                const bool is_task_data = path.endsWith(QStringLiteral("/ProjectTasksPage.qml"));
                ribbon_data_icons += is_ribbon_data ? 1 : 0;
                task_data_icons += is_task_data ? 1 : 0;
                bool preserves_source = false;
                if (!mode.isEmpty()) {
                    QVERIFY2(mode == QStringLiteral("true") || mode == QStringLiteral("false"),
                             qPrintable(path + QStringLiteral(": data icon mode must be a bool: ") +
                                        icon_id));
                    preserves_source = mode == QStringLiteral("true");
                } else if (is_ribbon_data || is_task_data) {
                    // Ribbon 与导入任务模型由其视觉组件统一按资源原色呈现。
                    preserves_source = true;
                }
                QVERIFY2(preserves_source == !monochrome_ids.contains(icon_id),
                         qPrintable(path + QStringLiteral(": wrong data icon mode: ") + icon_id));
            }

            for (const QString& block : qml_object_blocks(qml, "ThemedIcon")) {
                const QString expression = qml_binding(block, "name");
                if (expression.isEmpty() || expression == QStringLiteral("\"\"") ||
                    expression == QStringLiteral("''")) {
                    continue;
                }
                const QString icon_id = qml_icon_literal(expression);
                const QString mode = qml_binding(block, "preserveSourceColors");
                if (icon_id.isEmpty()) {
                    if (expression == QStringLiteral("tile.iconName")) {
                        QCOMPARE(mode, QStringLiteral("true"));
                        ribbon_model_uses_source_colors = mode == QStringLiteral("true");
                    } else if (expression == QStringLiteral("button.iconName")) {
                        QCOMPARE(mode, QStringLiteral("button.preserveIconColors"));
                        tool_button_forwards_icon_mode = true;
                    } else if (expression == QStringLiteral("projectEntry.iconName")) {
                        QCOMPARE(mode, QStringLiteral("true"));
                    } else {
                        QFAIL(qPrintable(
                            path + QStringLiteral(": unrecognized dynamic ThemedIcon name: ") +
                            expression));
                    }
                    continue;
                }
                ++referenced_icon_count;
                QVERIFY2(resource_ids.contains(icon_id),
                         qPrintable(path + QStringLiteral(": missing SVG for ") + icon_id));
                bool preserves_source = false;
                if (mode == QStringLiteral("true")) {
                    preserves_source = true;
                } else if (mode != QStringLiteral("false") && !mode.isEmpty()) {
                    QFAIL(qPrintable(path +
                                     QStringLiteral(": static ThemedIcon mode must be a bool: ") +
                                     icon_id));
                }
                QVERIFY2(preserves_source == !monochrome_ids.contains(icon_id),
                         qPrintable(path + QStringLiteral(": wrong ThemedIcon mode: ") + icon_id));
            }

            for (const QString& block : qml_object_blocks(qml, "ThemedToolButton")) {
                const QString expression = qml_binding(block, "iconName");
                if (expression.isEmpty() || expression == QStringLiteral("\"\"") ||
                    expression == QStringLiteral("''")) {
                    continue;
                }
                const QString icon_id = qml_icon_literal(expression);
                const QString mode = qml_binding(block, "preserveIconColors");
                if (icon_id.isEmpty()) {
                    if ((expression == QStringLiteral("modelData.icon") ||
                         expression == QStringLiteral("taskRow.modelData.icon")) &&
                        path.endsWith(QStringLiteral("/ProjectTasksPage.qml"))) {
                        QCOMPARE(mode, QStringLiteral("true"));
                        task_model_uses_source_colors = mode == QStringLiteral("true");
                    } else if (expression == QStringLiteral("modelData.icon") &&
                               path.endsWith(QStringLiteral("/LayersPanel.qml"))) {
                        QCOMPARE(mode, QStringLiteral("modelData.preserveIconColors === true"));
                        layer_model_uses_item_mode = true;
                    } else {
                        QFAIL(qPrintable(
                            path +
                            QStringLiteral(": unrecognized dynamic ThemedToolButton icon: ") +
                            expression));
                    }
                    continue;
                }
                ++referenced_icon_count;
                QVERIFY2(resource_ids.contains(icon_id),
                         qPrintable(path + QStringLiteral(": missing SVG for ") + icon_id));
                bool preserves_source = false;
                if (mode == QStringLiteral("true")) {
                    preserves_source = true;
                } else if (mode != QStringLiteral("false") && !mode.isEmpty()) {
                    QFAIL(qPrintable(
                        path + QStringLiteral(": static ThemedToolButton mode must be a bool: ") +
                        icon_id));
                }
                QVERIFY2(
                    preserves_source == !monochrome_ids.contains(icon_id),
                    qPrintable(path + QStringLiteral(": wrong ThemedToolButton mode: ") + icon_id));
            }
        }

        QVERIFY(qml_resources_found);
        QVERIFY(ribbon_content_forwards_model_icon);
        QVERIFY(ribbon_data_icons > 0);
        QVERIFY(task_data_icons > 0);
        QVERIFY(ribbon_model_uses_source_colors);
        QVERIFY(task_model_uses_source_colors);
        QVERIFY(layer_model_uses_item_mode);
        QVERIFY(tool_button_forwards_icon_mode);
        QVERIFY(referenced_icon_count > 0);
    }
    void svg_resources_render_with_caller_color() {
        QQmlEngine engine;
        panta::install_icon_provider(engine);
        auto* provider = static_cast<QQuickImageProvider*>(engine.imageProvider("panta-icons"));
        QVERIFY(provider != nullptr);
        const QString resource_root = QStringLiteral(":/qt/qml/Panta/Shell/icons/");
        const QStringList files =
            QDir(resource_root).entryList({QStringLiteral("*.svg")}, QDir::Files);
        QVERIFY(!files.isEmpty());
        for (const QString& file : files) {
            QImageReader reader(resource_root + file);
            QVERIFY2(reader.canRead(), qPrintable(file));
            QVERIFY2(reader.size().isValid(), qPrintable(file));
            QVERIFY2(!reader.read().isNull(), qPrintable(file));
        }

        QStringList monochrome_icons = panta::monochrome_icon_names().values();
        monochrome_icons.sort();
        for (const QString& name : monochrome_icons) {
            QImageReader reader(resource_root + name + QStringLiteral(".svg"));
            const QSize expected_size = reader.size();
            QVERIFY(expected_size.isValid());
            for (const int pixels : {16, 18, 24, 48}) {
                QSize original;
                const QImage frame = provider->requestImage(name + QStringLiteral("/ff2878b8"),
                                                            &original, QSize(pixels, pixels));
                QVERIFY2(!frame.isNull(), qPrintable(name));
                QCOMPARE(original, expected_size);
                QCOMPARE(frame.size(), QSize(pixels, pixels));
                int colored = 0;
                for (int y = 0; y < pixels; ++y) {
                    for (int x = 0; x < pixels; ++x) {
                        const QColor pixel = frame.pixelColor(x, y);
                        if (pixel.alpha() > 0) {
                            ++colored;
                        }
                        if (pixel.alpha() > 192) {
                            // Alpha mask 只保留几何覆盖，像素颜色必须完全来自调用方。
                            QVERIFY2(qAbs(pixel.red() - 40) <= 1, qPrintable(name));
                            QVERIFY2(qAbs(pixel.green() - 120) <= 1, qPrintable(name));
                            QVERIFY2(qAbs(pixel.blue() - 184) <= 1, qPrintable(name));
                        }
                    }
                }
                QVERIFY2(colored > 0, qPrintable(name));
            }
        }

        const QImage caret =
            provider->requestImage(QStringLiteral("caret/ff4a4a4a"), nullptr, QSize(24, 24));
        QVERIFY(!caret.isNull());
        QRect caret_bounds;
        for (int y = 0; y < caret.height(); ++y) {
            for (int x = 0; x < caret.width(); ++x) {
                if (caret.pixelColor(x, y).alpha() > 0) {
                    caret_bounds |= QRect(x, y, 1, 1);
                }
            }
        }
        QVERIFY(caret_bounds.isValid());
        QVERIFY(caret_bounds.width() < caret.width());
        QVERIFY(caret_bounds.height() < 16);

        QImageReader color_reader(resource_root + QStringLiteral("ribbon-project.svg"));
        color_reader.setScaledSize(QSize(42, 42));
        const QImage color_icon = color_reader.read();
        QVERIFY(!color_icon.isNull());
        bool has_blue = false;
        bool has_neutral = false;
        for (int y = 0; y < color_icon.height(); ++y) {
            for (int x = 0; x < color_icon.width(); ++x) {
                const QColor pixel = color_icon.pixelColor(x, y);
                has_blue = has_blue || (pixel.alpha() > 180 && pixel.blue() > pixel.red() * 1.4);
                has_neutral =
                    has_neutral || (pixel.alpha() > 180 && qAbs(pixel.red() - pixel.green()) < 5 &&
                                    qAbs(pixel.green() - pixel.blue()) < 5);
            }
        }
        QVERIFY(has_blue);
        QVERIFY(has_neutral);

        QSize original;
        const QImage white =
            provider->requestImage(QStringLiteral("close/ffffffff"), &original, QSize(24, 24));
        QVERIFY(white.pixelColor(6, 6).lightness() > 240);
        const QImage transparent =
            provider->requestImage(QStringLiteral("close/00ffffff"), nullptr, QSize(24, 24));
        QCOMPARE(transparent.pixelColor(6, 6).alpha(), 0);
        const QSize fallback_size =
            QImageReader(resource_root + QStringLiteral("close.svg")).size();
        QSize fallback_original;
        const QImage fallback =
            provider->requestImage(QStringLiteral("close/ff2878b8"), &fallback_original, {});
        QVERIFY(!fallback.isNull());
        QCOMPARE(fallback_original, fallback_size);
        QCOMPARE(fallback.size(), fallback_size);
        QVERIFY(provider->requestImage(QStringLiteral("../new/ffffffff"), nullptr, {}).isNull());
        QVERIFY(provider->requestImage(QStringLiteral("ribbon-project/ffffffff"), nullptr, {})
                    .isNull());
        QVERIFY(provider->requestImage(QStringLiteral("close/not-a-color"), nullptr, {}).isNull());
    }
};

QTEST_MAIN(ThemeAtomTest)
#include "theme_component_test.moc"
