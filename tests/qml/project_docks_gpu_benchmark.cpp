// 开发侧真实窗口基准：测量工程 / Tasks / Layers 场景端到端的帧呈现间隔。
#include "quick_item_helpers.hpp"
#include <QByteArray>
#include <QCoreApplication>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QGuiApplication>
#include <QObject>
#include <QPoint>
#include <QPointF>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSGRendererInterface>
#include <QSize>
#include <QString>
#include <QTimer>
#include <QUrl>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qlogging.h>
#include <QtCore/qnamespace.h>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtGui/qtestsupport_gui.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <algorithm>
#include <array>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <icon_provider.hpp>
#include <utility>
#include <vector>

namespace {

constexpr int kFrameCountPerSample = 60;
constexpr int kSampleCount = 3;
constexpr int kWarmupFrameCount = 30;
constexpr std::array kItemCounts = {0, 1, 100, 1000};

enum class Panels : std::uint8_t { Empty, Tasks, Layers, Both };

struct Scenario {
    const char* name;
    Panels panels;
    int item_count;
};

QStringList make_names(int count) {
    QStringList names;
    names.reserve(count);
    for (int index = 0; index < count; ++index) {
        names.append(QStringLiteral("part_%1.stl").arg(index));
    }
    return names;
}

std::vector<double> percentiles(std::vector<double> values) {
    std::sort(values.begin(), values.end());
    const auto p95_index =
        static_cast<std::size_t>(std::ceil(static_cast<double>(values.size()) * 0.95)) - 1;
    return {values[values.size() / 2], values[p95_index]};
}

const char* graphics_api_name(QSGRendererInterface::GraphicsApi api) {
    switch (api) {
    case QSGRendererInterface::Unknown:
        return "Unknown";
    case QSGRendererInterface::Software:
        return "Software";
    case QSGRendererInterface::OpenGL:
        return "OpenGL";
    case QSGRendererInterface::Direct3D11:
        return "Direct3D11";
    case QSGRendererInterface::Vulkan:
        return "Vulkan";
    case QSGRendererInterface::Metal:
        return "Metal";
    case QSGRendererInterface::Null:
        return "Null";
    case QSGRendererInterface::OpenVG:
        return "OpenVG";
    case QSGRendererInterface::Direct3D12:
        return "Direct3D12";
    }
    return "Other";
}

} // namespace

class ProjectDocksGpuBenchmark final : public QObject {
    Q_OBJECT

  private:
    QQmlEngine m_engine;
    QQuickWindow m_window;
    QQmlComponent m_emptyComponent{&m_engine};
    QQmlComponent m_tasksComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/TasksPanel.qml"))};
    QQmlComponent m_layersComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/LayersPanel.qml"))};

    QObject* create(QQmlComponent& component, const QVariantMap& properties, QObject& owner,
                    const QPoint& position, const QSize& size) {
        QObject* object = component.createWithInitialProperties(properties);
        if (object == nullptr) {
            QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
            return nullptr;
        }
        object->setParent(&owner);
        if (auto* item = qobject_cast<QQuickItem*>(object)) {
            item->setPosition(QPointF(position));
            item->setWidth(size.width());
            item->setHeight(size.height());
            item->setParentItem(m_window.contentItem());
        }
        return object;
    }

    std::vector<double> measure_frame_intervals(int frame_count) {
        QElapsedTimer clock;
        clock.start();
        qint64 previous_frame = -1;
        std::vector<double> intervals;
        intervals.reserve(frame_count);
        QEventLoop loop;
        const auto connection = connect(
            &m_window, &QQuickWindow::frameSwapped, &loop,
            [&] {
                const qint64 now = clock.nsecsElapsed();
                if (previous_frame >= 0) {
                    intervals.push_back(static_cast<double>(now - previous_frame) / 1.0e6);
                }
                previous_frame = now;
                if (static_cast<int>(intervals.size()) >= frame_count) {
                    loop.quit();
                } else {
                    m_window.update();
                }
            },
            Qt::QueuedConnection);
        QTimer::singleShot(15000, &loop, &QEventLoop::quit);
        m_window.update();
        loop.exec();
        disconnect(connection);
        if (static_cast<int>(intervals.size()) != frame_count) {
            QTest::qFail("Timed out waiting for the requested number of presented frames", __FILE__,
                         __LINE__);
            return {};
        }
        return intervals;
    }

