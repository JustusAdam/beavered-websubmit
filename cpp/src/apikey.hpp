#pragma once

#include <string>
#include "rocket/rocket.hpp"

namespace apikey {

class ApiKey {
public:
    explicit ApiKey(std::string key);
    const std::string& key() const;

private:
    std::string key_;
};

rocket::outcome<ApiKey> from_request(const rocket::request::Request& request);

} // namespace apikey