#include "../../support/qt/frame_submission_capture.hpp"
#include "benchmark_statistics.hpp"
#if defined(PANTA_TEST_WITH_VTK)
#include "vtk_error_capture.hpp"
#include <vtkCommand.h>
#include <vtkOutputWindow.h>
#endif
#include <QLoggingCategory>
#include <QObject>
#include <QString>
#include <QTimer>
#include <QtCore/qbytearrayalgorithms.h>
#include <QtCore/qlogging.h>
#include <QtCore/qtmetamacros.h>
#include <QtTest/qtest.h>
#include <QtTest/qtestcase.h>
#include <chrono>
#include <cstddef>
#include <limits>
#include <stdexcept>
#include <vector>

namespace {
Q_LOGGING_CATEGORY(viewportLog, "panta.viewport")
int forwarded = 0;
void previous_handler(QtMsgType, const QMessageLogContext&, const QString&) { ++forwarded; }
void disable_viewport(QLoggingCategory* category) {
    if (qstrcmp(category->categoryName(), "panta.viewport") == 0) {
        category->setEnabled(QtDebugMsg, false);
    }
}
} // namespace

class BenchmarkSupportTest final : public QObject {
    Q_OBJECT
  private slots:
#if defined(PANTA_TEST_WITH_VTK)
    void propagates_vtk_errors_and_restores_observer() {
        auto* output = vtkOutputWindow::GetInstance();
        const bool previously_observed = output->HasObserver(vtkCommand::ErrorEvent) != 0;
        {
            panta::test::VtkErrorCapture capture;
            capture.check();
            output->InvokeEvent(vtkCommand::ErrorEvent);
            QVERIFY_THROWS_EXCEPTION(std::runtime_error, capture.check());
        }
        QCOMPARE(output->HasObserver(vtkCommand::ErrorEvent) != 0, previously_observed);
        panta::test::VtkErrorCapture next;
        next.check();
    }
#endif
    void rejects_invalid_timings() {
        QVERIFY(!panta::test::summarize_timings({}));
        for (double value : {-1.0, std::numeric_limits<double>::infinity(),
                             std::numeric_limits<double>::quiet_NaN()}) {
            QVERIFY(!panta::test::summarize_timings(std::vector<double>{value}));
        }
    }
    void uses_nearest_rank() {
        const auto result = panta::test::summarize_timings(std::vector<double>{4, 0, 3, 1});
        if (!result) {
            QFAIL("Expected valid timing summary");
            return;
        }
        QCOMPARE(result->p50, 1.0);
        QCOMPARE(result->p95, 4.0);
    }
    void restores_filter() {
        const auto previous = QLoggingCategory::installFilter(disable_viewport);
        bool enabled = false;
        {
            panta::test::FrameSubmissionCapture capture;
            enabled = viewportLog().isDebugEnabled();
        }
        const bool restored = !viewportLog().isDebugEnabled();
        QLoggingCategory::installFilter(previous);
        QVERIFY(enabled);
        QVERIFY(restored);
    }
    void captures_and_restores_handler() {
        const auto previous = qInstallMessageHandler(previous_handler);
        forwarded = 0;
        {
            panta::test::FrameSubmissionCapture capture;
            QVERIFY_THROWS_EXCEPTION(std::logic_error, panta::test::FrameSubmissionCapture{});
            qCDebug(viewportLog) << "frame submitted";
            QCOMPARE(capture.count(), std::size_t{1});
            QVERIFY(capture.wait_after(0, std::chrono::milliseconds{10}));
            QTimer::singleShot(0, [] { qCDebug(viewportLog) << "frame submitted"; });
            QVERIFY(capture.wait_after(1, std::chrono::milliseconds{1000}));
            QVERIFY(!capture.wait_after(2, std::chrono::milliseconds{1}));
            qWarning("other diagnostic");
            QCOMPARE(forwarded, 1);
            qCDebug(viewportLog) << "surface synchronized";
            QCOMPARE(capture.surface_sync_count(), std::size_t{1});
            capture.reset();
            QCOMPARE(capture.count(), std::size_t{0});
            QCOMPARE(capture.surface_sync_count(), std::size_t{0});
        }
        qWarning("restored diagnostic");
        QCOMPARE(forwarded, 2);
        qInstallMessageHandler(previous);
    }
};

QTEST_GUILESS_MAIN(BenchmarkSupportTest)
#include "benchmark_support_test.moc"
