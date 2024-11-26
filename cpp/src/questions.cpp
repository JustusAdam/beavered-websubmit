#include "questions.hpp"
#include <stdexcept>
#include <algorithm>
#include <sstream>
#include "email.hpp"

using Value = mysql::Value;

namespace questions
{

    rocket::response::Template leclist(
        const apikey::ApiKey &apikey,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend,
        const rocket::State<std::shared_ptr<config::Config>> &config)
    {
        auto bg = (*backend)->lock();
        auto res = bg->prep_exec(
            "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
            std::vector<mysql::Value>());
        bg.unlock();

        std::string user = apikey.user;
        auto &admins = (*config)->admins;
        bool admin = std::find(admins.begin(), admins.end(), user) != admins.end();

        std::vector<LectureListEntry> lecs;
        for (const auto &r : res)
        {
            LectureListEntry entry;
            entry.id = mysql::from_value<int>(r[0]);
            entry.label = mysql::from_value<std::string>(r[1]);
            entry.num_qs = r[2].is_null() ? 0 : mysql::from_value<uint64_t>(r[2]);
            entry.num_answered = 0;
            lecs.push_back(entry);
        }

        LectureListContext ctx;
        ctx.admin = admin;
        ctx.lectures = lecs;
        ctx.parent = "layout";

        return rocket::response::Template::render("leclist", ctx);
    }

    rocket::response::Template answers(
        const admin::Admin &admin,
        int num,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend)
    {
        auto bg = (*backend)->lock();
        auto res = bg->prep_exec(
            "SELECT * FROM answers WHERE lec = ?",
            std::vector<Value>{Value((uint64_t)num)});
        bg.unlock();

        std::vector<LectureAnswer> answers;
        for (const auto &r : res)
        {
            LectureAnswer answer;
            answer.id = mysql::from_value<uint64_t>(r[2]);
            answer.user = mysql::from_value<std::string>(r[0]);
            answer.answer = mysql::from_value<std::string>(r[3]);
            if (r[4].get_type() == Value::Type::TIME)
            {
                answer.time = mysql::from_value<std::chrono::system_clock::time_point>(r[4]);
            }
            answers.push_back(answer);
        }

        LectureAnswersContext ctx;
        ctx.lec_id = num;
        ctx.answers = answers;
        ctx.parent = "layout";

        return rocket::response::Template::render("answers", ctx);
    }

    rocket::response::Template questions(
        const apikey::ApiKey &apikey,
        int num,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend)
    {
        std::unordered_map<uint64_t, std::string> answers;
        auto bg = (*backend)->lock();

        auto answers_res = bg->prep_exec(
            "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
            std::vector<Value>{Value((uint64_t)num), Value(apikey.user)});

        for (const auto &r : answers_res)
        {
            uint64_t id = mysql::from_value<uint64_t>(r[2]);
            std::string atext = mysql::from_value<std::string>(r[3]);
            answers[id] = atext;
        }

        auto res = bg->prep_exec(
            "SELECT * FROM questions WHERE lec = ?",
            std::vector<Value>{Value((uint64_t)num)});
        bg.unlock();

        std::vector<LectureQuestion> qs;
        for (const auto &r : res)
        {
            uint64_t id = mysql::from_value<uint64_t>(r[1]);
            LectureQuestion q;
            q.id = id;
            q.prompt = mysql::from_value<std::string>(r[2]);
            auto it = answers.find(id);
            if (it != answers.end())
            {
                q.answer = it->second;
            }
            qs.push_back(q);
        }

        std::sort(qs.begin(), qs.end(), [](const LectureQuestion &a, const LectureQuestion &b)
                  { return a.id < b.id; });

        LectureQuestionsContext ctx;
        ctx.lec_id = num;
        ctx.questions = qs;
        ctx.parent = "layout";

        return rocket::response::Template::render("questions", ctx);
    }

    rocket::response::Redirect questions_submit(
        const apikey::ApiKey &apikey,
        int num,
        const rocket::request::Form<LectureQuestionSubmission> &data,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend,
        const rocket::State<std::shared_ptr<config::Config>> &config)
    {
        auto bg = (*backend)->lock();

        mysql::Value vnum = Value((uint64_t)num);
        mysql::Value ts = Value(std::chrono::system_clock::now());

        for (const auto &elem : data->answers)
        {
            auto rec = std::vector<Value>{Value(apikey.user), vnum, Value(elem.first), Value(elem.second), ts};
            bg->replace("answers", rec);
        }

        std::stringstream answer_log;

        for (const auto &elem : data->answers)
        {
            answer_log << "Question " << elem.first
                       << ": " << std::endl
                       << elem.second << std::endl;
        }

        auto &cfg = *config;

        if (cfg->send_emails)
        {
            std::vector<std::string> recipients;
            if (num < 90)
            {
                recipients = cfg->staff;
            }
            else
            {
                recipients = cfg->admins;
            };

            email::send(
                apikey.user,
                recipients,
                "Lecture " + std::to_string(num) + " Answers",
                answer_log.str());
        }

        return rocket::response::Redirect::to("/leclist");
    }

    std::vector<LectureAnswer> get_answers(
        std::string user,
        const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend)
    {
        auto bg = (*backend)->lock();
        auto res = bg->prep_exec(
            "SELECT * FROM answers WHERE email = ?",
            std::vector<Value>{Value(user)});
        bg.unlock();

        std::vector<LectureAnswer> answers;
        for (const auto &r : res)
        {
            LectureAnswer answer;
            answer.id = mysql::from_value<uint64_t>(r[2]);
            answer.user = mysql::from_value<std::string>(r[0]);
            answer.answer = mysql::from_value<std::string>(r[3]);
            if (r[4].get_type() == Value::Type::TIME)
            {
                answer.time = mysql::from_value<std::chrono::system_clock::time_point>(r[4]);
            }
            answers.push_back(answer);
        }

        return answers;
    }

    rocket::response::Redirect forget_user(apikey::ApiKey apikey, const rocket::State<std::shared_ptr<mutex<backend::MySqlBackend>>> &backend)
    {
        auto bg = (*backend)->lock();

        std::string key = apikey.user;
        auto answers = get_answers(key, backend);

        for (const auto &answer : answers)
        {
            bg->delete_("answers", std::vector<std::string>{"id"}, std::vector{Value(answer.id)});
        }
    }
} // namespace questions