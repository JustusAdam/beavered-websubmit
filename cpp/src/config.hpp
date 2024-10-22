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

        const std::string &db_name() const;
        const std::string &smtp_server() const;
        int smtp_port() const;
        const std::string &smtp_user() const;
        const std::string &smtp_pass() const;
        const std::string &smtp_from() const;

        Config() = default;
        std::string class_;

        int max_questions;
        std::vector<std::string> admins;

    private:
        std::string db_name_;
        std::string smtp_server_;
        int smtp_port_;
        std::string smtp_user_;
        std::string smtp_pass_;
        std::string smtp_from_;
    };

} // namespace config