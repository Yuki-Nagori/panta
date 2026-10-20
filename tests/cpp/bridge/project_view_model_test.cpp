/// ProjectViewModel 回归：工程命令、错误映射及任务 080 的视口文档生命周期。

#include "mesh_frame_tracker.hpp"
#include "panta/visualization/mesh_source.hpp"
#include "panta/visualization/render_scene.hpp"
#include "project_operation_helpers.hpp"
#include "project_view_model.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QIODevice>
#include <QPair>
#include <QSignalSpy>
#include <QString>
#include <QTemporaryDir>
#include <QTimer>
#include <QUrl>
#include <QVariantMap>
#include <QtCore/qcontainerfwd.h>
#include <QtCore/qtestsupport_core.h>
#include <QtTest/qtest.h>
#include <array>
#include <functional>
#include <gtest/gtest.h>
#include <memory>
#include <system_error>
#include <thread>
#include <utility>
#include <vector>
#ifdef Q_OS_UNIX
#include <sys/stat.h>
#endif
#include <qtestcase.h>

using panta::bridge::ProjectViewModel;
using panta::test::finish_project_operation;
using panta::test::wait_for_active_mesh;

namespace {

class VariableWaiterFailureViewModel : public ProjectViewModel {
  public:
    bool failNextStart = false;

  protected:
    void startBackgroundWaiter(std::function<void()> work) override {
        if (std::exchange(failNextStart, false)) {
            throw std::system_error(std::make_error_code(std::errc::resource_unavailable_try_again),
                                    "injected variable waiter startup failure");
        }
        ProjectViewModel::startBackgroundWaiter(std::move(work));
    }
};

class HeldBackgroundViewModel : public ProjectViewModel {
  public:
    bool holdWork = false;
    std::vector<std::function<void()>> held;

    void releaseFirst() {
        auto work = std::move(held.front());
        held.erase(held.begin());
        std::thread(std::move(work)).join();
    }

  protected:
    void startBackgroundWaiter(std::function<void()> work) override {
        if (holdWork) {
            held.push_back(std::move(work));
        } else {
            ProjectViewModel::startBackgroundWaiter(std::move(work));
        }
    }
};

void expect_same_plan_settings(QVariantMap actual, QVariantMap expected) {
    // 创建路径与 QUrl 重开路径在 Windows 可使用不同分隔符；比较同一文件身份。
    const auto canonical = [](const QVariantMap& settings) {
        return QFileInfo(settings.value(QStringLiteral("projectPath")).toString())
            .canonicalFilePath();
    };
    const auto actualPath = canonical(actual);
    const auto expectedPath = canonical(expected);
    ASSERT_FALSE(actualPath.isEmpty());
    ASSERT_FALSE(expectedPath.isEmpty());
    EXPECT_EQ(actualPath, expectedPath);
    actual.remove(QStringLiteral("projectPath"));
    expected.remove(QStringLiteral("projectPath"));
    EXPECT_EQ(actual, expected);
}

} // namespace

TEST(ProjectViewModelTest, CreatesOpensRenamesAndSavesThroughRustService) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    ProjectViewModel view_model;
    QSignalSpy created(&view_model, &ProjectViewModel::projectCreated);
    const QString location = QDir(fixture.path()).absolutePath();
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), location)));
    ASSERT_EQ(created.count(), 1);
    EXPECT_EQ(view_model.currentName(), QStringLiteral("Demo"));
    EXPECT_FALSE(view_model.dirty());

    const QString project_path = view_model.currentPath();
    EXPECT_TRUE(
        QDir::fromNativeSeparators(project_path).endsWith(QStringLiteral("/Demo/Demo.panta")));
    EXPECT_TRUE(QFile::exists(project_path));
    EXPECT_TRUE(view_model.renameProject(QStringLiteral("Renamed")));
    EXPECT_TRUE(view_model.dirty());
    EXPECT_TRUE(finish_project_operation(view_model, view_model.saveProject()));
    EXPECT_FALSE(view_model.dirty());

    ProjectViewModel reopened;
    QSignalSpy opened(&reopened, &ProjectViewModel::projectOpened);
    ASSERT_TRUE(finish_project_operation(
        reopened, reopened.openProjectUrl(QUrl::fromLocalFile(project_path))));
    ASSERT_EQ(opened.count(), 1);
    EXPECT_EQ(reopened.currentName(), QStringLiteral("Renamed"));
    EXPECT_FALSE(reopened.dirty());
}

TEST(ProjectViewModelTest, ProjectCommandsReturnAdmissionAndPublishWithoutBlockingGuiEvents) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    HeldBackgroundViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("First"), fixture.path())));
    ASSERT_TRUE(model.renameProject(QStringLiteral("Renamed")));
    model.holdWork = true;
    QSignalSpy finished(&model, &ProjectViewModel::projectOperationFinished);
    ASSERT_TRUE(model.saveProject());
    EXPECT_TRUE(model.projectOperationPending());
    EXPECT_TRUE(model.dirty());
    EXPECT_TRUE(finished.isEmpty());
    ASSERT_EQ(model.held.size(), 1U);
    EXPECT_FALSE(model.createProject(QStringLiteral("Conflicting"), fixture.path()));
    EXPECT_FALSE(model.renameProject(QStringLiteral("Conflicting")));
    EXPECT_FALSE(QDir(fixture.filePath(QStringLiteral("Conflicting"))).exists());
    bool guiEventDelivered = false;
    QTimer::singleShot(0, &model, [&guiEventDelivered] { guiEventDelivered = true; });
    QTRY_VERIFY(guiEventDelivered);
    EXPECT_TRUE(model.projectOperationPending());
    model.releaseFirst();
    QTRY_COMPARE(finished.count(), 1);
    EXPECT_EQ(finished.at(0).at(0).toString(), QStringLiteral("save"));
    EXPECT_TRUE(finished.at(0).at(1).toBool());
    EXPECT_FALSE(model.projectOperationPending());
    EXPECT_FALSE(model.dirty());
    EXPECT_TRUE(model.errorCode().isEmpty());
}

TEST(ProjectViewModelTest, ProjectWaiterStartupFailureNeverAdmitsFilesystemWork) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    VariableWaiterFailureViewModel model;
    model.failNextStart = true;
    EXPECT_FALSE(model.createProject(QStringLiteral("NotCreated"), fixture.path()));
    EXPECT_FALSE(model.projectOperationPending());
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.io"));
    EXPECT_FALSE(QDir(fixture.filePath(QStringLiteral("NotCreated"))).exists());
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Existing"), fixture.path())));
    QFile manifest(model.currentPath());
    ASSERT_TRUE(manifest.open(QIODevice::ReadOnly));
    const auto original = manifest.readAll();
    manifest.close();
    ASSERT_TRUE(model.renameProject(QStringLiteral("Dirty")));
    model.failNextStart = true;
    EXPECT_FALSE(model.saveProject());
    EXPECT_FALSE(model.projectOperationPending());
    EXPECT_TRUE(model.dirty());
    ASSERT_TRUE(manifest.open(QIODevice::ReadOnly));
    EXPECT_EQ(manifest.readAll(), original);
    manifest.close();
    model.failNextStart = true;
    EXPECT_FALSE(model.openProject(model.currentPath()));
    EXPECT_EQ(model.currentName(), QStringLiteral("Dirty"));
    ASSERT_TRUE(finish_project_operation(model, model.saveProject()));
}

TEST(ProjectViewModelTest, CompletionDoesNotEraseAnErrorFromTheNextReentrantCommand) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    VariableWaiterFailureViewModel model;
    QSignalSpy finished(&model, &ProjectViewModel::projectOperationFinished);
    QSignalSpy created(&model, &ProjectViewModel::projectCreated);
    bool attemptedSave = false;
    bool acceptedSave = true;
    QObject::connect(&model, &ProjectViewModel::projectOperationPendingChanged, &model, [&] {
        if (model.projectOperationPending() || attemptedSave) {
            return;
        }
        attemptedSave = true;
        model.failNextStart = true;
        acceptedSave = model.saveProject();
    });
    ASSERT_TRUE(model.createProject(QStringLiteral("Completed"), fixture.path()));
    EXPECT_TRUE(model.projectOperationPending());
    QTRY_COMPARE(finished.count(), 1);
    EXPECT_EQ(finished.at(0).at(0).toString(), QStringLiteral("create"));
    EXPECT_TRUE(finished.at(0).at(1).toBool());
    EXPECT_TRUE(attemptedSave);
    EXPECT_FALSE(acceptedSave);
    EXPECT_FALSE(model.projectOperationPending());
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.io"));
    EXPECT_TRUE(
        model.errorDetail().contains(QStringLiteral("injected variable waiter startup failure")));
    EXPECT_EQ(created.count(), 1);
    EXPECT_EQ(model.currentName(), QStringLiteral("Completed"));
    EXPECT_TRUE(QFile::exists(fixture.filePath(QStringLiteral("Completed/Completed.panta"))));
}

