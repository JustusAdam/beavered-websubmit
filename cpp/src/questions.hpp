#pragma once

#include <string>
#include <vector>
#include <memory>
#include "rocket/rocket.hpp"
#include "backend.hpp"
#include "config.hpp"
#include "apikey.hpp"
#include "admin.hpp"

namespace mt {

template<typename T>
struct lock_guard {
    std::mutex& m;
    T& t;
    lock_guard(std::mutex& m, T& t) : m(m), t(t) {
        m.lock();
    }
    ~lock_guard() { m.unlock(); }

    T& operator*() { return t; }
    T* operator->() { return &t; }
};

template<typename T>
struct mutex {
    std::mutex m;
    T t;

    lock_guard<T> lock() {
        return lock_guard<T>(m, t);
    }
};

} // namespace mt

template<typename T>
using mutex = mt::mutex<T>;

namespace questions {

struct LectureQuestionSubmission {
    std::string question;
};

struct LectureQuestion {
    int id;
    std::string question;
    std::string asker;
};

struct LectureQuestionsContext {
    int num;
    std::vector<LectureQuestion> questions;
};

struct LectureAnswer {
    int id;
    int user;
    std::string question;
    std::string answer;
    std::chrono::system_clock::time_point time;
};

struct LectureAnswersContext {
    int num;
    int lec_id;
    std::chrono::system_clock::time_point time;
    std::vector<LectureAnswer> answers;
    std::string parent;
};

struct LectureListEntry {
    int num;
    std::string date;
    int questions;
};

struct LectureListContext {
    std::vector<LectureListEntry> lectures;
};

rocket::response::Template leclist(
    const apikey::ApiKey& apikey,
    const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>>& backend,
    const rocket::State<std::shared_ptr<config::Config>>& config
);

rocket::response::Template answers(
    const admin::Admin& admin,
    int num,
    const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>>& backend
);

rocket::response::Template questions(
    const apikey::ApiKey& apikey,
    int num,
    const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>>& backend
);

rocket::response::Redirect questions_submit(
    const apikey::ApiKey& apikey,
    int num,
    const rocket::request::Form<LectureQuestionSubmission>& data,
    const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>>& backend,
    const rocket::State<std::shared_ptr<config::Config>>& config
);

} // namespace questions