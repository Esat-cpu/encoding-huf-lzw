use std::collections::HashMap;


enum NodeKind {
    Leaf(char),
    Internal,
}

struct Node {
    val: NodeKind,
    freq: u32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}


impl Node {
    fn new(val: char, freq: u32) -> Self {
        Self { val: NodeKind::Leaf(val), freq, left: None, right: None }
    }

    fn merge(left: Node, right: Node) -> Node {
        Node {
            val: NodeKind::Internal,
            freq: left.freq + right.freq,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
        }
    }
}


#[derive(Default)]
pub struct Huffman {
    pub freq_table: HashMap<char, u32>,
    pub code_table: HashMap<char, String>,
    tree_root: Option<Box<Node>>,
}


impl Huffman {
    pub fn encode(s: &str) -> Huffman {
        let mut h = Huffman::default();
        if s.is_empty() { return h; }

        // Frequency table
        for ch in s.chars() {
            *h.freq_table.entry(ch).or_insert(0) += 1;
        }

        // The tree
        // Firstly collect nodes
        let mut node_list: Vec<Node> = vec![];

        for (&k, &v) in h.freq_table.iter() {
            node_list.push(
                Node::new(k, v)
            )
        }

        // Sort by frequency, smallest to largest
        node_list.sort_by_key(|n| n.freq);

        // Create internal nodes
        while node_list.len() > 1 {
            // Get first two elements
            let first: Node = node_list.remove(0);
            let second: Node = node_list.remove(0);

            // merge it
            // than insert the merged one to the right position in the list
            let internal = Node::merge(first, second);
            let position = node_list.partition_point(|n| n.freq < internal.freq);
            node_list.insert(position, internal);
        }

        h.tree_root = Some(Box::new(node_list.remove(0)));

        h.code_elements();

        h
    }


    fn code_elements(&self) {
        // TODO: do
    }
}

