#pragma once

#include <string>

namespace toml {

class value {
    // Placeholder for toml::value
};

value parse(const std::string& filename);

template<typename T>
T find(const value& v, const std::string& key1, const std::string& key2);

} // namespace toml