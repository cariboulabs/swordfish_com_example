project(swordfish_com_cpp)
cmake_minimum_required(VERSION 3.0)
set(CMAKE_CXX_STANDARD 17)
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/../wrappers/cpp)

find_library(SWORDFISH_COM_LIB swordfish_com
    HINTS
    ${CMAKE_CURRENT_SOURCE_DIR}/../wrappers/cpp/target
)

#ensure that the library is found
if(NOT SWORDFISH_COM_LIB)
    message(FATAL_ERROR "swordfish_com library not found")
endif()

add_executable(main main.cpp)
target_link_libraries(main PRIVATE ${SWORDFISH_COM_LIB})
