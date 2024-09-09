# 4. 🚦 Restful API 구현하기

이 장에서는 POST, PUT, DELETE, POST를 사용한 코멘트 작성 부분을 다룬다

```
API Routes
GET    /questions (empty body; return JSON)
```

## 4.1 인메모리 스토리지에서 질문 가져오기

---

### 4.1.1 모의 데이터베이스 설정하기

**코드 4.2 질문에 대한 로컬 저장소를 만들기**

```rust
use std::collections::HashMap;

struct Store {
    questions: HashMap<QuestionId, Question>,
}
```

세 가지 메서드로 저장소를 구현
- `new`
    - 값에 접근하고 전달할 수 있는 새로운 저장소 객체를 만든다
- `init`
    - 로컬 JSON 파일이나 코드로 예시 질문을 초기화한다
- `add_question`
    - 이후에 더 많은 질문을 추가한다

러스트에는 생성자를 생성하는 표준화되는 방법이 없으므로 new 키워드를 사용해 새로운 Store를 만들고 반환한다


**코드 4-3 저장소에 생성자 추가하기**

```rust
use std::collections::HashMap;

impl Store {
    fn new() -> Self {
        Store {
            questions: HashMap::new(),
        }
    }
}
```

**코드 4-4 질문을 추가하는 메서드를 저장소에 추가하기**

```rust
impl Store {
    fn add_question(mut self, question: Question) -> Self {
        self.questions.insert(question.id.clone(), question);
        self
    }
}
```

**코드 4-5 derive 매크로에 비교 트레이트 구현하기**

```rust
#[derive(Serialize, Debug, Clone, Eq, Hash)]
struct QuestionId(String);
```

**코드 4-6 QuestionId 구조체에 PartialEq 트레이트 추가하기**

```rust
#[derive(Serialize, Debug, Clone, Eq, Hash, PartialEq)]
struct QuestionId(String);
```

HashMap의 키/인덱스로 사용되는 모든 객체는 Eq, PartialEq, Hash 트레이트가 필요하다


### 4.1.2 테스트 데이터를 준비하기


**코드 4-7 init 메서드를 Store에 추가하고 예제 질문 추가하기**

```rust
impl Store {
    fn init(self) -> Self {
        let question = Question::new(
            QuestionId::from_str("1").expect("Id not set"),
            "How?".to_string(),
            "Please help!".to_string(),
            Some(vec!["general".to_string()]),
        );
        self.add_question(question)
    }

    fn add_question(mut self, question: Question) -> Self {
        self.questions.insert(question.id.clone(), question);
        self
    }
}
```


**코드 4-8 예제 질문을 넣은 question.json 파일 만들기**

```json
{
  "1" : {
    "id": "1",
    "title": "How?",
    "content": "Please help!",
    "tags": ["general"]
  }
}

```


**코드 4-9 Serde JSON 라이브러리 추가하기**

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```


**코드 4-10 JSON 파일에서 질문을 읽어 저장소에 넣기**

```rust
use serde::{Deserialize, Serialize};

impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }

    // fn add_question(mut self, question: Question) -> Self {
    //     self.questions.insert(question.id.clone(), question);
    //     self
    // }
}

#[derive(Serialize, Debug, Deserialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Serialize, Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
struct QuestionId(String);
```

### 4.1.3 가짜 데이터베이스에서 읽어 들이기

**코드 4-11 서버 시작 전에 저장소의 새 인스턴스 생성하기**

```rust
#[tokio::main]
async fn main() {
    let store = Store::new();
}
```


**코드 4-12 저장소 필터를 만들어 경로에 전달하기**

```rust
#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

