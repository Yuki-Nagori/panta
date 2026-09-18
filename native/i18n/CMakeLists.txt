# i18n 编译模块（任务 034）：消费 Cargo 生成的 TS（PANTA_I18N_TS_DIR，由
# launcher build.rs 从 resources/i18n/*.pa 生成），用 Qt 供给的锁定 lrelease
# 编译 QM 并嵌入资源。QM 是唯一运行期翻译输入；TS/QM 均为构建树产物，不入库。
# 运行时加载与语言切换由 022 接入，此处只负责编译与资源入口（:/i18n/）。

if(NOT DEFINED PANTA_I18N_TS_DIR)
  message(
    FATAL_ERROR
      "PANTA_I18N_TS_DIR 未设置：TS 由 cargo build 的 launcher"
      " build.rs 从 resources/i18n/*.pa 生成；presets 纯净 configure 请先运行"
      " cargo build（与 FFI 输入同边界，任务 041）")
endif()
if(NOT EXISTS "${PANTA_I18N_TS_DIR}")
  message(FATAL_ERROR "PANTA_I18N_TS_DIR 指向的目录不存在：${PANTA_I18N_TS_DIR}；" "先运行 cargo build 生成 TS")
endif()

# 只消费 Qt 预编译供给内的 lrelease（qttools 归档，见 qt-provision.cmake）；
# 不回退系统 lrelease 或源码构建（standards/dependency-acquisition.md）。
find_program(
  PANTA_LRELEASE_EXECUTABLE
  NAMES lrelease
  PATHS "${QT_STAGING}/bin"
  NO_DEFAULT_PATH REQUIRED)
execute_process(
  COMMAND "${PANTA_LRELEASE_EXECUTABLE}" -version
  OUTPUT_VARIABLE _lrelease_version
  ERROR_VARIABLE _lrelease_version
  OUTPUT_STRIP_TRAILING_WHITESPACE)
message(STATUS "panta i18n lrelease：${_lrelease_version}")

file(GLOB PANTA_I18N_TS_FILES "${PANTA_I18N_TS_DIR}/*.ts")
if(NOT PANTA_I18N_TS_FILES)
  message(FATAL_ERROR "${PANTA_I18N_TS_DIR} 下没有 .ts；检查 resources/i18n/*.pa"
                      " 与 launcher build.rs 的 TS 生成步骤")
endif()

set(PANTA_I18N_QM_FILES)
foreach(_ts IN LISTS PANTA_I18N_TS_FILES)
  get_filename_component(_stem "${_ts}" NAME_WE)
  set(_qm "${CMAKE_CURRENT_BINARY_DIR}/${_stem}.qm")
  # lrelease 默认参数（034 决策：Qt 6 中 -idbased 已废弃）。
  add_custom_command(
    OUTPUT "${_qm}"
    COMMAND "${PANTA_LRELEASE_EXECUTABLE}" "${_ts}" -qm "${_qm}"
    DEPENDS "${_ts}"
    COMMENT "lrelease ${_stem}"
    VERBATIM)
  list(APPEND PANTA_I18N_QM_FILES "${_qm}")
endforeach()

# 生成 .qrc 列出全部 QM：qrc 内列出的文件按原始字节嵌入（QM 魔数以 '<'
# 开头，直接传给 rcc 的单文件模式会被误判为 XML）；条目用相对 qrc 的
# basename——绝对路径会被 rcc 展开成完整资源路径。CMake 解析 qrc 建立
# 对 QM 的构建依赖。资源前缀 /i18n，运行期经 :/i18n/<catalog>_<locale>.qm
# 访问。
set(_i18n_qrc "${CMAKE_CURRENT_BINARY_DIR}/panta_i18n.qrc")
set(_i18n_qrc_entries)
foreach(_qm IN LISTS PANTA_I18N_QM_FILES)
  get_filename_component(_qm_name "${_qm}" NAME)
  string(APPEND _i18n_qrc_entries "    <file>${_qm_name}</file>\n")
endforeach()
file(
  WRITE "${_i18n_qrc}"
  "<!DOCTYPE RCC>\n<RCC version=\"1.0\">\n  <qresource prefix=\"/i18n\">\n${_i18n_qrc_entries}  </qresource>\n</RCC>\n"
)

qt_add_resources(PANTA_I18N_RESOURCE_SOURCES "${_i18n_qrc}")
# OBJECT 库：资源注册符在 qrc_*.cpp 的静态初始化里，无被引用符号；普通
# STATIC 库会被消费者链接时整库丢弃，对象直接传递才保证 :/i18n/ 可用。
add_library(panta_i18n OBJECT ${PANTA_I18N_RESOURCE_SOURCES})
panta_native_defaults(panta_i18n)
add_library(panta::i18n ALIAS panta_i18n)

if(BUILD_TESTING)
  add_executable(panta_i18n_qm_test ../../tests/cpp/i18n/qm_load_test.cpp)
  panta_native_defaults(panta_i18n_qm_test)
  target_link_libraries(panta_i18n_qm_test PRIVATE panta_i18n Qt6::Core Qt6::Test)
  add_test(NAME I18n.CompiledQmLoads COMMAND panta_i18n_qm_test)
  set_tests_properties(I18n.CompiledQmLoads PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
endif()
