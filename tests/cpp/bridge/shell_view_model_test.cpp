// ShellViewModel 的属性通知与命令验证（gtest.md + qt.md）。
// QSignalSpy 属 QtTest；自定义 main 提供 QCoreApplication。

#include "shell_view_model.hpp"
#include <QCoreApplication>
#include <QSignalSpy>
#include <QString>
#include <QtTest/qsignalspy.h>
#include <gtest/gtest.h>

namespace {

constexpr int kInitialCount = 0;

} // namespace

TEST(ShellViewModel, TickAdvancesCountAndUpdatesCaption) {
    panta::bridge::ShellViewModel viewModel;
    QSignalSpy countSpy(&viewModel, &panta::bridge::ShellViewModel::countChanged);
    QSignalSpy captionSpy(&viewModel, &panta::bridge::ShellViewModel::captionChanged);

    viewModel.tick();

    EXPECT_EQ(viewModel.count(), kInitialCount + 1);
    EXPECT_EQ(viewModel.caption().toStdString(), "修订 1");
    EXPECT_EQ(countSpy.count(), 1);
    EXPECT_EQ(captionSpy.count(), 1);
}

TEST(ShellViewModel, RepeatedCaptionWriteEmitsNoNotification) {
    panta::bridge::ShellViewModel viewModel;
    viewModel.setCaption(QStringLiteral("稳定值"));
    QSignalSpy captionSpy(&viewModel, &panta::bridge::ShellViewModel::captionChanged);

    viewModel.setCaption(QStringLiteral("稳定值"));

    EXPECT_EQ(captionSpy.count(), 0);
}

TEST(ShellViewModel, ChangedCaptionEmitsSingleNotification) {
    panta::bridge::ShellViewModel viewModel;
    QSignalSpy captionSpy(&viewModel, &panta::bridge::ShellViewModel::captionChanged);

    viewModel.setCaption(QStringLiteral("新值"));

    EXPECT_EQ(captionSpy.count(), 1);
}

TEST(ShellViewModel, ErrorPropertyDeduplicates) {
    panta::bridge::ShellViewModel viewModel;
    QSignalSpy errorSpy(&viewModel, &panta::bridge::ShellViewModel::errorChanged);

    viewModel.setError(QStringLiteral("示例错误"));
    viewModel.setError(QStringLiteral("示例错误"));

    EXPECT_EQ(errorSpy.count(), 1);
    EXPECT_EQ(viewModel.error().toStdString(), "示例错误");
}

int main(int argc, char** argv) {
    QCoreApplication app(argc, argv);
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