#[tokio::main]
async fn main() {
    let store = Store::new();
    // any 필터에 별도의 제약이 없으므로 모든 요청과 일치하여 모든 요청을 실행한다
    let store_filter = warp::any()
        // 필터에서 map을 호출하여 받는 함수에 값을 전달한다
        // map 내부에서는 러스트 클로저를 사용한다
        // move 키워드는 값을 가로챔(capture by value)을 나타낸다
        // 즉, 값을 클로저로 이동시켜 소유권을 가져온다
        .map(move ||
            // Warp 필터가 적용되는 모든 함수가 저장소를 사용할 수 있도록 저장소의 복제본을 반환한다
            // 지금은 경로가 하나뿐이어서 복제할 필요까지는 없다
            // 그러나 이 다음에 경로 핸들러를 여러 개 만들고 이들 모두가 저장소에 접근해야 하므로 복제해야 한다
            store.clone());

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```


**코드 4-13 /questions 경로와 경로 핸들러에 저장소 추가하기**

```rust
#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // get_questions 함수는 매개변수 하나를 받게 된다
    // 이제 설정된 질문을 반환하는 대신 저장소에서 읽는다
    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        // Warp 프레임워크는 이제 저장소 객체를 경로 핸들러에 추가한다
        .and(store_filter) // 필터를 체인에 연결한다
        .and_then(get_questions)
        .recover(return_error);
    let routes = get_questions.with(cors);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```


**코드 4-14 get_questions 경로 핸들러로 저장소에서 질문 읽기**

```rust
// use std::str::FromStr;
// use std::io::{Error, ErrorKind};

// Clone 트레이트를 추가해서
// get_questions 함수에서 사용
#[derive(Deserialize, Serialize, Debug, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

// impl Question {
//     fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
//         Question {
//             id,
//             title,
//             content,
//             tags,
//         }
//     }
// }

// #[derive(Debug)]
// struct InvalidId;
// impl Reject for InvalidId {}

// impl FromStr for QuestionId {
//     type Err = std::io::Error;
// 
//     fn from_str(id: &str) -> Result<Self, Self::Err> {
//         match id.is_empty() {
//             false => Ok(QuestionId(id.to_string())),
//             true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
//         }
//     }
// }

// Warp가 사용할 수 있게 회신과 거부를 반환하는 첫 번째 경로 핸들러를 만든다
async fn get_questions() -> Result<impl Reply, Rejection> {
    // let question = Question::new( // 요청하는 클라이언트에 반환할 새로운 question을 생성한다
    //     QuestionId::from_str("1").expect("No id provided"),
    //     "First Question".to_string(),
    //     "Content of question".to_string(),
    //     Some(vec!["faq".to_string()]),
    // );

    // match question.id.0.parse::<i32>() {
    //     Err(_) => Err(warp::reject::custom(InvalidId)),
    //     Ok(_) => Ok(warp::reply::json(&question)),
    // }

    // 현재 가지고 있는 모든 질문 목록을 반환
    // HashMap의 values 메서드를 사용해서 해시 맵의 키(QuestionId)는 빼고 해시 맵의 값(Question) 전부를 복제
    // collect를 사용하려면 단순 참조가 아닌, 값의 소유권을 가져야 하므로 값을 복제 해야 한다
    let res: Vec<Question> = store.questions.values().cloned()
        .collect();
    Ok(warp::reply::json(&res))
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    // } else if let Some(InvalidId) = r.find() {
    //     Ok(warp::reply::with_status(
    //         "No valid ID presented".to_string(),
    //         StatusCode::UNPROCESSABLE_ENTITY,
    //     ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
```


### 4.14 쿼리 매개변수 파싱하기


**코드 4-15 쿼리 매개변수를 파싱하는 query 필터 추가하기**

```rust
#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        // Warp가 get_questions 함수를 호출할 때 추가 매개변수가 있어야 됨
        // warp::query를 추가하여 마지막 and_then에서 호출하는 함수에 해시 맵을 매개변수로 추가
        .and(warp::query())
        // 필터 순서에 맞게 배치
        .and(store_filter)
        .and_then(get_questions);
    // .recover(return_error);
}
```


**코드 4-16 쿼리 매개 변수 HashMap을 경로 핸들러에 추가하기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let res: Vec<Question> = store.questions.values().cloned().collect();
    Ok(warp::reply::json(&res))
}
```


**코드 4-17 구조를 알아보기 위해 매개변수 디버깅하기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    println!("{:?}", params);
    let res: Vec<Question> = store.questions.values().cloned().collect();
    Ok(warp::reply::json(&res))
}
```


**코드 4-18 매개변수에 값이 들어 있는지 패턴 검사하기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    match params.get("start") {
        Some(start) => println!("{}", start),
        None => println!("No start value"),
    }
    // println!("{:?}", params);
}
```


