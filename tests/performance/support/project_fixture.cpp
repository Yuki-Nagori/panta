// 通过正式工程服务生成性能套件专用的 STL 工程；输入和产物只写入调用方的新目录。
#include <QCoreApplication>
#include <QDir>
#include <QString>
#include <QStringList>
#include <array>
#include <bit>
#include <cstdint>
#include <exception>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <project_view_model.hpp>
#include <stdexcept>

namespace {
void write_u32(std::ofstream& stream, std::uint32_t value) {
    const std::array<char, 4> bytes{static_cast<char>(value), static_cast<char>(value >> 8),
                                    static_cast<char>(value >> 16), static_cast<char>(value >> 24)};
    stream.write(bytes.data(), bytes.size());
}
void write_stl(const QString& path, std::uint32_t triangles) {
    std::ofstream stream(std::filesystem::path(path.toStdU16String()), std::ios::binary);
    stream.exceptions(std::ios::badbit | std::ios::failbit);
    const std::array<char, 80> header{};
    stream.write(header.data(), header.size());
    write_u32(stream, triangles);
    for (std::uint32_t index = 0; index < triangles; ++index) {
        const float x = static_cast<float>(index % 100);
        const auto row = index / 100;
        const float y = static_cast<float>(row);
        for (float value :
             std::array<float, 12>{0, 0, 1, x, y, 0, x + 0.5F, y, 0, x, y + 0.5F, 0}) {
            write_u32(stream, std::bit_cast<std::uint32_t>(value));
        }
        stream.put(0);
        stream.put(0);
    }
}
} // namespace
int main(int argc, char* argv[]) try {
    QCoreApplication application(argc, argv);
    const auto arguments = application.arguments();
    if (arguments.size() != 2) {
        throw std::invalid_argument("usage: panta_performance_fixture <new-directory>");
    }
    const QDir directory(arguments[1]);
    if (directory.exists()) {
        throw std::invalid_argument("Fixture directory must not exist");
    }
    if (!QDir().mkpath(directory.path())) {
        throw std::runtime_error("Cannot create fixture directory");
    }
    panta::bridge::ProjectViewModel project;
    if (!project.createProject(QStringLiteral("Benchmark"), directory.absolutePath())) {
        throw std::runtime_error(project.error().toStdString());
    }
    for (std::uint32_t triangles : {10'000U, 20'000U}) {
        const auto path = directory.filePath(QStringLiteral("triangles_%1.stl").arg(triangles));
        write_stl(path, triangles);
        if (!project.importStl(path, QStringLiteral("solid-3d"), QStringLiteral("millimeters"),
                               false)) {
            throw std::runtime_error(project.error().toStdString());
        }
    }
    if (!project.saveProject()) {
        throw std::runtime_error(project.error().toStdString());
    }
    std::cout << "fixture=" << project.currentPath().toStdString()
              << " source=synthetic_planar_triangles units=millimeters triangles=10000,20000\n";
    return 0;
} catch (const std::exception& error) {
    std::cerr << "fixture: " << error.what() << '\n';
    return 1;
}
