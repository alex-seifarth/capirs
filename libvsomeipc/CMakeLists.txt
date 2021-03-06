# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
cmake_minimum_required(VERSION 3.6)
project(vsomeipc
        VERSION 0.1.0
        LANGUAGES CXX)

include(GenerateExportHeader)
add_library(vsomeipc STATIC vsomeipc.cpp)
generate_export_header(vsomeipc)

target_include_directories(vsomeipc
        PRIVATE . ${CMAKE_CURRENT_BINARY_DIR}
        PUBLIC .
)

set(HEADERS
    ${CMAKE_CURRENT_SOURCE_DIR}/vsomeipc.h
    ${CMAKE_CURRENT_BINARY_DIR}/vsomeipc_export.h
)

set_target_properties(vsomeipc PROPERTIES
    PUBLIC_HEADER "${HEADERS}"
)

find_package(vsomeip3)
target_link_libraries(vsomeipc PRIVATE vsomeip3)

install(TARGETS vsomeipc)