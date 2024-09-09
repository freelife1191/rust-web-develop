use std::{io::prelude::*, net::{TcpStream, TcpListener}};

///
/// 1-11 러스트로 작성한 TCP 서버 예제
fn main() {
	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
	for stream in listener.incoming() { 
		  let stream = stream.unwrap(); 
		  handle_stream(stream); 
	}
}

fn handle_stream(mut stream: TcpStream) {
    // 여기서 실제적인 일을 한다
    let mut buffer = [0; 1024]; 
    stream.read_exact(&mut buffer).unwrap(); 
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let response = "HTTP/1.1 200 OK\r\n\r\n";
    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap(); 
}
