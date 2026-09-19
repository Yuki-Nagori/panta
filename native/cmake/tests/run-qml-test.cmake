# CTest 在 Windows 启动失败时可能没有输出；保留子进程的真实退出状态。
execute_process(COMMAND "${TEST_EXECUTABLE}" RESULT_VARIABLE _result)
if(NOT "${_result}" STREQUAL "0")
  message(FATAL_ERROR "QML 测试失败：${TEST_EXECUTABLE}（退出状态 ${_result}）")
endif()
