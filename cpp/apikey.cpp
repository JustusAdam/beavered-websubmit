#include "apikey.hpp"

namespace apikey {

ApiKey::ApiKey(std::string key) : key_(std::move(key)) {}

const std::string& ApiKey::key() const {
    return key_;
}

rocket::outcome<ApiKey> from_request(const rocket::request::Request& request) {
    // Implementation of from_request
    // This is a placeholder and needs to be properly implemented
    return rocket::outcome<ApiKey>::success(ApiKey("dummy_key"));
}

} // namespace apikey