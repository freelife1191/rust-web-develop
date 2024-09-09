# 🚦 2. 기초 쌓기

## 2.1 러스트 플레이북 따라 하기

---


**코드 2-2 Option 값에 match 사용하기**

```rust
fn main() {
    struct Book {
        title: String,
        isbn: Option<String>,
    }

    let book = Book {
        title: "Great book".to_string(),
        isbn: Some(String::from("1-123-456")),
    };

    match book.isbn { 
        Some(i) => println!("The ISBN of the book: {} is: {}", book.title, i),
        None => println("We don't know the ISBN of the book"),
    }
}
```


- 텍스트를 소유하고 수정해야 하는 경우 String 타입을 만든다
- 기본 텍스트를 단지 보기만 할 때는 &str을 사용한다
- 구조체로 새 데이터 타입을 만들 때 문자열은 보통 String 필드 타입으로 만든다
- 문자열/텍스트를 함수에 전달할 때 일반적으로 &str을 사용한다


### 2.1.7 Result 다루기

**코드 2-20 러스트 표준 라이브러리의 Result 정의**

```rust
pub enum Result<T, E> {
    OK(T),
    Err(E),
}
```

Option 과 마찬가지로 Result 에는 다양한 메서드와 트레이트가 구현되어 있다  
이러한 메서드 중 하나는 expect 로 문서에 따르면 포함된 OK 값을 반환한다  
unwrap 을 사용하는 방법도 있지만 사용자가 지정한 에러 메시지 없이 패닉 상태가 된다

> 실제 서비스에서는 unwrap 이나 expect를 사용하지 않는 것이 좋다  
> 패닉이 발생할 것이고 애플리케이션이 충돌할 것이다  
> 항상 match 로 에러 사례를 처리하거나, 그렇지 않으면 에러를 포착하고 정상적으로 반환하는지 확인해야 한다  

Result와 Option과 동일하지만 주요 차이점은 Error 변형이다  
Option은 데이터가 있어도 되고 없어도 된다  
Result는 실제로 데이터가 있을 것으로 예상하며 그렇지 않은 경우를 적극적으로 관리하여 사용해야 된다

## 2.2 웹 서버 만들기

---

**Warp를 이용한 최소한의 러스트 웹 서버**

```rust
use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::get()
        .map(|| format!("Hello, World!"));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 1337)).await;
}
```

### 2.2.2 러스트의 비동기 환경

러스트의 모든 비동기 애플리케이션에서 가장 중요한 결정은 런타임을 선택하는 것이다  
런타임에는 이미 커널 API(대부분의 경우 Mio라는 라이브러리)에 대한 추상화가 포함되어 있지만 그래도 먼저 러스트에서 제공하는 구문과 타입을 살펴보자


### 2.2.3 러스트에서 async/await 다루기

```rust
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let resp = reqwest::get("https://httpbin.org/ip")
        .await?
        .json::<HashMap<String, String>>()
        .await?;
    println!(":#?", resp);
    OK(())
}
```

```
[dependencies]
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
```

### 2.2.4 러스트의 퓨처 타입 사용하기

```rust
use std::task::Poll;

pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) 
        -> Poll<Self::Output>;
}
```

- Future 에는 파일 또는 문자열이 될 수 있는 Output이라는 연관 타입과 poll 메서드가 있다  
- poll 메서드는 퓨처가 준비되었는지 확인할 때 자주 호출되며, Pending 또는 Ready 값을 갖는 poll 타입을 반환한다  
- 준비되면 poll은 두 번째 줄에 지정된 타입 또는 에러를 반환한다  
- 그리고 퓨처가 준비되면 결과를 반환하고 변수에 할당된다

### 2.2.5 런타임 고르기

가장 인기 있는 런타임 중 하나인 Tokio는 업계 전반에서 널리 사용된다  
따라서 애플리케이션에 대한 첫 번째 안전한 선택지이다  

런타임은 스레드를 생성하고, 퓨처를 폴링하고, 완료까지 담당한다  
또한 작업을 커널에 전달하고 비동기 커널 API를 사용해 병목 현상이 발생하지 않도록 하는 역할도 한다  
Tokio는 운영체제 커널과 비동기 통신하기 위해 Mio 크레이트를 사용한다


### 2.2.6 웹 프레임워크 고르기

**러스트가 제공하는 상위 네 개의 웹 프레임워크**

