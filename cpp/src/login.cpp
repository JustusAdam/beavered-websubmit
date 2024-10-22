#include "login.hpp"

namespace login {

rocket::Template login(const rocket::State<config::Config>& config) {
    std::unordered_map<std::string, std::string> ctx;
    ctx["CLASS_ID"] = config->class_;
    ctx["parent"] = std::string("layout");
    return rocket::Template::render("login", ctx);
}

} // namespace login