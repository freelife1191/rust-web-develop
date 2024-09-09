# 🚦 3. 첫 경로 핸들러 만들기


## 3.1 웹 프레임워크에 대해 알아보기: Warp

---

**Warp 웹 프레임워크 선택 이유**

- 꼭 필요한 기능만 가지고 있어 작고 활발하게 유지보수되고 있으며, 커뮤니티가 활성화되어 있다
- 사실상 현재 러스트 생태계의 표준 런타임인 Tokio 런타임을 기반으로 한다
- 잘 활성화된 디스코드 채널에서 프로젝트의 창시자와 다른 사용자가 질문에 답변을 잘 해준다
- 깃허브에서 활발하게 개발되고 문서화되고 업데이트 된다


**코드 3-1 경로 필터 객체를 붙여 Warp 시작하기**

```rust
// warp에서 Filter 트레이트를 가져온다
use warp::Filter;

#[tokio::main]
async fn main() {
    // 경로 필터를 만든다
    let hello = warp::path("hello").map(|| format!("Hello, World!"));

    // 서버를 시작하고 경로 필터를 서버에 전달한다
    warp::serve(hello).run(([127, 0, 0, 1], 1337)).await;
}
```

## 3.2 첫 JSON 응답을 GET 요청으로 받기

---


**코드 3-4 첫 경로 핸들러의 추가, question 출력은 삭제**

```rust
use warp::Filter;

// Warp가 사용할 수 있게 회신과 거부를 반환하는 첫 번째 경로 핸들러를 만든다
async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new( // 요청하는 클라이언트에 반환할 새로운 question을 생성한다
        QuestionId::from_str("1").expect("No id provided"),
        "First Question".to_string(),
        "Content of question".to_string(),
        Some(vec!["faq".to_string()]),
    );

    Ok(warp::reply::json(&question)) // Warp의 json 응답을 사용해 question의 JSON 버전을 반환한다
}

#[tokio::main]
async fn main() {
    // 하나 이상의 필터를 결합하는 Warp의 .and 함수를 사용해 큰 필터 하나를 생성하고 get_item에 할당한다
    let get_items = warp::get()
        .and(warp::path("questions"))
        // path::end를 써서 정확히 /questions(예를 들어 /questions/further/params 같은 것은 안 됨)에서만 수신을 받겠다고 신호를 보낸다
        .and(warp::path::end())
        .and_then(get_questions);
    
    let routes = get_items; // 나중의 편의를 위해 경로 변수 routes를 정의한다

    warp::serve(routes) // route 필터를 Warp의 serve 메서드로 전달하고 서버를 시작한다
        .run(([127, 0, 0, 1], 3030)).await;
}
```


Serde 라이브러리는 직렬화 및 역직렬화 메서드를 프레임워크 하나로 묶은 것이다  
기본으로 러스트 생태계의 표준 직렬화(및 역직렬화) 프레임워크이다  
구조체를 JSON, TOML, BSON과 같은 형식으로 변환하고 다시 역변환할 수도 있다

**코드 3-6 프로젝트에 Serde 추가하기**

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
```

**코드 3-7 JSON 반환에 Serde의 Serialize 사용하기**

```rust
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct QuestionId(String);
```

**코드 3-8 사용자 정의 에러를 더하고 반환하기**

```rust
use warp::{Filter, reject::Reject};

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
        QuestionId::from_str("1").expect("No id provided"),
        "First Question".to_string(),
        "Content of question".to_string(),
        Some(vec!["faq".to_string()]),
    );

    match question.id.0.parse::<i32>() {
        Err(_) => Err(warp::reject::custom(InvalidId)),
        Ok(_) => Ok(warp::reply::json(&question)),
    }
}
```

**코드 3-9 경로 필터에서 우리의 에러 필터 사용하기**

```rust
#[tokio::main]
async fn main() {
    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions)
        .recover(return_error);

    let routes = get_items;

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

**코드 3-10 에러 처리에 에러 사례 추가하기**

```rust
use warp::{Filter, reject::Reject, Rejection, Reply, http::StatusCode};

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(_InvalidId) = r.find::<InvalidId>() {
        Ok(warp::reply::with_status(
            "No valid ID presented",
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found",
            StatusCode::NOT_FOUND,
        ))
    }
}
```


## 3.3 CORS 헤더 다루기

---

**코드 3-13 정확한 CORS 헤더를 반환할 수 있도록 애플리케이션 준비하기**

```rust
use warp::{Filter, reject::Reject, Rejection, Reply, http::StatusCode, http::Method};

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type") // 모든 출처 허용은 실제 운영 환경에서는 해서는 안됨
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions);

    let routes = get_items.with(cors).recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

### 3.3.2 CORS 응답 검사

**코드 3-14 OPTIONS 요청을 curl을 통해 보내기**

```shell
$ curl -X OPTIONS localhost:3030/questions \
    -H "Access-Control-Request-Method: PUT" \
    -H "Access-Control-Request-Headers: content-type" \
    -H "Origin: https://not-origin.io" -verbose
```

**코드 3-16 CORS가 실패할 때 받는 에러 타입을 디버깅하기**

```rust
async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", r); // 에러 처리 구문 추가
    if let Some(InvalidId) = r.find() {
        Ok(warp::reply::with_status(
            "No valid ID presented".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("not-in-the-request")  // 허용되지 않은 헤더를 추가
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions);

    let routes = get_items.with(cors).recover(return_error);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
```

**코드 3-19 CORS가 허용되지 않을 때 의미 있는 에러를 추가하기**

```rust
use warp::{Filter, reject::Reject, Rejection, Reply, http::StatusCode, http::Method, 
       filters::{
           cors::CorsForbidden
       }
};

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    // 현재 OPTIONS 요청을 거부하는 겨웅 에러 상황을 처리하지 않으므로 기본적으로 404 Not Found aptlwlfmf tkdydgksek
    // CorsForbidden 거부 타입치 포함되어 있으므로 사용
    if let Some(error) = r.find::<CorsForbidden>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else if let Some(InvalidId) = r.find() {
        Ok(warp::reply::with_status(
            "No valid ID presented".to_string(),
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


## 3.4 요약

---

- 선택한 라이브러리에서 어떤 스택을 다루는지 이해하는 것이 중요하다
- 일반적으로 선택한 웹 프레임워크의 비동기 작업 방식을 지원하려면 런타임을 포함해야 한다
- 모든 웹 프레임워크는 적절한 HTTP 메시지를 반환하는 웹 서버와 타입을 함께 제공한다
- 선택한 프레임워크가 가지고 있는 철학을 이해하고 몇 가지 사용 사례와 이러한 철학을 바탕으로 구현하는 방법을 생각해 본다
- 처음에는 문제없이 성공하는 작은 경로에서 시작하고, 보통 특정한 자원에 대한 GET 요청으로 시작한다
- Serde 라이브러리를 사용해 생성한 구조체를 직렬화 및 역직렬화한다
- 먼저 실패하는 경로 등의 방식을 고려한 후 사용자 정의 에러 처리를 구현한다
- 브라우저에서 HTTP 요청이 들어오고 서버가 배포된 도메인과 다른 도메인에서 시작하는 경우, CORS 워크플로의 일부인 OPTIONS 요청을 처리해야 한다
- Warp 프레임워크에는 요청에 적절하게 응답할 수 있는 cors 필터가 내장되어 있다