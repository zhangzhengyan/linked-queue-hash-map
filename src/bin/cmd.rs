use queue_hash_map::LinkedHashMap;

fn main() {
    let mut map = LinkedHashMap::new();

    map.insert(1, 10);
    map.print();
    println!("--------------------");

    map.insert(2, 20);
    map.print();
    println!("--------------------");

    map.delete_first();
    map.print();
    println!("--------------------");

    map.delete_first();
    map.print();
    println!("--------------------");

    map.insert(3, 30);
    map.print();
    println!("--------------------");

    map.insert(4, 40);
    map.print();
    println!("--------------------");

}