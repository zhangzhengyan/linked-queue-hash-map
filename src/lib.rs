use std::borrow::Borrow;
use std::collections::hash_map::{self, HashMap};
use std::ptr::{self};
use std::hash::{BuildHasher, Hash, Hasher};
use std::mem;
use std::fmt::Debug;
use std::time::{Duration, Instant};

struct KeyRef<K> {
    k: *const K,
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}

impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

#[derive(Hash, PartialEq, Eq)]
#[repr(transparent)]
struct Qey<Q: ?Sized>(Q);

impl<Q> Qey<Q> 
where
    Q: ?Sized,
{
    fn from_ref(q: &Q) -> &Self {
        unsafe { mem::transmute(q) }
    }
}

impl<K, Q> Borrow<Qey<Q>> for KeyRef<K>
where
    K: Borrow<Q>,
    Q: ?Sized,
{
    fn borrow(&self) -> &Qey<Q> {
        Qey::from_ref(unsafe { (*self.k).borrow() })
    }
}




struct Node<K: Debug, V: Debug + PartialOrd> {
    key: K,
    value: V,
    time: Instant,
    pre: *mut Node<K, V>,
    next: *mut Node<K, V>,
}

struct Head<K: Debug, V: Debug + PartialOrd> {
    first: *mut Node<K, V>,
    cur_time: Instant,
}

pub struct LinkedHashMap<K: Debug, V: Debug + PartialOrd, S = hash_map::RandomState> {
    map: HashMap<KeyRef<K>, *mut Node<K, V>, S>,

    head: *mut Head<K, V>,
    cur:  *mut Node<K, V>,      //支持取出value 使用而不是直接删除
    tail: *mut Node<K, V>,
}

impl<K: Debug, V: Debug + PartialOrd> Node<K, V> {
    fn new(k: K, v: V, time: Instant) -> Self {
        Node { 
            key: k, 
            value: v,
            time: time,  
            pre: ptr::null_mut(),
            next: ptr::null_mut(),
        }
    }
}

