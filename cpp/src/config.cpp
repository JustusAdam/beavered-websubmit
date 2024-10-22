#include "config.hpp"
#include <fstream>
#include <stdexcept>
#include "toml/toml.hpp"

namespace config {

std::shared_ptr<Config> Config::from_file(const std::string& filename) {
    auto config = std::make_shared<Config>();
    
    try {
        auto data = toml::parse(filename);
        
        config->db_name_ = toml::find<std::string>(data, "db", "name");
        config->smtp_server_ = toml::find<std::string>(data, "smtp", "server");
        config->smtp_port_ = toml::find<int>(data, "smtp", "port");
        config->smtp_user_ = toml::find<std::string>(data, "smtp", "user");
        config->smtp_pass_ = toml::find<std::string>(data, "smtp", "pass");
        config->smtp_from_ = toml::find<std::string>(data, "smtp", "from");
    } catch (const std::exception& e) {
        throw std::runtime_error("Failed to parse config file: " + std::string(e.what()));
    }

    return config;
}

const std::string& Config::db_name() const { return db_name_; }
const std::string& Config::smtp_server() const { return smtp_server_; }
int Config::smtp_port() const { return smtp_port_; }
const std::string& Config::smtp_user() const { return smtp_user_; }
const std::string& Config::smtp_pass() const { return smtp_pass_; }
const std::string& Config::smtp_from() const { return smtp_from_; }

} // namespace config