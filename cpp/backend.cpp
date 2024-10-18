#include "backend.hpp"
#include <stdexcept>

namespace backend {

MySqlBackend::MySqlBackend(const std::string& dbname, std::shared_ptr<slog::Logger> log, bool prime)
    : log_(log ? std::move(log) : std::make_shared<slog::Logger>(slog::Logger::root(slog::Discard, o!()))) {
    try {
        std::string connection_string = "mysql://root:password@127.0.0.1/" + dbname;
        pool_ = mysql::Pool::new(connection_string);

        // Check if connection is successful
        auto conn = pool_->get_conn();
        assert(conn->ping());

        // Read schema
        std::ifstream schema_file("src/schema.sql");
        schema_ = std::string((std::istreambuf_iterator<char>(schema_file)), std::istreambuf_iterator<char>());

        if (prime) {
            conn->query_drop("DROP DATABASE IF EXISTS " + dbname);
            conn->query_drop("CREATE DATABASE " + dbname);

            // Re-establish connection to the new database
            pool_ = mysql::Pool::new(connection_string);
            conn = pool_->get_conn();

            // Execute schema
            std::istringstream schema_stream(schema_);
            std::string line;
            while (std::getline(schema_stream, line)) {
                if (!line.empty() && line.substr(0, 2) != "--") {
                    conn->query_drop(line);
                }
            }
        }

        slog::debug(log_, "Connected to MySQL DB and initialized schema {}", dbname);
    } catch (const std::exception& e) {
        slog::error(log_, "Failed to initialize MySQL backend: {}", e.what());
        throw std::runtime_error("Failed to initialize MySQL backend: " + std::string(e.what()));
    }
}

std::vector<std::vector<mysql::Value>> MySqlBackend::prep_exec(const std::string& sql, const std::vector<mysql::Value>& params) {
    try {
        auto conn = pool_->get_conn();
        mysql::Statement stmt;
        
        auto it = prep_stmts_.find(sql);
        if (it == prep_stmts_.end()) {
            stmt = conn->prepare(sql);
            prep_stmts_.insert({sql, stmt});
        } else {
            stmt = it->second;
        }

        auto result = stmt.execute(params);
        std::vector<std::vector<mysql::Value>> rows;
        while (auto row = result.fetch()) {
            rows.push_back(row);
        }
        slog::debug(log_, "Executed query {} with params {:?}", sql, params);
        return rows;
    } catch (const mysql::Error& e) {
        slog::error(log_, "MySQL error: {}", e.what());
        throw;
    } catch (const std::exception& e) {
        slog::error(log_, "Failed to execute query: {}", e.what());
        throw std::runtime_error("Failed to execute query: " + std::string(e.what()));
    }
}

void MySqlBackend::do_insert(const std::string& table, const std::vector<mysql::Value>& vals, bool replace) {
    std::string op = replace ? "REPLACE" : "INSERT";
    std::string placeholders(vals.size(), '?');
    std::replace(placeholders.begin(), placeholders.end(), '?', ',');
    placeholders.pop_back();
    std::string query = op + " INTO " + table + " VALUES (" + placeholders + ")";
    
    slog::debug(log_, "Executing insert query {} for row {:?}", query, vals);
    
    try {
        auto conn = pool_->get_conn();
        conn->execute(query, vals);
    } catch (const mysql::Error& e) {
        slog::error(log_, "MySQL error: Failed to insert into {}, query {}: {}", table, query, e.what());
        throw;
    } catch (const std::exception& e) {
        slog::error(log_, "Failed to insert into {}, query {}: {}", table, query, e.what());
        throw std::runtime_error("Failed to insert: " + std::string(e.what()));
    }
}

void MySqlBackend::insert(const std::string& table, const std::vector<mysql::Value>& vals) {
    do_insert(table, vals, false);
}

void MySqlBackend::replace(const std::string& table, const std::vector<mysql::Value>& vals) {
    do_insert(table, vals, true);
}

} // namespace backend