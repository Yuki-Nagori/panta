/// PathHost 验收测试（任务 023）：标准目录注入、工程根解析与 cwd 无关、
/// 引用规则拒绝矩阵、file URL 单次解码、非 Unicode 拒绝与根外符号链接。
#include "panta_ffi.h"
#include "path_host.hpp"
#include <QChar>
#include <QCoreApplication>
#include <QDir>
#include <QFile>
#include <QFileDevice>
#include <QFileInfo>
#include <QIODevice>
#include <QString>
#include <QTemporaryDir>
#include <QUrl>
#include <QtCore/qtypes.h>
#include <gtest/gtest.h>
#include <initializer_list>
#include <memory>
#include <utility>

namespace {

using panta::bridge::PathHost;

/// 测试标准根必须位于 QTemporaryDir；QStandardPaths 的 test mode 在 macOS
/// 仍会解析到用户 home 下的 .qttest，受沙箱权限影响，不能作为写入夹具。
struct IsolatedStandardRoots {
    QTemporaryDir base;

    [[nodiscard]] bool isValid() const { return base.isValid(); }

    [[nodiscard]] std::unique_ptr<PathHost> create(QString* error) const {
        const QString root = base.path();
        return PathHost::createWithStandardRoots(
            {panta::bridge::StandardRoot{panta::ffi::PathRootKind::UserConfig,
                                         root + QStringLiteral("/user-config"),
                                         QStringLiteral("user-config")},
             panta::bridge::StandardRoot{panta::ffi::PathRootKind::AppData,
                                         root + QStringLiteral("/app-data"),
                                         QStringLiteral("app-data")},
             panta::bridge::StandardRoot{panta::ffi::PathRootKind::Cache,
                                         root + QStringLiteral("/cache"), QStringLiteral("cache")},
             panta::bridge::StandardRoot{panta::ffi::PathRootKind::Session,
                                         root + QStringLiteral("/session"),
                                         QStringLiteral("session")}},
            error);
    }
};

[[nodiscard]] QString resolveOrDie(const PathHost& host, const QString& reference) {
    QString error;
    const QString resolved = host.resolve(reference, &error);
    if (!error.isEmpty()) {
        ADD_FAILURE() << "resolve failed: " << error.toStdString();
    }
    return resolved;
}

} // namespace

TEST(PathHostTest, StandardDirectoriesAreInjected) {
    IsolatedStandardRoots standardRoots;
    ASSERT_TRUE(standardRoots.isValid());
    QString error;
    auto host = standardRoots.create(&error);
    ASSERT_NE(host, nullptr) << error.toStdString();

    const QString cache = resolveOrDie(*host, QStringLiteral("cache:/tiles/v1"));
    EXPECT_TRUE(QDir::isAbsolutePath(cache)) << cache.toStdString();
    EXPECT_TRUE(cache.startsWith(standardRoots.base.path())) << cache.toStdString();

    EXPECT_FALSE(host->resolve(QStringLiteral("user-config:/settings.pa"), &error).isEmpty());
    EXPECT_TRUE(error.isEmpty()) << error.toStdString();
}

TEST(PathHostTest, ProjectResolutionIsCwdIndependentAndRelocatable) {
    IsolatedStandardRoots standardRoots;
    ASSERT_TRUE(standardRoots.isValid());
    QTemporaryDir projectRoot;
    ASSERT_TRUE(projectRoot.isValid());
    QString error;
    auto host = standardRoots.create(&error);
    ASSERT_NE(host, nullptr) << error.toStdString();
    ASSERT_TRUE(host->setProjectRoot(QDir(projectRoot.path()).absolutePath(), &error))
        << error.toStdString();

    const QString before = resolveOrDie(*host, QStringLiteral("project:/资产 齿轮/box v1/a.step"));

    const QString originalCwd = QDir::currentPath();
    const QTemporaryDir otherCwd;
    ASSERT_TRUE(QDir::setCurrent(otherCwd.path()));
    const QString moved = resolveOrDie(*host, QStringLiteral("project:/资产 齿轮/box v1/a.step"));
    EXPECT_EQ(before, moved);
    ASSERT_TRUE(QDir::setCurrent(originalCwd));

    // 搬迁工程根后相对引用指向新根：引用语义跟随根，而非旧位置。
    QTemporaryDir relocated;
    ASSERT_TRUE(relocated.isValid());
    ASSERT_TRUE(host->setProjectRoot(QDir(relocated.path()).absolutePath(), &error))
        << error.toStdString();
    const QString after = resolveOrDie(*host, QStringLiteral("project:/资产 齿轮/box v1/a.step"));
    EXPECT_NE(before, after);
    EXPECT_TRUE(after.startsWith(QDir(relocated.path()).absolutePath()));
}

