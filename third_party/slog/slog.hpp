#pragma once

#include <memory>

namespace slog {

class Logger {
public:
    static Logger root(slog::Discard, slog::o);
};

enum class Level {
    Debug,
    Info,
    Warning,
    Error
};

template<typename... Args>
void log(std::shared_ptr<Logger> logger, Level level, const char* format, Args... args);

template<typename... Args>
void debug(std::shared_ptr<Logger> logger, const char* format, Args... args);

template<typename... Args>
void error(std::shared_ptr<Logger> logger, const char* format, Args... args);

class Discard {};

class o {};

} // namespace slog