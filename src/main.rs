use macipr::MacAddr;

fn main() {
    let mac = MacAddr::new(0, 1, 2, 3, 4, 5);
    println!("{}", mac);
}
