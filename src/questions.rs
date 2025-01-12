use crate::admin::Admin;
use crate::apikey::ApiKey;
use crate::backend::{DataType, NoriaBackend};
use crate::config::Config;
use crate::email;
use chrono::naive::NaiveDateTime;
use chrono::Local;
use rocket::request::{Form, FormItems, FromForm};
use rocket::response::Redirect;
use rocket::State;
use rocket_contrib::templates::Template;
use std::sync::{Arc, Mutex};

pub(crate) enum LectureQuestionFormError {
    Invalid,
}

#[derive(Debug)]
pub(crate) struct LectureQuestionSubmission {
    answers: Vec<(u64, String)>,
}

#[derive(Serialize)]
pub(crate) struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

#[derive(Serialize)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: u8,
    pub questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

#[derive(Serialize)]
struct LectureAnswer {
    id: u64,
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
    backend: State<Arc<Mutex<NoriaBackend>>>,
    config: State<Config>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("leclist").unwrap().into_sync();

    let user = apikey.user.clone();
    let admin = config.admins.contains(&user);

    let res = h
        .lookup(&[(0 as u64).into()], true)
        .expect("lecture list lookup failed");

    let lecs: Vec<_> = res
        .into_iter()
        .filter(|r| !r[2].is_none())
        /*.map(|mut r| {
            if let DataType::None = r[3] {
                r[3] = DataType::UnsignedInt(0);
            }
            if let DataType::None = r[4] {
                r[4] = DataType::UnsignedInt(0);
            }
            r
        })*/
        .map(|r| LectureListEntry {
            id: r[0].clone().into(),
            label: r[1].clone().into(),
            num_qs: r[2].clone().into(),
            num_answered: /*r[4].clone().into()*/ 0u64,
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
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let mut h = bg.handle.view("answers_by_lec").unwrap().into_sync();

    let key: DataType = (num as u64).into();

    let res = h.lookup(&[key], true).expect("failed to look up answers!");
    let answers: Vec<_> = res
        .into_iter()
        .map(|r| LectureAnswer {
            id: r[2].clone().into(),
            user: r[0].clone().into(),
            answer: r[3].clone().into(),
            time: if let DataType::Timestamp(ts) = r[4] {
                Some(ts)
            } else {
                None
            },
        })
        .collect();

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
    backend: State<Arc<Mutex<NoriaBackend>>>,
) -> Template {
    use std::collections::HashMap;

    let mut bg = backend.lock().unwrap();
    let mut qh = bg.handle.view("qs_by_lec").unwrap().into_sync();
    let key: DataType = (num as u64).into();

    let mut ah = bg.handle.view("my_answers_for_lec").unwrap().into_sync();
    let answers_res = ah
        .lookup(&[(num as u64).into(), apikey.user.clone().into()], true)
        .expect("lecture questions lookup failed");
    let mut answers = HashMap::new();

    for r in answers_res {
        let id: u64 = r[2].clone().into();
        let atext: String = r[3].clone().into();
        answers.insert(id, atext);
    }

    let res = qh
        .lookup(&[key], true)
        .expect("lecture questions lookup failed");
    let mut qs: Vec<_> = res
        .into_iter()
        .map(|r| {
            let id: u64 = r[1].clone().into();
            let answer = answers.get(&id).map(|s| s.to_owned());
            LectureQuestion {
                id: id,
                prompt: r[2].clone().into(),
                answer: answer,
            }
        })
        .collect();
    qs.sort_by(|a, b| a.id.cmp(&b.id));

    let ctx = LectureQuestionsContext {
        lec_id: num,
        questions: qs,
        parent: "layout",
    };
    Template::render("questions", &ctx)
}

impl<'f> FromForm<'f> for LectureQuestionSubmission {
    // In practice, we'd use a more descriptive error type.
    type Error = LectureQuestionFormError;

    fn from_form(
        items: &mut FormItems<'f>,
        strict: bool,
    ) -> Result<LectureQuestionSubmission, Self::Error> {
        let mut answers: Vec<(u64, String)> = vec![];

        for item in items {
            let (key, value) = item.key_value_decoded();
            if key.as_str().starts_with("q_") {
                let num = u64::from_str_radix(&key.as_str()[2..], 10)
                    .map_err(|_| LectureQuestionFormError::Invalid)?;
                answers.push((num, value));
            } else {
                if strict {
                    return Err(LectureQuestionFormError::Invalid);
                } else {
                    /* allow extra value when not strict */
                }
            }
        }

        Ok(LectureQuestionSubmission { answers })
    }
}

#[post("/<num>", data = "<data>")]
pub(crate) fn questions_submit(
    apikey: ApiKey,
    num: u8,
    data: Form<LectureQuestionSubmission>,
    backend: State<Arc<Mutex<NoriaBackend>>>,
    config: State<Config>,
) -> Redirect {
    use noria::Modification;

    let mut bg = backend.lock().unwrap();

    let num: DataType = (num as u64).into();
    let ts: DataType = DataType::Timestamp(Local::now().naive_local());

    let mut table = bg.handle.table("answers").unwrap().into_sync();

    for (id, answer) in &data.answers {
        let rec: Vec<DataType> = vec![
            apikey.user.clone().into(),
            num.clone(),
            (*id).into(),
            answer.clone().into(),
            ts.clone(),
        ];
        table
            .insert_or_update(
                rec,
                vec![
                    (3, Modification::Set(answer.clone().into())),
                    (4, Modification::Set(ts.clone())),
                ],
            )
            .expect("failed to write answer!");
    }

    if config.send_emails {
        let recipients = if num < 90.into() {
            config.staff.clone()
        } else {
            config.admins.clone()
        };

        email::send(
            apikey.user.clone(),
            recipients,
            format!("{} meeting {} questions", config.class, num),
            format!(
                "{}",
                data.answers
                    .iter()
                    .map(|(i, t)| format!("Question {}:\n{}", i, t))
                    .collect::<Vec<_>>()
                    .join("\n-----\n")
            ),
        )
        .expect("failed to send email");
    }

    Redirect::to("/leclist")
}
