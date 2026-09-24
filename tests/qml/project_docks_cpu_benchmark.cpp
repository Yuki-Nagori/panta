// 开发侧 QML 构造消融基准：比较工程树、Layers 面板及组合的 CPU 构造成本。
#include "quick_item_helpers.hpp"
#include <QByteArray>
#include <QCoreApplication>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QGuiApplication>
#include <QObject>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QString>
#include <QStringList>
#include <QUrl>
#include <QVariantMap>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qlogging.h>
#include <QtCore/qobjectdefs.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <algorithm>
#include <array>
#include <cmath>
#include <cstdint>
#include <functional>
#include <icon_provider.hpp>
#include <iterator>
#include <utility>
#include <vector>

namespace {

constexpr int kSampleCount = 31;
constexpr int kItemCounts[] = {0, 1, 100, 1000};

enum class Panels : std::uint8_t { Empty, Tasks, Layers, Both };

struct Sample {
    qint64 elapsed_nanoseconds = 0;
    int task_model_rows = 0;
    int task_delegate_rows = 0;
    bool layers_tab_visible = false;
};

struct Scenario {
    QString suite;
    QString name;
    int input_count = 0;
    std::function<Sample()> run;
    std::function<void(const Sample&)> verify;
};

struct Summary {
    double median_microseconds = 0.0;
    double p95_microseconds = 0.0;
};

QStringList make_names(int count) {
    QStringList names;
    names.reserve(count);
    for (int index = 0; index < count; ++index) {
        names.append(QStringLiteral("part_%1.stl").arg(index));
    }
    return names;
}

Summary summarize(std::vector<double> values) {
    std::sort(values.begin(), values.end());
    const auto percentile_index = [](double percentile) {
        return static_cast<int>(std::ceil(percentile * kSampleCount)) - 1;
    };
    return {values[kSampleCount / 2], values[percentile_index(0.95)]};
}

} // namespace

class QmlPerformanceBenchmark final : public QObject {
    Q_OBJECT

  private:
    QQmlEngine m_engine;
    QQuickWindow m_window;
    QQmlComponent m_emptyComponent{&m_engine};
    QQmlComponent m_tasksComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/TasksPanel.qml"))};
    QQmlComponent m_layersComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/LayersPanel.qml"))};

    QObject* create(QQmlComponent& component, const QVariantMap& properties, QObject& owner) {
        QObject* object = component.createWithInitialProperties(properties);
        if (object == nullptr) {
            QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
            return nullptr;
        }
        object->setParent(&owner);
        auto* item = qobject_cast<QQuickItem*>(object);
        if (item != nullptr) {
            item->setWidth(440);
            item->setHeight(320);
            item->setParentItem(m_window.contentItem());
        }
        return object;
    }

    Sample measure_once(Panels panels, const QStringList& names) {
        QObject owner;
        QElapsedTimer timer;
        timer.start();

        const QVariantMap names_property{{QStringLiteral("importedPartNames"), names}};
        QObject* tasks = nullptr;
        QObject* layers = nullptr;
        if (panels == Panels::Empty) {
            create(m_emptyComponent, {}, owner);
        } else {
            if (panels == Panels::Tasks || panels == Panels::Both) {
                QVariantMap properties = names_property;
                properties.insert(QStringLiteral("projectOpen"), true);
                properties.insert(QStringLiteral("projectName"), QStringLiteral("benchmark"));
                properties.insert(QStringLiteral("importedPartName"),
                                  names.isEmpty() ? QString{} : names.constLast());
                tasks = create(m_tasksComponent, properties, owner);
            }
            if (panels == Panels::Layers || panels == Panels::Both) {
                layers = create(m_layersComponent, names_property, owner);
            }
        }

        QCoreApplication::processEvents(QEventLoop::AllEvents, 20);
        const qint64 elapsed = timer.nsecsElapsed();
        auto* tasks_item = qobject_cast<QQuickItem*>(tasks);
        auto* tree = tasks_item == nullptr
                         ? nullptr
                         : visual_item(tasks_item, QStringLiteral("projectTreeSection"));
        const int task_model_rows = tree == nullptr ? 0 : tree->property("count").toInt();
        const int task_delegate_rows =
            tasks_item == nullptr
                ? 0
                : static_cast<int>(
                      visual_items(tasks_item, QStringLiteral("importedPartEntry")).size());
        auto* layers_item = qobject_cast<QQuickItem*>(layers);
        auto* tab_row = layers_item == nullptr
                            ? nullptr
                            : visual_item(layers_item, QStringLiteral("layersTabRow"));
        const bool layers_tab_visible = tab_row != nullptr && tab_row->property("visible").toBool();
        return {elapsed, task_model_rows, task_delegate_rows, layers_tab_visible};
    }

