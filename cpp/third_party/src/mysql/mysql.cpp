#include "mysql.hpp"
#include <stdexcept>
#include <cstring>

namespace mysql
{

    Value::~Value()
    {
        switch (type)
        {
        case Value::Type::STRING:
            delete &value.string_value;
            break;
        default:
            break;
        }
    }

    Value::Value(uint64_t value) : type(Value::Type::INT), value(value) {}
    Value::Value(std::string v) : type(Value::Type::INT), value(v) {}
    Value::Value(std::chrono::system_clock::time_point v) : type(Value::Type::TIME), value(v.time_since_epoch().count()) {}
    Value::Value(const Value &value) : type(value.type), value(0)
    {
        if (value.type == Type::STRING)
        {
            value_t new_val(value.value.string_value);
            memcpy(&this->value, &new_val, sizeof(value_t));
        }
        else
        {
            memcpy(&this->value, &value, sizeof(value_t));
        }
    }

    bool Value::is_null() const
    {
        return type == Value::Type::EMPTY;
    }

    const Value::Type Value::get_type() const
    {
        return type;
    }

    Pool::Pool(const std::string &connection_string) {}

    Pool::Pool() {}

    Connection *Pool::get_conn()
    {
        // Dummy implementation
        return new Connection();
    }

    bool Connection::ping()
    {
        // Dummy implementation
        return true;
    }

    void Connection::query_drop(const std::string &query)
    {
        // Dummy implementation
    }

    Statement Connection::prepare(const std::string &query)
    {
        // Dummy implementation
        return Statement();
    }

    Result Statement::execute(const std::vector<Value> &params)
    {
        // Dummy implementation
        return Result();
    }

    Result Connection::execute(const std::string &query, const std::vector<Value> &params)
    {
        return this->prepare(query).execute(params);
    }

    result_iterator::result_iterator(result_iterator::elem_ty *pos) : pos(pos) {}

    bool result_iterator::operator!=(const result_iterator &other)
    {
        return false;
    }

    result_iterator &result_iterator::operator++()
    {
        pos++;
        return *this;
    }

    result_iterator::elem_ty &result_iterator::operator*()
    {
        return *pos;
    }

    result_iterator Result::begin()
    {
        return result_iterator(&*values.begin());
    }

    result_iterator Result::end()
    {
        return result_iterator(&*values.end());
    }

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

    template <>
    int from_value(const Value &value)
    {
        assert(value.get_type() == Value::Type::INT);
        return value.value.uint64_value;
    }
} // namespace mysql