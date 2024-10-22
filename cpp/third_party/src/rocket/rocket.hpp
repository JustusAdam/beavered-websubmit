#pragma once

#include <string>
#include <unordered_map>

namespace rocket
{

    class Fairing
    {
    };

    template <typename T>
    class outcome
    {
        std::optional<T> t;
        outcome(std::optional<T> t) : t(t) {}

    public:
        static outcome<T> success(T t) { return outcome<T>(std::move(t)); }
    };

    class ignite
    {
    public:
        template <typename T>
        ignite &manage(T &&state)
        {
            return *this;
        };

        template <typename... Routes>
        ignite &mount(const std::string &base, Routes... routes)
        {
            return *this;
        }

        ;

        ignite &attach(Fairing &&fairing)
        {
            return *this;
        };

        void launch();
    };

    namespace http
    {
        class CookieJar
        {
        };
    }

    namespace response
    {
        class Redirect
        {
        public:
            static Redirect to(const std::string &uri);
        };

        class Template
        {
        private:
            std::string name;
            void *state;
            Template(std::string name, void *state);

        public:
            static rocket::Fairing fairing();
            template <typename T>
            static Template render(const std::string &template_name, T &state)
            {
                return Template(template_name, static_cast<void *>(&state));
            };
        };
    }

    using Template = response::Template;

    template <typename T>
    class State
    {
    private:
        T value;

    public:
        T &operator*() { return value; }
        T *operator->() { return &value; }
        const T &operator*() const { return value; }
        const T *operator->() const { return &value; }
    };

    namespace fs
    {
        class FileServer
        {
        public:
            static FileServer from(const char *path);
        };
    }

    namespace request
    {
        class Request
        {
        };
        template <typename T>
        class Form
        {
            T inner;

        public:
            T &operator*()
            {
                return this->inner;
            }

            T *operator->()
            {
                return &this->inner;
            }

            const T *operator->() const
            {
                return &this->inner;
            }
        };

    }

} // namespace rocket