impl <K: Hash + Eq + Debug, V: Debug + PartialOrd, S: BuildHasher> LinkedHashMap<K, V, S> { 
    fn with_map(map: HashMap<KeyRef<K>, *mut Node<K, V>, S>) -> Self {
        LinkedHashMap {
            map,
            head: ptr::null_mut(),
            cur: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    fn ensure_guard_node(&mut self) {
        if self.head.is_null() {
            // allocate the guard node if not present
            unsafe {
                let node_layout = std::alloc::Layout::new::<Head<K, V>>();
                self.head = std::alloc::alloc(node_layout) as *mut Head<K, V>;
                (*self.head).first = ptr::null_mut();
                (*self.head).cur_time = Instant::now();
                self.cur = ptr::null_mut();
                self.tail = ptr::null_mut();
            }
        }
    }

    #[inline]
    fn detach_first(&mut self) {
        unsafe {
            let first = (*self.head).first;
            if !first.is_null() {  
                if first == self.cur {
                    self.cur = (*first).next
                }

                (*self.head).first = (*first).next;
                let first = (*self.head).first;
                if !first.is_null() {
                    (*self.head).cur_time = (*first).time;
                    (*first).pre = ptr::null_mut();
                } else {
                    self.cur = ptr::null_mut();
                    self.tail = ptr::null_mut();
                }
            } else {
                self.cur = ptr::null_mut();
                self.tail = ptr::null_mut();
            }
        }   
    }

    #[inline]
    fn detach(&mut self, node: *mut Node<K, V>) {
        unsafe {
            if node == (*self.head).first {
                self.delete_first();
                return ;
            }
            
            (*(*node).pre).next = (*node).next;
            (*(*node).next).pre = (*node).pre;
        }

        if node == self.tail {
            self.tail = ptr::null_mut();
        }

        unsafe {
            if node == self.cur {
                self.cur = (*node).next;
            }
        }
    }

    #[inline]
    fn drop(node: *mut Node<K, V>) {
        unsafe {
            Box::from_raw(node);
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool 
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.map.contains_key(Qey::from_ref(k))
    }

    pub fn insert(&mut self, k: K, v: V) {
        self.ensure_guard_node();
        
        let time = Instant::now();
        let node = Box::into_raw(Box::new(Node::new(k, v, time)));

        unsafe {
            (*node).next = ptr::null_mut();
        }

        let keyref = unsafe {
            &(*node).key
        };

        //nodes insert to map
        self.map.insert(KeyRef { k: keyref }, node);
        
        // nodes link to the end of list
        unsafe {
            if (*self.head).first.is_null() {
                (*self.head).first = node;
                (*self.head).cur_time = time;
            }

            if self.cur.is_null() {
                self.cur = node;
            }

            if self.tail.is_null() {
                self.tail = node;
                (*node).pre = ptr::null_mut();
            } else {
                (*self.tail).next = node;
                (*node).pre = self.tail;
                self.tail = node;
            }
        }
    }

    //
    pub fn delete_first (&mut self) {
       unsafe {
            let node =  (*self.head).first;

            if !node.is_null() {
                //从map中移除
                let k = &(*node).key;
                self.map.remove(&KeyRef {k});

                //先从链表中删除
                self.detach_first();
                
                //释放内存空间
                Self::drop(node);                
            }
        }
    }

    pub fn delete<Q> (&mut self, k: &Q) 
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        let node = self.map.get(Qey::from_ref(k));
        let node = match node {
            Some(node) => {
                *node
            }
            None => ptr::null_mut()
        };
        
        if !node.is_null() {
            //从map中移除
            self.map.remove(Qey::from_ref(k));

            //从链表中删除
            self.detach(node);
            //释放内存空间
            Self::drop(node);
        }
    }

    pub fn get_mut<Q: ?Sized>(&self, k: &Q) -> Option<&mut V> 
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        self.map
            .get(Qey::from_ref(k))
            .map(|e| unsafe { &mut (**e).value })
    }

    pub fn value_gt<Q: ?Sized>(&self, k: &Q, v: V) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        match self.get_mut(k) {
            Some(value) => (*value) < v,
            None => true,               //key 不存在的情况也返回true
        }
    }

    //panic if key not exists, so make sure the key already exists
    pub fn value_gt_cas<Q: ?Sized>(&self, k: &Q, v: V) -> bool
    where
        K: Borrow<Q>,
        Q: Eq + Hash,
    {
        if *self.get_mut(k).unwrap() < v {
            *self.get_mut(k).unwrap() = v;
            return true;
        }

        return false;
    }

    pub fn insert_or_gt_cas (&mut self, k: K, v: V) 
    {
        if self.contains_key(&k) {
            self.value_gt_cas(&k, v);
        } else {
            self.insert(k, v);
        }
    }

    pub fn pop_cur (&mut self) -> Option<(&K, &V)>{
        if self.cur.is_null() {
            return None;
        }

        let cur = self.cur;

        //指针向下移动就可以了
        unsafe {
            self.cur = (*self.cur).next;
        }

        //把当前的(key, value)返回就可以了
        unsafe {
            Some((&(*cur).key, &(*cur).value))
        }
    }

    pub fn print(&self) {
        unsafe {
            let mut node = (*self.head).first;
            while !node.is_null() {
                  println!("node is key {:?}, value {:?}, time {:?}", (*node).key,(*node).value, (*node).time);
                  node = (*node).next;
            }
        }
    }

    /// time_out, Unit of milliseconds
    pub fn release_timeout(&mut self, time_out: u64) {
        self.ensure_guard_node();

        unsafe {
            let mut first = (*self.head).first;
            if first.is_null() {
                return ;
            }

            let t_out = Duration::from_millis(time_out);
        
            while !first.is_null() && Instant::now() - (*first).time > t_out {
                self.delete_first();
                first = (*self.head).first;
            }
        }
    }

}

impl <K: Hash + Eq + Debug, V: Debug + PartialOrd> LinkedHashMap<K, V> {
    pub fn new() -> Self {
        Self::with_map(HashMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_map(HashMap::with_capacity(capacity))
    }
}

