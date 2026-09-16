// QML 原子组件的默认 token 与显式覆盖测试（029）。
#include <QColor>
#include <QGuiApplication>
#include <QQmlComponent>
#include <QQmlEngine>
#include <QtTest/QtTest>

namespace {

QObject* create_component(QQmlEngine& engine, const QString& path, QObject& owner) {
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

}  // namespace

class ThemeComponentTest final : public QObject {
    Q_OBJECT

private slots:
    void defaults_use_theme() {
        QQmlEngine engine;
        QObject owner;
        QObject* label = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedLabel.qml"), owner);
        QVERIFY(label != nullptr);
        QCOMPARE(label->property("textColor").value<QColor>(), QColor(QStringLiteral("#e8e8e8")));
        QCOMPARE(label->property("textSize").toInt(), 14);
    }

    void explicit_values_override_theme() {
        QQmlEngine engine;
        QObject owner;
        QObject* button = create_component(
            engine, QStringLiteral("qrc:/qt/qml/Panta/Shell/Components/Atoms/ThemedButton.qml"), owner);
        QVERIFY(button != nullptr);
        button->setProperty("controlHeight", 48);
        button->setProperty("contentPadding", 20);
        QCOMPARE(button->property("controlHeight").toInt(), 48);
        QCOMPARE(button->property("contentPadding").toInt(), 20);
        QCOMPARE(button->property("implicitHeight").toInt(), 48);
        QCOMPARE(button->property("leftPadding").toInt(), 20);
        QCOMPARE(button->property("rightPadding").toInt(), 20);
    }
};

QTEST_MAIN(ThemeComponentTest)
#include "theme_component_test.moc"
