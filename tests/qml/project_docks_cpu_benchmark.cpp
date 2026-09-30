// 开发侧 QML 构造消融基准：比较工程树、Layers 面板及组合的 CPU 构造成本。
#include "analysis_sequence_helpers.hpp"
#include "quick_item_helpers.hpp"
#include <QByteArray>
#include <QCoreApplication>
#include <QElapsedTimer>
#include <QEventLoop>
#include <QGuiApplication>
#include <QMetaEnum>
#include <QMetaProperty>
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
// 文档页签代表性规模：单页签（Welcome 态）、少量打开、多页签滚动。
constexpr int kDocumentTabCounts[] = {1, 8, 24};
constexpr int kDocumentSwitchRounds = 32;
constexpr int kIconCounts[] = {1, 8, 24};

enum class Panels : std::uint8_t {
    Empty,
    Tasks,
    Layers,
    Both,
    MeshTool,
    AnalysisSequence,
    AnalysisSequenceDialog
};
enum class DocumentTabWorkload : std::uint8_t { Construct, Switch, Close };
enum class IconSourceMode : std::uint8_t { Empty, MonochromeProvider, SourceColors };

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

struct IconLoadSample {
    qint64 elapsed_nanoseconds = 0;
    int loaded_icons = 0;
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
    QVariantList m_analysisSequences;
    QQuickWindow m_window;
    QQmlComponent m_emptyComponent{&m_engine};
    QQmlComponent m_tasksComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/TasksPanel.qml"))};
    QQmlComponent m_layersComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/LayersPanel.qml"))};
    QQmlComponent m_analysisSequenceDialogComponent{
        &m_engine,
        QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Dialogs/AnalysisSequenceDialog.qml"))};
    QQmlComponent m_analysisSequenceComponent{
        &m_engine,
        QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/AnalysisSequencePanel.qml"))};
    QQmlComponent m_meshToolComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Panels/MeshToolPanel.qml"))};
    QQmlComponent m_documentTabBarComponent{
        &m_engine,
        QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Composites/DocumentTabBar.qml"))};
    QQmlComponent m_themedIconComponent{
        &m_engine, QUrl(QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedIcon.qml"))};

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
            if (panels == Panels::AnalysisSequenceDialog) {
                create(m_analysisSequenceDialogComponent,
                       QVariantMap{{QStringLiteral("sequences"), m_analysisSequences}}, owner);
            }
            if (panels == Panels::AnalysisSequence) {
                create(m_analysisSequenceComponent,
                       QVariantMap{{QStringLiteral("sequences"), m_analysisSequences}}, owner);
            }
            if (panels == Panels::MeshTool) {
                create(m_meshToolComponent, {}, owner);
            }
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

    /// 文档页签的输入集合；全部就绪 + 每第 7 个失败态，覆盖状态渲染分支。
    QVariantList make_documents(int count) const {
        QVariantList documents;
        documents.reserve(count);
        for (int index = 0; index < count; ++index) {
            const bool failed = index > 0 && index % 7 == 6;
            documents.append(QVariantMap{
                {QStringLiteral("id"), QStringLiteral("doc-%1").arg(index)},
                {QStringLiteral("kind"),
                 index == 0 ? QStringLiteral("welcome") : QStringLiteral("import")},
                {QStringLiteral("state"),
                 failed ? QStringLiteral("failed") : QStringLiteral("ready")},
                {QStringLiteral("title"), failed ? QStringLiteral("broken_%1.stl").arg(index)
                                                 : QStringLiteral("part_%1.stl").arg(index)},
                {QStringLiteral("message"), QString{}}});
        }
        return documents;
    }

    QStringList document_ids(int count) const {
        QStringList ids;
        ids.reserve(count);
        for (int index = 0; index < count; ++index) {
            ids.append(QStringLiteral("doc-%1").arg(index));
        }
        return ids;
    }

    /// 单独计时构造、活动态更新和关闭后的模型更新；输入准备不计入耗时。
    Sample measure_document_tabs(int count, DocumentTabWorkload workload) {
        QObject owner;
        const QStringList ids = document_ids(count);
        const QVariantList documents = make_documents(count);
        const auto properties =
            QVariantMap{{QStringLiteral("objectName"), QStringLiteral("documentTabBar")},
                        {QStringLiteral("documents"), documents},
                        {QStringLiteral("activeDocumentId"), ids.constFirst()}};
        QElapsedTimer timer;
        timer.start();
        QObject* bar_object = create(m_documentTabBarComponent, properties, owner);
        if (bar_object == nullptr) {
            QTest::qFail(qPrintable(m_documentTabBarComponent.errorString()), __FILE__, __LINE__);
            return {};
        }
        auto* bar = qobject_cast<QQuickItem*>(bar_object);
        bar->setHeight(bar->implicitHeight());
        QCoreApplication::processEvents(QEventLoop::AllEvents, 20);
        if (workload != DocumentTabWorkload::Construct) {
            timer.restart();
            if (workload == DocumentTabWorkload::Switch) {
                for (int round = 0; round < kDocumentSwitchRounds; ++round) {
                    bar->setProperty("activeDocumentId", ids[round % ids.size()]);
                }
            } else {
                bar->setProperty("documents", documents.mid(0, count - 1));
            }
            QCoreApplication::processEvents(QEventLoop::AllEvents, 20);
        }
        const qint64 elapsed = timer.nsecsElapsed();
        const int realized_closes =
            static_cast<int>(visual_items(bar, QStringLiteral("documentTabClose")).size());
        return {elapsed, realized_closes, realized_closes, false};
    }

    QStringList icon_names(IconSourceMode mode, int count) const {
        static const QStringList monochrome_icons{
            QStringLiteral("undo"),    QStringLiteral("redo"),    QStringLiteral("print"),
            QStringLiteral("preview"), QStringLiteral("account"), QStringLiteral("cart"),
            QStringLiteral("help"),    QStringLiteral("close")};
        static const QStringList source_color_icons{
            QStringLiteral("new"),
            QStringLiteral("open"),
            QStringLiteral("save"),
            QStringLiteral("project-file"),
            QStringLiteral("project-folder"),
            QStringLiteral("stl-file"),
            QStringLiteral("plan-tasks"),
            QStringLiteral("status-ok"),
            QStringLiteral("task-analysis"),
            QStringLiteral("task-analysis-sequence"),
            QStringLiteral("task-injection"),
            QStringLiteral("task-material"),
            QStringLiteral("task-mesh"),
            QStringLiteral("task-optimization"),
            QStringLiteral("task-settings"),
            QStringLiteral("log"),
            QStringLiteral("ribbon-start-here"),
            QStringLiteral("ribbon-new-features"),
            QStringLiteral("ribbon-tutorials"),
            QStringLiteral("ribbon-videos"),
            QStringLiteral("ribbon-help"),
            QStringLiteral("ribbon-project"),
            QStringLiteral("ribbon-open-project"),
            QStringLiteral("ribbon-import"),
            QStringLiteral("ribbon-add"),
            QStringLiteral("ribbon-dual-domain"),
            QStringLiteral("ribbon-geometry"),
            QStringLiteral("ribbon-mesh"),
            QStringLiteral("ribbon-thermoplastics-injection-molding"),
            QStringLiteral("ribbon-analysis-sequence"),
            QStringLiteral("ribbon-select-material"),
            QStringLiteral("ribbon-injection-locations"),
            QStringLiteral("ribbon-process-settings"),
            QStringLiteral("ribbon-optimization"),
            QStringLiteral("ribbon-boundary-conditions"),
            QStringLiteral("ribbon-analyze"),
            QStringLiteral("ribbon-job-manager"),
            QStringLiteral("ribbon-results"),
            QStringLiteral("ribbon-reports"),
            QStringLiteral("ribbon-shared-views"),
            QStringLiteral("ribbon-logs")};

        QStringList names;
        names.reserve(count);
        if (mode == IconSourceMode::Empty) {
            return names;
        }
        const QStringList& palette =
            mode == IconSourceMode::MonochromeProvider ? monochrome_icons : source_color_icons;
        // Use separate symbol slices for 1/8/24 color-icon rows to keep their first samples useful.
        const int start = count == 1 ? 0 : count == 8 ? 1 : 9;
        for (int index = 0; index < count; ++index) {
            names.append(palette[(start + index) % palette.size()]);
        }
        return names;
    }

    QByteArray image_status_name(QObject& image) const {
        const QMetaObject* meta_object = image.metaObject();
        const int status_index = meta_object->indexOfProperty("status");
        if (status_index < 0) {
            return {};
        }
        const QMetaProperty status_property = meta_object->property(status_index);
        return status_property.enumerator().valueToKey(status_property.read(&image).toInt());
    }

    IconLoadSample measure_icon_loading(IconSourceMode mode, int count) {
        QObject owner;
        const QStringList names = icon_names(mode, count);
        std::vector<QObject*> icons;
        icons.reserve(count);
        QElapsedTimer timer;
        timer.start();

        for (int index = 0; index < count; ++index) {
            QVariantMap properties{
                {QStringLiteral("name"), names.isEmpty() ? QString{} : names[index]},
                {QStringLiteral("iconSize"), 26},
                {QStringLiteral("preserveSourceColors"), mode == IconSourceMode::SourceColors}};
            QObject* object = m_themedIconComponent.createWithInitialProperties(properties);
            if (object == nullptr) {
                QTest::qFail(qPrintable(m_themedIconComponent.errorString()), __FILE__, __LINE__);
                return {};
            }
            object->setParent(&owner);
            auto* item = qobject_cast<QQuickItem*>(object);
            if (item == nullptr) {
                QTest::qFail("ThemedIcon did not create a QQuickItem", __FILE__, __LINE__);
                return {};
            }
            const int x = (index % 12) * 28;
            const int y = (index / 12) * 28;
            item->setPosition(QPointF(static_cast<qreal>(x), static_cast<qreal>(y)));
            item->setWidth(26);
            item->setHeight(26);
            item->setParentItem(m_window.contentItem());
            icons.push_back(object);
        }

        int loaded_icons = 0;
        if (mode == IconSourceMode::Empty) {
            QCoreApplication::processEvents(QEventLoop::AllEvents, 10);
        } else {
            QElapsedTimer wait_timer;
            wait_timer.start();
            while (wait_timer.elapsed() < 5000) {
                QCoreApplication::processEvents(QEventLoop::AllEvents, 10);
                loaded_icons = 0;
                bool pending = false;
                for (QObject* icon : icons) {
                    const QByteArray status = image_status_name(*icon);
                    if (status == "Error") {
                        QTest::qFail(qPrintable(QStringLiteral("Failed to load icon '%1'")
                                                    .arg(icon->property("name").toString())),
                                     __FILE__, __LINE__);
                        return {};
                    }
                    if (status == "Ready") {
                        ++loaded_icons;
                    } else {
                        pending = true;
                    }
                }
                if (!pending) {
                    break;
                }
            }
            if (loaded_icons != count) {
                QTest::qFail("Timed out waiting for QML icon images to become ready", __FILE__,
                             __LINE__);
                return {};
            }
        }
        return {timer.nsecsElapsed(), loaded_icons};
    }

    void run_icon_loading_scenarios() {
        const std::array<std::pair<IconSourceMode, const char*>, 3> modes = {
            {{IconSourceMode::Empty, "empty source baseline"},
             {IconSourceMode::MonochromeProvider, "monochrome provider"},
             {IconSourceMode::SourceColors, "source-color qrc"}}};
        for (const auto& [mode, label] : modes) {
            for (const int count : kIconCounts) {
                const IconLoadSample first_load = measure_icon_loading(mode, count);
                QCOMPARE(first_load.loaded_icons, mode == IconSourceMode::Empty ? 0 : count);
                std::vector<double> cached_samples;
                cached_samples.reserve(kSampleCount);
                for (int sample_index = 0; sample_index < kSampleCount; ++sample_index) {
                    const IconLoadSample sample = measure_icon_loading(mode, count);
                    QCOMPARE(sample.loaded_icons, mode == IconSourceMode::Empty ? 0 : count);
                    cached_samples.push_back(static_cast<double>(sample.elapsed_nanoseconds) /
                                             1000.0);
                }
                const Summary summary = summarize(std::move(cached_samples));
                qInfo().nospace() << "QML CPU icon load (" << label << ", " << count
                                  << " items; initial sample / cached p50 / p95 microseconds): "
                                  << static_cast<double>(first_load.elapsed_nanoseconds) / 1000.0
                                  << "/" << summary.median_microseconds << "/"
                                  << summary.p95_microseconds;
            }
        }
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
        qInfo().nospace() << "QML CPU workload (" << scenario.suite << ", " << scenario.name << ", "
                          << scenario.input_count << " items, " << kSampleCount
                          << " samples; model/realized rows " << warmup.task_model_rows << "/"
                          << warmup.task_delegate_rows
                          << "; p50/p95 microseconds): " << summary.median_microseconds << "/"
                          << summary.p95_microseconds;
    }

  private slots:

    void initTestCase() {
        panta::install_icon_provider(m_engine);
        m_analysisSequences = analysis_sequence_catalog();
        m_emptyComponent.setData(QByteArrayLiteral("import QtQuick\nItem {}"),
                                 QUrl(QStringLiteral("qrc:/benchmark/Empty.qml")));
        QVERIFY(m_emptyComponent.isReady());
        QVERIFY2(m_tasksComponent.isReady(), qPrintable(m_tasksComponent.errorString()));
        QVERIFY2(m_layersComponent.isReady(), qPrintable(m_layersComponent.errorString()));
        QVERIFY2(m_meshToolComponent.isReady(), qPrintable(m_meshToolComponent.errorString()));
        QVERIFY2(m_themedIconComponent.isReady(), qPrintable(m_themedIconComponent.errorString()));
    }

    std::vector<Scenario> document_tab_scenarios() {
        std::vector<Scenario> scenarios;
        scenarios.reserve(std::size(kDocumentTabCounts) * 3);
        for (const int count : kDocumentTabCounts) {
            for (const auto [workload, label] :
                 {std::pair{DocumentTabWorkload::Construct, "construct"},
                  std::pair{DocumentTabWorkload::Switch, "switch 32 times"},
                  std::pair{DocumentTabWorkload::Close, "close last"}}) {
                if (count == 1 && workload != DocumentTabWorkload::Construct) {
                    continue;
                }
                scenarios.push_back(
                    {QStringLiteral("viewport document tabs"), QString::fromLatin1(label), count,
                     [this, count, workload] { return measure_document_tabs(count, workload); },
                     [count, workload](const Sample& sample) {
                         const int expected =
                             count - (workload == DocumentTabWorkload::Close ? 1 : 0);
                         QCOMPARE(sample.task_model_rows, expected);
                     }});
            }
        }
        return scenarios;
    }

    void measures_registered_scenarios() {
        const std::array<std::pair<Panels, const char*>, 4> cases = {{{Panels::Empty, "empty"},
                                                                      {Panels::Tasks, "tasks"},
                                                                      {Panels::Layers, "layers"},
                                                                      {Panels::Both, "both"}}};
        for (const Scenario& scenario : project_dock_scenarios(cases)) {
            run_scenario(scenario);
        }
        for (const Scenario& scenario : document_tab_scenarios()) {
            run_scenario(scenario);
        }
    }

    void measures_icon_loading() { run_icon_loading_scenarios(); }

    void measures_analysis_sequence_construct() {
#if !defined(PANTA_TEST_WITH_BRIDGE)
        QSKIP("Analysis sequence catalog requires the enabled Bridge module");
#endif
        QVERIFY(!m_analysisSequences.isEmpty());
        for (const auto [panels, name] :
             {std::pair{Panels::Empty, "empty"},
              std::pair{Panels::AnalysisSequence, "analysis sequence"},
              std::pair{Panels::AnalysisSequenceDialog, "analysis sequence dialog"}}) {
            run_scenario(
                {QStringLiteral("analysis sequence"), QString::fromLatin1(name), 1,
                 [this, panels] { return measure_once(panels, {}); },
                 [this, panels](const Sample& sample) { verify_sample(panels, 0, sample); }});
        }
    }

    void measures_mesh_tool_construct() {
        for (const auto [panels, name] :
             {std::pair{Panels::Empty, "empty"}, std::pair{Panels::MeshTool, "mesh tool"}}) {
            run_scenario(
                {QStringLiteral("mesh tool"), QString::fromLatin1(name), 1,
                 [this, panels] { return measure_once(panels, {}); },
                 [this, panels](const Sample& sample) { verify_sample(panels, 0, sample); }});
        }
    }
};

QTEST_MAIN(QmlPerformanceBenchmark)
#include "project_docks_cpu_benchmark.moc"
