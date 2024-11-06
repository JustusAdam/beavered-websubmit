#include "config.hpp"
#include <fstream>
#include <stdexcept>
#include "toml/toml.hpp"

namespace config
{

    std::shared_ptr<Config> Config::from_file(const std::string &filename)
    {
        auto config = std::make_shared<Config>();

        try
        {
            auto data = toml::parse(filename);

            config->class_ = data.find("class").as_string();
            for (auto &admin : data.find("admins").as_vec())
            {
                config->admins.push_back(admin.as_string());
            }
            for (auto &member : data.find("staff").as_vec())
            {
                config->staff.push_back(member.as_string());
            }
            config->template_dir = data.find("template_dir").as_string();
            config->resource_dir = data.find("resource_dir").as_string();
            config->secret = data.find("secret").as_string();
            config->send_emails = data.find("send_emails").as_bool();
            config->prime = data.find("prime").as_bool();
        }
        catch (const std::exception &e)
        {
            throw std::runtime_error("Failed to parse config file: " + std::string(e.what()));
        }

        return config;
    }

} // namespace config