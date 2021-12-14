use chrono::naive::NaiveDateTime;
use rocket::form::{FromForm};
use std::collections::HashMap;
use beaver::filter;
use beaver::policy;
use beaver::policy::{Policy, Policied, PolicyError, NonePolicy, PoliciedString, PoliciedStringOption};
use beaver::derive_policied;
use beaver::derive_policied_vec;
extern crate beaver_derive;
use beaver_derive::Policied;
use std::any::Any;

#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    pub answers: HashMap<u64, String>,
}

#[derive(Serialize, Policied, Clone)]
#[policied(PoliciedLectureQuestion)]
pub struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    #[policy_protected(PoliciedStringOption)] 
    answer: Option<String>,
}

derive_policied!(LectureQuestion, PoliciedLectureQuestion);

derive_policied_vec!(PoliciedLectureQuestionVec, LectureQuestion, PoliciedLectureQuestion);


#[derive(Serialize, Policied, Clone)]
#[policied(PoliciedLectureQuestionsContext)]
pub struct LectureQuestionsContext {
    pub lec_id: u8,
    #[policy_protected(PoliciedLectureQuestionVec)] 
    questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

derive_policied!(LectureQuestionsContext, PoliciedLectureQuestionsContext);

#[derive(Serialize, Policied, Clone)]
#[policied(PoliciedLectureAnswer)]
pub struct LectureAnswer {
    pub id: u64,
    pub user: String,
    #[policy_protected(PoliciedString)] 
    answer: String,
    pub time: Option<NaiveDateTime>,
}

derive_policied!(LectureAnswer, PoliciedLectureAnswer);

derive_policied_vec!(PoliciedLectureAnswerVec, LectureAnswer, PoliciedLectureAnswer);

#[derive(Serialize, Policied, Clone)]
#[policied(PoliciedLectureAnswersContext)]
pub struct LectureAnswersContext {
    pub lec_id: u8,
    #[policy_protected(PoliciedLectureAnswerVec)] 
    answers: Vec<LectureAnswer>,
    pub parent: &'static str,
}

derive_policied!(LectureAnswersContext, PoliciedLectureAnswersContext);

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
    fn check(&self, ctxt: &filter::Context) -> Result<(), PolicyError> {
        match ctxt {
            filter::Context::CustomContext(cc) => {
                match cc.as_any().downcast_ref::<TemplateRenderContext>() {
                    Some(trc) => {
                        // interface with SQL database
                        if trc.is_admin || trc.user.eq(&self.user) {
                            return Ok(());
                        } else {
                            return Err(PolicyError {
                                message: "Answer can only be shown to admin or answer's user".to_string()
                            });
                        }
                    },
                    None => {
                        return Err(PolicyError {
                            message: "Can only send answer over TemplateRenderContext".to_string()
                        });
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
            vec![Box::new(self.clone()), other.clone()],
        )))
     }
}

pub struct TemplateRenderContext {
    pub is_admin: bool,
    pub user: String,
}

impl filter::CustomContext for TemplateRenderContext {
    fn as_any(&self) -> &dyn Any {
        self
    }
} 