    void verify_sample(Panels panels, int expected_rows, const Sample& sample) {
        const bool includes_tasks = panels == Panels::Tasks || panels == Panels::Both;
        const bool includes_layers = panels == Panels::Layers || panels == Panels::Both;
        QCOMPARE(sample.task_model_rows, includes_tasks ? expected_rows : 0);
        QVERIFY(sample.task_delegate_rows <= sample.task_model_rows);
        if (includes_tasks && expected_rows > 0) {
            QVERIFY(sample.task_delegate_rows > 0);
        }
        QCOMPARE(sample.layers_tab_visible, includes_layers && expected_rows > 0);
    }

    using PanelCases = std::array<std::pair<Panels, const char*>, 4>;

    std::vector<Scenario> project_dock_scenarios(const PanelCases& cases) {
        std::vector<Scenario> scenarios;
        scenarios.reserve(std::size(cases) * std::size(kItemCounts));
        for (const auto& [panels, label] : cases) {
            for (const int item_count : kItemCounts) {
                const QStringList names = make_names(item_count);
                scenarios.push_back({QStringLiteral("project docks"), QString::fromLatin1(label),
                                     item_count,
                                     [this, panels, names] { return measure_once(panels, names); },
                                     [this, panels, item_count](const Sample& sample) {
                                         verify_sample(panels, item_count, sample);
                                     }});
            }
        }
        return scenarios;
    }

    void run_scenario(const Scenario& scenario) {
        const Sample warmup = scenario.run();
        scenario.verify(warmup);
        std::vector<double> samples;
        samples.reserve(kSampleCount);
        for (int sample_index = 0; sample_index < kSampleCount; ++sample_index) {
            const Sample sample = scenario.run();
            scenario.verify(sample);
            samples.push_back(static_cast<double>(sample.elapsed_nanoseconds) / 1000.0);
        }
        const Summary summary = summarize(std::move(samples));
        qInfo().nospace() << "QML CPU construction (" << scenario.suite << ", " << scenario.name
                          << ", " << scenario.input_count << " items, " << kSampleCount
                          << " samples; model/realized rows " << warmup.task_model_rows << "/"
                          << warmup.task_delegate_rows
                          << "; p50/p95 microseconds): " << summary.median_microseconds << "/"
                          << summary.p95_microseconds;
    }

  private slots:
    void initTestCase() {
        panta::install_icon_provider(m_engine);
        m_emptyComponent.setData(QByteArrayLiteral("import QtQuick\nItem {}"),
                                 QUrl(QStringLiteral("qrc:/benchmark/Empty.qml")));
        QVERIFY(m_emptyComponent.isReady());
        QVERIFY2(m_tasksComponent.isReady(), qPrintable(m_tasksComponent.errorString()));
        QVERIFY2(m_layersComponent.isReady(), qPrintable(m_layersComponent.errorString()));
    }

    void measures_registered_scenarios() {
        const std::array<std::pair<Panels, const char*>, 4> cases = {{{Panels::Empty, "empty"},
                                                                      {Panels::Tasks, "tasks"},
                                                                      {Panels::Layers, "layers"},
                                                                      {Panels::Both, "both"}}};
        for (const Scenario& scenario : project_dock_scenarios(cases)) {
            run_scenario(scenario);
        }
    }
};

QTEST_MAIN(QmlPerformanceBenchmark)
#include "project_docks_cpu_benchmark.moc"