    void run_scenario(const Scenario& scenario, const char* api_name) {
        QObject owner;
        const QStringList names = make_names(scenario.item_count);
        const QVariantMap names_property{{QStringLiteral("importedPartNames"), names}};
        QQuickItem* tasks_item = nullptr;
        QQuickItem* layers_item = nullptr;
        if (scenario.panels == Panels::Empty) {
            create(m_emptyComponent, {}, owner, QPoint(0, 0), QSize(440, 700));
        } else {
            if (scenario.panels == Panels::Tasks || scenario.panels == Panels::Both) {
                QVariantMap properties = names_property;
                properties.insert(QStringLiteral("projectOpen"), true);
                properties.insert(QStringLiteral("projectName"), QStringLiteral("benchmark"));
                properties.insert(QStringLiteral("importedPartName"),
                                  names.isEmpty() ? QString{} : names.constLast());
                tasks_item = qobject_cast<QQuickItem*>(
                    create(m_tasksComponent, properties, owner, QPoint(0, 0),
                           QSize(440, scenario.panels == Panels::Both ? 420 : 700)));
            }
            if (scenario.panels == Panels::Layers || scenario.panels == Panels::Both) {
                layers_item = qobject_cast<QQuickItem*>(
                    create(m_layersComponent, names_property, owner,
                           QPoint(0, scenario.panels == Panels::Both ? 420 : 0),
                           QSize(440, scenario.panels == Panels::Both ? 280 : 700)));
            }
        }
        QCoreApplication::processEvents(QEventLoop::AllEvents, 50);
        QCOMPARE(static_cast<int>(owner.children().size()),
                 scenario.panels == Panels::Both ? 2 : 1);
        QQuickItem* tree = tasks_item == nullptr
                               ? nullptr
                               : visual_item(tasks_item, QStringLiteral("projectTreeSection"));
        QCOMPARE(tree == nullptr ? 0 : tree->property("count").toInt(),
                 scenario.panels == Panels::Tasks || scenario.panels == Panels::Both
                     ? scenario.item_count
                     : 0);
        const int realized_rows =
            tasks_item == nullptr
                ? 0
                : static_cast<int>(
                      visual_items(tasks_item, QStringLiteral("importedPartEntry")).size());
        QVERIFY(realized_rows <= scenario.item_count);
        if ((scenario.panels == Panels::Tasks || scenario.panels == Panels::Both) &&
            scenario.item_count > 0) {
            QVERIFY(realized_rows > 0);
        }
        QQuickItem* tab_row = layers_item == nullptr
                                  ? nullptr
                                  : visual_item(layers_item, QStringLiteral("layersTabRow"));
        QCOMPARE(tab_row != nullptr && tab_row->property("visible").toBool(),
                 (scenario.panels == Panels::Layers || scenario.panels == Panels::Both) &&
                     scenario.item_count > 0);

        if (measure_frame_intervals(kWarmupFrameCount).size() != kWarmupFrameCount) {
            return;
        }
        std::vector<double> intervals;
        intervals.reserve(static_cast<std::size_t>(kFrameCountPerSample) * kSampleCount);
        for (int sample = 0; sample < kSampleCount; ++sample) {
            const auto sample_intervals = measure_frame_intervals(kFrameCountPerSample);
            if (sample_intervals.size() != kFrameCountPerSample) {
                return;
            }
            intervals.insert(intervals.end(), sample_intervals.begin(), sample_intervals.end());
        }
        const auto summary = percentiles(std::move(intervals));
        qInfo().nospace() << "QML GPU frame presentation (" << api_name << ", " << scenario.name
                          << ", " << scenario.item_count << " imported items, " << kSampleCount
                          << " x " << kFrameCountPerSample << " frames; realized task rows "
                          << realized_rows << "; p50/p95 ms per frame): " << summary[0] << "/"
                          << summary[1];
    }

  private slots:
    void initTestCase() {
        panta::install_icon_provider(m_engine);
        m_emptyComponent.setData(QByteArrayLiteral("import QtQuick\nItem {}"),
                                 QUrl(QStringLiteral("qrc:/benchmark/Empty.qml")));
        QVERIFY(m_emptyComponent.isReady());
        QVERIFY2(m_tasksComponent.isReady(), qPrintable(m_tasksComponent.errorString()));
        QVERIFY2(m_layersComponent.isReady(), qPrintable(m_layersComponent.errorString()));

        if (QGuiApplication::platformName() == QStringLiteral("offscreen") ||
            QGuiApplication::platformName() == QStringLiteral("minimal")) {
            QSKIP("GPU frame benchmark requires a visible native graphics session");
        }
        m_window.resize(1000, 700);
        m_window.show();
        QVERIFY2(QTest::qWaitForWindowExposed(&m_window),
                 "Native benchmark window was not exposed");
        const auto api = m_window.rendererInterface()->graphicsApi();
        if (api == QSGRendererInterface::Unknown || api == QSGRendererInterface::Software ||
            api == QSGRendererInterface::Null) {
            QSKIP("A hardware-backed Qt Quick renderer is not available");
        }
    }

    void measures_visible_frame_presentation_ablation() {
        std::vector<Scenario> cases{{"empty", Panels::Empty, 0}};
        for (const int item_count : kItemCounts) {
            cases.push_back({"tasks", Panels::Tasks, item_count});
            cases.push_back({"layers", Panels::Layers, item_count});
            cases.push_back({"both", Panels::Both, item_count});
        }
        const char* api = graphics_api_name(m_window.rendererInterface()->graphicsApi());
        qInfo().nospace()
            << "Qt Quick renderer: " << api
            << "; timings include compositor/vsync and are end-to-end frame intervals, "
               "not GPU kernel timings";
        for (const Scenario& scenario : cases) {
            run_scenario(scenario, api);
        }
    }
};

QTEST_MAIN(ProjectDocksGpuBenchmark)
#include "project_docks_gpu_benchmark.moc"
