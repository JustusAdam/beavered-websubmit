#pragma once

#include <string>

namespace rocket {

class ignite {
public:
    template<typename T>
    ignite& manage(T&& state);
    
    template<typename... Routes>
    ignite& mount(const char* base, Routes... routes);
    
    template<typename Fairing>
    ignite& attach(Fairing&& fairing);
    
    void launch();
};

namespace http {
class CookieJar {};
}

namespace response {
class Redirect {
public:
    static Redirect to(const std::string& uri);
};
}

class State {
public:
    template<typename T>
    const T& inner() const;
};

namespace fs {
class FileServer {
public:
    static FileServer from(const char* path);
};
}

} // namespace rocket

namespace rocket_dyn_templates {
class Template {
public:
    static rocket::Fairing fairing();
};
}