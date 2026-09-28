/// 管理当前应用翻译目录的 Qt 加载与生命周期。
#pragma once

#include <QString>
#include <QTranslator>
#include <memory>

namespace panta::qt_adapter {

/// 在应用线程管理一个 Qt QM 目录；对象析构时卸载其拥有的 translator。
class TranslationCatalog final {
  public:
    TranslationCatalog() = default;
    ~TranslationCatalog();

    TranslationCatalog(const TranslationCatalog&) = delete;
    TranslationCatalog& operator=(const TranslationCatalog&) = delete;

    /// 从资源路径加载并替换目录；失败时保留旧目录，error 接收稳定错误码。
    [[nodiscard]] bool install(const QString& resourcePath, QString* error);
    /// 卸载当前目录；无目录时不产生变化。
    void clear();

  private:
    std::unique_ptr<QTranslator> m_translator;
};

} // namespace panta::qt_adapter
