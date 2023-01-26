use crate::admin::Admin;
use crate::apikey::ApiKey;
use crate::backend::{MySqlBackend, Value};
use crate::config::Config;
use crate::email;
use chrono::naive::NaiveDateTime;
use chrono::Local;
use mysql::from_value;
use rocket::form::{Form, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(sensitive))]
#[cfg_attr(not(feature = "v-ann-lib"), dfpp::output_types(LectureAnswer))]
#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<u64, String>,
}

#[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(sensitive))]
#[derive(Serialize)]
pub(crate) struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: u8,
    pub title: String,
    pub presenters: Vec<String>,
    pub questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

#[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(sensitive))]
#[derive(Serialize)]
struct LectureAnswer {
    id: u64,
    lec: u64,
    user: String,
    answer: String,
    time: Option<NaiveDateTime>,
}

#[derive(Serialize)]
struct LectureAnswersContext {
    lec_id: u8,
    answers: Vec<LectureAnswer>,
    parent: &'static str,
}

#[derive(Serialize)]
struct LectureListEntry {
    id: u64,
    label: String,
    num_qs: u64,
    num_answered: u64,
}

#[derive(Serialize)]
struct LectureListContext {
    admin: bool,
    lectures: Vec<LectureListEntry>,
    parent: &'static str,
}

#[get("/")]
pub(crate) fn leclist(
    apikey: ApiKey,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let res = bg.prep_exec(
        "SELECT lectures.id, lectures.label, lec_qcount.qcount \
         FROM lectures \
         LEFT JOIN lec_qcount ON (lectures.id = lec_qcount.lec)",
        vec![],
    );
    drop(bg);

    let user = apikey.user.clone();
    let admin = config.admins.contains(&user);

    let lecs: Vec<_> = res
        .into_iter()
        .map(|r| LectureListEntry {
            id: from_value(r[0].clone()),
            label: from_value(r[1].clone()),
            num_qs: if r[2] == Value::NULL {
                0u64
            } else {
                from_value(r[2].clone())
            },
            num_answered: 0u64,
        })
        .collect();

    let ctx = LectureListContext {
        admin: admin,
        lectures: lecs,
        parent: "layout",
    };

    Template::render("leclist", &ctx)
}

pub enum Either<A,B> {
    Left(A),
    Right(B),
}

#[dfpp::label(source)]
fn get_one_answer(bg: &mut MySqlBackend, user: &str, key: u64) -> LectureAnswer {
    let res = bg.prep_exec("SELECT * FROM answers WHERE email = ? AND num = ?", vec![user.into(), key.into()]);
    res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()),
            lec: from_value(r[1].clone()),
            user: from_value(r[0].clone()),
            answer: from_value(r[3].clone()),
            time: if let Value::Time(..) = r[4] {
                Some(from_value::<NaiveDateTime>(r[4].clone()))
            } else {
                None
            },
        })
        .next()
        .unwrap()
}

#[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(source))]
fn get_answers(bg: &mut MySqlBackend, key: Either<u64, &str>) -> Vec<LectureAnswer> {
    let (where_, key) = match key {
        Either::Left(lec) => ("lec", lec.into()),
        Either::Right(usr) => ("email", usr.into()),
    };
    let res = bg.prep_exec(&format!("SELECT * FROM answers WHERE {where_} = ?"), vec![key]);
    res
        .into_iter()
        .map(|r| LectureAnswer {
            id: from_value(r[2].clone()),
            lec: from_value(r[1].clone()),
            user: from_value(r[0].clone()),
            answer: from_value(r[3].clone()),
            time: if let Value::Time(..) = r[4] {
                Some(from_value::<NaiveDateTime>(r[4].clone()))
            } else {
                None
            },
        })
        .collect()
}

#[get("/<num>")]
pub(crate) fn answers(
    _admin: Admin,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let answers = get_answers(&mut bg, Either::Left(num as u64));
    drop(bg);
    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers,
        parent: "layout",
    };
    Template::render("answers", &ctx)
}

#[get("/<num>")]
pub(crate) fn questions(
    apikey: ApiKey,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    use std::collections::HashMap;

    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();

    let answers_res = bg.prep_exec(
        "SELECT answers.* FROM answers WHERE answers.lec = ? AND answers.email = ?",
        vec![(num as u64).into(), apikey.user.clone().into()],
    );
    let mut answers = HashMap::new();

    for r in answers_res {
        let id: u64 = from_value(r[2].clone());
        let atext: String = from_value(r[3].clone());
        answers.insert(id, atext);
    }
    let res = bg.prep_exec("SELECT * FROM questions WHERE lec = ?", vec![key]);
    drop(bg);
    let mut qs: Vec<_> = res
        .into_iter()
        .map(|r| {
            let id: u64 = from_value(r[1].clone());
            let answer = answers.get(&id).map(|s| s.to_owned());
            LectureQuestion {
                id: id,
                prompt: from_value(r[2].clone()),
                answer: answer,
            }
        })
        .collect();
    qs.sort_by(|a, b| a.id.cmp(&b.id));

    let ctx = LectureQuestionsContext {
        lec_id: num,
        title: "".into(),   // not needed here
        presenters: vec![], // same
        questions: qs,
        parent: "layout",
    };
    Template::render("questions", &ctx)
}

impl LectureAnswer {

    #[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(deletes, arguments = [0]))]
    fn delete_answer(self, bg: &mut MySqlBackend) {
        bg.delete("answers", &[("lec", self.lec.into()), ("q", self.id.into()), ("email", self.user.into())]);
    }
}

