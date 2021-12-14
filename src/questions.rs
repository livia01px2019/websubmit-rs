use crate::apikey::ApiKey;
use crate::backend::{MySqlBackend, Value};
use crate::config::Config;
use crate::email;
use crate::questionstructs::{TemplateRenderContext, PoliciedLectureQuestionVec, PoliciedLectureAnswerVec, AnswerPolicy, LectureQuestionSubmission, PoliciedLectureQuestion, PoliciedLectureQuestionsContext, PoliciedLectureAnswer, PoliciedLectureAnswersContext, LectureListEntry, LectureListContext};
use chrono::naive::NaiveDateTime;
use chrono::Local;
use mysql::from_value;
use rocket::form::{Form};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::Template;
use std::sync::{Arc, Mutex};
use beaver::filter;
use beaver::policy::{Policied, NonePolicy, PoliciedString, PoliciedStringOption};
extern crate beaver_derive;

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
    // _admin: Admin,
    apikey: ApiKey,
    num: u8,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
) -> Template {
    let mut bg = backend.lock().unwrap();
    let key: Value = (num as u64).into();
    let res = bg.query_exec("answers_by_lec", vec![key]);
    drop(bg);
    let mut answers: PoliciedLectureAnswerVec = PoliciedLectureAnswerVec::make(vec![], Box::new(NonePolicy));
    for r in res {
        answers.push_policy(PoliciedLectureAnswer::make_decomposed(
            from_value(r[2].clone()),
            from_value(r[0].clone()),
            PoliciedString::make(from_value(r[3].clone()),
                                Box::new(AnswerPolicy { user: from_value(r[0].clone()) })),
            if let Value::Time(..) = r[4] {
                Some(from_value::<NaiveDateTime>(r[4].clone()))
            } else {
                None
            },
            Box::new(NonePolicy)))
    }

    let ctx = PoliciedLectureAnswersContext::make_decomposed(
        num,
        answers,
        "layout",
        Box::new(NonePolicy)
    );

    let mut bg = backend.lock().unwrap();
    let rs = bg.query_exec("users_by_apikey", vec![apikey.key.clone().into()]);
    drop(bg);
    let is_admin: bool = from_value::<bool>(rs[0][2].clone());

    let render_ctxt = Box::new(
        filter::Context::CustomContext(
            Box::new(TemplateRenderContext { is_admin, user: apikey.user })));
    Template::render("answers", ctx.export_check(&render_ctxt).unwrap())
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

    let answers_res = bg.query_exec(
        "my_answers_for_lec",
        vec![(num as u64).into(), apikey.user.clone().into()],
    );
    let mut answers = HashMap::new();

    for r in answers_res {
        let id: u64 = from_value(r[2].clone());
        let atext: PoliciedString = PoliciedString::make(
            from_value(r[3].clone()), Box::new(AnswerPolicy { user: apikey.user.clone() }));
        answers.insert(id, atext);
    }
    let res = bg.query_exec("qs_by_lec", vec![key]);
    drop(bg);
    let mut qs: PoliciedLectureQuestionVec = PoliciedLectureQuestionVec::make(vec![], Box::new(NonePolicy));
    for r in res {
        let id = from_value(r[1].clone());
        qs.push_policy(PoliciedLectureQuestion::make_decomposed(
            id,
            from_value(r[2].clone()),
            PoliciedStringOption::make_option(answers.get(&id).map(|s| s.to_owned())),
            Box::new(NonePolicy),
        ))
    }
    qs.sort_by(|a, b| a.id.cmp(&b.id));

    let ctx = PoliciedLectureQuestionsContext::make_decomposed(
        num,
        qs,
        "layout",
        Box::new(NonePolicy)
    );

    let mut bg = backend.lock().unwrap();
    let rs = bg.query_exec("users_by_apikey", vec![apikey.key.clone().into()]);
    drop(bg);
    let is_admin: bool = from_value::<bool>(rs[0][2].clone());

    let render_ctxt = Box::new(
        filter::Context::CustomContext(
            Box::new(TemplateRenderContext { is_admin, user: apikey.user })));
    Template::render("questions", ctx.export_check(&render_ctxt).unwrap())
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

    for (id, answer) in &data.answers {
        let rec: Vec<Value> = vec![
            apikey.user.clone().into(),
            vnum.clone(),
            (*id).into(),
            answer.clone().into(),
            ts.clone(),
        ];
        bg.insert_or_update(
            "answers",
            rec,
            vec![(3, answer.clone().into()), (4, ts.clone())],
        );
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
            answer_log,
        )
        .expect("failed to send email");
    }
    drop(bg);

    Redirect::to("/leclist")
}
