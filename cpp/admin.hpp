#pragma once

#include <string>
#include "rocket/rocket.hpp"

namespace admin {

class Admin {
public:
    explicit Admin(std::string username);
    const std::string& username() const;

private:
    std::string username_;
};

rocket::outcome<Admin> from_request(const rocket::request::Request& request);

} // namespace admin