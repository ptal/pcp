// Copyright 2016 Pierre Talbot (IRCAM)

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod concept;
pub mod copy_memory;
pub mod ops;
pub mod trail;
pub mod trail_memory;

pub use variable::memory::copy_memory::*;
pub use variable::memory::trail::*;
pub use variable::memory::trail_memory::*;

pub type SingleTrailMemory<Dom> = TrailMemory<SingleValueTrail<Dom>, Dom>;
pub type TimestampTrailMemory<Dom> = TrailMemory<TimestampTrail<Dom>, Dom>;

#[cfg(test)]
mod test {
    use super::*;
    use gcollections::ops::*;
    use gcollections::*;
    use interval::interval::*;
    use kernel::*;
    use std::ops::{Deref, DerefMut, Range};
    use variable::concept::*;

    type Domain = Interval<i32>;

    struct Test {
        pub tree_depth: usize,
        pub memory_config: String,
        pub queue_config: String,
        tree: Tree,
    }

    impl Test {
        pub fn new(depth: usize, shape: &TreeShape) -> Self {
            Test {
                tree_depth: depth,
                memory_config: String::new(),
                queue_config: String::new(),
                tree: Tree::new(depth, shape),
            }
        }

        pub fn assert_node_equality(&self, real: Domain, expected: Domain, from: Domain) {
            if real != expected {
                println!("\nRestoration from `{}` to `{}` failed.", from, expected);
                println!(
                    "We obtained the node `{}` instead of the expected `{}`.",
                    real, expected
                );
                println!("Configuration:");
                println!("  Depth of the tree: {}", self.tree_depth);
                println!("  Memory: {}", self.memory_config);
                println!("  Queue (exploration): {}\n", self.queue_config);
                assert!(false);
            }
        }
    }

    // Used for describing a tree more easily. It is later transformed to `Tree` that used a index-based representation.
    #[derive(Clone)]
    enum TreeShape {
        Node(Range<i32>, Vec<TreeShape>),
        Leaf(Range<i32>),
    }

    fn tree_depth_4() -> TreeShape {
        use self::TreeShape::*;
        //                         [1..10]
        //            [1..5]                     [6..10]
        //     [1..2]       [3..5]        [6..7]        [8..10]
        // [1..1] [2..2] [3..3] [4..5] [6..6] [7..7] [8..8] [9..10]
        //                   [4..4] [5..5]               [9..9] [10..10]
        Node(
            1..10,
            vec![
                Node(
                    1..5,
                    vec![
                        Node(1..2, vec![Leaf(1..1), Leaf(2..2)]),
                        Node(
                            3..5,
                            vec![Leaf(3..3), Node(4..5, vec![Leaf(4..4), Leaf(5..5)])],
                        ),
                    ],
                ),
                Node(
                    6..10,
                    vec![
                        Node(6..7, vec![Leaf(6..6), Leaf(7..7)]),
                        Node(
                            8..10,
                            vec![Leaf(8..8), Node(9..10, vec![Leaf(9..9), Leaf(10..10)])],
                        ),
                    ],
                ),
            ],
        )
    }

    #[test]
    fn restoration_strategies() {
        configure_depth();
    }

    fn configure_depth() {
        let tree_shape = tree_depth_4();
        for depth in 2..3 {
            //0..5 {
            let mut test = Test::new(depth, &tree_shape);
            configure_memory(&mut test);
        }
    }

    fn configure_memory(test: &mut Test) {
        type MCopy = CopyMemory<Domain>;
        type MSingleTrail = SingleTrailMemory<Domain>;
        type MTimestampTrail = TimestampTrailMemory<Domain>;

        test.memory_config = String::from("CopyMemory");
        configure_queue::<MCopy>(test);
        test.memory_config = String::from("TrailMemory with SingleValueTrail");
        configure_dfs_queue::<MSingleTrail>(test);
        test.memory_config = String::from("TrailMemory with TimestampTrail");
        configure_dfs_queue::<MTimestampTrail>(test);
    }

    fn configure_queue<Mem>(test: &mut Test)
    where
        Mem: MemoryConcept,
        Mem: Collection<Item = Domain>,
    {
        configure_dfs_queue::<Mem>(test);
        configure_bfs_queue::<Mem>(test);
    }