**코드 4-19 start를 출력하여 구조 알아보기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    // HashMap에 start 키에 대한 Some 값이 있으면 Some으로 값을 추출하고 변수 n을 만든다
    // HashMap에 start 키가 없으면 if는 실패하고 컴파일러는 다음 행으로 넘어간다
    if let Some(n) = params.get("start") {
        println!("{}", n);
    }
    // match params.get("start") {
    //   Some(start) => println!("{}", start),
    //   None => println!("No start value"),
    // }
}
```


**코드 4-20 start 매개변수를 usize 타입으로 파싱하기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    // HashMap::get 함수는 Option<&String>을 반환하므로 (값이 있는 경우) 문자열에 대한 참조를 얻는다
    if let Some(n) = params.get("start") {
        // parse 메서드는 Result를 반환하므로 Debug({:?})로 다시 전환하는 것이다
        // 콘솔에 결과를 출력하는 대신 에러 처리를 추가하고 시작 값을 할당한다
        println!("{:?}", n.parse::<usize>());
    }
}
```


**코드 4-21 값을 usize 타입으로 파싱하되 실패하면 에러 반환하기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut start = 0;

    if let Some(n) = params.get("start") {
        start = n.parse::<usize>().expect("Could not parse start");
    }

    println!("{}", start);
}
```


### 4.1.5 전용 에러 반환하기


**코드 4-22 사용자 정의 Error 열거 타입 추가하기**

```rust
#[derive(Debug)]
enum Error {
    // 매개변수에서 숫자를 파싱할 수 없다
    ParseError(std::num::ParseIntError),
    // start나 end 매개변수가 누락되었다
    MissingParameters,
}
```

사용자 정의 에러를 구현하기 위해 두 단계 추가
1. Display 트레이트를 구현해 러스트가 에러를 문자열로 출력하게 한다
2. Warp 경로 핸들러에서 반환하도록 에러에 Warp의 Reject 트레이트를 구현한다


**코드 4-23 Error 열거 타입에 Display 트레이트 추가하기**

```rust
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameters")
        }
    }
}
```


**코드 4-24 사용자 정의 에러를 Warp의 Reject 트레이트로 구현하기**

```rust
// Warp의 Reject 트레이트는 마커 트레이트(marker trait)이다
// 내용이 비어 있지만 특정 속성을 충족한다는 일종의 확인을 컴파일러에 알려주어야 할 때 마커 트레이트를 사용한다
impl Reject for Error {}
```

Warp의 경로 핸들러에서 에러를 받을 수 있지만 두가지를 보완해야 한다
- 전용 함수에서 매개변수 처리 부분 출력하기
- get_questions 경로 핸들러 내에서 함수를 호출하여 발생한 에러를 return_error 함수로 보내 처리하기


**코드 4-25 Pagination 구조체를 추가하여 받는 쿼리 매개변수를 구조화한다**

```rust
#[derive(Debug)] // 구조체를 println!로 출력할 수 있고, 다른 방식으로도 출력할 수 있다
struct Pagination {
    start: usize,
    end: usize,
}
```


**코드 4-26 쿼리 추출 코드를 전용 함수로 옮기기**

```rust
fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    // HashMap에 .contains 메서드를 써서 두 매개변수가 모두 있는지 확인한다
    if params.contains_key("start") && params.contains_key("end") {
        // 새로운 Pagination 객체를 만들고 start와 end 변호를 설정한다
        // 두 매개변수가 모두 있으면 Result를 반환(return OK())한다. 바로 돌아가기 위해서 return 키워드를 사용한다
        return Ok(Pagination {
            start: params
                // HashMap의 .get 메서드로 옵션을 반환한다
                // 해당 메서드로는 키가 확실히 존재하는지 보증할 수 없기 때문이다
                .get("start")
                // 몇 줄 전에 HashMap에 매개변수가 두 개인지 먼저 확인했으므로 안전하지 않은 .unwrap을 사용해도 된다
                .unwrap()
                // HashMap의 &str 값을 usize 정수 타입으로 파싱한다
                .parse::<usize>()
                // 파싱 결과로 Result를 반환하며, 값을 풀어내거나 파싱에 실패했을 때는 .map_err와 줄 끝의 물음푤르 이용해 에러를 반환한다
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
        });
    }
    // 그렇지 않은 경우 if 절은 실행되지 않고 바로 Err로 이동하여 사용자 정의 MissingParameters 에러를 반환한다
    // 여기서 이중 콜론(:)을 사용하여 Error 열거 타입에서 접근한다
    Err(Error::MissingParameters)
}
```


**코드 4-27 전달한 매개변수에 따라 다른 질문 반환하기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        // Paginataion 객체를 반환하거나 끝에 있는 물음표(?)로 사용자 정의 에러를 반환한다
        let pagination = extract_pagination(params)?;
        let res: Vec<Question> = store.questions.values().cloned().collect();
        // start, end 매개변수를 사용하여 Vec에서 슬라이스를 가져와 사용자가 지정한 질문을 반환
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
    // let mut start = 0;
    // if let Some(n) = params.get("start") {
    //   start = n.parse::<usize>().expect("Could not parse start");
    // }
    // println!("{}", start);
    // let res: Vec<Question> = store.questions.values().cloned().collect();
    // Ok(warp::reply::json(&res))
}
```


