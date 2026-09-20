/// STEPControl 基础路径的 OCCT 适配器（任务 009）。本目录是仓库中唯一
/// 允许包含 OCCT 头的位置；装配/名称/颜色等 XDE 附属元数据不保留，取舍
/// 见公共头与任务记录。坐标在 OCCT 模型空间内一律毫米。
#include <BRepBndLib.hxx>
#include <Bnd_Box.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <NCollection_Sequence.hxx>
#include <STEPControl_Reader.hxx>
#include <Standard_Failure.hxx>
#include <TCollection_AsciiString.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS_Shape.hxx>
#include <cstdint>
#include <exception>
#include <filesystem>
#include <mutex>
#include <panta/geometry/step_import.hpp>
#include <string_view>

namespace panta::geometry {
namespace {

/// Interface_Static 与 translator 参数是进程级状态（standards/occt.md）：
/// 在并发隔离逐版本验证之前保守串行，全部导入共用一把进程级锁。
std::mutex& occt_import_mutex() {
    static std::mutex mutex;
    return mutex;
}

/// 各长度单位到毫米的换算系数；kUnknown 记 0 表示无法给出换算元数据
/// （坐标仍由读取器转换为毫米，只是来源单位未知）。
double unit_to_mm(LengthUnit unit) {
    switch (unit) {
    case LengthUnit::kMillimetre:
        return 1.0;
    case LengthUnit::kCentimetre:
        return 10.0;
    case LengthUnit::kMetre:
        return 1000.0;
    case LengthUnit::kMicron:
        return 0.001;
    case LengthUnit::kInch:
        return 25.4;
    case LengthUnit::kFoot:
        return 304.8;
    case LengthUnit::kMile:
        return 1609344.0;
    case LengthUnit::kMil:
        return 0.0254;
    case LengthUnit::kMicroinch:
        return 0.0000254;
    case LengthUnit::kUnknown:
        return 0.0;
    }
    return 0.0;
}

/// FileUnits 输出的 STEP 长度单位名（小写 ISO 拼写，实测见任务 009）。
LengthUnit length_unit_from_name(std::string_view name) {
    if (name == "millimetre")
        return LengthUnit::kMillimetre;
    if (name == "centimetre")
        return LengthUnit::kCentimetre;
    if (name == "metre")
        return LengthUnit::kMetre;
    if (name == "micron")
        return LengthUnit::kMicron;
    if (name == "inch")
        return LengthUnit::kInch;
    if (name == "foot")
        return LengthUnit::kFoot;
    if (name == "mile")
        return LengthUnit::kMile;
    if (name == "mil")
        return LengthUnit::kMil;
    if (name == "microinch")
        return LengthUnit::kMicroinch;
    return LengthUnit::kUnknown;
}

/// 从 FileUnits 的长度单位名序列取源单位：全部条目一致且可识别才返回该
/// 单位；序列为空、名称未识别或混合单位均按 kUnknown 处理。
LengthUnit source_length_unit(const NCollection_Sequence<TCollection_AsciiString>& names) {
    if (names.IsEmpty())
        return LengthUnit::kUnknown;
    LengthUnit unit = length_unit_from_name(names.First().ToCString());
    if (unit == LengthUnit::kUnknown)
        return LengthUnit::kUnknown;
    for (const auto& name : names) {
        if (length_unit_from_name(name.ToCString()) != unit)
            return LengthUnit::kUnknown;
    }
    return unit;
}

/// 汇总去重拓扑计数与毫米包围盒；shape 为空时仅记录 has_geometry=false。
void fill_geometry_summary(const TopoDS_Shape& shape, StepImportSummary& summary) {
    if (shape.IsNull())
        return;
    summary.has_geometry = true;

    TopTools_IndexedMapOfShape solids;
    TopTools_IndexedMapOfShape faces;
    TopTools_IndexedMapOfShape edges;
    TopExp::MapShapes(shape, TopAbs_SOLID, solids);
    TopExp::MapShapes(shape, TopAbs_FACE, faces);
    TopExp::MapShapes(shape, TopAbs_EDGE, edges);
    summary.solids = static_cast<std::uint32_t>(solids.Extent());
    summary.faces = static_cast<std::uint32_t>(faces.Extent());
    summary.edges = static_cast<std::uint32_t>(edges.Extent());

    Bnd_Box box;
    BRepBndLib::Add(shape, box);
    if (!box.IsVoid()) {
        box.Get(summary.bbox_min[0], summary.bbox_min[1], summary.bbox_min[2], summary.bbox_max[0],
                summary.bbox_max[1], summary.bbox_max[2]);
    }
}

} // namespace

StepImportResult import_step_summary(const std::filesystem::path& file) {
    StepImportResult result;
    const std::lock_guard<std::mutex> lock(occt_import_mutex());
    try {
        // 读取器逐次新建：失败路径不残留会话状态，重复导入互不影响。
        STEPControl_Reader reader;
        switch (const IFSelect_ReturnStatus read = reader.ReadFile(file.string().c_str())) {
        case IFSelect_RetDone:
            break;
        case IFSelect_RetVoid:
            result.status = StepImportStatus::kEmptyModel;
            result.detail = "ReadFile completed without producing a model";
            return result;
        case IFSelect_RetError:
            result.status = StepImportStatus::kFileUnreadable;
            result.detail = "ReadFile rejected the input";
            return result;
        case IFSelect_RetFail:
            result.status = StepImportStatus::kParseFailed;
            result.detail = "ReadFile ran but failed to parse the input";
            return result;
        }

        // 源单位来自文件内的长度单位名；FileUnits 只在读取成功的模型上
        // 安全（损坏文件上实测会段错误），因此置于 RetDone 分支之后。
        NCollection_Sequence<TCollection_AsciiString> length_names;
        NCollection_Sequence<TCollection_AsciiString> angle_names;
        NCollection_Sequence<TCollection_AsciiString> solid_angle_names;
        reader.FileUnits(length_names, angle_names, solid_angle_names);
        result.summary.source_unit = source_length_unit(length_names);
        result.summary.source_unit_to_mm = unit_to_mm(result.summary.source_unit);

        result.summary.roots_total = static_cast<std::uint32_t>(reader.NbRootsForTransfer());
        result.summary.roots_transferred = static_cast<std::uint32_t>(reader.TransferRoots());
        if (result.summary.roots_total == 0 || result.summary.roots_transferred == 0) {
            result.status = StepImportStatus::kEmptyModel;
            result.detail = "no transferable root produced a shape";
            return result;
        }
        if (result.summary.roots_transferred < result.summary.roots_total) {
            result.status = StepImportStatus::kTransferIncomplete;
            result.detail = "some transferable roots failed to convert";
        }

        const TopoDS_Shape shape = reader.OneShape();
        if (shape.IsNull()) {
            result.status = StepImportStatus::kEmptyResult;
            result.detail = "transferred shapes collapsed to a null shape";
            return result;
        }
        fill_geometry_summary(shape, result.summary);
        return result;
    } catch (const Standard_Failure& failure) {
        result.status = StepImportStatus::kInternalFailure;
        result.detail = failure.GetMessageString() != nullptr ? failure.GetMessageString()
                                                              : "Standard_Failure without message";
        return result;
    } catch (const std::exception& error) {
        result.status = StepImportStatus::kInternalFailure;
        result.detail = error.what();
        return result;
    }
}

} // namespace panta::geometry
