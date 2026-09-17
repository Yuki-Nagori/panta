# CMake 最终链接 Rust staticlib，验证 CXX 生成头和双向调用只编译/链接一次。
add_executable(panta_ffi_boundary_test tests/boundary_test.cpp)
panta_native_defaults(panta_ffi_boundary_test)
target_include_directories(panta_ffi_boundary_test PRIVATE
  "${PANTA_FFI_INCLUDE_DIR}"
  "${CMAKE_SOURCE_DIR}/../crates/panta-ffi/include"
)
target_link_libraries(panta_ffi_boundary_test PRIVATE
  "${PANTA_FFI_STATIC_LIB}"
  GTest::gtest_main
)
if(UNIX)
  find_package(Threads REQUIRED)
  target_link_libraries(panta_ffi_boundary_test PRIVATE Threads::Threads ${CMAKE_DL_LIBS})
elseif(WIN32)
  # Rust 的 MSVC staticlib 由 cc/rustc 以动态 release CRT（/MD）构建；
  # CMake Debug 默认是 /MDd + iterator level 2，会在边界测试链接时触发
  # LNK2038。只对这个跨 Rust ABI 的测试和其 GTest 依赖收敛到 /MD/level 0，
  # 不改变其它 native Debug target 的诊断配置。
  set_property(TARGET panta_ffi_boundary_test PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
  target_compile_definitions(panta_ffi_boundary_test PRIVATE _ITERATOR_DEBUG_LEVEL=0)
  foreach(gtest_target IN ITEMS gtest gtest_main)
    if(TARGET ${gtest_target})
      set_property(TARGET ${gtest_target} PROPERTY MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
      target_compile_definitions(${gtest_target} PUBLIC _ITERATOR_DEBUG_LEVEL=0)
    endif()
  endforeach()

  # Rust std 不会把这些 Windows 系统库作为 CMake target 依赖传播给
  # 外部 staticlib 消费者；边界测试是最终链接者，需在此显式声明。
  target_link_libraries(panta_ffi_boundary_test PRIVATE ws2_32 userenv ntdll)
endif()

add_test(
  NAME Ffi.RustCppBoundary
  COMMAND panta_ffi_boundary_test
)
