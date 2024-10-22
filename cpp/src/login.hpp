#pragma once

#include "rocket/rocket.hpp"
#include "backend.hpp"
#include "config.hpp"

namespace login {

rocket::Template login(const rocket::State<config::Config>& config);

} // namespace login