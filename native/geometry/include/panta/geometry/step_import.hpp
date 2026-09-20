/// STEP 导入冒烟的公共几何契约（任务 009，architecture/geometry.md 导入流程
/// 的最小可验证边界）。公共头只含自有类型与标准库；OCCT 类型限定在适配器
/// 内部（standards/occt.md）。坐标与包围盒一律毫米：源文件单位经读取器
/// 换算，摘要同时记录源单位与其到毫米的系数。
///
/// 冒烟采用 STEPControl 基础路径：装配结构、名称、颜色等 XDE 附属元数据
/// 不保留；完整导入产品流程另建业务任务。
#pragma once

#include <cstdint>
#include <filesystem>
#include <string>

namespace panta::geometry {

/// STEP 文件声明的长度单位；kUnknown 表示文件未声明或名称未被适配器识别。
/// 坐标无论如何都由读取器转换为毫米，kUnknown 只影响换算元数据。
/// 显式基础类型固定取值域宽度，避免公共枚举的 ABI 尺寸随实现漂移。
enum class LengthUnit : std::uint8_t {
    kUnknown,
    kMillimetre,
    kCentimetre,
    kMetre,
    kMicron,
    kInch,
    kFoot,
    kMile,
    kMil,
    kMicroinch,
};

/// 导入结果分类；kNone 表示成功。失败时 summary 仅用于诊断，不得当作
/// 有效导入结果提交工程。
enum class StepImportStatus : std::uint8_t {
    kNone,
    /// 文件不可读或不是可识别的 STEP 输入。
    kFileUnreadable,
    /// 文件已读取但无法解析出可用的模型内容。
    kParseFailed,
    /// 模型为空或没有可转移的根实体。
    kEmptyModel,
    /// 存在未转移成功的候选根；summary 记录已转移部分。
    kTransferIncomplete,
    /// 转移完成但结果不含任何几何实体。
    kEmptyResult,
    /// OCCT 异常或其他内部失败；detail 携带上下文。
    kInternalFailure,
};

/// 一次成功导入的自有摘要：单位、包围盒与去重拓扑计数。
struct StepImportSummary {
    /// 源文件长度单位。
    LengthUnit source_unit = LengthUnit::kUnknown;
    /// 源单位到毫米的换算系数（如 Inch=25.4）；kUnknown 时为 0，坐标仍为
    /// 读取器转换后的毫米值。
    double source_unit_to_mm = 0.0;
    /// 是否得到非空几何；包围盒仅在该值为 true 时有效。
    bool has_geometry = false;
    /// 包围盒角点（毫米）。这是含形状公差的保守包络（Bnd_Box gap，典型
    /// Precision::Confusion ≈ 1e-7 mm），不等于名义几何角点。
    double bbox_min[3] = {};
    double bbox_max[3] = {};
    /// 去重拓扑计数：共享拓扑只计一次。
    std::uint32_t solids = 0;
    std::uint32_t faces = 0;
    std::uint32_t edges = 0;
    /// 候选根总数与实际转移成功数；两者相等表示完整转换。
    std::uint32_t roots_total = 0;
    std::uint32_t roots_transferred = 0;
};

struct StepImportResult {
    StepImportStatus status = StepImportStatus::kNone;
    /// 面向日志的英文诊断上下文（OCCT 消息原文或失败阶段），不直接展示给
    /// 最终用户；用户可读文案由后续业务任务与 022 的语言字典承接。
    std::string detail;
    StepImportSummary summary;
};

/// 同步读取 STEP 文件并生成摘要。
///
/// 适配器内部对全部 OCCT 访问串行化（Interface_Static 与 translator 参数
/// 为进程级状态，standards/occt.md），调用方无须额外加锁；当前不提供中途
/// 取消，超小样例外耗时亚秒级，异步任务与取消边界归后续业务任务。
/// 失败返回结构化 status 并携带诊断，不抛异常、不终止进程；调用返回后
/// 适配器不保留任何形状或读取器状态，重复导入互不影响。
StepImportResult import_step_summary(const std::filesystem::path& file);

} // namespace panta::geometry