**코드 4-28 매개변수 추출 에러 처리하기**
use warp::{http::StatusCode, reject::Reject, Filter, Rejection, Reply};

```rust
async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
```

**내부적으로 개선된 부분**
- 로컬 JSON 파일에서 읽기
- 많은 양의 코드를 제거하기
- 경로 핸들러에 상태 전달하기
- 사용자 정의 에러 추가하기



## 4.2 질문을 POST, PUT, DELETE 하기

---

스토리지를 수정하는 데 몇 가지 새로운 작업이 필요하다
- 매개변수가 있는 HTTP PUT 요청에 대한 경로 열기
- HTTP POST 요청에 대한 경로 열기
- PUT, POST 요청 본문에서 JSON을 받아와 읽기
- 스레드에 안전한 방식으로 인메모리 스토리지 수정하기


두 가지 문제 해결
- 프로세스 둘 이상이 동시에 데이터를 변경하는 것을 방지해야 한다
- 변경이 필요한 경우 각 경로 핸들러에 데이터 저장소의 소유권을 부여해야 한다

러스트는 힙에 있는 이 구조의 소유권을 스택에 있는 여러 포인터 중 하나만 가질 수 있도록 한다  
그리고 소유권을 가진 포인터만 수정할 수 있다

두가지 옵션 고려
- 경로 핸들러마다 저장소 사본을 만든다
- 경로 핸들러 하나가 끝날 때까지 기다렸다가 저장소 소유권을 돌려준다. 그리고 다음 경로 핸들러에 넘긴다

이런 문제를 처리할 수 있는 기능
- `Rc<T>`
- `Arc<T>`

`Rc`, `Arc` 타입은 기본 데이터 구조 `T`를 힙에 배치하고 스택에 포인터를 생성한다  
그러면 동일한 데이터를 참조하는 해당 포인터의 복사본을 만들 수 있다  
`Rc`는 단일 스레드 시스템에서만 작동하고  
`Arc`는 다중 스레드를 위한 것으로 여러 스레드 간에 데이터를 공유할 수 있다는 것이 이 둘의 차이점이다

`Arc` 타입은 **원자적 참조 카운터**이다

한 스레드의 HTTP POST 요청으로 질문을 추가할 수 있고 다른 스레드의 HTTP PUT 요청으로 기존 질문을 변경 해야되는 상황에  
두 가지 타입 중 하나를 사용할 수 있다

- `Mutex`
    - 한 번에 쓰기나 읽기 하나만 허용하고 나머지는 차단한다
- `RwLock`
    - 읽기를 여러 개 허용하고 쓰기는 하나만 허용한다

둘 다 읽는 주체와 쓰는 주체가 해당 데이터에 대한 고유한 권한을 가지고 있는지 확인한다  
쓰는 주체나 읽는 주체가 접근을 요청하면 바로 데이터를 잠그고 이전의 작업이 완료되면 다음 작업을 위해서 잠근을 풀어 준다

두 타입 모두 동기 작업에 중점을 둔 `std::sync` 모듈의 일부이므로 비동기 환경에는 적합하지 않다  
비동기 환경에서는 `RwLock` 타입의 구현을 사용할 수 있다


