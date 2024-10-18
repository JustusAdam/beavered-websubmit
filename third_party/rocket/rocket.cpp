#include "rocket.hpp"

namespace rocket {

ignite& ignite::launch() {
    // Dummy implementation
    return *this;
}

namespace response {
Redirect Redirect::to(const std::string& uri) {
    // Dummy implementation
    return Redirect();
}
}

namespace fs {
FileServer FileServer::from(const char* path) {
    // Dummy implementation
    return FileServer();
}
}

} // namespace rocket

namespace rocket_dyn_templates {
rocket::Fairing Template::fairing() {
    // Dummy implementation
    return rocket::Fairing();
}
}