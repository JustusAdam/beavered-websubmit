#include "questions.hpp"
#include <stdexcept>

namespace questions {

rocket::response::Template leclist(
    const apikey::ApiKey& apikey,
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend,
    const rocket::State<std::shared_ptr<config::Config>>& config
) {
    auto bg = backend->lock();
    auto res = bg->prep_exec(
        "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
        std::vector<Value>()
    );
    bg.unlock();

    std::string user = apikey.user;
    bool admin = config->admins.find(user) != config->admins.end();

    std::vector<LectureListEntry> lecs;
    for (const auto& r : res) {
        LectureListEntry entry;
        entry.id = from_value<int>(r[0]);
        entry.label = from_value<std::string>(r[1]);
        entry.num_qs = r[2].is_null() ? 0 : from_value<uint64_t>(r[2]);
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
    const admin::Admin& admin,
    int num,
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend
) {
    auto bg = backend->lock();
    auto res = bg->prep_exec(
        "SELECT * FROM answers WHERE lec = ?",
        std::vector<Value>{Value((uint64_t)num)}
    );
    bg.unlock();

    std::vector<LectureAnswer> answers;
    for (const auto& r : res) {
        LectureAnswer answer;
        answer.id = from_value<uint64_t>(r[2]);
        answer.user = from_value<std::string>(r[0]);
        answer.answer = from_value<std::string>(r[3]);
        if (r[4].get_type() == Value::Type::TIME) {
            answer.time = from_value<std::chrono::system_clock::time_point>(r[4]);
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
    const apikey::ApiKey& apikey,
    int num,
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend
) {
    std::unordered_map<uint64_t, std::string> answers;
    auto bg = backend->lock();
    
    auto answers_res = bg->prep_exec(
        "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
        std::vector<Value>{Value((uint64_t)num), Value(apikey.user)}
    );

    for (const auto& r : answers_res) {
        uint64_t id = from_value<uint64_t>(r[2]);
        std::string atext = from_value<std::string>(r[3]);
        answers[id] = atext;
    }

    auto res = bg->prep_exec(
        "SELECT * FROM questions WHERE lec = ?",
        std::vector<Value>{Value((uint64_t)num)}
    );
    bg.unlock();

    std::vector<LectureQuestion> qs;
    for (const auto& r : res) {
        uint64_t id = from_value<uint64_t>(r[1]);
        LectureQuestion q;
        q.id = id;
        q.prompt = from_value<std::string>(r[2]);
        auto it = answers.find(id);
        if (it != answers.end()) {
            q.answer = it->second;
        }
        qs.push_back(q);
    }

    std::sort(qs.begin(), qs.end(), [](const LectureQuestion& a, const LectureQuestion& b) {
        return a.id < b.id;
    });

    LectureQuestionsContext ctx;
    ctx.lec_id = num;
    ctx.questions = qs;
    ctx.parent = "layout";

    return rocket::response::Template::render("questions", ctx);
}

rocket::response::Redirect questions_submit(
    const apikey::ApiKey& apikey,
    int num,
    const rocket::request::Form<LectureQuestionSubmission>& data,
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend,
    const rocket::State<std::shared_ptr<config::Config>>& config
) {
    auto bg = backend->lock();
    auto res = bg->prep_exec(
        "SELECT COUNT(*) FROM questions WHERE lec = ?",
        std::vector<Value>{Value((uint64_t)num)}
    );
    
    uint64_t count = from_value<uint64_t>(res[0][0]);
    
    if (count >= config->max_questions) {
        return rocket::response::Redirect::to("/questions/" + std::to_string(num));
    }
    
    bg->prep_exec(
        "INSERT INTO questions (lec, qtext) VALUES (?, ?)",
        std::vector<Value>{Value((uint64_t)num), Value(data.question)}
    );
    bg.unlock();
    
    return rocket::response::Redirect::to("/questions/" + std::to_string(num));
}

} // namespace questions