**코드 4-29 스레드에 안전한 HashMap 만들기**

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
        }
    }
}
```


**코드 4-30 저장소 읽는 방식을 수정하기**

```rust
async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        let res = &res[pagination.start..pagination.end];
        Ok(warp::reply::json(&res))
    } else {
        let res: Vec<Question> = store.questions.read().await.values().cloned().collect();
        Ok(warp::reply::json(&res))
    }
}
```

### 4.2.2 질문 추가하기


`/questions` 경로의 HTTP POST 요청에는 새로운 질문이 포함될 것으로 기대한다

```
API Routes
GET    /questions (empty body; return JSON)
POST   /questions (JSON body; return HTTP status code)
```

**코드 4-31 저장소에 질문을 추가하는 경로 핸들러 추가하기**

```rust
async fn add_question(
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    // 저장소에서 구현한 RwLock을 사용
    store
        .questions
        .write() // 쓰기 요청
        .await
        // 러스트의 소유권 원칙 
        // 첫 번째 매개변수는 질문 ID에 접근하는데 이렇게 하면 질문의 소유권을 해시 맵의 insert 메서드로 보낸다
        // 함수 다른 곳에서 질문을 사용하지 않는다면 괜찮겠지만 두 번째 인수로 질문을 받아 해시 맵에 저장하려고 한다
        // 따라서 첫 번째 매개변수의 question.id를 복제하여 사본을 만든 다음
        // 두 번째 매개변수 질문의 소유권을 insert 메서드에 넘긴다
        .insert(question.id.clone(), question); // 새 질문 삽입

    Ok(warp::reply::with_status("Question added", StatusCode::OK))
}
```


**코드 4-32 /question에 POST 경로 추가하기**

```rust
#[tokio::main]
async fn main() {

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    // 새로운 변수를 만들어 warp::post로 HTTP POST 요청에 대한 필터를 만든다
    let add_question = warp::post()
        // 아직은 동일한 최상위 경로 /questions 에서 요청을 받는다
        .and(warp::path("questions"))
        // 경로 정의를 마친다
        .and(warp::path::end())
        // 이 경로에 저장소를 추가해서 나중에 경로 핸들러에 전달한다
        .and(store_filter.clone())
        // 내용을 JSON으로 추출한다. 추출한 내용은 매개변수로 추가된다
        .and(warp::body::json())
        // 저장소와 추출한 json 값으로 add_question을 실행한다
        .and_then(add_question);

    let routes = get_questions
        .or(add_question)
        .with(cors)
        // Not Found 경로로 끝나기 전에 다른 경로로 전달하기 위해 
        // get_questions 필터 끝에 있던 recover를 삭제하고 경로 끝에 추가
        .recover(return_error);
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```


curl 테스트 시 add_question 경로 핸들러가 실패하는 것 확인(의도적으로 JSON에 id 항목을 빼고 넣음)

```shell
$ curl --location --request POST 'localhost:3030/questions' \
    --header 'Content-Type: application/json' \
    --data-raw '{
        "title": "New question",
        "content": "How does this work again?"
  }'
Route not found
```

### 4.2.3 질문 업데이트하기

웹 프레임워크 Warp는 URL 매개변수를 파싱해서 경로 핸들러로 전달해야 한다  
그래야 나중에 해시 맵을 인덱싱해서 값을 업데이트할 수 있다


PUT 메서드에는 Warp에서 파싱하게끔 URL 매개변수가 추가되어 경로 핸들러에 추가된다

```
API Routes
GET    /questions (empty body; return JSON)
POST   /questions (JSON body; return HTTP status code)
PUT    /questions/:questionId (JSON body, return HTTP status code)
```


**코드 4-33 질문을 수정하고 질문을 찾지 못하면 404를 반환한다**

```rust
#[derive(Debug)]
enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    QuestionNotFound,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            Error::MissingParameters => write!(f, "Missing parameters"),
            Error::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    // HashMap 객체에 직접 쓰던 add_question 경로 핸들러와 달리
    // 질문의 변경 가능한 참조를 요청해 내용을 변경한다
    // match 블록을 사용하여 HashMap 객체에 전달하려는 ID에 맞는 질문이 있는지 확인한다
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        // match 표현식의 가지(arm)를 사용해서 찾은 질문을 풀어낸 후 *q = question으로 내용을 덮어쓴다
        Some(q) => *q = question,
        // 질문이 없으면 즉시 중단하고 사용자 정의 에러인 QuestionNotFound를 반환한다
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }
    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}
