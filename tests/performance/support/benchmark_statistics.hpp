#pragma once

#include <algorithm>
#include <cmath>
#include <cstddef>
#include <optional>
#include <span>
#include <vector>

namespace panta::test {

struct TimingSummary {
    double p50;
    double p95;
};

/// 最近秩分位数；空、负值和非有限计时拒绝汇总，不伪造零耗时结果。
[[nodiscard]] inline auto summarize_timings(std::span<const double> samples)
    -> std::optional<TimingSummary> {
    if (samples.empty() || std::any_of(samples.begin(), samples.end(), [](double value) {
            return !std::isfinite(value) || value < 0;
        })) {
        return std::nullopt;
    }
    std::vector<double> sorted(samples.begin(), samples.end());
    std::sort(sorted.begin(), sorted.end());
    const auto at = [&](double percentile) {
        const auto rank =
            static_cast<std::size_t>(std::ceil(percentile * static_cast<double>(sorted.size())));
        return sorted[rank - 1];
    };
    return TimingSummary{at(0.50), at(0.95)};
}

} // namespace panta::test
