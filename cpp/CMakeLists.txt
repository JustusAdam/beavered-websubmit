cmake_minimum_required(VERSION 3.10)

project(Websubmit VERSION 1.0)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Add all source files
file(GLOB_RECURSE SOURCES 
    "src/*.cpp"
)

# Add all header files
file(GLOB_RECURSE HEADERS
    "src/*.hpp"
)

add_executable(
    websubmit
    ${SOURCES}
)

function(link_my_lib name)
    find_library(${name} NAMES 
        ${name}
        PATHS 
        ${CMAKE_CURRENT_SOURCE_DIR}/third_party/build
    )
    target_link_libraries(websubmit PRIVATE ${name})
endfunction()

link_my_lib(mysql)
link_my_lib(lettre)
link_my_lib(slog)
link_my_lib(serde)
link_my_lib(toml)
link_my_lib(rocket)


target_include_directories(websubmit PRIVATE third_party/src)