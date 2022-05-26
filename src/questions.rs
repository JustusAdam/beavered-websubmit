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
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

extern crate beaver_derive;

use beaver::policy::PoliciedString;
use beaver::policy::Policied;
use beaver::generic_policied::*;

//pub(crate) enum LectureQuestionFormError {
//   Invalid,
//}

#[derive(Clone, Serialize, Deserialize)]
struct AnswerPolicy {
    student_id: String,
}



#[typetag::serde]
impl beaver::policy::Policy for AnswerPolicy {
    fn check(&self, ctxt: &beaver::filter::Context) -> Result<(), beaver::policy::PolicyError> {
        match ctxt {
            beaver::filter::Context::CustomContext(any) if any.is::<Admin>() => Ok(()),
            beaver::filter::Context::KVContext(m) if (m.get("user") == Some(&self.student_id) && m.get("method") == Some(&"website".to_string())) || (m.get("method") == Some(&"email-notify".to_string()) && m.get("role") == Some(&"staff".to_string())) => {
                Ok(())
            }
            _ => Err(beaver::policy::PolicyError { message: "Failed export check".to_string()})
        }
    }
    fn merge(&self, other: &Box<dyn beaver::policy::Policy>) -> Result<Box<dyn beaver::policy::Policy>, beaver::policy::PolicyError> {
        Ok(Box::new(beaver::policy::MergePolicy::make( 
            Box::new(self.clone()),
            other.clone(),
        ))) 
    }
}


#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    answers: HashMap<u64, String>,
}

#[derive(Serialize, Clone)]
pub(crate) struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

trait GPoliciedLectureQuestionExt {
    fn get_id(&self) -> &u64;
    fn get_prompt(&self) -> &String;
    fn get_answer(&self) -> GPolicied<&Option<String>>;
}

impl GPoliciedLectureQuestionExt for GPolicied<LectureQuestion> {
    fn get_id(&self) -> &u64 {
        &self.unsafe_borrow_inner().id
    }
    fn get_prompt(&self) -> &String {
        &self.unsafe_borrow_inner().prompt
    }
    fn get_answer(&self) -> GPolicied<&Option<String>> {
        GPolicied::make(&self.unsafe_borrow_inner().answer, self.get_policy().clone())
    }
}

#[derive(Serialize)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: u8,
    pub questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

#[derive(Serialize, Clone, Deserialize)]
struct LectureAnswer {
    id: u64,
    user: String,
    answer: String,
    time: Option<NaiveDateTime>,
}

trait GPoliciedLectureAnswerExt {
    fn get_id(&self) -> &u64;
    fn get_user(&self) -> &String;
    fn get_answer(&self) -> GPolicied<&String>;
    fn get_time(&self) -> GPolicied<&Option<NaiveDateTime>>;
}

impl GPoliciedLectureAnswerExt for GPolicied<LectureAnswer> {
    fn get_id(&self) -> &u64 {
        &self.unsafe_borrow_inner().id
    }
    fn get_user(&self) -> &String {
        &self.unsafe_borrow_inner().user
    }
    fn get_answer(&self) -> GPolicied<&String> {
        (&self.unsafe_borrow_inner().answer).policied_with(self.get_policy().clone())
    }
    fn get_time(&self) -> GPolicied<&Option<NaiveDateTime>> {
        (&self.unsafe_borrow_inner().time).policied_with(self.get_policy().clone())
    }
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
    let res = bg.query_exec("leclist", vec![]);//vec![(0 as u64).into()]);
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

#[get("/<num>")]
pub(crate) fn answers(
    _admin: Admin,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();
    let answers = bg.query_exec_policied("answers_by_lec", vec![key], 
        |r| LectureAnswer {
                id: from_value(r[2].clone()),
                user: from_value(r[0].clone()),
                answer: from_value(r[3].clone()),
                time: if let Value::Time(..) = r[4] {
                    Some(from_value::<NaiveDateTime>(r[4].clone()))
                } else {
                    None
                },
            }
        );
    drop(bg);

    let ctx = LectureAnswersContext {
        lec_id: num,
        answers: answers.externalize_policy().export_check(&beaver::filter::Context::CustomContext(Box::new(_admin))).unwrap(),
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

    let answers_res = bg.query_exec_policied(
        "my_answers_for_lec",
        vec![(num as u64).into(), apikey.user.clone().into()],
        |e| 
            (from_value(e[2].clone()),
            from_value(e[3].clone())),
    );
    let mut answers = HashMap::new().policied();

    for r in answers_res {
        answers.insert_kv(r);
    }
    let res = bg.query_exec("qs_by_lec", vec![key]);
    drop(bg);
    let mut qs: Vec<_> = res
        .into_iter()
        .map(|r| {
            let id: u64 = from_value(r[1].clone());
            let answer = answers.get(&id).map(|p| p.map(|s: &String| s.to_owned()));
            GPolicied::make_default(
                |answer|
                    LectureQuestion {
                        id: id,
                        prompt: from_value(r[2].clone()),
                        answer: answer,
                    }
            ).apply(answer.externalize_policy())
        })
        .collect();
    qs.sort_by(|a, b| a.get_id().cmp(&b.get_id()));

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: qs.externalize_policy().export_check(&kv_ctx!("user" => apikey.user.clone(), "method" => "website")).unwrap(),
        parent: "layout",
    };
    Template::render("questions", &ctx)
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    config: &State<Config>,
) -> Redirect {
    let mut bg = backend.lock().unwrap();
    let vnum: Value = (num as u64).into();
    let ts: Value = Local::now().naive_local().into();
    let data = data.policied_with(Box::new(AnswerPolicy { student_id: apikey.user.clone().into() }));
    let answers : HashMap<u64, GPolicied<String>> = data.map(|d| d.into_inner().answers).internalize_policy_2_1();

    for (id, answer) in &answers {
        let (answer, policy) = answer.unsafe_borrow_decompose();
        let rec: Vec<Value> = vec![
            apikey.user.clone().into(),
            vnum.clone(),
            (*id).into(),
            answer.clone().into(),
            ts.clone(),
        ];
        bg.insert_or_update_policied(
            "answers",
            rec,
            vec![(3, answer.clone().into()), (4, ts.clone())],
            policy.as_ref()
        );
    }

    let answer_log =
        answers.iter()
            .map(|(i, t)| t.as_ref().map(|t| format!("Question {}:\n{}", i, t)))
            .collect::<Vec<_>>()
            .externalize_policy()
            .map(|v| v.join("\n-----\n"));
    if config.send_emails {
        let recipients = if num < 90 {
            config.staff.clone()
        } else {
            config.admins.clone()
        };

        email::send(
            bg.log.clone(),
            apikey.user.clone(),
            recipients,
            format!("{} meeting {} questions", config.class, num),
            answer_log.export_check(&kv_ctx!("method" => "email-notify", "role" => "staff")).unwrap(),
        )
        .expect("failed to send email");
    }
    drop(bg);

    Redirect::to("/leclist")
}
