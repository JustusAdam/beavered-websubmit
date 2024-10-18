#pragma once

#include <string>
#include <vector>
#include <memory>
#include "rocket/rocket.hpp"
#include "backend.hpp"
#include "config.hpp"
#include "apikey.hpp"
#include "admin.hpp"

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
    std::string question;
    std::string answer;
};

struct LectureAnswersContext {
    int num;
    std::vector<LectureAnswer> answers;
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
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend,
    const rocket::State<std::shared_ptr<config::Config>>& config
);

rocket::response::Template answers(
    const admin::Admin& admin,
    int num,
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend
);

rocket::response::Template questions(
    const apikey::ApiKey& apikey,
    int num,
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend
);

rocket::response::Redirect questions_submit(
    const apikey::ApiKey& apikey,
    int num,
    const rocket::request::Form<LectureQuestionSubmission>& data,
    const rocket::State<std::shared_ptr<std::mutex<backend::MySqlBackend>>>& backend,
    const rocket::State<std::shared_ptr<config::Config>>& config
);

} // namespace questions