#pragma once

#include <string>

namespace args {

struct Args {
    std::string config;
};

Args parse_args(int argc, char* argv[]);

} // namespace args