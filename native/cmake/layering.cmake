# Rust/CMake 分层检查依赖实际配置后的 target link 属性；不从源码文本猜测边。
function(_panta_collect_directory_targets directory output)
  get_property(
    _directory_targets
    DIRECTORY "${directory}"
    PROPERTY BUILDSYSTEM_TARGETS)
  get_property(
    _subdirectories
    DIRECTORY "${directory}"
    PROPERTY SUBDIRECTORIES)
  foreach(_subdirectory IN LISTS _subdirectories)
    _panta_collect_directory_targets("${_subdirectory}" _child_targets)
    list(APPEND _directory_targets ${_child_targets})
  endforeach()
  set("${output}"
      "${_directory_targets}"
      PARENT_SCOPE)
endfunction()

# 验证生成表达式的输出分支可静态归约到完整依赖项，拒绝字面量或多个
# 生成表达式拼接 target 名。Qt 生成的 TARGET_OBJECTS/TARGET_NAME 与
# LINK_ONLY/条件包装按结构递归处理。
function(_panta_validate_link_output source expression link output_operator output_value)
  if(NOT expression MATCHES "\\$<")
    set("${output_operator}"
        ""
        PARENT_SCOPE)
    set("${output_value}"
        "${expression}"
        PARENT_SCOPE)
    return()
  endif()
  if(NOT expression MATCHES "^\\$<")
    message(FATAL_ERROR "分层检查拒绝拼接或不完整的链接输出：${source} -> ${link}")
  endif()

  string(LENGTH "${expression}" _length)
  set(_depth 0)
  set(_separator -1)
  set(_outer_close -1)
  set(_index 0)
  while(_index LESS _length)
    math(EXPR _remaining "${_length} - ${_index}")
    if(_remaining GREATER 1)
      string(SUBSTRING "${expression}" ${_index} 2 _pair)
    else()
      set(_pair "")
    endif()
    if(_pair STREQUAL "$<")
      math(EXPR _depth "${_depth} + 1")
      math(EXPR _index "${_index} + 2")
      continue()
    endif()
    string(SUBSTRING "${expression}" ${_index} 1 _character)
    if(_character STREQUAL ">")
      math(EXPR _depth "${_depth} - 1")
      if(_depth EQUAL 0)
        set(_outer_close "${_index}")
        break()
      endif()
    elseif(_character STREQUAL ":" AND _depth EQUAL 1)
      set(_separator "${_index}")
    endif()
    math(EXPR _index "${_index} + 1")
  endwhile()
  math(EXPR _last_index "${_length} - 1")
  if(NOT _outer_close EQUAL _last_index OR _separator LESS 0)
    message(FATAL_ERROR "分层检查拒绝拼接或不完整的链接输出：${source} -> ${link}")
  endif()

  if(expression MATCHES "^\\$<([A-Za-z_][A-Za-z0-9_]*)[:>]")
    set(_operator "${CMAKE_MATCH_1}")
  else()
    set(_operator "")
  endif()
  math(EXPR _value_start "${_separator} + 1")
  math(EXPR _value_length "${_length} - ${_value_start} - 1")
  string(SUBSTRING "${expression}" ${_value_start} ${_value_length} _value)

  if(_operator STREQUAL "TARGET_NAME")
    if(_value MATCHES "\\$<" OR _value MATCHES "[,;]")
      message(FATAL_ERROR "分层检查拒绝动态 TARGET_NAME 输出：${source} -> ${link}")
    endif()
  elseif(
    _operator STREQUAL "LINK_ONLY"
    OR _operator STREQUAL "BUILD_INTERFACE"
    OR _operator STREQUAL "INSTALL_INTERFACE"
    OR _operator STREQUAL "TARGET_OBJECTS"
    OR _operator STREQUAL "")
    _panta_validate_link_output("${source}" "${_value}" "${link}" _nested_operator _nested_value)
  else()
    message(FATAL_ERROR "分层检查不支持链接输出表达式 ${_operator}：${source} -> ${link}")
  endif()
  set("${output_operator}"
      "${_operator}"
      PARENT_SCOPE)
  set("${output_value}"
      "${_value}"
      PARENT_SCOPE)
endfunction()

