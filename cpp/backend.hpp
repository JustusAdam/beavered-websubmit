#pragma once

#include <string>
#include <vector>
#include <memory>
#include <unordered_map>
#include "mysql/mysql.hpp"
#include "slog/slog.hpp"

namespace backend {

class MySqlBackend {
public:
    MySqlBackend(const std::string& dbname, std::shared_ptr<slog::Logger> log, bool prime);

    std::vector<std::vector<mysql::Value>> prep_exec(const std::string& sql, const std::vector<mysql::Value>& params);
    void insert(const std::string& table, const std::vector<mysql::Value>& vals);
    void replace(const std::string& table, const std::vector<mysql::Value>& vals);

private:
    void do_insert(const std::string& table, const std::vector<mysql::Value>& vals, bool replace);

    mysql::Pool pool_;
    std::shared_ptr<slog::Logger> log_;
    std::unordered_map<std::string, mysql::Statement> prep_stmts_;
    std::string schema_;
};

} // namespace backend