    fn configure_dfs_queue<Mem>(test: &mut Test)
    where
        Mem: MemoryConcept,
        Mem: Collection<Item = Domain>,
    {
        type Stack<Label> = VectorStack<QueueItem<Label>>;
        test.queue_config = String::from("VectorStack (Depth-first search)");
        test_restoration::<Mem, Stack<_>>(test);
    }

    fn configure_bfs_queue<Mem>(test: &mut Test)
    where
        Mem: MemoryConcept,
        Mem: Collection<Item = Domain>,
    {
        type Queue<Label> = DequeFrontBackQueue<QueueItem<Label>>;
        test.queue_config = String::from("DequeFrontBackQueue (Breadth-first search)");
        test_restoration::<Mem, Queue<_>>(test);
    }

    // Given `(parent, child, label)`, `parent` is the node index associated to the label to restore (for comparing real and expected values) and `child` is the index of the node to apply in the memory after restoration.
    type QueueItem<Label> = (usize, usize, Label);

    // We simulate the updates in the memory `M` according to the exploration `Q` of a static tree (in `test.tree`).
    // Since the tree is already fully built, we can test the values when restoring a node.
    fn test_restoration<M, Q>(test: &Test)
    where
        M: MemoryConcept,
        M: Collection<Item = Domain>,
        Q: Multiset,
        Q: Collection<Item = QueueItem<<M::FrozenState as Snapshot>::Label>>,
    {
        let tree = &test.tree;
        let mut queue = Q::empty();
        let mut mem = M::empty();
        mem.push(tree.root_value());
        let mut current = tree.root;
        let mut frozen = mem.freeze();
        for child in tree.children(tree.root) {
            println!("test_restoration: label");
            queue.insert((tree.root, child, frozen.label()));
        }
        while let Some((parent, child, label)) = queue.extract() {
            mem = frozen.restore(label);
            println!("test_restoration: restore to {}", mem[0]);
            test.assert_node_equality(mem[0], tree[parent].value, tree[current].value);
            println!("replace: {} with {}", mem[0], tree[child].value);
            mem.replace(0, tree[child].value);
            current = child;
            frozen = mem.freeze();
            for grandchild in tree.children(child) {
                println!("test_restoration: label");
                queue.insert((child, grandchild, frozen.label()));
            }
        }
    }

    #[derive(Clone)]
    struct Node {
        value: Domain,
        children: Vec<usize>,
    }

    impl Node {
        pub fn new(value: Domain, children: Vec<usize>) -> Self {
            Node { value, children }
        }
    }

    #[derive(Clone)]
    struct Tree {
        pub root: usize,
        nodes: Vec<Node>,
    }

    impl Tree {
        pub fn new(depth: usize, shape: &TreeShape) -> Self {
            let mut tree = Tree {
                root: 0,
                nodes: vec![],
            };
            tree.root = tree.from_shape(depth, shape);
            tree
        }

        pub fn root_value(&self) -> Domain {
            self.nodes[self.root].value
        }

        pub fn children(&self, node: usize) -> Vec<usize> {
            self[node].children.clone()
        }

        fn from_shape(&mut self, depth: usize, shape: &TreeShape) -> usize {
            use self::TreeShape::*;
            match shape {
                Node(value, children) if depth > 0 => {
                    let children = children
                        .iter()
                        .map(|child| self.from_shape(depth - 1, child))
                        .collect();
                    self.alloc(value.clone(), children)
                }
                &Node(ref value, _) | &Leaf(ref value) => self.leaf(value.clone()),
            }
        }

        fn alloc(&mut self, value: Range<i32>, children: Vec<usize>) -> usize {
            let node = Node::new((value.start, value.end).to_interval(), children);
            self.push(node);
            self.len() - 1
        }

        fn leaf(&mut self, value: Range<i32>) -> usize {
            self.alloc(value, vec![])
        }
    }

    impl Deref for Tree {
        type Target = Vec<Node>;
        fn deref(&self) -> &Self::Target {
            &self.nodes
        }
    }

    impl DerefMut for Tree {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.nodes
        }
    }
}
