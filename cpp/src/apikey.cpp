#include "apikey.hpp"

namespace apikey
{

    rocket::outcome<ApiKey> from_request(const rocket::request::Request &request)
    {
        // Implementation of from_request
        // This is a placeholder and needs to be properly implemented
        ApiKey apikey;
        return rocket::outcome<ApiKey>::success(apikey);
    }

} // namespace apikey