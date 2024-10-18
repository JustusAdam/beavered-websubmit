#pragma once

#include "rocket/rocket.hpp"
#include "backend.hpp"
#include "config.hpp"

namespace login {

rocket::response::Redirect login(const rocket::request::Form<Login>& form, rocket::http::CookieJar& cookies);
rocket::response::Redirect logout(rocket::http::CookieJar& cookies);

} // namespace login