TEST(PathHostTest, WriteTargetCreatesThenExistingResolves) {
    IsolatedStandardRoots standardRoots;
    ASSERT_TRUE(standardRoots.isValid());
    QTemporaryDir projectRoot;
    ASSERT_TRUE(projectRoot.isValid());
    QString error;
    auto host = standardRoots.create(&error);
    ASSERT_NE(host, nullptr) << error.toStdString();
    ASSERT_TRUE(host->setProjectRoot(QDir(projectRoot.path()).absolutePath(), &error))
        << error.toStdString();

    const QString target =
        host->resolveWriteTarget(QStringLiteral("project:/pockets/新 口袋/pocket.pa"), &error);
    ASSERT_TRUE(error.isEmpty()) << error.toStdString();

    const QDir parent = QFileInfo(target).dir();
    ASSERT_TRUE(parent.mkpath(QStringLiteral(".")));
    QFile file(target);
    ASSERT_TRUE(file.open(QIODevice::WriteOnly));
    file.write("pa");
    file.close();

    const QString existing =
        host->resolveExisting(QStringLiteral("project:/pockets/新 口袋/pocket.pa"), &error);
    ASSERT_TRUE(error.isEmpty()) << error.toStdString();
    EXPECT_EQ(QFileInfo(existing).canonicalFilePath(), QFileInfo(target).canonicalFilePath());

    const QString missing =
        host->resolveExisting(QStringLiteral("project:/pockets/missing.pa"), &error);
    EXPECT_TRUE(error.startsWith(QStringLiteral("path.not_found"))) << error.toStdString();
}

TEST(PathHostTest, InvalidReferencesAreRejectedWithStableCodes) {
    IsolatedStandardRoots standardRoots;
    ASSERT_TRUE(standardRoots.isValid());
    QTemporaryDir projectRoot;
    ASSERT_TRUE(projectRoot.isValid());
    QString error;
    auto host = standardRoots.create(&error);
    ASSERT_NE(host, nullptr) << error.toStdString();
    ASSERT_TRUE(host->setProjectRoot(QDir(projectRoot.path()).absolutePath(), &error))
        << error.toStdString();

    for (const auto& [reference, code] : std::initializer_list<std::pair<QString, QString>>{
             {QStringLiteral("project:/../escape"), QStringLiteral("path.parent_escape")},
             {QStringLiteral("project:/C:/win"), QStringLiteral("path.absolute_rejected")},
             {QStringLiteral("project:/CON"), QStringLiteral("path.reserved_name")},
             {QStringLiteral("project:/tail."), QStringLiteral("path.trailing_dot_or_space")},
             {QStringLiteral("workspace:/x"), QStringLiteral("path.unknown_scheme")},
             {QStringLiteral("assets/x"), QStringLiteral("path.missing_scheme")},
             {QStringLiteral("qrc:/icons/x.svg"), QStringLiteral("path.qrc_not_native")},
         }) {
        error.clear();
        const QString resolved = host->resolve(reference, &error);
        EXPECT_TRUE(resolved.isEmpty()) << reference.toStdString();
        EXPECT_TRUE(error.startsWith(code))
            << reference.toStdString() << " -> " << error.toStdString();
    }

    // 未注入工程根的类别立即失败，不回退 cwd。
    auto bareHost = standardRoots.create(&error);
    ASSERT_NE(bareHost, nullptr) << error.toStdString();
    error.clear();
    const QString unresolved = bareHost->resolve(QStringLiteral("project:/a"), &error);
    EXPECT_TRUE(error.startsWith(QStringLiteral("path.root_missing"))) << error.toStdString();
}

TEST(PathHostTest, FileUrlsDecodeExactlyOnce) {
    QString error;
    const QString path = QStringLiteral("/tmp/panta 空格%20名字.pa");
    const QUrl url = QUrl::fromLocalFile(path);
    const QString decoded = PathHost::fileUrlToPath(url, &error);
    EXPECT_TRUE(error.isEmpty()) << error.toStdString();
    // 编码后的 %20 不再二次解码：结果与原始路径一致。
    EXPECT_EQ(decoded, path);

    error.clear();
    const QString rejected =
        PathHost::fileUrlToPath(QUrl(QStringLiteral("qrc:///icons/x.svg")), &error);
    EXPECT_TRUE(rejected.isEmpty());
    EXPECT_TRUE(error.startsWith(QStringLiteral("path.not_file_url"))) << error.toStdString();
}

TEST(PathHostTest, NonRoundTrippableTextIsRejected) {
    IsolatedStandardRoots standardRoots;
    ASSERT_TRUE(standardRoots.isValid());
    // 未配对代理项无法往返 UTF-8：按当前契约拒绝，不做有损转换。
    const QString surrogate(QChar(0xD800));
    std::string utf8;
    QString error;
    EXPECT_FALSE(PathHost::toBoundaryUtf8(surrogate, &utf8, &error));
    EXPECT_EQ(error, QStringLiteral("path.non_unicode"));

    error.clear();
    auto host = standardRoots.create(&error);
    ASSERT_NE(host, nullptr) << error.toStdString();
    const QString resolved =
        host->resolve(QStringLiteral("project:/") + surrogate + QStringLiteral(".pa"), &error);
    EXPECT_TRUE(resolved.isEmpty());
    EXPECT_EQ(error, QStringLiteral("path.non_unicode"));
}