```


**코드 4-34 /questions/:questionId에 PUT 경로 추가하기**

```rust
#[tokio::main]
async fn main() {
    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    // 새로운 변수를 만들고 warp::put을 사용해 HTTP PUT 요청에 대한 필터를 구성한다
    let update_question = warp::put()
        // 아직까지는 동일한, 최상위 경로 /questions를 쓴다
        .and(warp::path("questions"))
        // String 매개변수를 추가하여 /questions/1234 같은 경로에서 동작하도록 한다
        .and(warp::path::param::<String>())
        // 경로 정의를 끝낸다
        .and(warp::path::end())
        // 이 경로에 저장소를 추가해서 나중에 경로 핸들러로 전달한다
        .and(store_filter.clone())
        // JSON 내용을 추출해서 매개변수로 추가한다
        .and(warp::body::json())
        // 저장소와 JSON을 매개변수로 하여 update_question을 호출한다
        .and_then(update_question);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```


새로 만든 경로에 PUT 요청을 실행할 때 ID를 누락했다면 HTTP 메서드와 경로에 대응하는 Warp 경로가 없으므로 서버는 404를 반환한다

```shell
$ curl --location --request PUT 'localhost:3030/questions' \
    --header 'Content-Type: application/json' \
    --data-raw '{
        "id": "1",
        "title": "NEW TITLE",
        "content": "OLD CONTENT"
    }'
Route not found
```


### 4.2.4 잘못된 요청 처리하기


**코드 4-35 PUT 요청 내용에서 질문을 읽지 못할 때 에러 추가하기**

```rust
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::Method,
    http::StatusCode,
    reject::Reject,
    Filter, Rejection, Reply,
};

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    // Warp에서 BodyDeserializeError를 가져와 Rejection에 이러한 타입의 에러가 있는지 return_error 함수에서 확인한다
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            // 해당 에러가 있다면 에러 메시지를 String 객체로 반환하고 응답에 StatusCode를 추가한다
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
```


content 필드가 누락된 질문을 추가하는 경우 애플리케이션에서 에러를 돌려준다

```shell
$ curl --location --request POST 'localhost:3030/questions' \
    --header 'Content-Type: application/json' \
    --data-raw '{
        "id": "5",
        "title": "NEW TITLE"
    }'
Request body deserialization error: missing field content at line 4 column 1
```


### 4.2.5 저장소에서 질문 삭제하기


질문 관련 항목을 완성하는 마지막 메서드는 HTTP DELETE이다

```
API Routes
GET    /questions (empty body; return JSON)
POST   /questions (JSON body; return HTTP status code)
PUT    /questions/:questionId (JSON body, return HTTP status code)
DELETE /questions/:questionId (empty body; return HTTP status code)
```


**코드 4-36 질문을 삭제하는 경로 핸들러 추가하기**

```rust
async fn delete_question(
    id: String,
    store: Store
) -> Result<impl warp::Reply, warp::Rejection> {
    // HashMap에서 키로 값을 가져올 때 질문을 찾지 못할 수 있기 때문에 match 사용
    match store.questions.write().await
        // 질문 ID를 전달할 수 있음
        .remove(&QuestionId(id)) {
        // 무언가 찾으면 올바른 상태 코드, 메시지와 함께 OK를 반환
        // _는 반환되는 값이 필요 없음을 알려준다
        Some(_) => return Ok(warp::reply::with_status("Question deleted", StatusCode::OK)),
        None => return Err(warp::reject::custom(Error::QuestionNotFound)),
    }
}
```


**코드 4-37 질문 삭제를 위한 경로 추가하기**

```rust
#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```



## 4.3 url-form-encoded로 answers POST 요청하기

---


### 4.3.1 url-form-encoded 와 JSON의 차이점

마지막으로 구현하는 경로: POST와 www-url-encoded 내용으로 답변 추가하기

```
API Routes
GET    /questions (empty body; return JSON)
POST   /questions (JSON body; return HTTP status code)
PUT    /questions/:questionId (JSON body, return HTTP status code)
DELETE /questions/:questionId (empty body; return HTTP status code)
POST   /answers (www-url-encoded body; return HTTP status code)
```


POST 요청을 보내는 예

```shell
POST /test HTTP/1.1
Host: foo.example
Content-Type: application/x-www-form-urlencoded
Content-Length: 27

field1=value1&field2=value2
```

`application/x-www-form-urlencoded` 요청을 보내는 POST curl의 예

```shell
$ curl --location --request POST 'localhost:3030/questions' \
    --header 'Content-Type: application/x-www-form-urlencoded' \
    --data-urlencode 'question_id=1' \
    --data-urlencode 'title=First question' \
    --data-urlencode 'content=This is the question I had.'
```

JSON으로 보내는 POST 요청은 다음과 같다(차이점은 음영으로 표시함)

```shell
$ curl --location --request POST 'localhost:3030/questions' \
    --header 'Content-Type: application/json' \
    --data-raw '{
        "id": "1",
        "title": "New Question",
        "content": "How and why?"
    }'
```


