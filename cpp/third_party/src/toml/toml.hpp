#pragma once

#include <string>
#include <variant>
#include <unordered_map>
#include <vector>
#include <cstddef>
#include <memory>

namespace toml
{

    class value
    {
        using key_t = std::string;
        using map_value = std::shared_ptr<std::unordered_map<key_t, value>>;
        using arr_value = std::vector<value>;
        std::variant<uint64_t, std::string, map_value, nullptr_t, arr_value, bool> value_f;

    public:
        value &find(std::string key);
        uint64_t as_int();
        std::vector<value> as_vec();
        std::string as_string();
        bool as_bool();
        bool is_null();
    };
    value parse(const std::string &filename);
} // namespace toml