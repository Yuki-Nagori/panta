cmake_minimum_required(VERSION 3.22)
project(panta_layering_fixture LANGUAGES CXX)

set(PANTA_ENABLE_BRIDGE_MODULE ON)
set(PANTA_FFI_STATIC_LIB "${CMAKE_BINARY_DIR}/libpanta_ffi.a")
include("${CMAKE_CURRENT_LIST_DIR}/../../layering.cmake")

foreach(
  _target IN
  ITEMS foundation
        geometry
        mesh
        qt_adapter
        i18n
        visualization
        bridge
        shell
        bridgeplugin
        shellplugin
        visualizationplugin
        native_app)
  add_library("panta_${_target}" INTERFACE)
endforeach()

foreach(_target IN ITEMS Experimental NightlyMemoryCheck all_qmltyperegistrations all_qmllint
                         all_qmllint_json all_qmllint_module)
  add_custom_target("${_target}" COMMENT "模拟 Qt/CTest 生成的 utility target")
endforeach()

add_library(panta::foundation ALIAS panta_foundation)
add_library(panta::qt_adapter ALIAS panta_qt_adapter)
add_library(panta::i18n ALIAS panta_i18n)
add_library(panta::visualization ALIAS panta_visualization)
add_library(panta::bridge ALIAS panta_bridge)
add_library(panta::shell ALIAS panta_shell)

target_link_libraries(panta_mesh INTERFACE "${PANTA_FFI_STATIC_LIB}")
target_link_libraries(panta_bridge INTERFACE panta::visualization panta::qt_adapter
                                             "${PANTA_FFI_STATIC_LIB}")
target_link_libraries(panta_shell INTERFACE panta::bridge panta::visualization)
target_link_libraries(panta_bridgeplugin INTERFACE panta_bridge panta_visualizationplugin)
target_link_libraries(panta_shellplugin INTERFACE panta_shell)
target_link_libraries(panta_visualizationplugin INTERFACE panta_visualization)
target_link_libraries(
  panta_native_app
  INTERFACE panta::foundation
            panta::shell
            panta_shellplugin
            panta::i18n
            "${PANTA_FFI_STATIC_LIB}"
            panta_visualization
            panta_bridgeplugin
            panta_visualizationplugin)

if(PANTA_FIXTURE_ILLEGAL)
  add_library(panta_core INTERFACE IMPORTED GLOBAL)
  target_link_libraries(panta_mesh INTERFACE panta_core)
endif()

if(PANTA_FIXTURE_UNREGISTERED_TARGET)
  add_library(mesh_helper INTERFACE)
  target_link_libraries(panta_mesh INTERFACE mesh_helper)
endif()

if(PANTA_FIXTURE_PREFIX_COLLISION)
  target_link_libraries(panta_bridge INTERFACE "$<$<BOOL:1>:panta_visualizationplugin>")
endif()

if(PANTA_FIXTURE_HIDDEN_ALIAS)
  add_library(hidden_alias ALIAS panta_bridge)
  target_link_libraries(panta_mesh INTERFACE "$<LINK_ONLY:hidden_alias>")
endif()

if(PANTA_FIXTURE_NAMESPACED_ALIAS)
  add_library(hidden::bridge ALIAS panta_bridge)
  target_link_libraries(panta_mesh INTERFACE "$<LINK_ONLY:hidden::bridge>")
endif()

if(PANTA_FIXTURE_DIRECT_PROPERTY)
  set_property(TARGET panta_mesh PROPERTY INTERFACE_LINK_LIBRARIES_DIRECT panta_bridge)
endif()

if(PANTA_FIXTURE_COMPUTED_GENEX)
  target_link_libraries(panta_mesh INTERFACE "$<JOIN:panta_;bridge,>")
endif()

if(PANTA_FIXTURE_DYNAMIC_PROPERTY)
  target_link_libraries(panta_mesh INTERFACE "$<TARGET_PROPERTY:INTERFACE_LINK_LIBRARIES>")
endif()

if(PANTA_FIXTURE_ADJACENT_GENEX)
  target_link_libraries(panta_mesh INTERFACE "$<1:panta_>$<1:bridge>")
endif()

if(PANTA_FIXTURE_NESTED_CONCAT)
  target_link_libraries(panta_mesh INTERFACE "$<1:panta_$<1:bridge>>")
endif()

if(PANTA_FIXTURE_LITERAL_GENEX_CONCAT)
  add_library(hidden_bridge ALIAS panta_bridge)
  target_link_libraries(panta_mesh INTERFACE "hidden_$<1:bridge>")
endif()

cmake_language(DEFER CALL panta_verify_layering)
