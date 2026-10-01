#pragma once

#include <QAnimationDriver>
#include <QByteArray>
#include <QElapsedTimer>
#include <QQuickGraphicsConfiguration>
#include <QQuickItem>
#include <QQuickRenderControl>
#include <QQuickRenderTarget>
#include <QQuickWindow>
#include <QSize>
#include <QString>
#include <functional>
#include <memory>
#include <rhi/qrhi.h>

namespace panta::test {

/// 离屏场景按 60Hz 逻辑帧推进，避免动画速度随渲染速度变化。
class FrameAnimationDriver final : public QAnimationDriver {
  public:
    void advance() override {
        ++m_frame;
        advanceAnimation();
    }
    [[nodiscard]] auto elapsed() const -> qint64 override { return m_frame * 1000 / 60; }

  private:
    qint64 m_frame = 0;
};

struct RenderTiming {
    double completionMs;
    double gpuMs;
};

/// Qt Quick 测试侧硬件离屏目标；场景与采样策略由调用方管理。
class OffscreenQuickRenderer final {
  public:
    [[nodiscard]] auto window() -> QQuickWindow& { return m_window; }

    [[nodiscard]] auto initialize(const QSize& logicalSize, qreal devicePixelRatio) -> QString {
        m_window.resize(logicalSize);
        // 内容尺寸直接采用目标逻辑尺寸，避免布局依赖隐藏窗口的平台 resize 事件。
        m_window.contentItem()->setSize(logicalSize);
        QQuickGraphicsConfiguration config;
        config.setTimestamps(true);
        m_window.setGraphicsConfiguration(config);
        if (!m_control.initialize()) {
            return QStringLiteral("Cannot initialize offscreen hardware renderer");
        }
        QRhi* rhi = m_control.rhi();
        if (rhi == nullptr || rhi->backend() == QRhi::Null) {
            return QStringLiteral("A hardware RHI is required");
        }
        const auto deviceType = rhi->driverInfo().deviceType;
        if (deviceType != QRhiDriverInfo::IntegratedDevice &&
            deviceType != QRhiDriverInfo::DiscreteDevice &&
            deviceType != QRhiDriverInfo::ExternalDevice) {
            return QStringLiteral("An identifiable hardware GPU is required");
        }
        if (!rhi->isFeatureSupported(QRhi::Timestamps)) {
            return QStringLiteral("GPU timestamps are unsupported by this device");
        }
        const QSize pixelSize = logicalSize * devicePixelRatio;
        m_texture.reset(
            rhi->newTexture(QRhiTexture::RGBA8, pixelSize, 1, QRhiTexture::RenderTarget));
        if (!m_texture->create()) {
            return QStringLiteral("Cannot create offscreen color texture");
        }
        m_depthStencil.reset(rhi->newRenderBuffer(QRhiRenderBuffer::DepthStencil, pixelSize, 1));
        if (!m_depthStencil->create()) {
            return QStringLiteral("Cannot create offscreen depth-stencil buffer");
        }
        QRhiTextureRenderTargetDescription description{QRhiColorAttachment(m_texture.get())};
        description.setDepthStencilBuffer(m_depthStencil.get());
        m_target.reset(rhi->newTextureRenderTarget(description));
        m_pass.reset(m_target->newCompatibleRenderPassDescriptor());
        m_target->setRenderPassDescriptor(m_pass.get());
        if (!m_target->create()) {
            return QStringLiteral("Cannot create offscreen render target");
        }
        auto target = QQuickRenderTarget::fromRhiRenderTarget(m_target.get());
        target.setDevicePixelRatio(devicePixelRatio);
        m_window.setRenderTarget(target);
        return {};
    }

    [[nodiscard]] auto deviceName() const -> QByteArray {
        return m_control.rhi()->driverInfo().deviceName;
    }

    [[nodiscard]] auto render(const std::function<void()>& update = {}) -> RenderTiming {
        QElapsedTimer timer;
        timer.start();
        if (update) {
            update();
        }
        m_control.polishItems();
        m_control.beginFrame();
        if (!m_control.rhi()->isRecordingFrame()) {
            return {static_cast<double>(timer.nsecsElapsed()) / 1.0e6, 0};
        }
        m_control.sync();
        m_control.render();
        // endFrame 等待离屏帧完成，返回后的时间戳对应本帧。
        m_control.endFrame();
        auto* commands = m_control.commandBuffer();
        return {static_cast<double>(timer.nsecsElapsed()) / 1.0e6,
                commands == nullptr ? 0 : commands->lastCompletedGpuTime() * 1000.0};
    }

    [[nodiscard]] auto deviceLost() const -> bool { return m_control.rhi()->isDeviceLost(); }

    ~OffscreenQuickRenderer() {
        m_window.setRenderTarget({});
        m_control.invalidate();
        m_target.reset();
        m_pass.reset();
        m_depthStencil.reset();
        m_texture.reset();
    }

  private:
    QQuickRenderControl m_control;
    QQuickWindow m_window{&m_control};
    std::unique_ptr<QRhiTexture> m_texture;
    std::unique_ptr<QRhiRenderBuffer> m_depthStencil;
    std::unique_ptr<QRhiRenderPassDescriptor> m_pass;
    std::unique_ptr<QRhiTextureRenderTarget> m_target;
};

} // namespace panta::test
