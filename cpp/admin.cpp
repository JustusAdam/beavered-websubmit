#include "admin.hpp"

namespace admin {

Admin::Admin(std::string username) : username_(std::move(username)) {}

const std::string& Admin::username() const {
    return username_;
}

rocket::outcome<Admin> from_request(const rocket::request::Request& request) {
    // Implementation of from_request
    // This is a placeholder and needs to be properly implemented
    return rocket::outcome<Admin>::success(Admin("admin"));
}

} // namespace admin