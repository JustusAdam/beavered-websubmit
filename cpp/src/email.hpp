#pragma once

#include <string>
#include "config.hpp"

namespace email
{

    void send(const std::string &sender, const std::vector<std::string> &to, const std::string &subject, const std::string &body);

} // namespace email