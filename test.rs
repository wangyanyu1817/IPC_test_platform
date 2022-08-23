fn main() {
    let mut s0: String = String::from("hello");
    let s1: String = String::from("hello");
    println!("lee");
    s0 = format!("{}\n{}",s0,s1);
    // s0 += &s1;
    println!("{:?}",s0);
}