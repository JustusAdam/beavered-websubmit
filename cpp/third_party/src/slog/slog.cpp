#include "slog.hpp"
#include <iostream>

namespace slog
{

    Logger Logger::root(slog::Discard, slog::o)
    {
        // Dummy implementation
        return Logger();
    }

} // namespace slog