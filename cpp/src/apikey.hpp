#pragma once

#include <string>
#include "rocket/rocket.hpp"

namespace apikey
{

    class ApiKey
    {
    public:
        std::string key;
        std::string user;
    };

    rocket::outcome<ApiKey> from_request(const rocket::request::Request &request);

} // namespace apikey