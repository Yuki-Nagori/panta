/// ProjectViewModel 回归：工程命令、错误映射及任务 080 的视口文档生命周期。

#include "panta/visualization/mesh_source.hpp"
#include "project_view_model.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QIODevice>
#include <QSignalSpy>
#include <QString>
#include <QTemporaryDir>
#include <QUrl>
#include <QVariantMap>
#include <QtCore/qcontainerfwd.h>
#include <QtTest/qtest.h>
#include <array>
#include <gtest/gtest.h>
#include <memory>
#include <qtestcase.h>

using panta::bridge::ProjectViewModel;

namespace {

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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), location));
    ASSERT_EQ(created.count(), 1);
    EXPECT_EQ(view_model.currentName(), QStringLiteral("Demo"));
    EXPECT_FALSE(view_model.dirty());

    const QString project_path = view_model.currentPath();
    EXPECT_TRUE(
        QDir::fromNativeSeparators(project_path).endsWith(QStringLiteral("/Demo/Demo.panta")));
    EXPECT_TRUE(QFile::exists(project_path));
    EXPECT_TRUE(view_model.renameProject(QStringLiteral("Renamed")));
    EXPECT_TRUE(view_model.dirty());
    EXPECT_TRUE(view_model.saveProject());
    EXPECT_FALSE(view_model.dirty());

    ProjectViewModel reopened;
    QSignalSpy opened(&reopened, &ProjectViewModel::projectOpened);
    ASSERT_TRUE(reopened.openProjectUrl(QUrl::fromLocalFile(project_path)));
    ASSERT_EQ(opened.count(), 1);
    EXPECT_EQ(reopened.currentName(), QStringLiteral("Renamed"));
    EXPECT_FALSE(reopened.dirty());
}

