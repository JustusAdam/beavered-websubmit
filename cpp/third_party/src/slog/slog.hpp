#pragma once

#include <memory>

namespace slog
{

    class Discard
    {
    };

    class o
    {
    };

    class Logger
    {
    public:
        static Logger root(slog::Discard, slog::o);
    };

    enum class Level
    {
        Debug,
        Info,
        Warning,
        Error
    };

    template <typename... Args>
    void log(std::shared_ptr<Logger> logger, Level level, const char *format, Args... args)
    {
        // left empty for now, unsure if this is important
    }

    template <typename... Args>
    void debug(std::shared_ptr<Logger> logger, const char *format, Args... args)
    {
        log(logger, Level::Debug, format, args...);
    }

    template <typename... Args>
    void error(std::shared_ptr<Logger> logger, const char *format, Args... args)
    {
        log(logger, Level::Error, format, args...);
    }

} // namespace slog