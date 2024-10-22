#include "mysql.hpp"
#include <stdexcept>

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

} // namespace mysql