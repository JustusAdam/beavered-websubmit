cmake_minimum_required(VERSION 3.10)

project(ThirdPartyLibs VERSION 1.0)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED True)

# Add all source files
file(GLOB_RECURSE SOURCES 
    "*.cpp"
)

# Add all header files
file(GLOB_RECURSE HEADERS
    "*.hpp"
)

# Create the shared library
add_library(third_party_libs SHARED ${SOURCES} ${HEADERS})

# Set the output name of the library
set_target_properties(third_party_libs PROPERTIES OUTPUT_NAME "third_party_libs")

# Specify include directories
target_include_directories(third_party_libs PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# Install the library
install(TARGETS third_party_libs
    LIBRARY DESTINATION lib
    PUBLIC_HEADER DESTINATION include
)