TEST(ProjectViewModelTest, DestroyingProjectRequestOwnerDoesNotAccessDeletedQtObjects) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    auto model = std::make_unique<HeldBackgroundViewModel>();
    model->holdWork = true;
    ASSERT_TRUE(model->createProject(QStringLiteral("Accepted"), fixture.path()));
    ASSERT_EQ(model->held.size(), 1U);
    auto work = std::move(model->held.front());
    model.reset();
    std::thread(std::move(work)).join();
    QCoreApplication::processEvents();
    EXPECT_TRUE(QFile::exists(fixture.filePath(QStringLiteral("Accepted/Accepted.panta"))));
}

TEST(ProjectViewModelTest, DisplayPreparationCoalescesSelectionAndRejectsTheOldDisplayResult) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const auto sourcePath = fixture.filePath(QStringLiteral("part.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    source.close();
    HeldBackgroundViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Display"), fixture.path())));
    model.holdWork = true;
    ASSERT_TRUE(model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                QStringLiteral("millimeters"), false));
    ASSERT_EQ(model.held.size(), 1U);
    EXPECT_EQ(model.mesh_snapshot(), nullptr);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Truncate));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 2 0 0\nvertex 0 1 0\n"), 0);
    source.close();
    ASSERT_TRUE(model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                QStringLiteral("millimeters"), false));
    EXPECT_EQ(model.activeDocumentId(), QStringLiteral("import-2"));
    ASSERT_EQ(model.held.size(), 1U);
    model.releaseFirst();
    QTRY_COMPARE(model.held.size(), 1U);
    EXPECT_EQ(model.mesh_snapshot(), nullptr);
    model.releaseFirst();
    ASSERT_TRUE(wait_for_active_mesh(model, QStringLiteral("import-2")));
    EXPECT_EQ(model.mesh_snapshot()->vertices[1], (std::array<double, 3>{2.0, 0.0, 0.0}));
}

TEST(ProjectViewModelTest, GpuSamplingRejectsEmptyFramesAndWaitsForTheTargetScene) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const auto path = fixture.filePath(QStringLiteral("part.stl"));
    QFile source(path);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    source.close();
    HeldBackgroundViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("GpuSampling"), fixture.path())));
    for (int index = 1; index <= 2; ++index) {
        ASSERT_TRUE(model.importStl(path, QStringLiteral("solid-3d"), QStringLiteral("millimeters"),
                                    false));
        ASSERT_TRUE(wait_for_active_mesh(model, QStringLiteral("import-%1").arg(index)));
    }

    panta::test::MeshFrameTracker frames;
    panta::visualization::SceneRevision revision = 0;
    QObject::connect(&model, &ProjectViewModel::meshChanged, &model, [&] {
        const auto mesh = model.mesh_snapshot();
        frames.record_scene(++revision, mesh);
        // 模拟离屏渲染：清空场景立即完成，目标模型帧由下方显式交付。
        if (!mesh)
            frames.record_frame(revision);
    });
    model.holdWork = true;
    for (const auto& id : {QStringLiteral("import-1"), QStringLiteral("import-2")}) {
        const auto before = frames.frame_count();
        model.activateDocument(id);
        ASSERT_EQ(model.held.size(), 1U);
        ASSERT_GT(frames.frame_count(), before); // 旧的“帧数增长”判定已被空帧满足。
        const auto emptyRevision = revision;
        EXPECT_TRUE(frames.scene_complete());
        EXPECT_FALSE(frames.ready(model.mesh_snapshot()));

        model.releaseFirst();
        ASSERT_TRUE(wait_for_active_mesh(model, id));
        EXPECT_GT(revision, emptyRevision);
        EXPECT_FALSE(frames.scene_complete());
        EXPECT_FALSE(frames.ready(model.mesh_snapshot()));
        // DTO 已就绪仍不够；先收到上一场景帧也不能结束目标等待。
        frames.record_frame(emptyRevision);
        EXPECT_FALSE(frames.ready(model.mesh_snapshot()));
        frames.record_frame(revision);
        EXPECT_TRUE(frames.ready(model.mesh_snapshot()));
        EXPECT_FALSE(frames.ready(std::make_shared<panta::visualization::SurfaceMeshSnapshot>()));
    }
    std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot> prior = model.mesh_snapshot();
    model.activateDocument(QStringLiteral("welcome"));
    EXPECT_TRUE(prior.expired());
    EXPECT_FALSE(frames.ready(nullptr));
    EXPECT_TRUE(frames.scene_complete());
}

TEST(ProjectViewModelTest, DestroyingDisplayPreparationOwnerDoesNotAccessDeletedQtObjects) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const auto sourcePath = fixture.filePath(QStringLiteral("part.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    source.close();
    auto model = std::make_unique<HeldBackgroundViewModel>();
    ASSERT_TRUE(finish_project_operation(
        *model, model->createProject(QStringLiteral("Display"), fixture.path())));
    model->holdWork = true;
    ASSERT_TRUE(model->importStl(sourcePath, QStringLiteral("solid-3d"),
                                 QStringLiteral("millimeters"), false));
    ASSERT_EQ(model->held.size(), 1U);
    auto work = std::move(model->held.front());
    model.reset();
    std::thread(std::move(work)).join();
    QCoreApplication::processEvents();
}

TEST(ProjectViewModelTest, CreatesAndOpensProjectThroughUnicodeFilesystemPaths) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    const QString location = QDir(fixture.path()).filePath(QStringLiteral("项目资料"));
    ProjectViewModel created;
    ASSERT_TRUE(finish_project_operation(
        created, created.createProject(QStringLiteral("工程-测试"), location)))
        << created.errorCode().toStdString() << ": " << created.errorDetail().toStdString();

    const QString projectPath = created.currentPath();
    ASSERT_TRUE(QFile::exists(projectPath));
    EXPECT_EQ(QDir::fromNativeSeparators(projectPath),
              QDir::fromNativeSeparators(
                  QDir(location).filePath(QStringLiteral("工程-测试/工程-测试.panta"))));

    ProjectViewModel reopened;
    const QUrl projectUrl = QUrl::fromLocalFile(projectPath);
    ASSERT_TRUE(finish_project_operation(reopened, reopened.openProjectUrl(projectUrl)))
        << reopened.errorCode().toStdString() << ": " << reopened.errorDetail().toStdString();
    EXPECT_EQ(reopened.currentName(), QStringLiteral("工程-测试"));
    EXPECT_EQ(QFileInfo(reopened.currentPath()).canonicalFilePath(),
              QFileInfo(projectPath).canonicalFilePath());
}

