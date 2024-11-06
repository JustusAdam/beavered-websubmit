#include "toml.hpp"
#include <variant>
#include <string>
#include <vector>

namespace toml
{

    value parse(const std::string &filename)
    {
        // Dummy implementation
        return value();
    }

    value value::find(std::string key)
    {
        return std::get<value::map_value>(this->value_f).at(key);
    }
    uint64_t value::as_int()
    {
        return std::get<uint64_t>(this->value_f);
    }
    std::vector<value> value::as_vec()
    {
        return std::get<value::arr_value>(this->value_f);
    }
    std::string value::as_string()
    {
        return std::get<std::string>(this->value_f);
    }
    bool value::as_bool()
    {
        return std::get<bool>(this->value_f);
    }
    bool value::is_null()
    {
        return std::holds_alternative<nullptr_t>(this->value_f);
    }

} // namespace toml