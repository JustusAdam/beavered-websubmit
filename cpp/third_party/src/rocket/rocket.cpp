#include <unordered_map>
#include <string>

#include "rocket.hpp"

namespace rocket
{

    void ignite::launch() {}

    namespace response
    {
        Redirect Redirect::to(const std::string &uri)
        {
            // Dummy implementation
            return Redirect();
        }
    }

    namespace fs
    {
        FileServer FileServer::from(const char *path)
        {
            // Dummy implementation
            return FileServer();
        }
    }

    Fairing Template::fairing()
    {
        // Dummy implementation
        return rocket::Fairing();
    }

    Template::Template(std::string name, void *state) : name(name), state(state) {}
}