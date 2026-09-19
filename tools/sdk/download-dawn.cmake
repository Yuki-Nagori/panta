# 下载并解包 VTK 9.7 RenderingWebGPU 所需的固定 GitHub Dawn native 资产。

cmake_minimum_required(VERSION 3.22)

foreach(_required IN ITEMS PANTA_DAWN_DOWNLOAD_DIR PANTA_DAWN_URL PANTA_DAWN_ARCHIVE
                           PANTA_DAWN_SHA256 PANTA_DAWN_LICENSE_URL PANTA_DAWN_LICENSE_SHA256)
  if(NOT DEFINED ${_required})
    message(FATAL_ERROR "Dawn 下载缺少变量 ${_required}")
  endif()
endforeach()

set(_download_dir "${PANTA_DAWN_DOWNLOAD_DIR}")
set(_archive "${_download_dir}/${PANTA_DAWN_ARCHIVE}")
set(_dawn_dir "${_download_dir}/dawn")
file(MAKE_DIRECTORY "${_download_dir}")

set(_archive_valid FALSE)
if(EXISTS "${_archive}")
  file(SHA256 "${_archive}" _actual_sha256)
  if(_actual_sha256 STREQUAL PANTA_DAWN_SHA256)
    set(_archive_valid TRUE)
  else()
    file(REMOVE "${_archive}")
  endif()
endif()

if(NOT _archive_valid)
  message(STATUS "下载 Dawn 预编译资产：${PANTA_DAWN_ARCHIVE}")
  file(
    DOWNLOAD "${PANTA_DAWN_URL}" "${_archive}"
    EXPECTED_HASH "SHA256=${PANTA_DAWN_SHA256}"
    STATUS _download_status
    SHOW_PROGRESS)
  list(GET _download_status 0 _download_code)
  if(_download_code)
    list(GET _download_status 1 _download_message)
    file(REMOVE "${_archive}")
    message(FATAL_ERROR "Dawn 下载失败：${_download_message}")
  endif()
endif()

file(REMOVE_RECURSE "${_dawn_dir}")
execute_process(
  COMMAND "${CMAKE_COMMAND}" -E tar xf "${_archive}"
  WORKING_DIRECTORY "${_download_dir}"
  RESULT_VARIABLE _extract_code
  ERROR_VARIABLE _extract_error
  ERROR_STRIP_TRAILING_WHITESPACE)
if(_extract_code)
  message(FATAL_ERROR "Dawn 解包失败：${_extract_error}")
endif()

get_filename_component(_archive_stem "${PANTA_DAWN_ARCHIVE}" NAME_WE)
if(PANTA_DAWN_ARCHIVE MATCHES "\\.tar\\.gz$")
  string(REGEX REPLACE "\\.tar$" "" _archive_stem "${_archive_stem}")
endif()
set(_extracted_dir "${_download_dir}/${_archive_stem}")
if(NOT IS_DIRECTORY "${_extracted_dir}")
  if(PANTA_DAWN_ARCHIVE MATCHES "\\.tar\\.gz$" AND IS_DIRECTORY
                                                   "${_download_dir}/${_archive_stem}.tar")
    set(_extracted_dir "${_download_dir}/${_archive_stem}.tar")
  else()
    message(FATAL_ERROR "Dawn 归档缺少预期目录：${_extracted_dir}")
  endif()
endif()
file(RENAME "${_extracted_dir}" "${_dawn_dir}")
if(NOT EXISTS "${_dawn_dir}/lib/cmake/Dawn/DawnConfig.cmake")
  message(FATAL_ERROR "Dawn 制品缺少 lib/cmake/Dawn/DawnConfig.cmake：${_dawn_dir}")
endif()

set(_license "${_dawn_dir}/LICENSE")
set(_license_valid FALSE)
if(EXISTS "${_license}")
  file(SHA256 "${_license}" _actual_license_sha256)
  if(_actual_license_sha256 STREQUAL PANTA_DAWN_LICENSE_SHA256)
    set(_license_valid TRUE)
  else()
    file(REMOVE "${_license}")
  endif()
endif()
if(NOT _license_valid)
  file(
    DOWNLOAD "${PANTA_DAWN_LICENSE_URL}" "${_license}"
    EXPECTED_HASH "SHA256=${PANTA_DAWN_LICENSE_SHA256}"
    STATUS _license_status)
  list(GET _license_status 0 _license_code)
  if(_license_code)
    list(GET _license_status 1 _license_message)
    file(REMOVE "${_license}")
    message(FATAL_ERROR "Dawn LICENSE 下载失败：${_license_message}")
  endif()
endif()
