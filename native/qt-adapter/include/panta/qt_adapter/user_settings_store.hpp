/// QSettings 存储 DTO；只包含 Qt 持久化层可读写的值，不承载 schema 校验。
#pragma once

#include <QString>
#include <QStringList>
#include <QtCore/qsettings.h>

namespace panta::qt_adapter {

struct UserSettingsRecord {
    quint32 schemaVersion = 0;
    QString themeId;
    QString language;
    quint32 windowWidth = 0;
    quint32 windowHeight = 0;
    bool windowMaximized = false;
    QStringList recentProjectPaths;
    friend bool operator==(const UserSettingsRecord&, const UserSettingsRecord&) = default;
};

/// 使用 QSettings 原子同步完整快照；测试可注入 IniFormat 文件或隔离的原生设置标识。
class UserSettingsStore final {
  public:
    UserSettingsStore();
    UserSettingsStore(QString fileName, QSettings::Format format);
    UserSettingsStore(QString organizationName, QString applicationName);

    [[nodiscard]] bool load(UserSettingsRecord* record, bool* found, QString* error) const;
    [[nodiscard]] bool save(const UserSettingsRecord& record, QString* error) const;

  private:
    [[nodiscard]] QSettings createSettings() const;

    QString m_fileName;
    QString m_organizationName;
    QString m_applicationName;
    QSettings::Format m_format = QSettings::NativeFormat;
};

} // namespace panta::qt_adapter