TEST(ProjectViewModelTest, VariableDocumentsPreserveDiagnosticsAndRejectStaleUpdates) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Variables"), fixture.path())));
    QSignalSpy finished(&model, &ProjectViewModel::variableDocumentFinished);
    const QString source = QStringLiteral(
        "version: 1\nkind: variables\n\nvalues:\n  count: int = 12\n  doubled: int = count * 2\n");
    ASSERT_TRUE(model.beginSaveVariableDocument(source, model.variableGeneration(), 0))
        << model.errorCode().toStdString() << ": " << model.errorDetail().toStdString();
    EXPECT_EQ(model.variableRevision(), 0U);
    QTRY_VERIFY(!finished.isEmpty());
    ASSERT_TRUE(finished.takeFirst().at(1).toBool());
    EXPECT_EQ(model.variableRevision(), 1U);
    EXPECT_EQ(model.variableSource(), source);
    ASSERT_EQ(model.variableValues().size(), 2);
    const auto doubled = model.variableValues().at(1).toMap();
    EXPECT_EQ(doubled.value(QStringLiteral("name")).toString(), QStringLiteral("doubled"));
    EXPECT_EQ(doubled.value(QStringLiteral("type")).toString(), QStringLiteral("int"));
    EXPECT_EQ(doubled.value(QStringLiteral("value")).toString(), QStringLiteral("24"));

    QFile variableFile(QDir(QFileInfo(model.currentPath()).absolutePath())
                           .filePath(QStringLiteral("variables.pa")));
    ASSERT_TRUE(variableFile.open(QIODevice::ReadOnly));
    const QByteArray previousFile = variableFile.readAll();
    variableFile.close();

    const QString staleSource =
        QStringLiteral("version: 1\nkind: variables\n\nvalues:\n  count: int = 99\n");
    EXPECT_FALSE(model.beginSaveVariableDocument(staleSource, model.variableGeneration(), 0));
    EXPECT_EQ(model.errorCode(), QStringLiteral("variables.stale_revision"));
    EXPECT_EQ(model.variableRevision(), 1U);
    EXPECT_EQ(model.variableValues().at(0).toMap().value(QStringLiteral("value")).toString(),
              QStringLiteral("12"));
    ASSERT_TRUE(variableFile.open(QIODevice::ReadOnly));
    EXPECT_EQ(variableFile.readAll(), previousFile);
    variableFile.close();

    const QString invalidSource =
        QStringLiteral("version: 1\nkind: variables\n\nvalues:\n  count: int = unknown-name\n");
    ASSERT_TRUE(model.beginSaveVariableDocument(invalidSource, model.variableGeneration(), 1));
    QTRY_VERIFY(!finished.isEmpty());
    EXPECT_FALSE(finished.takeFirst().at(1).toBool());
    ASSERT_EQ(model.variableDiagnostics().size(), 1);
    const auto diagnostic = model.variableDiagnostics().constFirst().toMap();
    EXPECT_EQ(diagnostic.value(QStringLiteral("code")).toString(),
              QStringLiteral("variables.unknown_reference"));
    EXPECT_EQ(diagnostic.value(QStringLiteral("line")).toULongLong(), 5U);
    EXPECT_EQ(model.variableRevision(), 1U);
    EXPECT_EQ(model.variableValues().at(0).toMap().value(QStringLiteral("value")).toString(),
              QStringLiteral("12"));

    const QString invalidResource = QStringLiteral(
        "version: 1\nkind: variables\n\nvalues:\n  mesh: resource = project:/../outside.stl\n");
    ASSERT_TRUE(model.beginSaveVariableDocument(invalidResource, model.variableGeneration(), 1));
    QTRY_VERIFY(!finished.isEmpty());
    EXPECT_FALSE(finished.takeFirst().at(1).toBool());
    ASSERT_EQ(model.variableDiagnostics().size(), 1);
    const auto resourceDiagnostic = model.variableDiagnostics().constFirst().toMap();
    EXPECT_EQ(resourceDiagnostic.value(QStringLiteral("code")).toString(),
              QStringLiteral("variables.invalid_resource"));
    EXPECT_EQ(resourceDiagnostic.value(QStringLiteral("line")).toULongLong(), 5U);
    EXPECT_EQ(model.variableRevision(), 1U);
    ASSERT_TRUE(variableFile.open(QIODevice::ReadOnly));
    EXPECT_EQ(variableFile.readAll(), previousFile);
    variableFile.close();

    ProjectViewModel reopened;
    ASSERT_TRUE(finish_project_operation(reopened, reopened.openProject(model.currentPath())));
    QSignalSpy loaded(&reopened, &ProjectViewModel::variableDocumentFinished);
    ASSERT_TRUE(reopened.beginLoadVariableDocument(reopened.variableGeneration(), 0))
        << reopened.errorCode().toStdString() << ": " << reopened.errorDetail().toStdString();
    ASSERT_TRUE(loaded.wait(5000));
    EXPECT_TRUE(loaded.takeFirst().at(1).toBool());
    EXPECT_EQ(reopened.variableRevision(), 1U);
    EXPECT_EQ(reopened.variableValues().size(), 2);
    EXPECT_EQ(reopened.variableSource(), source);
}

TEST(ProjectViewModelTest, OldVariableRequestCannotWriteIntoAnotherProject) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("First"), fixture.path())));
    const auto oldGeneration = model.variableGeneration();
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Second"), fixture.path())));
    EXPECT_NE(model.variableGeneration(), oldGeneration);

    const QString source =
        QStringLiteral("version: 1\nkind: variables\n\nvalues:\n  count: int = 7\n");
    EXPECT_FALSE(model.beginSaveVariableDocument(source, oldGeneration, 0));
    EXPECT_EQ(model.errorCode(), QStringLiteral("variables.stale_generation"));
    EXPECT_EQ(model.variableRevision(), 0U);
    EXPECT_TRUE(model.variableValues().isEmpty());
    EXPECT_FALSE(QFileInfo(QDir(QFileInfo(model.currentPath()).absolutePath())
                               .filePath(QStringLiteral("variables.pa")))
                     .exists());
}

TEST(ProjectViewModelTest, VariableWaiterStartupFailureDoesNotSubmitOrFreezeProject) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    VariableWaiterFailureViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("First"), fixture.path())));
    QSignalSpy finished(&model, &ProjectViewModel::variableDocumentFinished);
    const QString source =
        QStringLiteral("version: 1\nkind: variables\n\nvalues:\n  count: int = 7\n");
    ASSERT_TRUE(model.beginSaveVariableDocument(source, model.variableGeneration(), 0));
    QTRY_VERIFY(!finished.isEmpty());
    ASSERT_TRUE(finished.takeFirst().at(1).toBool());
    QFile variableFile(QDir(QFileInfo(model.currentPath()).absolutePath())
                           .filePath(QStringLiteral("variables.pa")));
    ASSERT_TRUE(variableFile.open(QIODevice::ReadOnly));
    const auto original = variableFile.readAll();
    variableFile.close();

    model.failNextStart = true;
    EXPECT_FALSE(model.beginSaveVariableDocument(
        QStringLiteral("version: 1\nkind: variables\nvalues:\n  count: int = 99\n"),
        model.variableGeneration(), 1));
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.io"));
    EXPECT_EQ(model.variableRevision(), 1U);
    EXPECT_EQ(model.variableSource(), source);
    ASSERT_TRUE(variableFile.open(QIODevice::ReadOnly));
    EXPECT_EQ(variableFile.readAll(), original);
    variableFile.close();
    model.failNextStart = true;
    EXPECT_FALSE(model.beginLoadVariableDocument(model.variableGeneration(), 1));
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.io"));
    QCoreApplication::processEvents();
    EXPECT_TRUE(finished.isEmpty());

    // 两类启动失败均未准入 Rust 任务；工程可切换，后续请求仍能正常完成。
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Second"), fixture.path())));
    ASSERT_TRUE(model.beginSaveVariableDocument(source, model.variableGeneration(), 0));
    QTRY_VERIFY(!finished.isEmpty());
    EXPECT_TRUE(finished.takeFirst().at(1).toBool());
    EXPECT_EQ(model.variableRevision(), 1U);
    ASSERT_TRUE(variableFile.open(QIODevice::ReadOnly));
    EXPECT_EQ(variableFile.readAll(), original);
}

TEST(ProjectViewModelTest, VariableSaveRejectsConflictingSwitchUntilQueuedCompletion) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("First"), fixture.path())));
    QSignalSpy finished(&model, &ProjectViewModel::variableDocumentFinished);
    const QString source =
        QStringLiteral("version: 1\nkind: variables\n\nvalues:\n  count: int = 7\n");
    ASSERT_TRUE(model.beginSaveVariableDocument(source, model.variableGeneration(), 0));
    // 即使 worker 已完成，GUI 尚未消费回执，冻结的候选仍不能被工程切换或另一次保存覆盖。
    EXPECT_EQ(model.variableRevision(), 0U);
    EXPECT_FALSE(finish_project_operation(
        model, model.createProject(QStringLiteral("Second"), fixture.path())));
    EXPECT_EQ(model.currentName(), QStringLiteral("First"));
    EXPECT_FALSE(model.beginSaveVariableDocument(source, model.variableGeneration(), 0));
    QTRY_VERIFY(!finished.isEmpty());
    EXPECT_TRUE(finished.takeFirst().at(1).toBool());
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Second"), fixture.path())));
}

TEST(ProjectViewModelTest, VariableReadCompletionFromOldProjectDoesNotReplaceNewProjection) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("First"), fixture.path())));
    QSignalSpy finished(&model, &ProjectViewModel::variableDocumentFinished);
    const QString source =
        QStringLiteral("version: 1\nkind: variables\n\nvalues:\n  count: int = 7\n");
    ASSERT_TRUE(model.beginSaveVariableDocument(source, model.variableGeneration(), 0));
    QTRY_VERIFY(!finished.isEmpty());
    finished.clear();
    const auto oldGeneration = model.variableGeneration();
    ASSERT_TRUE(model.beginLoadVariableDocument(oldGeneration, 1));
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Second"), fixture.path())));
    const auto nextGeneration = model.variableGeneration();
    QTRY_VERIFY(!finished.isEmpty());
    const auto completion = finished.takeFirst();
    EXPECT_EQ(completion.at(0).toULongLong(), oldGeneration);
    EXPECT_FALSE(completion.at(1).toBool());
    EXPECT_EQ(model.variableGeneration(), nextGeneration);
    EXPECT_EQ(model.variableRevision(), 0U);
    EXPECT_TRUE(model.variableSource().isEmpty());
    EXPECT_TRUE(model.variableValues().isEmpty());
    EXPECT_TRUE(model.errorCode().isEmpty());
}

