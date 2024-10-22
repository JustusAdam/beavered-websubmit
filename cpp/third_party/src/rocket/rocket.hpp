#pragma once

#include <string>
#include <unordered_map>

namespace rocket {

class Fairing {};

template<typename T>
class outcome {
    std::optional<T> t;
    outcome(std::optional<T> t) : t(t) {}
public:
    static outcome<T> success(T t) { return outcome<T>(std::move(t)); }
};

class ignite {
public:
    template<typename T>
    ignite& manage(T&& state);
    
    template<typename... Routes>
    ignite& mount(const std::string& base, Routes... routes);
    
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

class Template {
private: 
    std::string name;
    std::unordered_map<std::string, std::string> context;
    Template(std::string name, std::unordered_map<std::string, std::string> context) : name(name), context(context) {}
public:
    static rocket::Fairing fairing();
    static Template render(const std::string& template_name, std::unordered_map<std::string, std::string> state);
};
}

using Template = response::Template;

template<typename T>
class State {
private:
    T value;
public:
    T& operator*() { return value; }
    T* operator->() { return &value; }
    const T& operator*() const { return value; }
    const T* operator->() const { return &value; }
};

namespace fs {
class FileServer {
public:
    static FileServer from(const char* path);
};
}

namespace request {
class Request {};

template<typename T>
class Form {
private:
    T value;
};
}


} // namespace rocket