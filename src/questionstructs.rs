use crate::apikey::ApiKey;
use crate::backend::MySqlBackend;
use chrono::naive::NaiveDateTime;
use rocket::form::{FromForm};
use rocket::State;
use std::collections::HashMap;
use beaver::filter;
use beaver::policy;
use beaver::policy::{Policy, Policied, PolicyError, NonePolicy, PoliciedString};
extern crate beaver_derive;
use beaver_derive::Policied;
use std::any::Any;
use std::sync::{Arc, Mutex};
use mysql::from_value;

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
#[derive(Serialize)]
pub(crate) struct LectureQuestionUnpolicied {
    pub id: u64,
    pub prompt: String,
    pub answer: Option<String>,
}

impl LectureQuestion {
    pub fn make(id: u64, prompt: String, answer: Option<PoliciedString>, policy: Box<dyn Policy>) -> LectureQuestion {
        match answer {
            Some(mut ps) => {
                let new_policy = policy.merge(&ps.get_policy()).unwrap();

                // TODO: This will be a new function that does not take in a context. This is a hacky solution
                ps.remove_policy();
                let ctxt = Box::new(
                        filter::Context::CustomContext(
                            Box::new(HackContext{})));
                let answer_unpolicied = ps.export(&ctxt).unwrap();

                LectureQuestion {
                    id, prompt, answer: Some(answer_unpolicied), policy: new_policy
                }
            }
            None => {
                LectureQuestion {
                    id, prompt, answer: None, policy
                }
            }
        }
    }
}

#[derive(Serialize, Policied)]
pub(crate) struct LectureQuestionsContext {
    pub lec_id: u8,
    // #[policy_protected(Vec<LectureQuestion>)] 
    questions: Vec<LectureQuestionUnpolicied>,
    pub parent: &'static str,
    policy: Box<dyn Policy>,
}

#[derive(Serialize)]
pub(crate) struct LectureQuestionsContextUnpolicied {
    pub lec_id: u8,
    pub questions: Vec<LectureQuestionUnpolicied>,
    pub parent: &'static str,
}

impl LectureQuestionsContext {
    pub fn make(lec_id: u8, questions: Vec<LectureQuestion>, parent: &'static str, 
        init_policy: Box<dyn Policy>) -> LectureQuestionsContext {
            let mut policy = init_policy;
            for question in &questions {
                policy = policy.merge(question.get_policy()).unwrap()
            }
            let questions_unpolicied = questions.into_iter().map(
                |a| LectureQuestionUnpolicied {
                    id: a.id,
                    prompt: a.prompt,
                    answer: a.answer
                }
            ).collect();
            LectureQuestionsContext {
                lec_id, questions: questions_unpolicied, parent, policy
            }
    }

    pub fn export(self, ctxt: &filter::Context) -> Result<LectureQuestionsContextUnpolicied, PolicyError> {
        match self.policy.export_check(&ctxt) {
            Ok(_) => {
                Ok(LectureQuestionsContextUnpolicied {
                    lec_id: self.lec_id,
                    questions: self.questions,
                    parent: self.parent
                })
            }, 
            Err(pe) => { Err(pe) }
        }
    }
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
        match self.policy.export_check(&ctxt) {
            Ok(_) => {
                Ok(LectureAnswersContextUnpolicied {
                    lec_id: self.lec_id,
                    answers: self.answers,
                    parent: self.parent
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
                match cc.as_any().downcast_ref::<TemplateRenderContext>() {
                    Some(trc) => {
                        // interface with SQL database
                        let mut bg = trc.backend.lock().unwrap();
                        let rs = bg.query_exec("users_by_apikey", vec![trc.key.clone().into()]);
                        drop(bg);

                        let is_admin: bool = from_value::<bool>(rs[0][2].clone());

                        if is_admin || trc.apikey.user.eq(&self.user) {
                            return Ok(());
                        } else {
                            return Err(PolicyError {
                                message: "Answer can only be shown to admin or answer's user".to_string()
                            });
                        }
                    },
                    None => {
                        match cc.as_any().downcast_ref::<HackContext>() {
                            Some(hc) => { Ok(()) },
                            None => { Err(PolicyError{message: "Must be either TemplateRenderContext or HackContext"}) }
                        }
                    }
                }; 
                
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

pub struct TemplateRenderContext<'a> {
    // pub admin: bool,
    // Note: this is broken right now
    pub apikey: ApiKey,
    pub(crate) backend: &State<Arc<Mutex<MySqlBackend>>>,
}

impl filter::CustomContext for TemplateRenderContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
} 

pub struct HackContext {}

impl filter::CustomContext for HackContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
} 