TEST(PathHostTest, StandardRootInjectionCreatesMissingDirectories) {
    QTemporaryDir scratch;
    ASSERT_TRUE(scratch.isValid());
    const QString nested = scratch.path() + QStringLiteral("/laid/out/cache");
    QString error;
    auto host = panta::bridge::PathHost::createWithStandardRoots(
        {panta::bridge::StandardRoot{panta::ffi::PathRootKind::Cache, QDir(nested).absolutePath(),
                                     QStringLiteral("cache")}},
        &error);
    ASSERT_NE(host, nullptr) << error.toStdString();
    EXPECT_TRUE(QDir(nested).exists());
    // 写探针由临时文件承担,注入完成后不残留。
    const auto entries = QDir(nested).entryList(QDir::Files | QDir::NoDotAndDotDot);
    EXPECT_TRUE(entries.isEmpty()) << entries.join(QStringLiteral(",")).toStdString();
}

TEST(PathHostTest, StandardRootInjectionRejectsEmptyAndRelativeEntries) {
    QString error;
    auto host = panta::bridge::PathHost::createWithStandardRoots(
        {panta::bridge::StandardRoot{panta::ffi::PathRootKind::Cache, QString(),
                                     QStringLiteral("cache")}},
        &error);
    EXPECT_EQ(host, nullptr);
    EXPECT_TRUE(error.startsWith(QStringLiteral("path.standard_dir_unavailable")))
        << error.toStdString();

    error.clear();
    host = panta::bridge::PathHost::createWithStandardRoots(
        {panta::bridge::StandardRoot{panta::ffi::PathRootKind::AppData,
                                     QStringLiteral("relative/dir"), QStringLiteral("app-data")}},
        &error);
    EXPECT_EQ(host, nullptr);
    EXPECT_TRUE(error.startsWith(QStringLiteral("path.standard_dir_unavailable")))
        << error.toStdString();
}

#ifdef Q_OS_UNIX
TEST(PathHostTest, StandardRootInjectionRejectsUnwritableDirectory) {
    QTemporaryDir scratch;
    ASSERT_TRUE(scratch.isValid());
    const QString locked = scratch.path() + QStringLiteral("/locked");
    ASSERT_TRUE(QDir().mkpath(locked));
    // 只读目录(含执行位,可进入不可写):写探针必须失败并给出明确错误。
    QFile permissions(locked);
    ASSERT_TRUE(permissions.setPermissions(QFileDevice::ReadOwner | QFileDevice::ExeOwner |
                                           QFileDevice::ReadGroup | QFileDevice::ExeGroup |
                                           QFileDevice::ReadOther | QFileDevice::ExeOther));

    QString error;
    auto host = panta::bridge::PathHost::createWithStandardRoots(
        {panta::bridge::StandardRoot{panta::ffi::PathRootKind::Cache, QDir(locked).absolutePath(),
                                     QStringLiteral("cache")}},
        &error);
    EXPECT_EQ(host, nullptr);
    EXPECT_TRUE(error.startsWith(QStringLiteral("path.standard_dir_unwritable")))
        << error.toStdString();

    // 恢复权限以便 QTemporaryDir 清理。
    (void)permissions.setPermissions(QFileDevice::ReadOwner | QFileDevice::WriteOwner |
                                     QFileDevice::ExeOwner);
}
#endif

#ifdef Q_OS_UNIX
TEST(PathHostTest, SymlinkEscapeOutsideRootIsRejected) {
    IsolatedStandardRoots standardRoots;
    ASSERT_TRUE(standardRoots.isValid());
    QTemporaryDir projectRoot;
    QTemporaryDir outside;
    ASSERT_TRUE(projectRoot.isValid());
    ASSERT_TRUE(outside.isValid());
    QFile secret(outside.path() + QStringLiteral("/secret.pa"));
    ASSERT_TRUE(secret.open(QIODevice::WriteOnly));
    ASSERT_EQ(secret.write("x"), qint64(1));
    secret.close();

    const QString link = projectRoot.path() + QStringLiteral("/leak");
    ASSERT_TRUE(QFile::link(outside.path(), link));

    QString error;
    auto host = standardRoots.create(&error);
    ASSERT_NE(host, nullptr) << error.toStdString();
    ASSERT_TRUE(host->setProjectRoot(QDir(projectRoot.path()).absolutePath(), &error))
        << error.toStdString();

    const QString leaked = host->resolveExisting(QStringLiteral("project:/leak/secret.pa"), &error);
    EXPECT_TRUE(leaked.isEmpty());
    EXPECT_TRUE(error.startsWith(QStringLiteral("path.not_contained"))) << error.toStdString();
}
#endif

int main(int argc, char** argv) {
    QCoreApplication app(argc, argv);
    ::testing::InitGoogleTest(&argc, argv);
    return RUN_ALL_TESTS();
}
