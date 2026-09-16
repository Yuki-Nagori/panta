# 桌面入口先导骨架：任务 005 在此接入 Qt 并替换实现；可执行名 panta-native
# 是 launcher（crates/launcher/build.rs）的定位约定，改名须同步该脚本。
add_executable(panta_native_app main.cpp)
set_target_properties(panta_native_app PROPERTIES
  OUTPUT_NAME panta-native
  EXPORT_NAME native-app
)
panta_native_defaults(panta_native_app)
target_link_libraries(panta_native_app PRIVATE panta::foundation)

install(TARGETS panta_native_app EXPORT pantaNativeTargets
  RUNTIME DESTINATION bin
)
