#include "icon_provider.hpp"

#include <QColor>
#include <QImage>
#include <QPainter>
#include <QQmlEngine>
#include <QQuickImageProvider>
#include <QRegularExpression>
#include <QSize>
#include <QString>
#include <QSvgRenderer>
#include <qnamespace.h>

namespace panta {
namespace {

// SVG 的 currentColor 不继承 QML 的颜色；先渲染 alpha，再用调用方颜色着色。
// 每次请求的 renderer/painter 均为局部对象，不共享可变绘图状态。
class IconProvider final : public QQuickImageProvider {
  public:
    IconProvider() : QQuickImageProvider(QQuickImageProvider::Image) {}

    QImage requestImage(const QString& id, QSize* size, const QSize& requested_size) override {
        static const QRegularExpression valid_id(
            QStringLiteral("^([a-z][a-z0-9-]*)/([0-9a-fA-F]{8})$"));
        const auto match = valid_id.match(id);
        if (!match.hasMatch()) {
            return {};
        }
        QSvgRenderer renderer(
            QStringLiteral(":/qt/qml/Panta/Shell/icons/%1.svg").arg(match.captured(1)));
        if (!renderer.isValid()) {
            return {};
        }
        if (size != nullptr) {
            *size = renderer.defaultSize();
        }
        const QSize pixels = requested_size.isValid() && !requested_size.isEmpty()
                                 ? requested_size
                                 : renderer.defaultSize();
        QImage image(pixels, QImage::Format_ARGB32_Premultiplied);
        image.fill(Qt::transparent);
        QPainter painter(&image);
        renderer.render(&painter);
        painter.setCompositionMode(QPainter::CompositionMode_SourceIn);
        painter.fillRect(image.rect(), QColor(QStringLiteral("#") + match.captured(2)));
        return image;
    }
};

} // namespace

void install_icon_provider(QQmlEngine& engine) {
    engine.addImageProvider(QStringLiteral("panta-icons"), new IconProvider);
}
} // namespace panta
