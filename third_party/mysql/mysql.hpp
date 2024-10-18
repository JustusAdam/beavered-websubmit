#pragma once

#include <string>
#include <vector>

namespace mysql {

class Pool {
public:
    static Pool* new(const std::string& connection_string);
    class Connection* get_conn();
};

class Connection {
public:
    bool ping();
    void query_drop(const std::string& query);
    class Statement prepare(const std::string& query);
};

class Statement {
public:
    class Result execute(const std::vector<Value>& params);
};

class Result {
public:
    std::vector<Value> fetch();
};

class Value {
    // Placeholder for mysql::Value
};

class Error : public std::exception {
public:
    const char* what() const noexcept override { return "MySQL Error"; }
};

} // namespace mysql