#ifdef Q_OS_UNIX
TEST(ProjectViewModelTest, VariableReadKeepsGuiResponsiveWhileFilesystemIsBlocked) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("BlockedRead"), fixture.path())));
    const auto variablePath = QDir(QFileInfo(model.currentPath()).absolutePath())
                                  .filePath(QStringLiteral("variables.pa"));
    const auto nativePath = QFile::encodeName(variablePath);
    ASSERT_EQ(::mkfifo(nativePath.constData(), 0600), 0);
    QSignalSpy finished(&model, &ProjectViewModel::variableDocumentFinished);
    // FIFO 的 open/read 直到 GUI 发出下一事件才获准完成，捕捉把 I/O 移回 GUI 的回归。
    ASSERT_TRUE(model.beginLoadVariableDocument(model.variableGeneration(), 0));
    EXPECT_EQ(model.variableRevision(), 0U);
    bool guiEventDelivered = false;
    std::jthread writer;
    QTimer::singleShot(0, &model, [&] {
        guiEventDelivered = true;
        writer = std::jthread([variablePath] {
            QFile stream(variablePath);
            if (stream.open(QIODevice::WriteOnly)) {
                stream.write("version: 1\nkind: variables\nvalues:\n  count: int = 7\n");
            }
        });
    });
    QTRY_VERIFY(!finished.isEmpty());
    EXPECT_TRUE(guiEventDelivered);
    EXPECT_TRUE(finished.takeFirst().at(1).toBool());
    EXPECT_EQ(model.variableRevision(), 1U);
}
#endif

TEST(ProjectViewModelTest, DestroyingVariableRequestOwnerDoesNotWaitForQtCompletion) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    auto model = std::make_unique<ProjectViewModel>();
    ASSERT_TRUE(finish_project_operation(
        *model, model->createProject(QStringLiteral("Dropped"), fixture.path())));
    ASSERT_TRUE(model->beginLoadVariableDocument(model->variableGeneration(), 0));
    model.reset();
    // worker 不拥有 Qt 对象；销毁后排队的完成通知由连接的 context 自动撤销。
    QTest::qWait(20);
    QCoreApplication::processEvents();
}

TEST(ProjectViewModelTest, MapsRustErrorsWithoutCreatingInvalidTargets) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    ProjectViewModel view_model;
    const QString location = QDir(fixture.path()).absolutePath();

    EXPECT_FALSE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("../escape"), location)));
    EXPECT_EQ(view_model.errorCode(), QStringLiteral("project.invalid_name"));
    EXPECT_EQ(view_model.errorCategory(), QStringLiteral("validation"));
    EXPECT_EQ(view_model.errorDetail(), QStringLiteral("../escape"));
    EXPECT_FALSE(view_model.error().isEmpty());
    EXPECT_FALSE(QDir(fixture.path()).exists(QStringLiteral("escape")));

    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), location)));
    EXPECT_TRUE(view_model.errorCategory().isEmpty());
    EXPECT_TRUE(view_model.errorDetail().isEmpty());
    EXPECT_FALSE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), location)));
    EXPECT_EQ(view_model.errorCode(), QStringLiteral("project.already_exists"));
    EXPECT_EQ(view_model.errorCategory(), QStringLiteral("conflict"));
    EXPECT_FALSE(view_model.errorDetail().isEmpty());

    ProjectViewModel empty;
    EXPECT_FALSE(finish_project_operation(empty, empty.saveProject()));
    EXPECT_EQ(empty.errorCode(), QStringLiteral("project.no_project"));
    EXPECT_EQ(empty.errorCategory(), QStringLiteral("state"));
    EXPECT_TRUE(empty.errorDetail().isEmpty());
}

TEST(ProjectViewModelTest, ExposesRustProcessConstraintsAndPerFieldDraftErrors) {
    ProjectViewModel view_model;
    const auto plan = view_model.planSettings();
    const auto constraints = plan.value(QStringLiteral("processConstraints")).toMap();
    const auto temperature = constraints.value(QStringLiteral("temperature")).toMap();
    const auto flow = constraints.value(QStringLiteral("flowRate")).toMap();
    const auto pressure = constraints.value(QStringLiteral("holdingPressure")).toMap();
    const auto gate_count = constraints.value(QStringLiteral("numberOfGates")).toMap();
    EXPECT_DOUBLE_EQ(temperature.value(QStringLiteral("minimum")).toDouble(), -273.15);
    EXPECT_FALSE(flow.value(QStringLiteral("minimumInclusive")).toBool());
    EXPECT_DOUBLE_EQ(pressure.value(QStringLiteral("maximum")).toDouble(), 200.0);
    EXPECT_TRUE(gate_count.value(QStringLiteral("integer")).toBool());

    auto fill = plan.value(QStringLiteral("fillSettings")).toMap();
    fill.insert(QStringLiteral("flowRate"), 0.0);
    const auto fill_result = view_model.validateFillSettingsDraft(fill);
    EXPECT_FALSE(fill_result.value(QStringLiteral("valid")).toBool());
    EXPECT_TRUE(fill_result.value(QStringLiteral("errors"))
                    .toStringList()
                    .contains(QStringLiteral("flowRate")));

    auto gate = plan.value(QStringLiteral("gateLocationSettings")).toMap();
    gate.insert(QStringLiteral("numberOfGates"), 1.5);
    const auto gate_result = view_model.validateGateLocationSettingsDraft(gate);
    EXPECT_FALSE(gate_result.value(QStringLiteral("valid")).toBool());
    EXPECT_TRUE(gate_result.value(QStringLiteral("errors"))
                    .toStringList()
                    .contains(QStringLiteral("numberOfGates")));

    const auto profile_result = view_model.validateHoldingProfileDraft(
        {QVariantMap{{QStringLiteral("duration"), 0.0}, {QStringLiteral("pressure"), 201.0}}});
    EXPECT_FALSE(profile_result.value(QStringLiteral("valid")).toBool());
    EXPECT_TRUE(profile_result.value(QStringLiteral("errors"))
                    .toStringList()
                    .contains(QStringLiteral("holdingProfile.0.pressure")));
}