impl ApiKey {
    #[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(deletes, arguments = [0]))]
    fn delete_apikey(self, bg: &mut MySqlBackend) {
        bg.delete("users", &[("email", self.user.into())])
    }
}

#[cfg(feature = "edit-del-3-b")]
#[dfpp::analyze]
#[post("/answer/delete/<num>")]
pub(crate) fn delete_answer_handler(apikey: ApiKey, num: u64, backend: &State<Arc<Mutex<MySqlBackend>>>) -> Redirect {
    get_one_answer(&mut backend.lock().unwrap(), &apikey.user, num);
    Redirect::to("/")
}

fn delete_my_answers(bg: &mut MySqlBackend, answers: Vec<LectureAnswer>) {
    for answer in answers {
        answer.delete_answer(bg);
    }
}

#[cfg(feature = "edit-del-3-c")]
#[post("/forget_answers")]
fn delete_my_answers_controller(apikey: ApiKey, backend: &State<Arc<Mutex<MySqlBackend>>>) -> Redirect {
    let mut bg = backend.lock().unwrap();
    let key = apikey.user.as_str();
    let mut answers = get_answers(&mut bg, Either::Right(key));
    for answer in answers {
        answer.delete_answer(&mut bg);
    }
    Redirect::to("/")
}

#[dfpp::analyze]
#[post("/forget")]
pub(crate) fn forget_user(apikey: ApiKey, backend: &State<Arc<Mutex<MySqlBackend>>>) -> Redirect {
    let mut bg = backend.lock().unwrap();
    let key = apikey.user.as_str();
    let mut answers = get_answers(&mut bg, Either::Right(key));

    #[cfg(feature = "edit-del-1-a")]
    apikey.delete_apikey(&mut bg);

    cfg_if! {
        if #[cfg(feature = "edit-del-2-c")] {
            answers.pop().unwrap().delete_answer(&mut bg);
        } else if #[cfg(feature = "edit-del-1-c")] {
            LectureAnswer {
                id: 0,
                lec: 0,
                user: "test@test.com".to_string(),
                answer: "dummy".to_string(),
                time: None,
            }.delete_answer(&mut bg);
        } else if #[cfg(feature = "edit-del-2-a")] {
            answers.into_iter().for_each(|ans| {
                ans.delete_answer(&mut bg);
            });
        } else if #[cfg(feature = "edit-del-3-a")] {
            delete_my_answers(&mut bg, answers);
        } else if #[cfg(feature = "edit-del-3-c")] {
            if apikey.user == "impossible" {
                delete_my_answers_controller(apikey.clone(), backend);
            }
        } else {
            for answer in answers {
                cfg_if! {
                    if #[cfg(any(feature = "edit-del-1-b", feature = "edit-del-3-b"))] {
                    } else if #[cfg(feature = "edit-del-2-b")] {
                        bg.delete("answers", &[("lec", answer.lec.into()), ("q", answer.id.into())]);
                    } else {
                        answer.delete_answer(&mut bg);
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "edit-del-1-a"))]
    apikey.delete_apikey(&mut bg);
    Redirect::to("/")
}

#[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(presenter, return))]
#[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(safe_source, return))]
fn get_presenters(bg: &mut MySqlBackend, num: u8) -> Vec<String> {
    let mut presenter_emails = vec![];
    let presenters_res = bg.prep_exec("SELECT * FROM presenters WHERE lec = ?;", vec![num.into()]);
    for p in presenters_res {
        let email: String = from_value(p[1].clone());
        presenter_emails.push(email);
    }
    presenter_emails
}

#[dfpp::label(bless_safe_source, return)]
fn get_num(num: u8) -> u8 {
	num
}

#[dfpp::label(safe_source_with_bless, return)]
fn get_staff(config: &State<Config>) -> Vec<String> {
	config.staff.clone()
}

#[dfpp::label(safe_source, return)]
fn get_admins(config: &State<Config>) -> Vec<String> {
	config.admins.clone()
}

#[cfg_attr(not(feature = "v-ann-lib"), dfpp::label(scopes, return))]
fn scopes_argument<T: Clone>(field: &T) -> T {
    return field.clone();
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Redirect {
    questions_submit_internal(apikey, num, data, backend, config)
}
#[dfpp::analyze]
pub(crate) fn questions_submit_internal(
    apikey: ApiKey,
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    let vnum: Value = (num as u64).into();
    let ts: Value = Local::now().naive_local().into();

    let mut presenter_emails = get_presenters(&mut bg, num);

    for (id, answer) in &data.answers {
        let rec: Vec<Value> = vec![ 
            scopes_argument(&apikey.user).into(),
            vnum.clone(),
            (*id).into(),
            answer.clone().into(),
            ts.clone(),
        ];
        bg.replace("answers", rec);
    }

    let answer_log = format!(
        "{}",
        data.answers
            .iter()
            .map(|(i, t)| format!("Question {}:\n{}", i, t))
            .collect::<Vec<_>>()
            .join("\n-----\n")
    );
    if config.send_emails {
        let mut recipients = if get_num(num) < 90 {
			get_staff(config)
		} else {
			get_admins(config)
		};

        recipients.append(&mut presenter_emails);

        //println!("");
        email::send(
            bg.log.clone(),
            apikey.user.clone(),
            recipients,
            format!("{} meeting {} questions", config.class, num),
            answer_log,
        )
        .expect("failed to send email");
    }
    //drop(bg);
    //presenter_emails.push("".to_string());
    Redirect::to("/leclist")
}
