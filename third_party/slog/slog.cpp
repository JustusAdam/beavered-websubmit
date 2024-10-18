#include "slog.hpp"
#include <iostream>

namespace slog {

Logger Logger::root(slog::Discard, slog::o) {
    // Dummy implementation
    return Logger();
}

template<typename... Args>
void log(std::shared_ptr<Logger> logger, Level level, const char* format, Args... args) {
    // Dummy implementation
    std::cout << "Log: " << format << std::endl;
}

template<typename... Args>
void debug(std::shared_ptr<Logger> logger, const char* format, Args... args) {
    // Dummy implementation
    std::cout << "Debug: " << format << std::endl;
}

template<typename... Args>
void error(std::shared_ptr<Logger> logger, const char* format, Args... args) {
    // Dummy implementation
    std::cout << "Error: " << format << std::endl;
}

} // namespace slog