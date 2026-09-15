# SPDX-License-Identifier: GPL-3.0-or-later
# Socle Rust / Qt 6 de MMdedit — chaîne de fabrication à vide (fenêtre minimale).
cmake_minimum_required(VERSION 3.24)
project(mmdedit VERSION 0.3.0 LANGUAGES CXX)

if(CMAKE_CXX_COMPILER_ID STREQUAL "MSVC")
  set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
endif()

set(CMAKE_AUTOMOC ON)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 6.4 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2)
if(NOT ANDROID)
  find_package(Qt6 REQUIRED COMPONENTS QmlImportScanner)
endif()
qt_standard_project_setup()

if(MSVC)
  set_property(TARGET Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick Qt6::QuickControls2
               PROPERTY MAP_IMPORTED_CONFIG_DEBUG "RELEASE")
endif()

if(ANDROID)
  if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(Rust_CARGO_TARGET aarch64-linux-android)
  elseif(ANDROID_ABI STREQUAL "x86_64")
    set(Rust_CARGO_TARGET x86_64-linux-android)
  endif()
endif()

find_package(CxxQt QUIET)
if(NOT CxxQt_FOUND)
  include(FetchContent)
  FetchContent_Declare(CxxQt
    GIT_REPOSITORY https://github.com/kdab/cxx-qt-cmake.git
    GIT_TAG 0.10.0)
  FetchContent_MakeAvailable(CxxQt)
endif()

cxx_qt_import_crate(
  MANIFEST_PATH core/Cargo.toml
  CRATES mmdedit_core
  LOCKED
  QT_MODULES Qt::Core Qt::Gui Qt::Qml Qt::Quick Qt::QuickControls2)

cxx_qt_import_qml_module(mmdedit_qml_module
  URI "fr.mmedia.mmdedit"
  SOURCE_CRATE mmdedit_core)

qt_add_executable(mmdedit cpp/main.cpp)
target_link_libraries(mmdedit PRIVATE mmdedit_qml_module Qt::Quick Qt::QuickControls2)
target_compile_definitions(mmdedit PRIVATE MMDEDIT_VERSION="${PROJECT_VERSION}")

set_target_properties(mmdedit PROPERTIES
  WIN32_EXECUTABLE ON
  MACOSX_BUNDLE ON
  MACOSX_BUNDLE_GUI_IDENTIFIER fr.mmedia.mmdedit
  MACOSX_BUNDLE_BUNDLE_NAME MMdedit
  MACOSX_BUNDLE_BUNDLE_VERSION ${PROJECT_VERSION}
  MACOSX_BUNDLE_SHORT_VERSION_STRING ${PROJECT_VERSION}
  QT_ANDROID_PACKAGE_NAME fr.mmedia.mmdedit
  QT_ANDROID_APP_NAME MMdedit
  QT_ANDROID_VERSION_NAME ${PROJECT_VERSION})

if(NOT ANDROID)
  qt_import_qml_plugins(mmdedit)
endif()

if(NOT ANDROID)
  install(TARGETS mmdedit
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR})
endif()

enable_testing()
add_test(NAME smoke COMMAND mmdedit --smoke)
set_tests_properties(smoke PROPERTIES ENVIRONMENT "QT_QPA_PLATFORM=offscreen")
