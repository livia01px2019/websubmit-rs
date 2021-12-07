// pub trait Policied {
//     pub fn export_check(self, ctxt: Context);
//     pub fn add_policy(self, s: String, policy: fn(Box<dyn Policied>) -> Result<(), PolicyError>);
//     pub fn remove_policy(self, s: String);
// }

// // struct PoliciedString {
// // 	s: String,
// // 	policies: Map<String, fn(Box<dyn Policied>) -> Result<(), PolicyError>>
// // }


// pub(crate) struct LectureQuestion {
//     pub id: u64,
//     pub prompt: String,
//     answer: Option<String>,
//     policies: Map<String, fn(Box<dyn Policied>) -> Result<(), PolicyError>>
// }

// struct AnswerPolicyInfo {
//     is_admin: bool
// }
// let a = LectureQuestion {
//     0, "hi".to_string(), None, emptyMap
// }
// api = AnswerPolicyInfo {
//     is_admin: true
// }
// a.add_policy("answer_policy", answer_policy_generator(api))

// pub(crate) struct LectureQuestionsContext {
//     pub lec_id: u8,
//     questions: Vec<LectureQuestion>,
//     pub parent: &'static str,
//     policies: Map<String, fn(Box<dyn Policied>) -> Result<(), PolicyError>>
// }

// pub fn answer_policy_generator(a: AnswerPolicyInfo) 
//     -> fn(Box<dyn Policied>) -> Result<(), PolicyError> {   
// 	| policied_thing | {
        
//     }
// }


// impl LectureQuestionsContext {
//     fn make(lec_id: u8, questions: Vec<LectureQuestion>, parent: &'static str, policy: Map<String, fn(Box<dyn Policied>) -> Result<(), PolicyError>>) {
//         let a = |lcq| {
//             for q in lcq.questions {
//                 q.export_check()
//                 if failed, return failed
//             }
//         }
//         return LectureQuestionsContext {
//             lec_id: lec_id,
//             questions: questions,
//             parent: parent,
//             policies: new Map {"str": a}
//         }
//     }
// }
use beaver::policy::Policy;

struct Policied<T: Clone> {
    inner: T,
    pub policy: Box<dyn Policy>
}

impl<T: Clone> Policied<T> {
    pub fn make(inner: T, policy: Box<dyn Policy>) -> Policied<T> {
        Policied { inner, policy }
    }

    pub fn get_policy(&self) -> &Box<dyn Policy> { 
        &self.policy 
    }
    pub fn remove_policy(self) -> T { 
        self.inner
    }
    // DO NOT USE
    pub fn inner(&self) -> T {
        selt.inner.clone()
    }
}

//#[derive(Policied("PoliciedA"))]
struct A {
    #[policy_protected]
    fa: String,
    #[policy_protected]
    fb: Number,
    pub public: String
}

// derived
impl Policied<A> {
    pub fn make(fa: Policied<String>, fb: Policied<Number>, public: String) -> Policied<A> {
        let a = A {
            fa: fa.remove_policy(),
            fb: fb.remove_policy(),
            public,
        };
        Policied<A>::make(a, fa.get_policy().merge(fb.get_policy()));
    }

    pub fn fa(&self) -> Policied<String> {
        Policied<String>::make(self.inner().fa, policy.clone())
    }
    pub fn fb(&self) -> Policied<Number> {
        Policied<Number>::make(self.inner().fb, policy.clone())
    }
    pub fn public(&self) -> String {
        self.inner().public
    }
}
