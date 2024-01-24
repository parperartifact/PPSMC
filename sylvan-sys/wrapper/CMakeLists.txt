cmake_minimum_required(VERSION 3.14)

project(wrapper VERSION 1.7.1)
add_library(wrapper src/sylvan-wrapper.c)

enable_language(C CXX)

include_directories(${CMAKE_CURRENT_SOURCE_DIR}/src)

set(SYLVAN "${CMAKE_CURRENT_SOURCE_DIR}/../../sylvan")
add_subdirectory(${SYLVAN} sylvan)
target_include_directories(wrapper PUBLIC ${SYLVAN}/src)
target_link_directories(wrapper PUBLIC ${SYLVAN}/build/src)
target_link_libraries(wrapper PUBLIC sylvan lace)

install(TARGETS wrapper sylvan lace
        LIBRARY DESTINATION lib
        PUBLIC_HEADER DESTINATION include
        )
