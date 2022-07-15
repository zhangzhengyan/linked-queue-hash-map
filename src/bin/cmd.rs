use queue_hash_map::LinkedHashMap;

fn main() {
    let mut map = LinkedHashMap::with_capacity(30);

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

    *map.get_mut(&4).unwrap() = 50;
    map.print();
    println!("--------------------");

    map.insert(5, 55);
    map.print();
    println!("--------------------");

    map.insert(6, 66);
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

    map.delete_first();
    map.print();
    println!("--------------------");

    map.insert_or_gt_cas(8, 90);
    map.print();
    println!("--------------------");

    map.insert_or_gt_cas(9, 88);
    map.print();
    println!("--------------------");

    println!("compare is {}", map.value_gt(&9, 10));

}