use chrono::naive::NaiveDateTime;
use rocket::form::{FromForm};
use std::collections::HashMap;
use beaver::filter;
use beaver::policy;
use beaver::policy::{Policy, Policied, PolicyError, NonePolicy, PoliciedString, PoliciedStringOption};
extern crate beaver_derive;
use beaver_derive::Policied;
use std::any::Any;

#[derive(Debug, FromForm)]
pub(crate) struct LectureQuestionSubmission {
    pub answers: HashMap<u64, String>,
}

// #[derive(Serialize, Policied)]
// #[policied(PoliciedLectureQuestion)]
#[derive(Serialize, Clone)]
pub struct LectureQuestion {
    pub id: u64,
    pub prompt: String,
    //#[policy_protected(PoliciedStringOption)] 
    answer: Option<String>,
}
// ############ ############ can be macro-ed ############ ############ ############
#[derive(Serialize)]
pub struct PoliciedLectureQuestion {
    inner: LectureQuestion,
    policy: Box<dyn Policy>
}

impl Policied<LectureQuestion> for PoliciedLectureQuestion {
    fn make(inner: LectureQuestion, policy: Box<dyn Policy>) -> PoliciedLectureQuestion {
        PoliciedLectureQuestion {
            inner, policy
        }
    }
    fn get_policy(&self) -> &Box<dyn Policy> {
        &self.policy
    }
    fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    fn export_check(&self, ctxt: &filter::Context) -> Result<LectureQuestion, PolicyError> {
        match self.get_policy().check(&ctxt) {
            Ok(_) => {
                Ok(self.inner.clone())
            }, 
            Err(pe) => { Err(pe) }
        }
    }
    fn export(&self) -> LectureQuestion {
        self.inner.clone()
    }
}

impl PoliciedLectureQuestion {
    pub fn make_decompose(id: u64, prompt: String, answer: PoliciedStringOption, policy: Box<dyn Policy>) -> PoliciedLectureQuestion {
        // Merge the given policy with policies from policied inputs
        let new_policy = policy.merge(answer.get_policy()).unwrap();
        PoliciedLectureQuestion::make(
            LectureQuestion {
                id, prompt, answer: answer.export()
            },
            new_policy
        )
    }
}

//derive_policied_vec!(PoliciedLectureQuestionVec, String, PoliciedLectureQuestion);
#[derive(Serialize)]
pub struct PoliciedLectureQuestionVec {
    inner: Vec<LectureQuestion>,
    policy: Box<dyn Policy>,
}

impl Policied<Vec<LectureQuestion>> for PoliciedLectureQuestionVec {
    fn make(inner: Vec<LectureQuestion>, policy: Box<dyn Policy>) -> PoliciedLectureQuestionVec {
        PoliciedLectureQuestionVec {
            inner, policy
        }
    }
    fn get_policy(&self) -> &Box<dyn Policy> {
        &self.policy
    }
    fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    fn export_check(&self, ctxt: &filter::Context) -> Result<Vec<LectureQuestion>, PolicyError> {
        match self.get_policy().check(&ctxt) {
            Ok(_) => {
                Ok(self.inner.clone())
            }, 
            Err(pe) => { Err(pe) }
        }
    }
    fn export(&self) -> Vec<LectureQuestion> {
        self.inner.clone()
    }
}

impl PoliciedLectureQuestionVec {
    pub fn push(&mut self, value: LectureQuestion) {
        self.inner.push(value);
    }

    pub fn push_policy(&mut self, value: PoliciedLectureQuestion) {
        self.policy = self.policy.merge(value.get_policy()).unwrap();
        self.inner.push(value.export());
    }

    pub fn pop(&mut self) -> Option<PoliciedLectureQuestion> {
        match self.inner.pop() {
            Some(v) => Some(PoliciedLectureQuestion::make(v, self.policy.clone() )),
            None => None
        }
    }

