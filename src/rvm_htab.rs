const HTAB_SIZE : usize = 4096;
const KEY_LENGTH : usize = 64;
const HTAB_LOAD_FACTOR : f64 = 0.7;

#[derive(Clone)]
struct RvmHtabNode {
    key: String,
    value : i32,
    value_str : String,
    next: Option<Box<RvmHtabNode>>
}

impl RvmHtabNode {
    fn new(key: String, value: i32, value_str: String) -> Self {
        RvmHtabNode {
            key : key.clone(),
            value : value,
            value_str,
            next: None
        }
    }
}

pub struct RvmHtabCtx {
    num_nodes : u32,
    size : usize,
    nodes : Vec<Option<Box<RvmHtabNode>>>
}

impl RvmHtabCtx {
    pub fn new() -> Self {
        RvmHtabCtx {
            num_nodes: 0,
            size : HTAB_SIZE,
            nodes: vec![None; HTAB_SIZE]
        }
    }

    fn htab_hash(key: &str, size: usize) -> usize {
        let mut hash: usize = 0;
        for c in key.chars() {
            hash = hash.wrapping_add((hash << c as usize));
            hash = hash.wrapping_sub(c as usize);
            
        }
        hash % size
    }

    fn htab_rehash(&mut self, size : usize) {
        let old_size = self.size;
        let mut old_nodes = std::mem::take(&mut self.nodes);

        self.size = size;
        self.nodes = vec![None; size];

        /* Traverse the original hash table, rehashing
        * every entry into the new table        */

        for i in 0..old_size {
            if let Some(mut node) = old_nodes[i].take() {
                loop {
                    self.rvm_htab_add(&node.key,node.value, &node.value_str);
                    let next_node = node.next.take();
                    if next_node.is_none() {
                        break;
                    } else {
                        node = next_node.unwrap();
                    }
                }
            }
        }
    }

    pub fn rvm_htab_add(&mut self, key: &str, value : i32, value_str: &str) {
        self.num_nodes += 1;
        let current_load : f64 = self.num_nodes as f64 / self.size as f64;
        if current_load > HTAB_LOAD_FACTOR {
            self.htab_rehash(self.num_nodes as usize * 2);
        }

        let hash = RvmHtabCtx::htab_hash(key, self.size);
        let node : Box<RvmHtabNode> = Box::new(RvmHtabNode::new(key.to_string(), value, value_str.to_string()));
        
        if self.nodes[hash].is_none() {
            self.nodes[hash] = Some(node);
        } else {
            let mut current_node = self.nodes[hash].as_mut().unwrap();
            while current_node.next.is_some() {
                current_node = current_node.next.as_mut().unwrap();
            }
            current_node.next = Some(node);
        }
    }

    pub fn rvm_htab_find(&self, key: &str) -> Option<i32> {
        let hash = RvmHtabCtx::htab_hash(key, self.size);
        let mut current_node = self.nodes[hash].as_ref();
        while let Some(node) = current_node {
            if node.key == key {
                return Some(node.value);
            }
            current_node = node.next.as_ref();
        }
        None
    }

    pub fn rvm_htab_find_ref(&self, key: &str) -> Option<String> {
        let hash = RvmHtabCtx::htab_hash(key, self.size);
        let mut current_node = self.nodes[hash].as_ref();
        while let Some(node) = current_node {
            if node.key == key {
                return Some(node.value_str.clone());
            }
            current_node = node.next.as_ref();
        }
        None
    }
}