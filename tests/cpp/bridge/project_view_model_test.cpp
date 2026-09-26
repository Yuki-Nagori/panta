/// ProjectViewModel 验收测试（任务 057）：Qt 字符串适配、Rust 工程服务命令
/// 生命周期、清单创建/打开/保存和可恢复错误。

#include "panta/visualization/mesh_source.hpp"
#include "project_view_model.hpp"
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QIODevice>
#include <QSignalSpy>
#include <QString>
#include <QTemporaryDir>
#include <QUrl>
#include <QVariantMap>
#include <QtTest/qtest.h>
#include <array>
#include <gtest/gtest.h>

using panta::bridge::ProjectViewModel;

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

    const QString secondSourcePath =
        QDir(fixture.path()).filePath(QStringLiteral("sample-second.stl"));
    ASSERT_TRUE(QFile::copy(sourcePath, secondSourcePath));
    ASSERT_TRUE(view_model.importStl(secondSourcePath, QStringLiteral("dual-domain"),
                                     QStringLiteral("millimeters"), false));
    EXPECT_EQ(view_model.importedPartNames(),
              QStringList({QStringLiteral("sample.stl"), QStringLiteral("sample-second.stl")}));
    EXPECT_EQ(view_model.importedPartName(), QStringLiteral("sample-second.stl"));

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

    // 关闭活动导入页签：无就绪邻位，回到 Welcome（空串并非 Welcome 场景）。
    view_model.closeDocument(QStringLiteral("import-1"));
    EXPECT_EQ(view_model.openDocuments().size(), 0);
    EXPECT_EQ(view_model.activeDocumentId(), QString{});
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
    QTRY_COMPARE(loading[QStringLiteral("state")].toString(), QStringLiteral("ready"));
    QTRY_COMPARE(reopened.activeDocumentId(), QStringLiteral("import-1"));
    QTRY_VERIFY(reopened.mesh_snapshot() != nullptr);
    EXPECT_EQ(reopened.mesh_snapshot()->vertices.size(), 3U);

    // 未知记录同步拒绝，不建页签。
    const auto before = reopened.openDocuments().size();
    reopened.openImportRecord(QStringLiteral("import-99"));
    EXPECT_EQ(reopened.openDocuments().size(), before);
}
