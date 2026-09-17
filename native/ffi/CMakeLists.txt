# CMake 最终链接 Rust staticlib，验证 CXX 生成头和双向调用只编译/链接一次。
add_executable(panta_ffi_boundary_test tests/boundary_test.cpp)
panta_native_defaults(panta_ffi_boundary_test)
target_include_directories(panta_ffi_boundary_test PRIVATE
  "${PANTA_FFI_INCLUDE_DIR}"
  "${CMAKE_SOURCE_DIR}/../crates/panta-ffi/include"
)
target_link_libraries(panta_ffi_boundary_test PRIVATE
  "${PANTA_FFI_STATIC_LIB}"
  GTest::gtest
)
if(UNIX)
  find_package(Threads REQUIRED)
  target_link_libraries(panta_ffi_boundary_test PRIVATE Threads::Threads ${CMAKE_DL_LIBS})
endif()

add_test(
  NAME Ffi.RustCppBoundary
  COMMAND panta_ffi_boundary_test
)