TEST(ProjectViewModelTest, MapsRustErrorsWithoutCreatingInvalidTargets) {
    QTemporaryDir fixture;
    ASSERT_TRUE(fixture.isValid());
    ProjectViewModel view_model;
    const QString location = QDir(fixture.path()).absolutePath();

    EXPECT_FALSE(view_model.createProject(QStringLiteral("../escape"), location));
    EXPECT_EQ(view_model.errorCode(), QStringLiteral("project.invalid_name"));
    EXPECT_FALSE(view_model.error().isEmpty());
    EXPECT_FALSE(QDir(fixture.path()).exists(QStringLiteral("escape")));

    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), location));
    EXPECT_FALSE(view_model.createProject(QStringLiteral("Demo"), location));
    EXPECT_EQ(view_model.errorCode(), QStringLiteral("project.already_exists"));

    ProjectViewModel empty;
    EXPECT_FALSE(empty.saveProject());
    EXPECT_EQ(empty.errorCode(), QStringLiteral("project.no_project"));
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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), fixture.path()));
    EXPECT_TRUE(view_model.importedPartNames().isEmpty());
    ASSERT_TRUE(view_model.inspectStl(sourcePath));
    EXPECT_TRUE(view_model.importPreviewReady());
    EXPECT_EQ(view_model.importPreviewName(), QStringLiteral("sample.stl"));
    EXPECT_EQ(view_model.importPreviewDimensions(), QStringLiteral("1.00 × 1.00 × 0.00"));
    EXPECT_EQ(view_model.importPreviewTriangleCount(), 1U);

    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("dual-domain"),
                                     QStringLiteral("millimeters"), true));
    EXPECT_EQ(view_model.importedPartNames(), QStringList{QStringLiteral("sample.stl")});
    EXPECT_EQ(view_model.importedPartName(), QStringLiteral("sample.stl"));
    EXPECT_EQ(view_model.importedMeshType(), QStringLiteral("dual-domain"));
    EXPECT_EQ(view_model.importedUnits(), QStringLiteral("millimeters"));
    EXPECT_EQ(view_model.importedDimensions(), QStringLiteral("1.00 × 1.00 × 0.00 mm"));
    EXPECT_EQ(view_model.importedTriangleCount(), 1U);
    EXPECT_TRUE(QFile::exists(view_model.importedAssetPath()));
    const auto mesh = view_model.mesh_snapshot();
    ASSERT_NE(mesh, nullptr);
    EXPECT_EQ(mesh->vertices.size(), 3U);
    EXPECT_EQ(mesh->vertices[1], (std::array<double, 3>{1.0, 0.0, 0.0}));

    const auto firstPlan = view_model.planSettings();
    QSignalSpy planChanged(&view_model, &ProjectViewModel::planSettingsChanged);
    EXPECT_EQ(firstPlan.value(QStringLiteral("meshType")).toString(),
              QStringLiteral("dual-domain"));
    ASSERT_TRUE(view_model.setAnalysisSequence(
        view_model.currentPath(), firstPlan.value(QStringLiteral("revision")).toULongLong(),
        firstPlan.value(QStringLiteral("importId")).toString(), QStringLiteral("fill-pack")));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("sequenceId")).toString(),
              QStringLiteral("fill-pack"));
    EXPECT_EQ(planChanged.count(), 1);

    const QString secondSourcePath =
        QDir(fixture.path()).filePath(QStringLiteral("sample-second.stl"));
    ASSERT_TRUE(QFile::copy(sourcePath, secondSourcePath));
    ASSERT_TRUE(view_model.importStl(secondSourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    EXPECT_EQ(view_model.importedPartNames(),
              QStringList({QStringLiteral("sample.stl"), QStringLiteral("sample-second.stl")}));
    EXPECT_EQ(view_model.importedPartName(), QStringLiteral("sample-second.stl"));

    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("meshType")).toString(),
              QStringLiteral("solid-3d"));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("sequenceId")).toString(),
              QStringLiteral("fill"));
    view_model.activateDocument(firstPlan.value(QStringLiteral("importId")).toString());
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("meshType")).toString(),
              QStringLiteral("dual-domain"));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("sequenceId")).toString(),
              QStringLiteral("fill-pack"));
    view_model.activateDocument(QStringLiteral("welcome"));
    EXPECT_EQ(view_model.planSettings().value(QStringLiteral("meshType")).toString(),
              QStringLiteral("solid-3d"));

    ProjectViewModel reopened;
    ASSERT_TRUE(reopened.openProject(view_model.currentPath()));
    EXPECT_EQ(reopened.importedPartNames(),
              QStringList({QStringLiteral("sample.stl"), QStringLiteral("sample-second.stl")}));
    EXPECT_EQ(reopened.importedPartName(), QStringLiteral("sample-second.stl"));
    EXPECT_EQ(reopened.importedAssetPath(), view_model.importedAssetPath());
    EXPECT_EQ(reopened.importedDimensions(), QStringLiteral("1.00 × 1.00 × 0.00 mm"));
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
    EXPECT_EQ(view_model.importedPartNames(),
              QStringList({QStringLiteral("sample.stl"), QStringLiteral("sample-second.stl"),
                           QStringLiteral("sample.stl")}));
    EXPECT_EQ(view_model.importedPartName(), QStringLiteral("sample.stl"));
    EXPECT_EQ(mesh_changed.count(), 1);
    const auto replacement = view_model.mesh_snapshot();
    ASSERT_NE(replacement, nullptr);
    EXPECT_EQ(replacement->vertices[1], (std::array<double, 3>{2.0, 0.0, 0.0}));
    EXPECT_EQ(mesh->vertices[1], (std::array<double, 3>{1.0, 0.0, 0.0}));
    ASSERT_TRUE(view_model.inspectStl(sourcePath));
    EXPECT_FALSE(view_model.inspectStl(sourcePath + QStringLiteral(".missing")));
    EXPECT_FALSE(view_model.importPreviewReady());
    EXPECT_EQ(view_model.importedPartNames().size(), 3);
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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), fixture.path()));

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
    ASSERT_EQ(view_model.openDocuments().size(), 2);
    const auto imported = view_model.openDocuments()[1].toMap();
    EXPECT_EQ(imported[QStringLiteral("id")].toString(), QStringLiteral("import-1"));
    EXPECT_EQ(imported[QStringLiteral("state")].toString(), QStringLiteral("ready"));
    EXPECT_EQ(imported[QStringLiteral("title")].toString(), QStringLiteral("sample.stl"));
    EXPECT_EQ(view_model.activeDocumentId(), QStringLiteral("import-1"));
    EXPECT_EQ(view_model.activeDocumentTitle(), QStringLiteral("sample.stl"));
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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), fixture.path()));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));

    view_model.activateDocument(QStringLiteral("import-1"));
    const auto first_mesh = view_model.mesh_snapshot();
    ASSERT_NE(first_mesh, nullptr);
    view_model.activateDocument(QStringLiteral("import-2"));
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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), fixture.path()));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(view_model.saveProject());

    ProjectViewModel reopened;
    ASSERT_TRUE(reopened.openProject(view_model.currentPath()));
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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), fixture.path()));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    const std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot> first_snapshot =
        view_model.mesh_snapshot();
    ASSERT_FALSE(first_snapshot.expired());

    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    const std::weak_ptr<const panta::visualization::SurfaceMeshSnapshot> second_snapshot =
        view_model.mesh_snapshot();
    ASSERT_FALSE(second_snapshot.expired());
    EXPECT_TRUE(first_snapshot.expired());

    for (int round = 0; round < 32; ++round) {
        view_model.activateDocument(QStringLiteral("import-1"));
        ASSERT_NE(view_model.mesh_snapshot(), nullptr);
        EXPECT_TRUE(second_snapshot.expired());
        view_model.activateDocument(QStringLiteral("import-2"));
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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), fixture.path()));

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
    ASSERT_TRUE(view_model.createProject(QStringLiteral("Demo"), fixture.path()));
    ASSERT_TRUE(view_model.importStl(sourcePath, QStringLiteral("solid-3d"),
                                     QStringLiteral("millimeters"), false));
    ASSERT_TRUE(view_model.saveProject());
    const auto assetPath = view_model.importedAssetPath();

    ProjectViewModel reopened;
    ASSERT_TRUE(reopened.openProject(view_model.currentPath()));

    // 资产损坏 → 加载失败：页签保留 Failed 态并携带原因，视口保持 Welcome。
    ASSERT_TRUE(QFile::rename(assetPath, assetPath + QStringLiteral(".bak")));
    reopened.openImportRecord(QStringLiteral("import-1"));
    QTRY_COMPARE(reopened.openDocuments().size(), 2);
    QTRY_COMPARE(reopened.openDocuments()[1].toMap()[QStringLiteral("state")].toString(),
                 QStringLiteral("failed"));
    EXPECT_EQ(reopened.openDocuments()[1].toMap()[QStringLiteral("message")].toString(),
              QStringLiteral("The saved STL asset is missing from the project package."));
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
    ASSERT_TRUE(model.createProject(QStringLiteral("Demo"), fixture.path()));
    ASSERT_TRUE(model.importStl(sourcePath, QStringLiteral("dual-domain"),
                                QStringLiteral("millimeters"), false));
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
    ASSERT_TRUE(finished.wait(5000));
    ASSERT_EQ(finished.count(), 1);
    EXPECT_TRUE(finished.at(0).at(0).toBool());
    EXPECT_FALSE(model.fillSettingsConfirmationPending());
    EXPECT_TRUE(model.planSettings().value(QStringLiteral("fillSettingsConfirmed")).toBool());
    EXPECT_EQ(model.planSettings().value(QStringLiteral("fillSettings")).toMap(), candidate);
    ProjectViewModel reopened;
    ASSERT_TRUE(reopened.openProjectUrl(QUrl::fromLocalFile(model.currentPath())));
    expect_same_plan_settings(reopened.planSettings(), model.planSettings());
    candidate.insert(QStringLiteral("flowRate"), -1.0);
    ASSERT_TRUE(model.setFillSettings(
        model.currentPath(), model.planSettings().value(QStringLiteral("revision")).toULongLong(),
        initial.value(QStringLiteral("importId")).toString(), candidate));
    ASSERT_TRUE(finished.wait(5000));
    EXPECT_FALSE(finished.at(1).at(0).toBool());
    EXPECT_FALSE(model.fillSettingsConfirmationPending());
    expect_same_plan_settings(reopened.planSettings(), model.planSettings());
}