TEST(ProjectViewModelTest, StructuredImportFailuresPreserveTheCommittedProjectAndViewport) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const QString sourcePath = fixture.filePath(QStringLiteral("零件 sample.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    source.close();

    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(model, model.activeDocumentId()));
    const auto mesh = model.mesh_snapshot();
    ASSERT_NE(mesh, nullptr);
    const auto settings = model.planSettings();
    const auto documents = model.openDocuments();
    const auto parts = model.importedPartNames();
    const auto active = model.activeDocumentId();
    QFile manifest(model.currentPath());
    ASSERT_TRUE(manifest.open(QIODevice::ReadOnly));
    const auto before = manifest.readAll();
    manifest.close();
    QSignalSpy imported(&model, &ProjectViewModel::projectImported);

    // 参数本身包含冒号及中文，不能通过拼接文案还原诊断字段。
    EXPECT_FALSE(model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                 QStringLiteral("unknown:单位"), false));
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.import_unsupported_units"));
    EXPECT_EQ(model.errorCategory(), QStringLiteral("format"));
    EXPECT_EQ(model.errorDetail(), QStringLiteral("unknown:单位"));
    EXPECT_EQ(model.error(), QStringLiteral("Choose valid STL import options."));
    EXPECT_EQ(imported.count(), 0);
    EXPECT_EQ(model.mesh_snapshot(), mesh);
    EXPECT_EQ(model.planSettings(), settings);
    EXPECT_EQ(model.openDocuments(), documents);
    EXPECT_EQ(model.importedPartNames(), parts);
    EXPECT_EQ(model.activeDocumentId(), active);
    ASSERT_TRUE(manifest.open(QIODevice::ReadOnly));
    EXPECT_EQ(manifest.readAll(), before);
    manifest.close();

    const QString missing = fixture.filePath(QStringLiteral("不存在 sample.stl"));
    ASSERT_TRUE(model.inspectStl(missing));
    QTRY_VERIFY(!model.importPreviewPending());
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.import_file_missing"));
    EXPECT_EQ(model.errorCategory(), QStringLiteral("missing"));
    EXPECT_EQ(model.errorDetail(), missing);
    EXPECT_FALSE(model.importPreviewReady());
    EXPECT_EQ(model.mesh_snapshot(), mesh);
    EXPECT_EQ(model.planSettings(), settings);

    const auto revision = settings.value(QStringLiteral("revision")).toULongLong();
    const auto importId = settings.value(QStringLiteral("importId")).toString();
    QSignalSpy materialFinished(&model, &ProjectViewModel::materialConfirmationFinished);
    QSignalSpy fillFinished(&model, &ProjectViewModel::fillSettingsConfirmationFinished);
    QSignalSpy gateFinished(&model, &ProjectViewModel::gateLocationSettingsConfirmationFinished);
    const auto expectStaleRequest = [&model](bool accepted) {
        EXPECT_FALSE(accepted);
        EXPECT_EQ(model.errorCode(), QStringLiteral("project.command_invalid"));
        EXPECT_EQ(model.errorCategory(), QStringLiteral("validation"));
        EXPECT_EQ(model.errorDetail(), QStringLiteral("plan settings changed"));
    };
    expectStaleRequest(finish_project_operation(
        model, model.setAnalysisSequence(model.currentPath(), revision + 1, importId,
                                         QStringLiteral("fill"))));
    expectStaleRequest(
        model.setMaterial(model.currentPath(), revision + 1, importId,
                          model.defaultMaterial().value(QStringLiteral("id")).toString()));
    expectStaleRequest(
        model.setFillSettings(model.currentPath(), revision + 1, importId,
                              settings.value(QStringLiteral("fillSettings")).toMap()));
    expectStaleRequest(model.setGateLocationSettings(
        model.currentPath(), revision + 1, importId,
        settings.value(QStringLiteral("gateLocationSettings")).toMap()));
    EXPECT_FALSE(model.materialConfirmationPending());
    EXPECT_FALSE(model.fillSettingsConfirmationPending());
    EXPECT_FALSE(model.gateLocationSettingsConfirmationPending());
    EXPECT_EQ(materialFinished.count(), 0);
    EXPECT_EQ(fillFinished.count(), 0);
    EXPECT_EQ(gateFinished.count(), 0);
    EXPECT_EQ(model.planSettings(), settings);
    EXPECT_EQ(model.mesh_snapshot(), mesh);

    EXPECT_FALSE(finish_project_operation(
        model, model.openProject(fixture.filePath(QStringLiteral("missing.panta")))));
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.file_missing"));
    EXPECT_EQ(model.errorCategory(), QStringLiteral("missing"));
    EXPECT_EQ(model.planSettings(), settings);
    EXPECT_EQ(model.mesh_snapshot(), mesh);
    EXPECT_EQ(model.openDocuments(), documents);

    // 临时把资产目录替换为普通文件，稳定触发 sidecar 目录创建失败。
    const QString projectRoot = QFileInfo(model.currentPath()).absolutePath();
    const QString assetsDirectory = projectRoot + QStringLiteral("/assets");
    const QString parkedAssetsDirectory = projectRoot + QStringLiteral("/assets-parked");
    ASSERT_TRUE(QDir().rename(assetsDirectory, parkedAssetsDirectory));
    QFile assetsBlocker(assetsDirectory);
    ASSERT_TRUE(assetsBlocker.open(QIODevice::WriteOnly));
    assetsBlocker.write("not a directory");
    assetsBlocker.close();
    EXPECT_FALSE(model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                 QStringLiteral("millimeters"), false));
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.import_asset_write_failed"));
    EXPECT_EQ(model.errorCategory(), QStringLiteral("io"));
    EXPECT_EQ(imported.count(), 0);
    EXPECT_EQ(model.planSettings(), settings);
    EXPECT_EQ(model.mesh_snapshot(), mesh);
    EXPECT_EQ(model.importedPartNames(), parts);
    ASSERT_TRUE(manifest.open(QIODevice::ReadOnly));
    EXPECT_EQ(manifest.readAll(), before);
    manifest.close();
    ASSERT_TRUE(QFile::remove(assetsDirectory));
    ASSERT_TRUE(QDir().rename(parkedAssetsDirectory, assetsDirectory));

    ASSERT_TRUE(model.inspectStl(sourcePath));
    EXPECT_TRUE(model.importPreviewPending());
    QTRY_VERIFY(!model.importPreviewPending());
    EXPECT_TRUE(model.error().isEmpty());
    EXPECT_TRUE(model.errorCode().isEmpty());
    EXPECT_TRUE(model.errorCategory().isEmpty());
    EXPECT_TRUE(model.errorDetail().isEmpty());
    EXPECT_TRUE(model.importPreviewReady());
}

TEST(ProjectViewModelTest, PreviewsImportsAndPersistsLatestRecord) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("sample.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Text));
    ASSERT_GT(source.write("solid sample\n"
                           "facet normal 0 0 1\n"
                           " outer loop\n"
                           "  vertex 0 0 0\n"
                           "  vertex 1 0 0\n"
                           "  vertex 0 1 0\n"
                           " endloop\n"
                           "endfacet\n"
                           "endsolid sample\n"),
              0);
    source.close();

    ProjectViewModel view_model;
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), fixture.path())));
    EXPECT_TRUE(view_model.importedPartNames().isEmpty());
    ASSERT_TRUE(view_model.inspectStl(sourcePath));
    EXPECT_TRUE(view_model.importPreviewPending());
    QTRY_VERIFY(!view_model.importPreviewPending());
    EXPECT_TRUE(view_model.importPreviewReady());
    EXPECT_EQ(view_model.importPreviewName(), QStringLiteral("sample.stl"));
    EXPECT_EQ(view_model.importPreviewDimensions(), QStringLiteral("1.00 × 1.00 × 0.00"));
    EXPECT_EQ(view_model.importPreviewTriangleCount(), 1U);

    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("dual-domain"),
                                     QStringLiteral("millimeters"), true));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    EXPECT_EQ(view_model.importedPartNames(), QStringList{QStringLiteral("sample")});
    const auto mesh = view_model.mesh_snapshot();
    ASSERT_NE(mesh, nullptr);
    EXPECT_EQ(mesh->vertices.size(), 3U);
    EXPECT_EQ(mesh->vertices[1], (std::array<double, 3>{1.0, 0.0, 0.0}));

    const auto firstPlan = view_model.planSettings();
    QSignalSpy planChanged(&view_model, &ProjectViewModel::planSettingsChanged);
    EXPECT_EQ(firstPlan.value(QStringLiteral("meshType")).toString(),
              QStringLiteral("dual-domain"));
    ASSERT_TRUE(finish_project_operation(
        view_model,
        view_model.setAnalysisSequence(
            view_model.currentPath(), firstPlan.value(QStringLiteral("revision")).toULongLong(),
            firstPlan.value(QStringLiteral("importId")).toString(), QStringLiteral("fill-pack"))));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("sequenceId")).toString(),
              QStringLiteral("fill-pack"));
    EXPECT_EQ(planChanged.count(), 1);

    const QString secondSourcePath =
        QDir(fixture.path()).filePath(QStringLiteral("sample-second.stl"));
    ASSERT_TRUE(QFile::copy(sourcePath, secondSourcePath));
    ASSERT_TRUE(view_model.importStl(secondSourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    EXPECT_EQ(view_model.importedPartNames(),
              QStringList({QStringLiteral("sample"), QStringLiteral("sample-second")}));

    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("meshType")).toString(),
              QStringLiteral("solid-3d"));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("sequenceId")).toString(),
              QStringLiteral("fill"));
    view_model.activateDocument(firstPlan.value(QStringLiteral("importId")).toString());
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("meshType")).toString(),
              QStringLiteral("dual-domain"));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("sequenceId")).toString(),
              QStringLiteral("fill-pack"));
    view_model.activateDocument(QStringLiteral("welcome"));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("meshType")).toString(),
              QStringLiteral("solid-3d"));

    ProjectViewModel reopened;
    ASSERT_TRUE(finish_project_operation(reopened, reopened.openProject(view_model.currentPath())));
    EXPECT_EQ(reopened.importedPartNames(),
              QStringList({QStringLiteral("sample"), QStringLiteral("sample-second")}));
    // 打开工程只读清单（073/080）：已保存网格须经只读 FSM 激活异步恢复，
    // 重开后视口快照为空；记录投影与修订不变。
    const auto reopened_mesh = reopened.mesh_snapshot();
    EXPECT_EQ(reopened_mesh, nullptr);

    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Truncate | QIODevice::Text));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 2 0 0\nvertex 0 1 0\n"), 0);
    source.close();
    QSignalSpy mesh_changed(&view_model, &panta::visualization::MeshSource::meshChanged);
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("dual-domain"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    EXPECT_EQ(view_model.importedPartNames(),
              QStringList({QStringLiteral("sample"), QStringLiteral("sample-second"),
                           QStringLiteral("sample")}));
    EXPECT_EQ(mesh_changed.count(), 2);
    const auto replacement = view_model.mesh_snapshot();
    ASSERT_NE(replacement, nullptr);
    EXPECT_EQ(replacement->vertices[1], (std::array<double, 3>{2.0, 0.0, 0.0}));
    EXPECT_EQ(mesh->vertices[1], (std::array<double, 3>{1.0, 0.0, 0.0}));
    ASSERT_TRUE(view_model.inspectStl(sourcePath));
    EXPECT_TRUE(view_model.importPreviewPending());
    QTRY_VERIFY(!view_model.importPreviewPending());
    ASSERT_TRUE(view_model.inspectStl(sourcePath + QStringLiteral(".missing")));
    QTRY_VERIFY(!view_model.importPreviewPending());
    EXPECT_EQ(view_model.errorCode(), QStringLiteral("project.import_file_missing"));
    EXPECT_FALSE(view_model.importPreviewReady());
    EXPECT_EQ(view_model.importedPartNames().size(), 3);
}

