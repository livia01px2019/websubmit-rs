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
use beaver::filter;
use beaver::policy;
use beaver::policy::{Policy, Policied, PolicyError, NonePolicy, PoliciedNumber, PoliciedString};
extern crate beaver_derive;
use beaver_derive::Policied;
use std::any::Any;

//pub(crate) enum LectureQuestionFormError {
//   Invalid,
//}

#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    pub answers: HashMap<u64, String>,
}

#[derive(Serialize, Policied)]
pub(crate) struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    //#[policy_protected(PoliciedString)] 
    answer: Option<String>,
    policy: Box<dyn Policy>, 
}

impl LectureQuestion {
    pub fn make(id: u64, prompt: String, answer: Option<PoliciedString>, policy: Box<dyn Policy>) -> LectureQuestion {
        // TODO: merge policy and policy from answer 
        // TODO: call export on PoliciedString
        // create LectureQuestion struct 
    }
}

pub(crate) struct LectureQuestionUnpolicied {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

// TODO: Make this policied by deriving and then creating a make() fn, also an unpolicied version. 
// Then follow example of answers() to use these structs and give unpolicied struct to render
#[derive(Serialize)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: u8,
    pub questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

#[derive(Serialize, Policied)]
pub(crate) struct LectureAnswer {
    pub id: u64,
    pub user: String,
    #[policy_protected(PoliciedString)] 
    answer: String,
    pub time: Option<NaiveDateTime>,
    policy: Box<dyn Policy>,
}

#[derive(Serialize)]
pub(crate) struct LectureAnswerUnpolicied {
    pub id: u64,
    pub user: String,
    pub answer: String,
    pub time: Option<NaiveDateTime>,
}

impl LectureAnswer {
    pub fn make(id: u64, user: String, answer: String, time: Option<NaiveDateTime>, 
        policy: Box<dyn Policy>) -> LectureAnswer {
            LectureAnswer {
                id, user, answer, time, policy
            }
    }
}

#[derive(Serialize, Policied)]
pub(crate) struct LectureAnswersContext {
    pub lec_id: u8,
    // #[policy_protected(Vec<LectureAnswer>)] 
    answers: Vec<LectureAnswerUnpolicied>,
    pub parent: &'static str,
    policy: Box<dyn Policy>,
}

#[derive(Serialize)]
pub(crate) struct LectureAnswersContextUnpolicied {
    pub lec_id: u8,
    pub answers: Vec<LectureAnswerUnpolicied>,
    pub parent: &'static str,
}

impl LectureAnswersContext {
    pub fn make(lec_id: u8, answers: Vec<LectureAnswer>, parent: &'static str, 
        init_policy: Box<dyn Policy>) -> LectureAnswersContext {
            let mut policy = init_policy;
            for answer in &answers {
                policy = policy.merge(answer.get_policy()).unwrap()
            }
            let answers_unpolicied = answers.into_iter().map(
                |a| LectureAnswerUnpolicied {
                    id: a.id,
                    user: a.user,
                    answer: a.answer,
                    time: a.time
                }
            ).collect();
            LectureAnswersContext {
                lec_id, answers: answers_unpolicied, parent, policy
            }
    }

    pub fn export(self, ctxt: &filter::Context) -> Result<LectureAnswersContextUnpolicied, PolicyError> {
        match policied_item.get_policy().export_check(&ctxt) {
            Ok(_) => {
                Ok(LectureAnswersContextUnpolicied {
                    lec_id: policied_item.lec_id,
                    answers: policied_item.answers,
                    parent: policied_item.parent
                })
            }, 
            Err(pe) => { Err(pe) }
        }
    }
}

#[derive(Serialize)]
pub(crate) struct LectureListEntry {
    pub id: u64,
    pub label: String,
    pub num_qs: u64,
    pub num_answered: u64,
}

#[derive(Serialize)]
pub(crate) struct LectureListContext {
    pub admin: bool,
    pub lectures: Vec<LectureListEntry>,
    pub parent: &'static str,
}

#[derive(Serialize, Clone)]
pub struct AnswerPolicy {
    pub user: String
}

impl Policy for AnswerPolicy {
    fn export_check(&self, ctxt: &filter::Context) -> Result<(), PolicyError> {
        match ctxt {
            filter::Context::CustomContext(cc) => {
                let trc: &TemplateRenderContext = match cc.as_any().downcast_ref::<TemplateRenderContext>() {
                    Some(trc) => trc,
                    None => panic!("&cc isn't a TemplateRenderContext!"),
                }; 
                if (trc.admin || trc.user.eq(&self.user)) {
                    return Ok(());
                } else {
                    return Err(PolicyError {
                        message: "Answer can only be shown to admin or answer's user".to_string()
                    });
                }
            }
            _ => {Err(PolicyError {
                message: "Can only send answer over TemplateRenderContext".to_string()
            })}
        }
    }
    fn merge(&self, other: &Box<dyn Policy>) ->  Result<Box<dyn Policy>, PolicyError>{
        Ok(Box::new(policy::MergePolicy::make( 
            Box::new(self.clone()),
            other.clone(),
        )))
     }
}

pub struct TemplateRenderContext {
    pub admin: bool,
    pub user: String,
}

impl filter::CustomContext for TemplateRenderContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
} 
