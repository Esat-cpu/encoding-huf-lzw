use std::{
    cmp::Reverse,
    collections::HashMap
};


pub enum NodeKind {
    Leaf(char),
    Internal,
}

pub struct Node {
    pub val: NodeKind,
    pub freq: u32,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
    pub order: usize,
}


impl Node {
    fn new(val: char, freq: u32) -> Self {
        Self { val: NodeKind::Leaf(val), freq, left: None, right: None, order: 0 }
    }

    fn merge(left: Node, right: Node, merge_count: usize) -> Node {
        Node {
            val: NodeKind::Internal,
            freq: left.freq + right.freq,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            order: merge_count,
        }
    }
}


#[derive(Default)]
pub struct Huffman {
    pub freq_table: HashMap<char, u32>,
    pub code_table: HashMap<char, String>,
    pub tree_root: Option<Box<Node>>,
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

        // Create code table
        h.code_elements();

        h
    }


    // Create frequency table
    fn frequency_table(&mut self, s: &str) {
        for ch in s.chars() {
            *self.freq_table.entry(ch).or_insert(0) += 1;
        }
    }


    // Create tree
    fn create_tree(&mut self) {
        // Firstly collect nodes
        let mut node_list: Vec<Node> = vec![];

        for (&k, &v) in self.freq_table.iter() {
            node_list.push(Node::new(k, v))
        }

        // Sort by frequency, largest to smallest
        node_list.sort_unstable_by_key(|n| Reverse(n.freq));

        // Create internal nodes and the tree structure
        let mut merge_count = 0;
        while node_list.len() > 1 {
            // Get first two elements
            let first: Node = node_list.pop().unwrap();
            let second: Node = node_list.pop().unwrap();

            // merge it
            // than insert the merged one to the right position in the list
            merge_count += 1;
            let internal = Node::merge(first, second, merge_count);
            let position = node_list.partition_point(|n| n.freq > internal.freq);
            node_list.insert(position, internal);
        }

        self.tree_root = Some(Box::new(node_list.pop().unwrap()));
    }


    // Create code table
    fn code_elements(&mut self) {
        match &self.tree_root {
            Some(node) if node.left.is_none() && node.right.is_none() => {
                if let NodeKind::Leaf(ch) = node.val {
                    self.code_table.insert(ch, "0".to_string());
                }
            }
            _ => travel(&mut self.code_table, &self.tree_root, String::new()),
        }
    }
}