TEST(ProjectViewModelTest, PreviewCancellationReplacementAndProjectSwitchIgnoreOldResults) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const auto a = fixture.filePath(QStringLiteral("a.stl"));
    const auto b = fixture.filePath(QStringLiteral("b.stl"));
    for (const auto& path : {a, b}) {
        QFile source(path);
        ASSERT_TRUE(source.open(QIODevice::WriteOnly));
        ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    }
    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Demo"), fixture.path())));
    const auto plan = model.planSettings();
    QSignalSpy finished(&model, &ProjectViewModel::importPreviewFinished);
    ASSERT_TRUE(model.inspectStl(a));
    EXPECT_TRUE(model.importPreviewPending());
    EXPECT_FALSE(model.importPreviewReady());
    model.cancelImportPreview();
    EXPECT_FALSE(model.importPreviewPending());
    EXPECT_FALSE(model.importPreviewReady());
    ASSERT_TRUE(model.inspectStl(a));
    ASSERT_TRUE(model.inspectStl(b));
    QTRY_VERIFY(!model.importPreviewPending());
    ASSERT_TRUE(model.importPreviewReady());
    EXPECT_EQ(model.importPreviewName(), QStringLiteral("b.stl"));
    ASSERT_EQ(finished.count(), 1);
    EXPECT_TRUE(finished.at(0).at(1).toBool());
    EXPECT_EQ(model.planSettings(), plan);
    EXPECT_TRUE(model.importedPartNames().isEmpty());
    ASSERT_TRUE(model.inspectStl(a));
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Other"), fixture.path())));
    EXPECT_FALSE(model.importPreviewPending());
    EXPECT_FALSE(model.importPreviewReady());
    finished.clear();
    ASSERT_TRUE(model.inspectStl(b));
    QTRY_VERIFY(!model.importPreviewPending());
    EXPECT_EQ(model.importPreviewName(), QStringLiteral("b.stl"));
    EXPECT_EQ(finished.count(), 1);
    EXPECT_EQ(model.currentName(), QStringLiteral("Other"));
}

TEST(ProjectViewModelTest, ConfirmedImportCompletesThroughBackgroundViewModelRequest) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const auto path = fixture.filePath(QStringLiteral("part.stl"));
    QFile source(path);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    source.close();

    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(model.inspectStl(path));
    QTRY_VERIFY(!model.importPreviewPending());
    ASSERT_TRUE(model.importPreviewReady());
    QSignalSpy finished(&model, &ProjectViewModel::projectImportFinished);

    ASSERT_TRUE(model.beginImportStl(path, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    EXPECT_TRUE(model.importPending() || finished.count() == 1);
    QTRY_COMPARE(finished.count(), 1);
    EXPECT_TRUE(finished.at(0).at(1).toBool());
    EXPECT_FALSE(model.importPending());
    EXPECT_EQ(model.importedPartNames(), QStringList{QStringLiteral("part")});
    EXPECT_EQ(model.openDocuments().size(), 2);
    EXPECT_EQ(model.activeDocumentId(), QStringLiteral("import-1"));
    ASSERT_TRUE(wait_for_active_mesh(model, QStringLiteral("import-1")));
    EXPECT_NE(model.mesh_snapshot(), nullptr);
    EXPECT_EQ(model.errorCode(), QString{});

    QFile manifest(model.currentPath());
    ASSERT_TRUE(manifest.open(QIODevice::ReadOnly));
    const auto data = manifest.readAll();
    EXPECT_TRUE(data.contains("\"id\": \"import-1\""));
    EXPECT_TRUE(
        QFile::exists(QDir(fixture.filePath(QStringLiteral("Demo")))
                          .filePath(QStringLiteral("assets/meshes/mesh-import-1-r1.pmesh"))));
}

TEST(ProjectViewModelTest, PreviewNotificationsCanReplaceOrCancelTheCompletedRequest) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const auto path = fixture.filePath(QStringLiteral("preview.stl"));
    QFile source(path);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    source.close();
    ProjectViewModel model;
    QSignalSpy finished(&model, &ProjectViewModel::importPreviewFinished);
    bool replaced = false;
    const auto replacement =
        QObject::connect(&model, &ProjectViewModel::importPreviewChanged, &model, [&] {
            if (!replaced && model.importPreviewReady()) {
                replaced = true;
                EXPECT_TRUE(model.inspectStl(path));
            }
        });
    ASSERT_TRUE(model.inspectStl(path));
    QTRY_VERIFY(!model.importPreviewPending());
    EXPECT_TRUE(replaced);
    EXPECT_TRUE(model.importPreviewReady());
    EXPECT_EQ(finished.count(), 1);
    QObject::disconnect(replacement);
    const auto cancellation =
        QObject::connect(&model, &ProjectViewModel::errorChanged, &model, [&] {
            if (!model.errorCode().isEmpty())
                model.cancelImportPreview();
        });
    ASSERT_TRUE(model.inspectStl(path + QStringLiteral(".missing")));
    QTRY_VERIFY(!model.errorCode().isEmpty());
    EXPECT_FALSE(model.importPreviewPending());
    EXPECT_FALSE(model.importPreviewReady());
    EXPECT_EQ(finished.count(), 1);
    QObject::disconnect(cancellation);
}

int main(int argc, char** argv) {
    QCoreApplication app(argc, argv);
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}

TEST(ProjectViewModelTest, DocumentTabsFollowWelcomeImportAndClose) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("sample.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Text));
    ASSERT_GT(source.write("solid sample\n"
                           "facet normal 0 0 1\n"
                           " outer loop\n"
                           "  vertex 0 0 0\n"
                           "  vertex 1 0 0\n"
                           "  vertex 0 1 0\n"
                           " endloop\n"
                           "endfacet\n"
                           "endsolid sample\n"),
              0);
    source.close();

    ProjectViewModel view_model;
    QSignalSpy documentsChanged(&view_model, &ProjectViewModel::documentsChanged);
    QSignalSpy activeChanged(&view_model, &ProjectViewModel::activeDocumentChanged);
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), fixture.path())));

    // 进入工程工作区：默认只有就绪 Welcome 且为活动文档。
    EXPECT_EQ(view_model.openDocuments().size(), 1);
    const auto welcome = view_model.openDocuments().front().toMap();
    EXPECT_EQ(welcome[QStringLiteral("id")].toString(), QStringLiteral("welcome"));
    EXPECT_EQ(welcome[QStringLiteral("state")].toString(), QStringLiteral("ready"));
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("welcome"));
    EXPECT_EQ(view_model.activeDocumentTitle(), QStringLiteral("Welcome"));
    EXPECT_EQ(view_model.mesh_snapshot(), nullptr);

    // 导入成功：自动新增并激活就绪导入页签，快照复用导入产物。
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("dual-domain"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    ASSERT_EQ(view_model.openDocuments().size(), 2);
    const auto imported = view_model.openDocuments()[1].toMap();
    EXPECT_EQ(imported[QStringLiteral("id")].toString(), QStringLiteral("import-1"));
    EXPECT_EQ(imported[QStringLiteral("state")].toString(), QStringLiteral("ready"));
    EXPECT_EQ(imported[QStringLiteral("title")].toString(), QStringLiteral("sample"));
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("import-1"));
    EXPECT_EQ(view_model.activeDocumentTitle(), QStringLiteral("sample"));
    EXPECT_EQ(activeChanged.count(), 2);
    EXPECT_NE(view_model.mesh_snapshot(), nullptr);

    // 关闭非活动 Welcome：不改变当前视口。
    documentsChanged.clear();
    view_model.closeDocument(QStringLiteral("welcome"));
    EXPECT_EQ(documentsChanged.count(), 1);
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("import-1"));
    EXPECT_NE(view_model.mesh_snapshot(), nullptr);

    // 关闭最后一个活动页签后进入空白视口。
    view_model.closeDocument(QStringLiteral("import-1"));
    EXPECT_EQ(view_model.openDocuments().size(), 0);
    EXPECT_EQ(view_model.activeDocumentId(), QString{});
    EXPECT_EQ(view_model.mesh_snapshot(), nullptr);
}

