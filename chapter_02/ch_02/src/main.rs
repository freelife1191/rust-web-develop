// ch02/src/main.rs
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use warp::Filter;

///
/// 코드 2-1 Question 타입의 생성과 구현
#[derive(Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug)]
struct QuestionId(String);

impl Question {
    fn new(
        id: QuestionId,
        title: String,
        content: String,
        tags: Option<Vec<String>>
    ) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl std::fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter)
        -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{}, title: {}, content: {}, tags: {:?}",
            self.id, self.title, self.content, self.tags
        )
    }
}

impl std::fmt::Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter) 
        -> Result<(), std::fmt::Error> {
        write!(f, "id: {}", self.0)
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            false => Ok(QuestionId(id.to_string())),
            true => Err(
                Error::new(ErrorKind::InvalidInput, "No id provided")
            ),
        }
    }
}

#[tokio::main]
async fn main() {
    // let question = Question::new(
    //     QuestionId::from_str("1").expect("No id provided"),
    //     "First Question".to_string(),
    //     "Content of question".to_string(),
    //     Some(vec!["faq".to_string()]), // ["faq".to_string()],
    // );
    // println!("{:#?}", question); // #을 추가하여 좀 더 예쁘게 출력할 수도 있다, 이렇게 하면 긴 문자열 하나 대신 여러 줄로 데이터 구조를 출력해준다

    let hello = warp::get().map(|| format!("Hello, World!"));
    warp::serve(hello)
    .run(([127, 0, 0, 1], 3030))
    .await;
}