function(panta_verify_layering)
  set(_production_targets
      panta_foundation
      panta_geometry
      panta_mesh
      panta_qt_adapter
      panta_i18n
      panta_visualization
      panta_bridge
      panta_shell
      panta_bridgeplugin
      panta_shellplugin
      panta_visualizationplugin
      panta_native_app)

  _panta_collect_directory_targets("${CMAKE_SOURCE_DIR}" _configured_targets)
  foreach(_target IN LISTS _configured_targets)
    get_target_property(_target_source_dir "${_target}" SOURCE_DIR)
    if(NOT _target_source_dir)
      continue()
    endif()
    cmake_path(IS_PREFIX CMAKE_SOURCE_DIR "${_target_source_dir}" _project_owned_target NORMALIZE)
    if(NOT _project_owned_target OR _target IN_LIST _production_targets)
      continue()
    endif()
    get_target_property(_target_type "${_target}" TYPE)
    # CTest、Qt 等会在调用方目录创建聚合 custom target；它们不能作为链接库
    # 绕过依赖图检查，只审计可参与链接图的仓库自有 targets。
    if(_target_type STREQUAL "UTILITY")
      continue()
    endif()
    if(_target MATCHES "(_test|_benchmark)(_other_files)?$"
       OR _target STREQUAL "panta_performance_fixture"
       OR _target MATCHES "^panta_(bridge|shell|visualization)(_resources_[0-9]+|_tooling)$"
       OR _target MATCHES "^panta_(bridge|shell|visualization)plugin_init$"
       OR _target MATCHES "^panta_(bridge|shell|visualization)_other_files$"
       OR _target MATCHES "^panta_(bridge|shell|visualization)_qml[a-zA-Z0-9_]*$"
       OR _target MATCHES "^panta_(bridge|shell|visualization)_.*_qml$"
       OR _target MATCHES "^panta_(bridge|shell|visualization)_copy_(qml|res)$"
       OR _target MATCHES
          "^panta_native_app_(qml[a-zA-Z0-9_]*|.*_qml|copy_(qml|res)|other_files|tooling)$"
       OR _target MATCHES "^(panta_native_app_tooling|panta_benchmark_moc)$")
      continue()
    endif()
    message(FATAL_ERROR "项目 native target 未登记分层（须使用已登记的 panta_ target 名）：${_target}")
  endforeach()

  foreach(_target IN LISTS _production_targets)
    if(_target
       MATCHES
       "^(panta_qt_adapter|panta_visualization|panta_bridge|panta_bridgeplugin|panta_visualizationplugin)$"
       AND NOT PANTA_ENABLE_BRIDGE_MODULE)
      continue()
    endif()
    if(NOT TARGET "${_target}")
      message(FATAL_ERROR "分层检查无法解析项目 target '${_target}'")
    endif()
  endforeach()

  foreach(_source IN LISTS _production_targets)
    if(_source
       MATCHES
       "^(panta_qt_adapter|panta_visualization|panta_bridge|panta_bridgeplugin|panta_visualizationplugin)$"
       AND NOT PANTA_ENABLE_BRIDGE_MODULE)
      continue()
    endif()

    set(_supported_link_genex
        LINK_ONLY
        BUILD_INTERFACE
        INSTALL_INTERFACE
        BOOL
        IF
        CONFIG
        AND
        NOT
        STREQUAL
        TARGET_OBJECTS
        TARGET_NAME)
    set(_allowed_dependencies)
    set(_allowed_aliases)
    if(_source STREQUAL "panta_mesh")
      set(_allowed_dependencies panta_ffi)
    elseif(_source STREQUAL "panta_bridge")
      set(_allowed_dependencies panta_visualization panta_qt_adapter panta_ffi)
      set(_allowed_aliases panta::visualization panta::qt_adapter)
    elseif(_source STREQUAL "panta_shell")
      set(_allowed_dependencies panta_bridge panta_visualization)
      set(_allowed_aliases panta::bridge panta::visualization)
    elseif(_source STREQUAL "panta_bridgeplugin")
      set(_allowed_dependencies panta_bridge panta_visualizationplugin)
      set(_allowed_aliases panta::bridge)
    elseif(_source STREQUAL "panta_shellplugin")
      set(_allowed_dependencies panta_shell)
      set(_allowed_aliases panta::shell)
    elseif(_source STREQUAL "panta_visualizationplugin")
      set(_allowed_dependencies panta_visualization)
      set(_allowed_aliases panta::visualization)
    elseif(_source STREQUAL "panta_native_app")
      set(_allowed_dependencies
          panta_foundation
          panta_shell
          panta_shellplugin
          panta_i18n
          panta_ffi
          panta_visualization
          panta_bridgeplugin
          panta_visualizationplugin)
      set(_allowed_aliases panta::foundation panta::shell panta::i18n)
    endif()

    set(_links)
    foreach(_property IN ITEMS LINK_LIBRARIES INTERFACE_LINK_LIBRARIES
                               INTERFACE_LINK_LIBRARIES_DIRECT)
      get_target_property(_property_links "${_source}" "${_property}")
      if(_property_links AND NOT _property_links MATCHES "-NOTFOUND$")
        list(APPEND _links ${_property_links})
      endif()
    endforeach()
    list(REMOVE_DUPLICATES _links)

    foreach(_link IN LISTS _links)
      if(_link MATCHES "\\$<")
        # 表达式可能藏在字面量后缀/前缀中；先验证所有含生成表达式的项，
        # 再按完整表达式处理，避免把合成库名当作外部依赖忽略。
        _panta_validate_link_output("${_source}" "${_link}" "${_link}" _root_operator _link_value)
      endif()
      # 条件链接表达式不能安全地按字符串拆解；分层图中的仓库内边必须显式
      # 可解析。只接受结构已知的表达式，防止 JOIN/TARGET_PROPERTY 等计算
      # 型表达式拼出静态 token 扫描看不到的项目 target。
      if(_link MATCHES "^\\$<")
        if(_root_operator)
          if(NOT _root_operator IN_LIST _supported_link_genex)
            message(FATAL_ERROR "分层检查不支持链接生成表达式：${_source} -> ${_link}（${_root_operator}）")
          endif()
        endif()

        string(REGEX MATCHALL "\\$<[A-Za-z_][A-Za-z0-9_]*[:>]" _genex_operators "${_link_value}")
        foreach(_expression IN LISTS _genex_operators)
          string(REGEX REPLACE "^\\$<|[:>]$" "" _operator "${_expression}")
          if(NOT _operator IN_LIST _supported_link_genex)
            message(FATAL_ERROR "分层检查不支持无法静态验证的链接生成表达式：${_source} -> ${_link}（${_operator}）")
          endif()
        endforeach()

        string(REGEX MATCHALL "panta(::|_)[A-Za-z0-9_.+-]+" _project_tokens "${_link}")
        foreach(_dependency IN LISTS _project_tokens)
          if(_dependency IN_LIST _allowed_dependencies OR _dependency IN_LIST _allowed_aliases)
            continue()
          endif()
          if(_dependency
             MATCHES
             "^panta_(bridge|shell|visualization)(_resources_[0-9]+|_tooling|_other_files|_qml[a-zA-Z0-9_]*|_.*_qml|_copy_(qml|res)|plugin_init)$"
             OR _dependency MATCHES
                "^panta_native_app_(qml[a-zA-Z0-9_]*|.*_qml|copy_(qml|res)|other_files|tooling)$")
            continue()
          endif()
          message(FATAL_ERROR "条件链接表达式含未登记的项目 target：${_source} -> ${_dependency}（表达式：${_link}）")
        endforeach()

        # 非 panta 命名的 ALIAS 也可能指向项目 target。检查表达式中的标识符，
        # 并将 CMake alias 解析到真实 target 后按实际依赖边校验。
        string(REGEX MATCHALL "[A-Za-z_][A-Za-z0-9_.+-]*(::[A-Za-z_][A-Za-z0-9_.+-]*)*"
                     _expression_tokens "${_link}")
        foreach(_token IN LISTS _expression_tokens)
          if(NOT TARGET "${_token}")
            continue()
          endif()
          get_target_property(_resolved_target "${_token}" ALIASED_TARGET)
          if(NOT _resolved_target)
            set(_resolved_target "${_token}")
          endif()
          if(NOT _resolved_target MATCHES "^panta_")
            continue()
          endif()
          if(_resolved_target IN_LIST _allowed_dependencies
             OR _resolved_target
                MATCHES
                "^panta_(bridge|shell|visualization)(_resources_[0-9]+|_tooling|_other_files|_qml[a-zA-Z0-9_]*|_.*_qml|_copy_(qml|res)|plugin_init)$"
             OR _resolved_target MATCHES
                "^panta_native_app_(qml[a-zA-Z0-9_]*|.*_qml|copy_(qml|res)|other_files|tooling)$")
            continue()
          endif()
          message(
            FATAL_ERROR "条件链接表达式经 alias 解析后违反分层规则：${_source} -> ${_resolved_target}（表达式：${_link}）")
        endforeach()
        continue()
      endif()

      if("${_link}" STREQUAL "${PANTA_FFI_STATIC_LIB}")
        set(_dependency panta_ffi)
      elseif(TARGET "${_link}")
        get_target_property(_dependency "${_link}" ALIASED_TARGET)
        if(NOT _dependency)
          set(_dependency "${_link}")
        endif()
      elseif(_link MATCHES "^panta(::|_)")
        message(FATAL_ERROR "分层检查无法解析项目 target：${_source} -> ${_link}")
      else()
        continue()
      endif()

      if(NOT _dependency MATCHES "^panta_")
        continue()
      endif()

      if(_dependency MATCHES
         "^panta_(bridge|shell|visualization)(_resources_[0-9]+|_tooling|plugin_init)$"
         OR _dependency MATCHES "^panta_(bridge|shell|visualization)_other_files$"
         OR _dependency MATCHES "^panta_(bridge|shell|visualization)_qml[a-zA-Z0-9_]*$"
         OR _dependency MATCHES "^panta_(bridge|shell|visualization)_.*_qml$"
         OR _dependency MATCHES "^panta_(bridge|shell|visualization)_copy_(qml|res)$"
         OR _dependency MATCHES
            "^panta_native_app_(qml[a-zA-Z0-9_]*|.*_qml|copy_(qml|res)|other_files|tooling)$"
         OR _dependency STREQUAL "panta_native_app_tooling")
        continue()
      endif()

      if(NOT _dependency IN_LIST _allowed_dependencies)
        message(FATAL_ERROR "违反 Rust/CMake 分层规则：${_source} -> ${_dependency}（未登记允许边）")
      endif()
    endforeach()
  endforeach()
endfunction()
