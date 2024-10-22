#include "mysql.hpp"
#include <stdexcept>

namespace mysql {

Pool::Pool(const std::string& connection_string) {}

Connection* Pool::get_conn() {
    // Dummy implementation
    return new Connection();
}

bool Connection::ping() {
    // Dummy implementation
    return true;
}

void Connection::query_drop(const std::string& query) {
    // Dummy implementation
}

Statement Connection::prepare(const std::string& query) {
    // Dummy implementation
    return Statement();
}

Result Statement::execute(const std::vector<Value>& params) {
    // Dummy implementation
    return Result();
}

std::vector<Value> Result::fetch() {
    // Dummy implementation
    return std::vector<Value>();
}

} // namespace mysql