fn main() {
    let mut address = String::from("Street 1"); // 가변 변수를 선언하고 String 값을 할당한다
    add_postal_code(&mut address); // add_postal_code 함수에 address 참조를 전달한다
    println!("{}", address); // 수정된 address를 출력한다
}

fn add_postal_code(address: &mut String) { // 함수 매개변수는 String의 가변 타입 참조를 기대한다
    address.push_str(", 1234 Kingston"); // push_str 메서드는 String을 직접 변경한다
}
