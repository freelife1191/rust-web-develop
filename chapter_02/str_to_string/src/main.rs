use std::io::{Error, ErrorKind};
use std::str::FromStr;

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

/// FromStr 트레이트로 &str에서 QuestionId 만들기
/// &self 를 취하지 않으므로 마침표(.)를 사용해 호출할 수 있는 메서드가 아니라
/// 이중 콜론(::)을 사용해 호출하는 연관 함수
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

fn main() {
    let question = Question::new(
        // QuestionId("1".to_string()),
        QuestionId::from_str("1").expect("No id provided"),
        "First Question".to_string(),
        "Content of question".to_string(),
        Some(vec!["faq".to_string()]), // ["faq".to_string()],
    );
    // println!("{:?}", question);
    println!("{:#?}", question); // #을 추가하여 좀 더 예쁘게 출력할 수도 있다, 이렇게 하면 긴 문자열 하나 대신 여러 줄로 데이터 구조를 출력해준다
}
