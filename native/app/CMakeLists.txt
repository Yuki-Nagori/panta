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

# Panta.Bridge 的静态 plugin 由 main.cpp 中的 Q_IMPORT_QML_PLUGIN 显式保留。
# App.qml 已作为 Panta.Shell 模块资源编译，app 目标没有可供 qmlimportscanner
# 扫描的源文件；调用扫描器只会产生空结果，并要求工具额外加载平台 ICU。

install(TARGETS panta_native_app EXPORT pantaNativeTargets
  RUNTIME DESTINATION bin
)
