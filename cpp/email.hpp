#pragma once

#include <string>
#include "config.hpp"

namespace email {

void send_email(const config::Config& config, const std::string& to, const std::string& subject, const std::string& body);

} // namespace email