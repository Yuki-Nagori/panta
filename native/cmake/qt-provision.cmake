# Qt 预编译产物供给（任务 005；维护者决策：不源码构建，直接使用官方预编译包）。
#
# 来源：Qt 在线安装器仓库 qtsdkrepository（与安装器同源、无需账号）。
# SHA256 于 2026-09-16 下载实测记录于此，下载与缓存均强校验；升级 Qt 时
# 同步更新三平台的 URL/SHA 并回写 ai-docs/standards/dependency-acquisition.md。
# 解包使用 `cmake -E tar`（内建 libarchive 支持 7z，三平台零额外工具）。
# 供给结果缓存在 QT_PROVISION_DIR（默认构建树 qt/；任务 041 起由
# build.rs 传入 target/panta-deps/qt，跨 profile 与 presets 共享）。
# 模块构成：qtbase + qtdeclarative（005）与 qttools（034，含 lrelease，
# i18n 的 QM 编译经 `native/i18n` 消费）；qtsvg/qt5compat 等未取，需要时
# 按 task 扩展。

if(NOT DEFINED QT_PROVISION_DIR)
  set(QT_PROVISION_DIR "${CMAKE_BINARY_DIR}/qt")
endif()
set(QT_STAGING "${QT_PROVISION_DIR}/staging")
# 每个归档解包完成后的指纹目录；staging 被清空时全部重解。
set(QT_EXTRACTED "${QT_PROVISION_DIR}/extracted")

if(APPLE)
  set(_qt_repo "https://download.qt.io/online/qtsdkrepository/mac_x64/desktop/qt6_6112/qt6_6112/qt.qt6.6112.clang_64")
  set(_qt_archives
    "6.11.2-0-202608131016qtbase-MacOS-MacOS_15-Clang-MacOS-MacOS_15-X86_64-ARM64.7z|9592f84f7e26d532c5c56824d1da7c9214a766cb0a17beb5af71022bcfbcd271"
    "6.11.2-0-202608131016qtdeclarative-MacOS-MacOS_15-Clang-MacOS-MacOS_15-X86_64-ARM64.7z|ceb8e3f3830531a52de5007ef3f50ffc5525021feb132ee322f59583eb2903cf"
    "6.11.2-0-202608131016qttools-MacOS-MacOS_15-Clang-MacOS-MacOS_15-X86_64-ARM64.7z|415b5008059d0066ac0d4806de98f6e8dacf7b5c80ca0305875160d2107d3719"
  )
elseif(WIN32)
  set(_qt_repo "https://download.qt.io/online/qtsdkrepository/windows_x86/desktop/qt6_6112/qt6_6112_msvc2022_64/qt.qt6.6112.win64_msvc2022_64")
  set(_qt_archives
    "6.11.2-0-202608131017qtbase-Windows-Windows_11_24H2-MSVC2022-Windows-Windows_11_24H2-X86_64.7z|fd984b7264361b4dd3fd2a417702ca1258e4086268f2ee6a69b9a393d9c3f6bb"
    "6.11.2-0-202608131017qtdeclarative-Windows-Windows_11_24H2-MSVC2022-Windows-Windows_11_24H2-X86_64.7z|5591ca564c1a9299a45a15b6b1d324c86e0aaed7cf6c47b7e6fe464aecae2587"
    "6.11.2-0-202608131017qttools-Windows-Windows_11_24H2-MSVC2022-Windows-Windows_11_24H2-X86_64.7z|5f2b387a1f8055102b1388ff4ea743124ed6389d442f75c659728bb5c2a56946"
  )
elseif(UNIX)
  set(_qt_repo "https://download.qt.io/online/qtsdkrepository/linux_x64/desktop/qt6_6112/qt6_6112/qt.qt6.6112.linux_gcc_64")
  set(_qt_archives
    "6.11.2-0-202608131018qtbase-Linux-RHEL_9_6-GCC-Linux-RHEL_9_6-X86_64.7z|0f86f13b161141e77b1d056b54e2b6fc40fb16f243e71123346f9fb35d418027"
    "6.11.2-0-202608131018qtdeclarative-Linux-RHEL_9_6-GCC-Linux-RHEL_9_6-X86_64.7z|5f0ce87c077f749723db6e923142adb860c146ecaf93c68197feda5307f22dd"
    "6.11.2-0-202608131018qttools-Linux-RHEL_9_6-GCC-Linux-RHEL_9_6-X86_64.7z|42d5f3dbfc25647d9d95ef8b64401dc7e3ef7c83a39a29b548dfa0985f71c0ca"
  )
  # Qt Linux 工具使用与该发行版配套的 ICU 73。此归档由 Qt 官方仓库提供，
  # 文件直接放入 staging/lib，供 rcc、qtpaths、qmlimportscanner 等工具通过
  # $ORIGIN/../lib 解析；不使用系统 ICU，也不在本地编译 ICU。
  set(_qt_runtime_archives
    "6.11.2-0-202608131018icu-linux-Rhel8.6-x86_64.7z|111bdae30a66fff6ef65620e95766170fa5a6f425c360ea33b79cb2ec7e2fd86"
  )
