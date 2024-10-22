cmake_minimum_required(VERSION 3.10)
project(third_party_libs)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Function to create a shared library
function(add_third_party_library name)
    add_library(${name} SHARED
        src/${name}/${name}.cpp
        src/${name}/${name}.hpp
    )
    target_include_directories(${name} PUBLIC src/${name})
endfunction()

# Add each third-party library
add_third_party_library(mysql)
add_third_party_library(rocket)
add_third_party_library(lettre)
add_third_party_library(slog)
add_third_party_library(serde)
add_third_party_library(toml)