// Traverse the tree to build code table
fn travel(code_table: &mut HashMap<char, String>, node: &Option<Box<Node>>, code: String) {
    if let Some(n) = node {
        match n.val {
            NodeKind::Leaf(ch) => {
                code_table.insert(ch, code);
            },
            NodeKind::Internal => {
                travel(code_table, &n.left, format!("{code}0"));
                travel(code_table, &n.right, format!("{code}1"));
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn is_prefix_free(codes: &HashMap<char, String>) -> bool {
        let mut v: Vec<&String> = codes.values().collect();
        v.sort_unstable_by_key(|c| c.len());
        for i in 0..v.len() {
            for j in i + 1..v.len() {
                if v[j].starts_with(v[i].as_str()) {
                    return false;
                }
            }
        }
        true
    }

    fn leaf_depths(node: &Option<Box<Node>>, depth: usize) -> Vec<(char, usize)> {
        match node {
            Some(n) => match n.val {
                NodeKind::Leaf(ch) => vec![(ch, depth)],
                NodeKind::Internal => {
                    let mut v = leaf_depths(&n.left, depth + 1);
                    v.extend(leaf_depths(&n.right, depth + 1));
                    v
                }
            },
            None => vec![],
        }
    }

    fn assert_binary(node: &Option<Box<Node>>) {
        if let Some(n) = node {
            match n.val {
                NodeKind::Internal => {
                    assert!(n.left.is_some());
                    assert!(n.right.is_some());
                    assert_binary(&n.left);
                    assert_binary(&n.right);
                }
                NodeKind::Leaf(_) => {
                    assert!(n.left.is_none());
                    assert!(n.right.is_none());
                }
            }
        }
    }

    fn total_freq(node: &Option<Box<Node>>) -> u32 {
        match node {
            Some(n) => match n.val {
                NodeKind::Leaf(_) => n.freq,
                NodeKind::Internal => total_freq(&n.left) + total_freq(&n.right),
            },
            None => 0,
        }
    }

    fn assert_unique_orders(node: &Option<Box<Node>>, orders: &mut std::collections::HashSet<usize>) {
        if let Some(n) = node {
            match n.val {
                NodeKind::Internal => {
                    assert!(n.order > 0, "internal node order must be > 0");
                    assert!(orders.insert(n.order), "order {} is not unique", n.order);
                    assert_unique_orders(&n.left, orders);
                    assert_unique_orders(&n.right, orders);
                }
                NodeKind::Leaf(_) => {
                    assert_eq!(n.order, 0, "leaf node order must be 0");
                }
            }
        }
    }

    // Encoding an empty string should produce empty tables and no tree root.
    #[test]
    fn encode_empty_string() {
        let h = Huffman::encode("");
        assert!(h.freq_table.is_empty());
        assert!(h.code_table.is_empty());
        assert!(h.tree_root.is_none());
    }

    // A single character should produce code "0" with frequency 1.
    #[test]
    fn encode_single_char() {
        let h = Huffman::encode("a");
        assert_eq!(h.freq_table.len(), 1);
        assert_eq!(h.freq_table[&'a'], 1);
        assert_eq!(h.code_table.len(), 1);
        assert_eq!(h.code_table[&'a'], "0");
        assert!(h.tree_root.is_some());
    }

    // Repeated occurrences of the same character should yield a single code "0".
    #[test]
    fn encode_single_char_repeated() {
        let h = Huffman::encode("aaaa");
        assert_eq!(h.freq_table[&'a'], 4);
        assert_eq!(h.code_table.len(), 1);
        assert_eq!(h.code_table[&'a'], "0");
    }

    // Two distinct characters with equal frequency should each get a 1-bit code, prefix-free.
    #[test]
    fn encode_two_distinct_chars() {
        let h = Huffman::encode("ab");
        assert_eq!(h.freq_table.len(), 2);
        assert_eq!(h.freq_table[&'a'], 1);
        assert_eq!(h.freq_table[&'b'], 1);
        assert_eq!(h.code_table.len(), 2);
        assert!(h.code_table[&'a'] == "0" || h.code_table[&'a'] == "1");
        assert!(h.code_table[&'b'] == "0" || h.code_table[&'b'] == "1");
        assert_ne!(h.code_table[&'a'], h.code_table[&'b']);
        assert!(is_prefix_free(&h.code_table));
    }

    // Characters with unequal frequencies (a:3, b:2, c:1) produce a valid prefix-free code table.
    #[test]
    fn encode_unequal_frequencies() {
        let h = Huffman::encode("aaabbc");
        assert_eq!(h.freq_table[&'a'], 3);
        assert_eq!(h.freq_table[&'b'], 2);
        assert_eq!(h.freq_table[&'c'], 1);
        assert_eq!(h.code_table.len(), 3);
        assert!(is_prefix_free(&h.code_table));
    }

    // The frequency table correctly counts character occurrences.
    #[test]
    fn frequency_table_counts_correctly() {
        let mut h = Huffman::default();
        h.frequency_table("hello world");
        assert_eq!(h.freq_table[&'h'], 1);
        assert_eq!(h.freq_table[&'e'], 1);
        assert_eq!(h.freq_table[&'l'], 3);
        assert_eq!(h.freq_table[&'o'], 2);
        assert_eq!(h.freq_table[&' '], 1);
        assert_eq!(h.freq_table[&'w'], 1);
        assert_eq!(h.freq_table[&'r'], 1);
        assert_eq!(h.freq_table[&'d'], 1);
    }

    // A non-empty input should produce a tree root.
    #[test]
    fn tree_root_exists_for_nonempty() {
        let h = Huffman::encode("test");
        assert!(h.tree_root.is_some());
    }

    // Every internal node must have two children, every leaf must have none (full binary tree).
    #[test]
    fn tree_is_binary() {
        let h = Huffman::encode("abcdef");
        assert_binary(&h.tree_root);
    }

    // The sum of all frequencies in the tree must equal the input length.
    #[test]
    fn tree_freq_sum_matches() {
        let h = Huffman::encode("aaabbc");
        assert_eq!(total_freq(&h.tree_root), 6);
    }

    // All codes must consist only of '0' and '1'.
    #[test]
    fn code_table_only_contains_0_and_1() {
        let h = Huffman::encode("this is a test string");
        for code in h.code_table.values() {
            assert!(code.chars().all(|c| c == '0' || c == '1'));
        }
    }

    // No code should be a prefix of another code (prefix-free property).
    #[test]
    fn code_table_is_prefix_free() {
        let h = Huffman::encode("this is a test string");
        assert!(is_prefix_free(&h.code_table));
    }

    // Every character in the input must have an entry in the code table.
    #[test]
    fn all_chars_have_codes() {
        let s = "abcdefgh";
        let h = Huffman::encode(s);
        for ch in s.chars() {
            assert!(h.code_table.contains_key(&ch));
        }
    }

    // More frequent characters must receive codes no longer than less frequent ones.
    #[test]
    fn more_frequent_gets_shorter_or_equal_code() {
        let h = Huffman::encode("aaabbc");
        let depths = leaf_depths(&h.tree_root, 0);
        let mut pairs = depths.clone();
        pairs.sort_unstable_by_key(|(_, d)| *d);
        for (c1, d1) in &pairs {
            for (c2, d2) in &pairs {
                if h.freq_table[c1] > h.freq_table[c2] {
                    assert!(*d1 <= *d2, "{:?}(freq={}) depth={} > {:?}(freq={}) depth={}",
                        c1, h.freq_table[c1], d1, c2, h.freq_table[c2], d2);
                }
            }
        }
    }

    // Unicode characters (é, ö) must be handled correctly.
    #[test]
    fn encode_unicode_chars() {
        let h = Huffman::encode("héllo wörld");
        assert!(h.freq_table.contains_key(&'é'));
        assert!(h.freq_table.contains_key(&'ö'));
        assert!(h.code_table.contains_key(&'é'));
        assert!(h.code_table.contains_key(&'ö'));
        assert!(is_prefix_free(&h.code_table));
    }

    // Repeated encoding of the same input must always produce valid prefix-free codes.
    #[test]
    fn encode_repeated_encoding_is_consistent() {
        let h1 = Huffman::encode("deterministic test");
        let h2 = Huffman::encode("deterministic test");
        assert_eq!(h1.freq_table, h2.freq_table);
        assert!(is_prefix_free(&h1.code_table));
        assert!(is_prefix_free(&h2.code_table));
    }

    // All codes in the code table must be unique (no two characters share a code).
    #[test]
    fn code_table_unique_codes() {
        let h = Huffman::encode("aabbccddeeff");
        let mut seen = std::collections::HashSet::new();
        for code in h.code_table.values() {
            assert!(seen.insert(code), "duplicate code: {}", code);
        }
    }

    // A long string (pangram repeated 10 times) must encode all 27 distinct chars (26 letters + space).
    #[test]
    fn encode_long_string() {
        let s = "the quick brown fox jumps over the lazy dog".repeat(10);
        let h = Huffman::encode(&s);
        assert_eq!(h.freq_table.len(), 27);
        assert!(is_prefix_free(&h.code_table));
    }

    // Internal nodes must have order > 0 (unique), leaf nodes must have order == 0.
    // For n distinct characters, there should be n-1 internal nodes.
    #[test]
    fn tree_order_field_is_unique_for_internal_nodes() {
        let h = Huffman::encode("abcdefgh");
        let mut orders = std::collections::HashSet::new();
        assert_unique_orders(&h.tree_root, &mut orders);
        assert_eq!(orders.len(), 7);
    }
}