    pub fn sort_by<F>(&mut self, compare: F) where F: FnMut(&LectureQuestion, &LectureQuestion) -> std::cmp::Ordering, {
        self.inner.sort_by(compare)
    }
}
// ############ ############ end can be macro-ed ############ ############ ############


// #[derive(Serialize, Policied)]
// #[policied(PoliciedLectureQuestionsContext)]
#[derive(Serialize, Clone)]
pub struct LectureQuestionsContext {
    pub lec_id: u8,
    // #[policy_protected(PoliciedLectureQuestionVec)] 
    questions: Vec<LectureQuestion>,
    pub parent: &'static str,
}

// ############ ############ can be macro-ed ############ ############ ############
#[derive(Serialize)]
pub struct PoliciedLectureQuestionsContext {
    inner: LectureQuestionsContext,
    policy: Box<dyn Policy>,
}

impl Policied<LectureQuestionsContext> for PoliciedLectureQuestionsContext {
    fn make(inner: LectureQuestionsContext, policy: Box<dyn Policy>) -> PoliciedLectureQuestionsContext {
        PoliciedLectureQuestionsContext {
            inner, policy
        }
    }
    fn get_policy(&self) -> &Box<dyn Policy> {
        &self.policy
    }
    fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    fn export_check(&self, ctxt: &filter::Context) -> Result<LectureQuestionsContext, PolicyError> {
        match self.get_policy().check(&ctxt) {
            Ok(_) => {
                Ok(self.inner.clone())
            }, 
            Err(pe) => { Err(pe) }
        }
    }
    fn export(&self) -> LectureQuestionsContext {
        self.inner.clone()
    }
}

impl PoliciedLectureQuestionsContext {
    pub fn make_decompose(lec_id: u8, questions: PoliciedLectureQuestionVec, parent: &'static str, 
    policy: Box<dyn Policy>) -> PoliciedLectureQuestionsContext {
        // Merge the given policy with policies from policied inputs
        let new_policy = policy.merge(questions.get_policy()).unwrap();
        PoliciedLectureQuestionsContext::make(
            LectureQuestionsContext {
                lec_id, questions: questions.export(), parent
            },
            new_policy
        )
    }
}
// ############ ############ end can be macro-ed ############ ############ ############


// #[derive(Serialize, Policied)]
// #[policied(PoliciedLectureQuestion)]
#[derive(Serialize, Clone)]
pub struct LectureAnswer {
    pub id: u64,
    pub user: String,
    //#[policy_protected(PoliciedString)] 
    answer: String,
    pub time: Option<NaiveDateTime>,
}

#[derive(Serialize)]
pub struct PoliciedLectureAnswer {
    inner: LectureAnswer,
    policy: Box<dyn Policy>,
}

// ############ ############ can be macro-ed ############ ############ ############
impl Policied<LectureAnswer> for PoliciedLectureAnswer {
    fn make(inner: LectureAnswer, policy: Box<dyn Policy>) -> PoliciedLectureAnswer {
        PoliciedLectureAnswer {
            inner, policy
        }
    }
    fn get_policy(&self) -> &Box<dyn Policy> {
        &self.policy
    }
    fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    fn export_check(&self, ctxt: &filter::Context) -> Result<LectureAnswer, PolicyError> {
        match self.get_policy().check(&ctxt) {
            Ok(_) => {
                Ok(self.inner.clone())
            }, 
            Err(pe) => { Err(pe) }
        }
    }
    fn export(&self) -> LectureAnswer {
        self.inner.clone()
    }
}

impl PoliciedLectureAnswer {
    pub fn make_decompose(id: u64, user: String, answer: PoliciedString, time: Option<NaiveDateTime>, 
        policy: Box<dyn Policy>) -> PoliciedLectureAnswer {
        let new_policy = policy.merge(answer.get_policy()).unwrap();
        PoliciedLectureAnswer::make(
            LectureAnswer {
                id, user, answer: answer.export(), time
            },
            new_policy
        )
    }
}