### 4.3.2 url-form-encoded로 answers 추가하기


**코드 4-38 프로젝트에 answers 추가하기**

```rust
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct AnswerId(String);

// 새로운 구조체 Answer를 추가해 시스템에서 잡변이 어떻게 보여야 하는지에 대한 요구 사항을 지정한다
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Answer {
    id: AnswerId,
    content: String,
    // answers 구조체는 질문 속성과 동일한 서명을 가진다
    question_id: QuestionId,
}

// Store에 새로운 answers 구조체를 추가한다
#[derive(Clone)]
struct Store {
    questions: Arc<RwLock<HashMap<QuestionId, Question>>>,
    // 답변을 저장하기 위한 HashMap을 읽기-쓰기 잠금(RwLock)으로 래핑하여 데이터 무결성을 보장하고
    // 스레드 간에 구조를 전달할 수 있도록 Arc로 래핑한다
    answers: Arc<RwLock<HashMap<AnswerId, Answer>>>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Arc::new(RwLock::new(Self::init())),
            answers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

// add_answer 경로 핸들러 구현
async fn add_answer(
    store: Store,
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let answer = Answer {
        id: AnswerId("1".to_string()),
        content: params.get("content").unwrap().to_string(),
        // 핵심 부분은 해시 맵에서 매개변수를 읽는 것이다
        // 여기서는 실제 운용 목적이 아니어서 unwrap을 사용했다
        // 매개변수를 찾을 수 없으면 러스트 애플리케이션은 패닉 상태가 되어 비정상 종료된다
        // match를 사용하여 누락된 매개변수에서 발생하는 에러를 개별적으로 반환해야 한다
        question_id: QuestionId(params.get("questionId").unwrap().to_string()),
    };

    store
        .answers
        .write()
        .await
        .insert(answer.id.clone(), answer);

    Ok(warp::reply::with_status("Answer added", StatusCode::OK))
}
```


**코드 4-39 url-form으로 답변을 추가하는 경로 핸들러 추가하기**

```rust
#[tokio::main]
async fn main() {
    let store = Store::new();
    let store_filter = warp::any().map(move || store.clone());

    // add_answer 함수의 매개변수에 HashMap<String, String>을 추가한다
    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        // Warp의 body::form() 함수를 사용하여 URL-form-encoded 내용을 추출한다
        .and(warp::body::form())
        .and_then(add_answer);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(delete_question)
        .or(add_answer)
        .with(cors)
        .recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

**연습**

- 수 작업으로 만드는 대신 임의의 고유한 ID를 만든다
- 필요한 필드가 없는 경우 에러 처리를 추가한다
- 답변을 게시하려는 질문이 있는지 확인한다
- 답변 경로를 /questions/:questionId/answers로 바꾼다


## 4.4 요약

---

- 로컬 `HashMap` 객체를 인메모리 스토리지로 삼는 것으로 시작한다. 실제 데이터베이스로 진행하기 전에 개념 설계를 더 빠르게 진행할 수 있다
- `Serde` JSON 라이브러리를 사용하여 외부 JSON 파일을 파싱하고 사용자 정의 데이터 타입에 매핑할 수 있다
- 해시 맵은 인메모리 스토리지로는 쓸 만하지만, 사용하는 키는 서로 비교할 수 있도록 트레이트 세 개(`PartialEq`, `Eq`, `Hash`)를 반드시 구현해야 하는 것을 명심해야 한다
- 상태를 전달하려면 객체의 복사본을 반환하는 필터를 만들어서 경로 핸들러 둘 이상에 전달해야 한다
- HTTP로 받은 데이터는 `Warp`의 필터로 파싱할 수 있으며 프레임워크의 `json`, `query`, `param`, `form`을 사용할 수 있다
- 경로에서 데이터를 추출하려면 더 많은 필터를 추가해야 하며, `Warp`는 마지막에 호출하는 함수에 추출한 데이터를 매개변수로 자동 추가해 준다
- HTTP 내용이나 경로 매개변수에서 받아 파싱한 데이터 타입에 맞게 사용자 정의 타입을 만드는 것이 좋다
- 문제가 생겼을 때 `Warp`로 반환할 수 있도록 사용자 정의 에러에 트레이트를 구현해야 한다
- `Warp`는 적절한 HTTP 응답을 반환하는 데 쓰이는 HTTP 상태 코드 타입이 포함되어 있다