else()
  message(FATAL_ERROR "Qt 预编译供给暂不支持平台：${CMAKE_SYSTEM_NAME}（记录到 dependency-acquisition.md 再扩展）")
endif()

file(MAKE_DIRECTORY "${QT_PROVISION_DIR}/archives" "${QT_EXTRACTED}" "${QT_STAGING}")
foreach(entry IN LISTS _qt_archives)
  string(REPLACE "|" ";" _kv "${entry}")
  list(GET _kv 0 _archive_name)
  list(GET _kv 1 _archive_sha)
  set(_archive_path "${QT_PROVISION_DIR}/archives/${_archive_name}")
  set(_extract_marker "${QT_EXTRACTED}/${_archive_name}.sha256")
  # 已按同一哈希解包且 staging 仍完整则跳过；指纹不符（升级归档或清空
  # staging）才重新下载/解包，老缓存只补新模块。
  if(EXISTS "${_extract_marker}"
      AND EXISTS "${QT_STAGING}/bin"
      AND EXISTS "${_archive_path}")
    file(STRINGS "${_extract_marker}" _recorded_sha)
    if(_recorded_sha STREQUAL "${_archive_sha}")
      continue()
    endif()
  endif()
  if(EXISTS "${_archive_path}")
    file(SHA256 "${_archive_path}" _archive_actual)
    if(NOT _archive_actual STREQUAL _archive_sha)
      file(REMOVE "${_archive_path}")
      message(STATUS "Qt 归档校验不符，已删除并重新下载：${_archive_name}")
    endif()
  endif()
  if(NOT EXISTS "${_archive_path}")
    message(STATUS "下载 Qt 预编译包：${_archive_name}")
    file(DOWNLOAD "${_qt_repo}/${_archive_name}" "${_archive_path}"
      INACTIVITY_TIMEOUT 120
      TIMEOUT 900
      EXPECTED_HASH SHA256=${_archive_sha})
  endif()
  message(STATUS "解包 Qt 预编译包：${_archive_name}")
  execute_process(
    COMMAND "${CMAKE_COMMAND}" -E tar xf "${_archive_path}"
    WORKING_DIRECTORY "${QT_STAGING}"
    RESULT_VARIABLE _extract_result
  )
  if(NOT _extract_result EQUAL 0)
    message(FATAL_ERROR "Qt 归档解包失败（${_extract_result}）：${_archive_name}")
  endif()
  file(WRITE "${_extract_marker}" "${_archive_sha}\n")
endforeach()

if(DEFINED _qt_runtime_archives AND NOT EXISTS "${QT_STAGING}/lib/libicui18n.so.73")
  foreach(entry IN LISTS _qt_runtime_archives)
    string(REPLACE "|" ";" _kv "${entry}")
    list(GET _kv 0 _archive_name)
    list(GET _kv 1 _archive_sha)
    set(_archive_path "${QT_PROVISION_DIR}/archives/${_archive_name}")
    if(EXISTS "${_archive_path}")
      file(SHA256 "${_archive_path}" _archive_actual)
      if(NOT _archive_actual STREQUAL _archive_sha)
        file(REMOVE "${_archive_path}")
        message(STATUS "Qt ICU 归档校验不符，已删除并重新下载：${_archive_name}")
      endif()
    endif()
    if(NOT EXISTS "${_archive_path}")
      message(STATUS "下载 Qt ICU 预编译包：${_archive_name}")
      file(DOWNLOAD "${_qt_repo}/${_archive_name}" "${_archive_path}"
        INACTIVITY_TIMEOUT 120
        TIMEOUT 900
        EXPECTED_HASH SHA256=${_archive_sha})
    endif()
    message(STATUS "解包 Qt ICU 预编译包：${_archive_name}")
    execute_process(
      COMMAND "${CMAKE_COMMAND}" -E tar xf "${_archive_path}"
      WORKING_DIRECTORY "${QT_STAGING}/lib"
      RESULT_VARIABLE _extract_result
    )
    if(NOT _extract_result EQUAL 0)
      message(FATAL_ERROR "Qt ICU 归档解包失败（${_extract_result}）：${_archive_name}")
    endif()
  endforeach()
endif()

unset(_qt_repo)
unset(_qt_archives)
unset(_archive_path)
unset(_archive_name)
unset(_archive_sha)
unset(_qt_runtime_archives)
unset(_extract_marker)
unset(_recorded_sha)