//derive_policied_vec!(PoliciedLectureAnswerVec, String, PoliciedLectureAnswer);
#[derive(Serialize)]
pub struct PoliciedLectureAnswerVec {
    inner: Vec<LectureAnswer>,
    policy: Box<dyn Policy>,
}

impl Policied<Vec<LectureAnswer>> for PoliciedLectureAnswerVec {
    fn make(inner: Vec<LectureAnswer>, policy: Box<dyn Policy>) -> PoliciedLectureAnswerVec {
        PoliciedLectureAnswerVec {
            inner, policy
        }
    }
    fn get_policy(&self) -> &Box<dyn Policy> {
        &self.policy
    }
    fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    fn export_check(&self, ctxt: &filter::Context) -> Result<Vec<LectureAnswer>, PolicyError> {
        match self.get_policy().check(&ctxt) {
            Ok(_) => {
                Ok(self.inner.clone())
            }, 
            Err(pe) => { Err(pe) }
        }
    }
    fn export(&self) -> Vec<LectureAnswer> {
        self.inner.clone()
    }
}

impl PoliciedLectureAnswerVec {
    pub fn push(&mut self, value: LectureAnswer) {
        self.inner.push(value);
    }

    pub fn push_policy(&mut self, value: PoliciedLectureAnswer) {
        self.policy = self.policy.merge(value.get_policy()).unwrap();
        self.inner.push(value.export());
    }

    pub fn pop(&mut self) -> Option<PoliciedLectureAnswer> {
        match self.inner.pop() {
            Some(v) => Some(PoliciedLectureAnswer::make(v, self.policy.clone() )),
            None => None
        }
    }
}
// ############ ############ end can be macro-ed ############ ############ ############

// #[derive(Serialize, Policied)]
// #[policied(PoliciedLectureQuestionsContext)]
#[derive(Serialize, Clone)]
pub struct LectureAnswersContext {
    pub lec_id: u8,
    // #[policy_protected(PoliciedLectureAnswerVec)] 
    answers: Vec<LectureAnswer>,
    pub parent: &'static str,
}

// ############ ############ end can be macro-ed ############ ############ ############
#[derive(Serialize)]
pub struct PoliciedLectureAnswersContext {
    inner: LectureAnswersContext,
    policy: Box<dyn Policy>,
}

impl Policied<LectureAnswersContext> for PoliciedLectureAnswersContext {
    fn make(inner: LectureAnswersContext, policy: Box<dyn Policy>) -> PoliciedLectureAnswersContext {
        PoliciedLectureAnswersContext {
            inner, policy
        }
    }
    fn get_policy(&self) -> &Box<dyn Policy> {
        &self.policy
    }
    fn remove_policy(&mut self) -> () { self.policy = Box::new(NonePolicy); }
    fn export_check(&self, ctxt: &filter::Context) -> Result<LectureAnswersContext, PolicyError> {
        match self.get_policy().check(&ctxt) {
            Ok(_) => {
                Ok(self.inner.clone())
            }, 
            Err(pe) => { Err(pe) }
        }
    }
    fn export(&self) -> LectureAnswersContext {
        self.inner.clone()
    }
}

impl PoliciedLectureAnswersContext {
    pub fn make_decompose(lec_id: u8, answers: PoliciedLectureAnswerVec, parent: &'static str, 
        policy: Box<dyn Policy>) -> PoliciedLectureAnswersContext {
        // Merge the given policy with policies from policied inputs
        let new_policy = policy.merge(answers.get_policy()).unwrap();
        PoliciedLectureAnswersContext::make(
            LectureAnswersContext {
                lec_id, answers: answers.export(), parent
            },
            new_policy
        )
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
                        match cc.as_any().downcast_ref::<HackContext>() {
                            Some(_) => { return Ok(()); },
                            None => { return Err(PolicyError{
                                message: "Must be either TemplateRenderContext or HackContext".to_string() }); }
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

pub struct TemplateRenderContext {
    pub is_admin: bool,
    pub user: String,
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
