#pragma once

#include <atomic>
#include <memory>
#include <stdexcept>
#include <vtkCallbackCommand.h>
#include <vtkCommand.h>
#include <vtkNew.h>
#include <vtkOutputWindow.h>
#include <vtkSmartPointer.h>

namespace panta::test {

/// 附加观察 VTK 错误，不替换进程的输出窗口；回调可能来自设备线程。
class VtkErrorCapture final {
  public:
    VtkErrorCapture() : m_output(vtkOutputWindow::GetInstance()) {
        // 标志由 VTK 命令持有，避免晚到回调借用捕获器成员。
        auto failed = std::make_unique<std::atomic_bool>(false);
        m_callback->SetClientDataDeleteCallback(
            [](void* state) { delete static_cast<std::atomic_bool*>(state); });
        m_callback->SetClientData(failed.release());
        m_callback->SetCallback([](vtkObject*, unsigned long, void* state, void*) {
            static_cast<std::atomic_bool*>(state)->store(true);
        });
        m_tag = m_output->AddObserver(vtkCommand::ErrorEvent, m_callback);
    }
    ~VtkErrorCapture() { m_output->RemoveObserver(m_tag); }
    VtkErrorCapture(const VtkErrorCapture&) = delete;
    auto operator=(const VtkErrorCapture&) -> VtkErrorCapture& = delete;
    void check() const {
        if (static_cast<std::atomic_bool*>(m_callback->GetClientData())->load()) {
            throw std::runtime_error("VTK reported a rendering error; see diagnostics");
        }
    }

  private:
    vtkSmartPointer<vtkOutputWindow> m_output;
    vtkNew<vtkCallbackCommand> m_callback;
    unsigned long m_tag = 0;
};

} // namespace panta::test
