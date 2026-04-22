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
    // Fill the values in a Huffman struct with the given string slice
    pub fn encode(s: &str) -> Huffman {
        let mut h = Huffman::default();
        if s.is_empty() { return h; }

        // Frequency table
        h.frequency_table(s);

        // The tree
        h.create_tree();

        // coders
        h.code_elements();

        h
    }


    // Create frequency table
    fn frequency_table(&mut self, s: &str) {
        for ch in s.chars() {
            *self.freq_table.entry(ch).or_insert(0) += 1;
        }
    }


    // Create tree (Min Heap)
    fn create_tree(&mut self) {
        // Firstly collect nodes
        let mut node_list: Vec<Node> = vec![];

        for (&k, &v) in self.freq_table.iter() {
            node_list.push(Node::new(k, v))
        }

        // Sort by frequency, smallest to largest
        node_list.sort_unstable_by_key(|n| n.freq);

        // Create internal nodes and the tree structure
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

        self.tree_root = Some(Box::new(node_list.remove(0)));
    }


    // Create code table
    fn code_elements(&self) {
        // TODO: do
    }
}

