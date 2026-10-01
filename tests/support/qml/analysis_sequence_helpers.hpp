// 测试和基准从正式 ViewModel 读取 Rust 目录，避免维护另一套分析序列清单。
#pragma once
#include <QVariantList>
#if defined(PANTA_TEST_WITH_BRIDGE)
#include <project_view_model.hpp>
#endif

inline QVariantList analysis_sequence_catalog() {
#if defined(PANTA_TEST_WITH_BRIDGE)
    const panta::bridge::ProjectViewModel model;
    return model.analysisSequences();
#else
    return {};
#endif
}
