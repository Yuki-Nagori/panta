// 多个 QML 组件测试共享的轻量构造与源码检查辅助函数。
#pragma once

#include <QCoreApplication>
#include <QObject>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QRegularExpression>
#include <QString>
#include <QTranslator>
#include <QUrl>
#include <QVariantList>
#include <QVariantMap>
#include <QVector>
#include <QtTest/qtest.h>

namespace panta::test::qml {

inline QVector<QString> qml_object_blocks(const QString& text, const char* type_name) {
    QVector<QString> blocks;
    const QRegularExpression type_start(
        QStringLiteral("\\b") + QRegularExpression::escape(QString::fromLatin1(type_name)) +
            QStringLiteral("\\s*\\{([^{}]*)\\}"),
        QRegularExpression::DotMatchesEverythingOption);
    auto match = type_start.globalMatch(text);
    while (match.hasNext()) {
        blocks.push_back(match.next().captured(1));
    }
    return blocks;
}

inline QString qml_binding(const QString& block, const char* property_name) {
    const QRegularExpression binding(
        QStringLiteral("(?:^|\\n)[\\t ]*") +
        QRegularExpression::escape(QString::fromLatin1(property_name)) +
        QStringLiteral("\\s*:\\s*([^\\n]+)"));
    const auto match = binding.match(block);
    if (!match.hasMatch()) {
        return {};
    }
    QString value = match.captured(1).trimmed();
    if (value.endsWith(QLatin1Char(','))) {
        value.chop(1);
    }
    return value.trimmed();
}

inline QString qml_icon_literal(const QString& expression) {
    static const QRegularExpression literal(QStringLiteral("^[\\\"']([^\\\"']*)[\\\"']$"));
    const auto match = literal.match(expression);
    return match.hasMatch() ? match.captured(1) : QString{};
}

inline QVector<QString> qml_simple_object_blocks(const QString& text) {
    QVector<QString> blocks;
    static const QRegularExpression object_start(QStringLiteral("\\{([^{}]*)\\}"),
                                                 QRegularExpression::DotMatchesEverythingOption);
    auto match = object_start.globalMatch(text);
    while (match.hasNext()) {
        blocks.push_back(match.next().captured(1));
    }
    return blocks;
}

inline QObject* create_component(QQmlEngine& engine, const QString& path, QObject& owner) {
    QQmlComponent component(&engine, QUrl(path));
    if (!component.isReady()) {
        QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
        return nullptr;
    }
    QObject* object = component.create();
    if (object == nullptr) {
        QTest::qFail(qPrintable(component.errorString()), __FILE__, __LINE__);
        return nullptr;
    }
    object->setParent(&owner);
    return object;
}

inline QVariantMap output_run(const QString& context, const QString& id, const QString& text) {
    return {
        {QStringLiteral("contextId"), context},
        {QStringLiteral("id"), id},
        {QStringLiteral("categories"),
         QVariantList{QVariantMap{{QStringLiteral("id"), QStringLiteral("analysis")},
                                  {QStringLiteral("sourceText"), QStringLiteral("Analysis Log")},
                                  {QStringLiteral("text"), text}}}},
        {QStringLiteral("resultGroups"), QVariantList{}}};
}

class WelcomeTranslator final : public QTranslator {
  public:
    QString translate(const char* context, const char* source_text, const char*,
                      int) const override {
        if (QString::fromUtf8(context) == QStringLiteral("ProjectViewModel") &&
            QString::fromUtf8(source_text) == QStringLiteral("Welcome")) {
            return QStringLiteral("欢迎");
        }
        return {};
    }
};

} // namespace panta::test::qml
