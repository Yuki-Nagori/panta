# CMake 最终链接 Rust staticlib，验证 CXX 生成头和双向调用只编译/链接一次。
# 两个测试目标使用相同的 staticlib/生成头/系统库链入口，差异仅在源文件。
function(panta_add_ffi_test name source)
  add_executable(${name} ${source})
  panta_native_defaults(${name})
  target_include_directories(${name} PRIVATE "${PANTA_FFI_INCLUDE_DIR}"
                                             "${CMAKE_SOURCE_DIR}/../crates/panta-ffi/include")
  target_link_libraries(${name} PRIVATE "${PANTA_FFI_STATIC_LIB}" GTest::gtest_main)
  if(UNIX)
    find_package(Threads REQUIRED)
    target_link_libraries(${name} PRIVATE Threads::Threads ${CMAKE_DL_LIBS})
  elseif(WIN32)
    # Rust std 不会把这些 Windows 系统库作为 CMake target 依赖传播给
    # 外部 staticlib 消费者；边界测试是最终链接者，需在此显式声明。
    target_link_libraries(${name} PRIVATE ws2_32 userenv ntdll)
  endif()
endfunction()

panta_add_ffi_test(panta_ffi_boundary_test ../../tests/cpp/ffi/boundary_test.cpp)
panta_add_ffi_test(panta_ffi_task_service_test ../../tests/cpp/ffi/task_service_test.cpp)

add_test(NAME Ffi.RustCppBoundary COMMAND panta_ffi_boundary_test)

add_test(NAME Ffi.TaskServiceLifecycle COMMAND panta_ffi_task_service_test)
