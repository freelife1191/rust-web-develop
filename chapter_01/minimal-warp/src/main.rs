use warp::Filter;

///
/// 코드 1-13 Warp를 사용해서 러스트로 만든 최소한의 HTTP ㅅㅓ버
#[tokio::main]
async fn main() {

    let hello = warp::path("hello")
        .and(warp::path::param())
        .map(|name: String| format!("Hello, {}!", name));

    warp::serve(hello)
        .run(([127, 0, 0, 1], 1337))
        .await;

}
