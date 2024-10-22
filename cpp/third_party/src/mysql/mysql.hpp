#pragma once

#include <string>
#include <vector>
#include <chrono>
#include <cassert>

namespace mysql {


class Value {
    ~Value() {}
public:
    // Placeholder for mysql::Value
    enum Type {
        // Placeholder for mysql::Value::Type
        TIME,
        INT,
        STRING,
    };

    const Type get_type() const;

    Value(uint64_t value);

    union {
        uint64_t uint64_value;
        std::string string_value;
    } value;

    Type type;

};

class Connection {
public:
    bool ping();
    void query_drop(const std::string& query);
    class Statement prepare(const std::string& query);
    class Result execute(std::string query, const std::vector<Value>& params);
};

class Pool {
public:
    Pool();
    Pool(const std::string& connection_string);
    class Connection* get_conn();
};

class Statement {
public:
    class Result execute(const std::vector<Value>& params);
};


class result_iterator {
public:
    bool operator!=(const result_iterator& other);
    result_iterator& operator++();
    std::vector<Value> operator*();
};

class Result {
public:
    result_iterator begin();
    result_iterator end();
};

class Error : public std::exception {
public:
    const char* what() const noexcept override { return "MySQL Error"; }
};

template<typename T>
T from_value(const Value& value);

template<>
uint64_t from_value(const Value& value) {
    assert(value.get_type() == Value::Type::INT);
    return value.value.uint64_value;
}

template<>
std::string from_value(const Value& value) {
    assert(value.get_type() == Value::Type::STRING);
    return value.value.string_value;
}

template<>
std::chrono::system_clock::time_point from_value(const Value& value) {
    assert(value.get_type() == Value::Type::TIME);
    return std::chrono::system_clock::from_time_t(value.value.uint64_value);
}


} // namespace mysql