TEST(ProjectViewModelTest, ReorderingAndClosingDocumentsPreservesActiveMeshSelection) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("sample.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Text));
    ASSERT_GT(source.write("solid sample\n"
                           "facet normal 0 0 1\n"
                           " outer loop\n"
                           "  vertex 0 0 0\n"
                           "  vertex 1 0 0\n"
                           "  vertex 0 1 0\n"
                           " endloop\n"
                           "endfacet\n"
                           "endsolid sample\n"),
              0);
    source.close();

    ProjectViewModel view_model;
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));

    view_model.activateDocument(QStringLiteral("import-1"));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    const auto first_mesh = view_model.mesh_snapshot();
    ASSERT_NE(first_mesh, nullptr);
    view_model.activateDocument(QStringLiteral("import-2"));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    const auto second_mesh = view_model.mesh_snapshot();
    ASSERT_NE(second_mesh, nullptr);
    ASSERT_NE(first_mesh.get(), second_mesh.get());

    // 拖拽改变显示顺序时，活动身份及视口快照仍按稳定 ID 保持。
    view_model.moveDocument(2, 0);
    const auto reordered = view_model.openDocuments();
    ASSERT_EQ(reordered.size(), 3);
    EXPECT_EQ(reordered[0].toMap()[QStringLiteral("id")].toString(), QStringLiteral("import-2"));
    EXPECT_EQ(reordered[1].toMap()[QStringLiteral("id")].toString(), QStringLiteral("welcome"));
    EXPECT_EQ(reordered[2].toMap()[QStringLiteral("id")].toString(), QStringLiteral("import-1"));
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("import-2"));
    EXPECT_EQ(view_model.mesh_snapshot().get(), second_mesh.get());

    // 关闭非活动页签不改变场景；关闭活动页签优先激活右邻就绪文档。
    view_model.closeDocument(QStringLiteral("welcome"));
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("import-2"));
    EXPECT_EQ(view_model.mesh_snapshot().get(), second_mesh.get());
    view_model.closeDocument(QStringLiteral("import-2"));
    ASSERT_TRUE(wait_for_active_mesh(view_model, QStringLiteral("import-1")));
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("import-1"));
    EXPECT_NE(view_model.mesh_snapshot().get(), first_mesh.get());
    EXPECT_EQ(view_model.mesh_snapshot()->vertices, first_mesh->vertices);

    view_model.closeDocument(QStringLiteral("import-1"));
    EXPECT_TRUE(view_model.activeDocumentId().isEmpty());
    EXPECT_TRUE(view_model.openDocuments().isEmpty());
    EXPECT_EQ(view_model.mesh_snapshot(), nullptr);
}

TEST(ProjectViewModelTest, ReopenLoadsWelcomeOnlyAndActivatesSavedRecordOnDemand) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("sample.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Text));
    ASSERT_GT(source.write("solid sample\n"
                           "facet normal 0 0 1\n"
                           " outer loop\n"
                           "  vertex 0 0 0\n"
                           "  vertex 2 0 0\n"
                           "  vertex 0 1 0\n"
                           " endloop\n"
                           "endfacet\n"
                           "endsolid sample\n"),
              0);
    source.close();

    ProjectViewModel view_model;
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    ASSERT_TRUE(finish_project_operation(view_model, view_model.saveProject()));

    ProjectViewModel reopened;
    ASSERT_TRUE(finish_project_operation(reopened, reopened.openProject(view_model.currentPath())));
    // 重开工程只保留 Welcome 页签；不继承旧会话活动文档。
    EXPECT_EQ(reopened.openDocuments().size(), 1);
    EXPECT_EQ(reopened.activeDocumentId(), QStringLiteral("welcome"));
    EXPECT_EQ(reopened.mesh_snapshot(), nullptr);

    // 点击工程树未打开记录：Loading 页签先建立，激活仍由成功结果驱动。
    reopened.openImportRecord(QStringLiteral("import-1"));
    ASSERT_EQ(reopened.openDocuments().size(), 2);
    const auto loading = reopened.openDocuments()[1].toMap();
    EXPECT_EQ(loading[QStringLiteral("id")].toString(), QStringLiteral("import-1"));
    EXPECT_EQ(reopened.activeDocumentId(), QStringLiteral("welcome"));
    QTRY_COMPARE(reopened.openDocuments()[1].toMap()[QStringLiteral("state")].toString(),
                 QStringLiteral("ready"));
    QTRY_COMPARE(reopened.activeDocumentId(), QStringLiteral("import-1"));
    QTRY_VERIFY(reopened.mesh_snapshot() != nullptr);
    EXPECT_EQ(reopened.mesh_snapshot()->vertices.size(), 3U);

    // 未知记录同步拒绝，不建页签。
    const auto before = reopened.openDocuments().size();
    reopened.openImportRecord(QStringLiteral("import-99"));
    EXPECT_EQ(reopened.openDocuments().size(), before);

    // 关闭 Welcome 和网格页签后从空白视口重新打开，不依赖占位页签存在。
    reopened.closeDocument(QStringLiteral("import-1"));
    EXPECT_EQ(reopened.openDocuments().size(), 1);
    EXPECT_EQ(reopened.activeDocumentId(), QStringLiteral("welcome"));
    reopened.closeDocument(QStringLiteral("welcome"));
    EXPECT_TRUE(reopened.openDocuments().isEmpty());
    EXPECT_TRUE(reopened.activeDocumentId().isEmpty());
    EXPECT_FALSE(reopened.placeholder_visible());
    reopened.openImportRecord(QStringLiteral("import-1"));
    QTRY_COMPARE(reopened.activeDocumentId(), QStringLiteral("import-1"));
    QTRY_VERIFY(reopened.mesh_snapshot() != nullptr);
}

TEST(ProjectViewModelTest, DuplicateImportOrdinalsFollowImportOrderAcrossActivationAndClose) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("part.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly));
    ASSERT_GT(source.write("vertex 0 0 0\nvertex 1 0 0\nvertex 0 1 0\n"), 0);
    source.close();

    ProjectViewModel project;
    ASSERT_TRUE(finish_project_operation(
        project, project.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(project.importStl(sourcePath, QStringLiteral("solid-3d"),
                                  QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(project, project.activeDocumentId()));
    ASSERT_TRUE(project.importStl(sourcePath, QStringLiteral("solid-3d"),
                                  QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(project, project.activeDocumentId()));
    ASSERT_TRUE(finish_project_operation(project, project.saveProject()));

    ProjectViewModel reopened;
    ASSERT_TRUE(finish_project_operation(reopened, reopened.openProject(project.currentPath())));
    reopened.openImportRecord(QStringLiteral("import-2"));
    QTRY_COMPARE(reopened.activeDocumentId(), QStringLiteral("import-2"));
    reopened.openImportRecord(QStringLiteral("import-1"));
    QTRY_COMPARE(reopened.activeDocumentId(), QStringLiteral("import-1"));

    const auto ordinal_for = [&reopened](const QString& id) {
        for (const auto& entry : reopened.openDocuments()) {
            const auto document = entry.toMap();
            if (document.value(QStringLiteral("id")).toString() == id) {
                return qMakePair(document.value(QStringLiteral("displayOrdinal")).toInt(),
                                 document.value(QStringLiteral("displayCount")).toInt());
            }
        }
        return qMakePair(0, 0);
    };
    EXPECT_EQ(ordinal_for(QStringLiteral("import-1")), qMakePair(1, 2));
    EXPECT_EQ(ordinal_for(QStringLiteral("import-2")), qMakePair(2, 2));

    reopened.closeDocument(QStringLiteral("import-1"));
    EXPECT_EQ(ordinal_for(QStringLiteral("import-2")), qMakePair(2, 2));
}

TEST(ProjectViewModelTest, ClosedDocumentsReleaseTheirMeshSnapshotsAfterRepeatedSwitching) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("sample.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Text));
    ASSERT_GT(source.write("solid sample\n"
                           "facet normal 0 0 1\n"
                           " outer loop\n"
                           "  vertex 0 0 0\n"
                           "  vertex 1 0 0\n"
                           "  vertex 0 1 0\n"
                           " endloop\n"
                           "endfacet\n"
                           "endsolid sample\n"),
              0);
    source.close();

    ProjectViewModel view_model;
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    const std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot> first_snapshot =
        view_model.mesh_snapshot();
    ASSERT_FALSE(first_snapshot.expired());

    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    const std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot> second_snapshot =
        view_model.mesh_snapshot();
    ASSERT_FALSE(second_snapshot.expired());
    EXPECT_TRUE(first_snapshot.expired());

    for (int round = 0; round < 32; ++round) {
        view_model.activateDocument(QStringLiteral("import-1"));
        ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
        ASSERT_NE(view_model.mesh_snapshot(), nullptr);
        EXPECT_TRUE(second_snapshot.expired());
        view_model.activateDocument(QStringLiteral("import-2"));
        ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
        ASSERT_NE(view_model.mesh_snapshot(), nullptr);
        EXPECT_TRUE(first_snapshot.expired());
    }
    EXPECT_TRUE(first_snapshot.expired());
    EXPECT_TRUE(second_snapshot.expired());

    view_model.closeDocument(QStringLiteral("import-1"));
    EXPECT_TRUE(first_snapshot.expired());
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("import-2"));
    EXPECT_NE(view_model.mesh_snapshot(), nullptr);

    const std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot> active_snapshot =
        view_model.mesh_snapshot();
    view_model.closeDocument(QStringLiteral("import-2"));
    EXPECT_TRUE(active_snapshot.expired());
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("welcome"));
    EXPECT_EQ(view_model.mesh_snapshot(), nullptr);
}

TEST(ProjectViewModelTest, ClosingLastTabFlipsPlaceholderVisibility) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    ProjectViewModel view_model;
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), fixture.path())));

    EXPECT_TRUE(view_model.placeholder_visible());
    view_model.closeDocument(QStringLiteral("welcome"));

    EXPECT_FALSE(view_model.placeholder_visible());
    EXPECT_TRUE(view_model.activeDocumentId().isEmpty());
    EXPECT_EQ(view_model.mesh_snapshot(), nullptr);
}