- `Actix Web`
  - 가장 완벽하고 적극적으로 사용되는 웹 프레임워크이며 많은 기능을 담고 있다
  - 때때로 독자적인 부분이 있을 수 있다
- `Rocket`
  - 매크로를 사용하여 경로 핸들러를 표기하고, JSON 파싱 기능이 내장되어 있다
  - 견고한 웹 서버를 작성하는 데 필요한 모든 기능이 포함된 완전한 프레임워크이다
- `Warp`
  - 러스트를 위한 최초의 웹 프레임워크 중 하나다
  - Tokio 커뮤니티와 밀접하게 개발되었으며 상당히 자유롭다
  - 가장 기본적인 프레임워크로, 개발자에게 설계 결정을 많이 맡긴다
- `Axum`
  - 최신 프레임워크이며 Tokio 생태계의 기존 크레이트와 Warp 및 다른 프레임워크에서 배운 설계 교훈을 바탕으로 최대한 많은 것을 구축하려고 한다

`Actix Web`은 자체 런타임을 제공한다(하지만 Tokio를 선택할 수도 있음)  
`Rocket`, `Warp`, `Axum` 프레임워크는 Tokio를 사용한다

이 책에서는 `Warp`를 선택  
부담스럽지 않을 정도로 작고, 사용자가 많아 잘 관리되며, 매우 활발한 디스코드 채널도 있다

대부분의 책과 코드는 프레임워크에 구애 받지 않는다  
서버를 설정하고 경로 핸들러를 추가하면 우리는 다시 순수한 러스트 영역에 있게 되며, 이후에는 프레임워크의 많은 부분은 찾아볼 수 없다  
이 책에서는 이러한 부분을 명확하게 강조하고 있으며, 프레임워크를 선택한 후에 어느 부분에서 어떻게 작업할지 잘 알게 될 것이다


**코드 2-24 Warp를 이용한 최소한의 러스트 HTTP 서버**

```rust
use warp::Filter;

#[tokio::main]
async fn main() {
    let hello = warp::path("hello")
        .and(warp::path::param())
        .map(|name: String| format!("Hello, {}!", name)); // .map 함수는 이전 함수에서 (가능한) 인수를 가져와 변환하는 Warp 필터이다

    warp::serve(hello)
        .run(([127, 0, 0, 1], 1337))
        .await;
}
```

```
[dependencies]
tokio = { version = "1.2", feature = ["full"] }
warp = "0.3"
```


**코드 2-27 main.rs에서 Warp 서버 시작하기**

```rust
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use warp::Filter;





#[tokio::main]
async fn main() {
    let hello = warp::get().map(|| format!("Hello, World!"));
    warp::serve(hello)
        .run(([127, 0, 0, 1], 3030))
        .await;
}
```

## 2.3 요약

---

- 항상 구조체를 통해 리소스를 매핑하는 것으로 시작하고 타입 간의 관계를 생각한다
- 타입에 `new`나, 타입을 다른 타입으로 변환하는 도우미 메서드를 추가하여 단순화한다
- 러스트의 소유권, 대여 원칙과 이것이 코드 작성 방식에 어떤 영향을 미치고 이에 따라 컴파일러가 어떤 에러를 던질 수 있는지 이해하라
- 트레이트를 사용하면 기능을 추가해 사용자 정의 데이터 타입이 당신이 선택한 프레임워크와 잘 작동하도록 할 수 있다
- `derive` 매크로를 사용하여 범용 사용 사례에 대한 트레이트를 구현하면 작성해야 하는 코드를 많이 절약할 수 있다
- 타입과 프레임워크의 기능을 찾는 데 자주 사용하므로 러스트 문서와 친해져야 한다. 이는 언어를 더 잘 이해하는 데 도움이 된다
- 러스트는 비동기 구문과 타입이 함께 제공되지만, 비동기 애플리케이션을 작성하려면 더 많은 것이 필요하다
- 런타임은 동시에 여러 계산을 처리하면서 비동기 커널 API에 대한 추상화를 제공한다
- 적극적으로 유지보수되고 대규모 커뮤니티와 지원이 있으며 대기업에서 사용하는 웹 프레임워크를 선택한다
- 우리가 선택한 웹 프레임워크는 HTTP 구현, 서버 및 런타임을 추상화하므로 애플리케이션의 비즈니스 로직 작성에 집중할 수 있다
