#pragma once

#include <string>
#include <vector>
#include <chrono>
#include <cassert>

namespace mysql
{
    class Value;

    template <typename T>
    T from_value(const Value &value);

    class Value
    {

    public:
        enum Type
        {
            TIME,
            INT,
            STRING,
            EMPTY,
        };
        ~Value();

        const Type get_type() const;

        Value(uint64_t value);
        Value(std::string value);
        Value(const Value &value);

        bool is_null() const;

        template <typename T>
        friend T from_value(const Value &value);

    private:
        union value_t
        {
            uint64_t uint64_value;
            std::string string_value;
            value_t(uint64_t v) : uint64_value(v) {};
            value_t(std::string v) : string_value(v) {};
            ~value_t() {}
        } value;

        Type type;
    };

    class Connection
    {
    public:
        bool ping();
        void query_drop(const std::string &query);
        class Statement prepare(const std::string &query);
        class Result execute(std::string query, const std::vector<Value> &params);
    };

    class Pool
    {
    public:
        Pool();
        Pool(const std::string &connection_string);
        class Connection *get_conn();
    };

    class Statement
    {
    public:
        class Result execute(const std::vector<Value> &params);
    };

    class result_iterator
    {
        using elem_ty = std::vector<Value>;
        elem_ty *pos;
        result_iterator(elem_ty *pos);

    public:
        bool operator!=(const result_iterator &other);
        result_iterator &operator++();
        elem_ty &operator*();

        friend Result;
    };

    class Result
    {
        std::vector<std::vector<Value>> values;

    public:
        result_iterator begin();
        result_iterator end();
    };

    class Error : public std::exception
    {
    public:
        const char *what() const noexcept override { return "MySQL Error"; }
    };

    template <>
    uint64_t from_value(const Value &value)
    {
        assert(value.get_type() == Value::Type::INT);
        return value.value.uint64_value;
    }

    template <>
    std::string from_value(const Value &value)
    {
        assert(value.get_type() == Value::Type::STRING);
        return value.value.string_value;
    }

    template <>
    std::chrono::system_clock::time_point from_value(const Value &value)
    {
        assert(value.get_type() == Value::Type::TIME);
        return std::chrono::system_clock::from_time_t(value.value.uint64_value);
    }

} // namespace mysql