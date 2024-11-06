#pragma once

#include <string>
#include <memory>
#include <vector>
#include "slog/slog.hpp"

namespace config
{

    class Config
    {
    public:
        static std::shared_ptr<Config> from_file(const std::string &filename);

        Config() = default;

        std::string class_;
        std::vector<std::string> admins;
        std::vector<std::string> staff;
        std::string template_dir;
        std::string resource_dir;
        std::string secret;
        bool send_emails;
        bool prime;
    };

} // namespace config