/// native 入口先导骨架：打印 native 层版本；参数由 launcher 转发（任务 004）。
/// 任务 005 将以 Qt 应用替换本实现，入口目录与可执行名保持不变。
#include <panta/foundation/version.hpp>

#include <cstdio>
#include <string_view>

namespace {

/// 用法错误退出码，与 launcher 约定一致（BSD sysexits.h 的 EX_USAGE）。
constexpr int kExitUsage = 64;

void print_version() {
    const panta::foundation::Version version = panta::foundation::native_version();
    std::printf("panta-native %d.%d.%d\n", version.major, version.minor, version.patch);
}

}  // namespace

int main(int argc, char* argv[]) {
    for (int index = 1; index < argc; ++index) {
        const std::string_view argument = argv[index];
        if (argument == "--version") {
            print_version();
            return 0;
        }
        std::fprintf(stderr, "panta-native: 未知参数 '%.*s'（用法：panta-native [--version]）\n",
                     static_cast<int>(argument.size()), argument.data());
        return kExitUsage;
    }
    print_version();
    return 0;
}
