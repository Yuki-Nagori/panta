# 桌面入口（任务 005）：链接 QML 模块与 ViewModel 模块。
add_executable(panta_native_app
  main.cpp
)
set_target_properties(panta_native_app PROPERTIES
  OUTPUT_NAME panta-native
  EXPORT_NAME native-app
)
panta_native_defaults(panta_native_app)
target_link_libraries(panta_native_app PRIVATE
  panta::foundation
  panta_bridgeplugin
  panta::shell
  Qt6::Gui
  Qt6::Qml
  Qt6::Quick
  Qt6::QuickControls2
)

# 由 Panta.Shell 的 TARGET 依赖扫描并链接 Panta.Bridge 静态 plugin，
# 保证 QML_ELEMENT 注册代码在最终 executable 中保留。
qt_import_qml_plugins(panta_native_app)

install(TARGETS panta_native_app EXPORT pantaNativeTargets
  RUNTIME DESTINATION bin
)
