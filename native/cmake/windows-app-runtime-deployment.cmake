# Windows 开发裸启运行库部署（任务 083）：真实窗口验收直接启动
# target/native/debug 下的 exe，其 DLL 搜索路径不含任何 staging，必须把
# 依赖运行库与 Qt 插件复制到应用目录。仅覆盖开发验收场景；安装包与发布
# 分发另行立任务。
#
# 部署三段：
# 1. windeployqt（托管 Qt 供给）：Qt6 DLL、平台/图像格式/QML 插件与 Qt
#    QML 模块树；--qmldir 指向仓库 QML 源供推导 Qt 模块集（app 目标没有
#    可扫描的源文件，见 native/app/CMakeLists.txt 说明）。
# 2. 已供给 SDK 的 DLL 目录整目录复制：顶层 bin 与 OCCT 式嵌套 bin 都被
#    覆盖，目录集合随 manifest 登记与 staging 实况增长，调用方无需维护
#    名单（netgen bin、occt win64/vc14/bin、vtk bin 等）。
# 3. d3dcompiler_47.dll（System32 副本）：Dawn 的 HLSL 编译要求它位于应
#    用目录，仅 System32 可见时 LoadLibraryEx 报 Error 87、WebGPU 设备创
#    建失败并 SEH 崩溃。
#
# 调用：panta_deploy_windows_app_runtime(<target> <qmldir>)
function(panta_deploy_windows_app_runtime target qml_dir)
  if(NOT WIN32)
    return()
  endif()

  find_program(
    PANTA_WINDEPLOYQT_EXECUTABLE
    NAMES windeployqt
    PATHS "${QT_STAGING}/bin"
    NO_DEFAULT_PATH REQUIRED)
  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND "${PANTA_WINDEPLOYQT_EXECUTABLE}" --no-translations --qmldir "${qml_dir}"
            "$<TARGET_FILE:${target}>"
    COMMENT "部署 Qt 运行库到 $<TARGET_FILE_NAME:${target}> 目录"
    VERBATIM)

  get_property(_sdk_names GLOBAL PROPERTY panta_sdk_names)
  foreach(_name IN LISTS _sdk_names)
    get_property(_version GLOBAL PROPERTY "panta_sdk_version_${_name}")
    if(_version STREQUAL "")
      continue()
    endif()
    panta_sdk_triple(_triple)
    set(_staging "${PANTA_SDK_PROVISION_DIR}/${_name}/${_version}/${_triple}")
    file(GLOB_RECURSE _sdk_dlls "${_staging}/*.dll")
    set(_dll_dirs "")
    foreach(_dll IN LISTS _sdk_dlls)
      get_filename_component(_dir "${_dll}" DIRECTORY)
      list(APPEND _dll_dirs "${_dir}")
    endforeach()
    list(REMOVE_DUPLICATES _dll_dirs)
    foreach(_dir IN LISTS _dll_dirs)
      add_custom_command(
        TARGET ${target}
        POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E copy_directory "${_dir}" "$<TARGET_FILE_DIR:${target}>"
        COMMENT "拷贝 ${_name} SDK 运行库到 $<TARGET_FILE_NAME:${target}> 目录"
        VERBATIM)
    endforeach()
  endforeach()

  add_custom_command(
    TARGET ${target}
    POST_BUILD
    COMMAND "${CMAKE_COMMAND}" -E copy "$ENV{windir}/System32/d3dcompiler_47.dll"
            "$<TARGET_FILE_DIR:${target}>"
    COMMENT "拷贝 d3dcompiler_47.dll 到 $<TARGET_FILE_NAME:${target}> 目录"
    VERBATIM)
endfunction()
