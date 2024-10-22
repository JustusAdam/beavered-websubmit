#pragma once

#include <string>
#include <vector>
#include <memory>
#include "rocket/rocket.hpp"
#include "backend.hpp"
#include "config.hpp"
#include "apikey.hpp"
#include "admin.hpp"

namespace mt
{

    template <typename T>
    struct lock_guard
    {
        std::mutex &m;
        T &t;
        bool needs_unlock;
        lock_guard(std::mutex &m, T &t) : m(m), t(t), needs_unlock(true)
        {
            m.lock();
        }
        ~lock_guard()
        {
            if (needs_unlock)
            {
                m.unlock();
            }
        }

        T &operator*()
        {
            assert(!needs_unlock);
            return t;
        }
        T *operator->()
        {
            assert(!needs_unlock);
            return &t;
        }

        void unlock()
        {
            m.unlock();
            needs_unlock = false;
        }
    };

    template <typename T>
    struct mutex
    {
        std::mutex m;
        T t;

    public:
        lock_guard<T> lock()
        {
            return lock_guard<T>(m, t);
        }
    };

} // namespace mt

template <typename T>
using mutex = mt::mutex<T>;

namespace questions
{

    struct LectureQuestionSubmission
    {
        std::string question;
    };

    struct LectureQuestion
    {
        int id;
        std::string prompt;
        std::optional<std::string> answer;
    };

    struct LectureQuestionsContext
    {
        int lec_id;
        std::string parent;
        std::vector<LectureQuestion> questions;
    };

    struct LectureAnswer
    {
        int id;
        std::string user;
        std::string question;
        std::string answer;
        std::chrono::system_clock::time_point time;
    };

    struct LectureAnswersContext
    {
        int num;
        int lec_id;
        std::chrono::system_clock::time_point time;
        std::vector<LectureAnswer> answers;
        std::string parent;
    };

    struct LectureListEntry
    {
        int id;
        std::string label;
        int num_qs;
        int num_answered;
    };

    struct LectureListContext
    {
        std::string parent;
        bool admin;
        std::vector<LectureListEntry> lectures;
    };

    rocket::response::Template leclist(
        const apikey::ApiKey &apikey,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend,
        const rocket::State<std::shared_ptr<config::Config>> &config);

    rocket::response::Template answers(
        const admin::Admin &admin,
        int num,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend);

    rocket::response::Template questions(
        const apikey::ApiKey &apikey,
        int num,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend);

    rocket::response::Redirect questions_submit(
        const apikey::ApiKey &apikey,
        int num,
        const rocket::request::Form<LectureQuestionSubmission> &data,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend,
        const rocket::State<std::shared_ptr<config::Config>> &config);

} // namespace questions