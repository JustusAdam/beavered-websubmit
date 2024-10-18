#include "toml.hpp"
#include <stdexcept>

namespace toml {

value parse(const std::string& filename) {
    // Dummy implementation
    return value();
}

template<typename T>
T find(const value& v, const std::string& key1, const std::string& key2) {
    // Dummy implementation
    throw std::runtime_error("Not implemented");
}

} // namespace toml