TEST(ProjectViewModelTest, FailedLoadRetainsTabAndCloseReleasesActivationState) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());

    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("sample.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Text));
    ASSERT_GT(source.write("solid sample\n"
                           "facet normal 0 0 1\n"
                           " outer loop\n"
                           "  vertex 0 0 0\n"
                           "  vertex 1 0 0\n"
                           "  vertex 0 1 0\n"
                           " endloop\n"
                           "endfacet\n"
                           "endsolid sample\n"),
              0);
    source.close();

    ProjectViewModel view_model;
    ASSERT_TRUE(finish_project_operation(
        view_model, view_model.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(view_model, view_model.activeDocumentId()));
    ASSERT_TRUE(finish_project_operation(view_model, view_model.saveProject()));
    const auto assetPath = QDir(QFileInfo(view_model.currentPath()).absolutePath())
                               .filePath(QStringLiteral("assets/meshes/mesh-import-1-r1.pmesh"));

    ProjectViewModel reopened;
    ASSERT_TRUE(finish_project_operation(reopened, reopened.openProject(view_model.currentPath())));

    // 资产损坏 → 加载失败：页签保留 Failed 态并携带原因，视口保持 Welcome。
    ASSERT_TRUE(QFile::rename(assetPath, assetPath + QStringLiteral(".bak")));
    reopened.openImportRecord(QStringLiteral("import-1"));
    QTRY_COMPARE(reopened.openDocuments().size(), 2);
    QTRY_COMPARE(reopened.openDocuments()[1].toMap()[QStringLiteral("state")].toString(),
                 QStringLiteral("failed"));
    EXPECT_EQ(reopened.openDocuments()[1].toMap()[QStringLiteral("message")].toString(),
              QStringLiteral("The saved mesh asset is missing from the project package."));
    const auto diagnostic =
        reopened.openDocuments()[1].toMap().value(QStringLiteral("diagnostic")).toMap();
    EXPECT_EQ(diagnostic.value(QStringLiteral("code")).toString(),
              QStringLiteral("project.asset_missing"));
    EXPECT_EQ(diagnostic.value(QStringLiteral("category")).toString(), QStringLiteral("missing"));
    EXPECT_FALSE(diagnostic.value(QStringLiteral("detail")).toString().isEmpty());
    EXPECT_EQ(reopened.activeDocumentId(), QStringLiteral("welcome"));
    EXPECT_EQ(reopened.mesh_snapshot(), nullptr);

    // 关闭 Failed 页签：释放激活簿记，页签栏回到仅 Welcome。
    reopened.closeDocument(QStringLiteral("import-1"));
    EXPECT_EQ(reopened.openDocuments().size(), 1);
    EXPECT_EQ(reopened.activeDocumentId(), QStringLiteral("welcome"));

    // 资产恢复后重新打开：新 attempt 加载成功并激活。
    ASSERT_TRUE(QFile::rename(assetPath + QStringLiteral(".bak"), assetPath));
    reopened.openImportRecord(QStringLiteral("import-1"));
    QTRY_COMPARE(reopened.activeDocumentId(), QStringLiteral("import-1"));
    QTRY_VERIFY(reopened.mesh_snapshot() != nullptr);
    EXPECT_EQ(reopened.mesh_snapshot()->vertices.size(), 3U);
    EXPECT_TRUE(
        reopened.openDocuments()[1].toMap().value(QStringLiteral("diagnostic")).toMap().isEmpty());
}

TEST(ProjectViewModelTest, FillSettingsConfirmAsynchronouslyAndReopenFromRust) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    const QString sourcePath = QDir(fixture.path()).filePath(QStringLiteral("part.stl"));
    QFile source(sourcePath);
    ASSERT_TRUE(source.open(QIODevice::WriteOnly | QIODevice::Text));
    ASSERT_GT(source.write("solid case\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 "
                           "0\nvertex 0 1 0\nendloop\nendfacet\nendsolid case\n"),
              0);
    source.close();
    ProjectViewModel model;
    ASSERT_TRUE(finish_project_operation(
        model, model.createProject(QStringLiteral("Demo"), fixture.path())));
    ASSERT_TRUE(model.importStl(sourcePath, QStringLiteral("dual-domain"),
                                QStringLiteral("millimeters"), false));
    ASSERT_TRUE(wait_for_active_mesh(model, model.activeDocumentId()));
    const auto initial = model.planSettings();
    auto candidate = initial.value(QStringLiteral("fillSettings")).toMap();
    candidate.insert(QStringLiteral("meltTemperature"), 235.0);
    QSignalSpy finished(&model, &ProjectViewModel::fillSettingsConfirmationFinished);
    ASSERT_TRUE(model.setFillSettings(
        model.currentPath(), model.planSettings().value(QStringLiteral("revision")).toULongLong(),
        initial.value(QStringLiteral("importId")).toString(), candidate));
    EXPECT_TRUE(model.fillSettingsConfirmationPending());
    EXPECT_EQ(model.planSettings(), initial);
    EXPECT_FALSE(model.setFillSettings(
        model.currentPath(), model.planSettings().value(QStringLiteral("revision")).toULongLong(),
        initial.value(QStringLiteral("importId")).toString(), candidate));
    QTRY_VERIFY(!finished.isEmpty());
    ASSERT_EQ(finished.count(), 1);
    EXPECT_TRUE(finished.at(0).at(0).toBool());
    EXPECT_FALSE(model.fillSettingsConfirmationPending());
    EXPECT_TRUE(model.planSettings().value(QStringLiteral("fillSettingsConfirmed")).toBool());
    EXPECT_EQ(model.planSettings().value(QStringLiteral("fillSettings")).toMap(), candidate);
    ProjectViewModel reopened;
    ASSERT_TRUE(finish_project_operation(
        reopened, reopened.openProjectUrl(QUrl::fromLocalFile(model.currentPath()))));
    expect_same_plan_settings(reopened.planSettings(), model.planSettings());
    candidate.insert(QStringLiteral("flowRate"), -1.0);
    ASSERT_TRUE(model.setFillSettings(
        model.currentPath(), model.planSettings().value(QStringLiteral("revision")).toULongLong(),
        initial.value(QStringLiteral("importId")).toString(), candidate));
    QTRY_COMPARE(finished.count(), 2);
    EXPECT_FALSE(finished.at(1).at(0).toBool());
    EXPECT_FALSE(model.fillSettingsConfirmationPending());
    EXPECT_EQ(model.errorCode(), QStringLiteral("project.manifest_invalid"));
    EXPECT_EQ(model.errorCategory(), QStringLiteral("format"));
    EXPECT_FALSE(model.errorDetail().isEmpty());
    expect_same_plan_settings(reopened.planSettings(), model.planSettings());

    candidate = model.planSettings().value(QStringLiteral("fillSettings")).toMap();
    candidate.insert(QStringLiteral("meltTemperature"), 240.0);
    ASSERT_TRUE(model.setFillSettings(
        model.currentPath(), model.planSettings().value(QStringLiteral("revision")).toULongLong(),
        initial.value(QStringLiteral("importId")).toString(), candidate));
    QTRY_COMPARE(finished.count(), 3);
    ASSERT_EQ(finished.count(), 3);
    EXPECT_TRUE(finished.at(2).at(0).toBool());
    EXPECT_FALSE(model.fillSettingsConfirmationPending());
    EXPECT_TRUE(model.errorCode().isEmpty());
    EXPECT_TRUE(model.errorCategory().isEmpty());
    EXPECT_TRUE(model.errorDetail().isEmpty());
}
