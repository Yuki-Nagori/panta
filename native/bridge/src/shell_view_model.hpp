/// Shell 界面的最小 ViewModel：承载可观察状态与命令，供 QML 绑定。
///
/// 契约（ui-and-bridge.md / qt.md）：
/// - 只暴露可观察属性与命令，不做文件解析或数值计算；
/// - 属性写入值未变化时不发 NOTIFY 信号（qt.md：值未改变不重复通知）；
/// - 对象归属 GUI 线程，当前无可重入后台任务。
#pragma once

#include <QObject>
#include <QtQml/qqmlregistration.h>

namespace panta::bridge {

class ShellViewModel : public QObject {
    Q_OBJECT
    QML_ELEMENT
    /// 界面标题/说明文字；重复写入同一值不触发 captionChanged。
    Q_PROPERTY(QString caption READ caption WRITE setCaption NOTIFY captionChanged)
    /// 已推进的修订计数；由命令 tick() 推进，只读属性。
    Q_PROPERTY(int count READ count NOTIFY countChanged)
    /// 最近一次面向用户的错误摘要；空串表示无错误。
    Q_PROPERTY(QString error READ error WRITE setError NOTIFY errorChanged)

  public:
    explicit ShellViewModel(QObject* parent = nullptr);

    [[nodiscard]] auto caption() const -> const QString&;
    void setCaption(const QString& value);

    [[nodiscard]] auto count() const -> int;

    [[nodiscard]] auto error() const -> const QString&;
    void setError(const QString& message);

    /// 命令：推进修订计数并同步刷新说明文字。
    Q_INVOKABLE void tick();

  signals:
    void captionChanged();
    void countChanged();
    void errorChanged();

  private:
    QString m_caption;
    int m_count = 0;
    QString m_error;
};

} // namespace panta::bridge
