use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
/// An index of a node in a DAG
pub struct Node(usize);

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
pub enum Value {
    /// The value of the tape at a given offset from the cursor
    Tape(i32),
    /// A constant value
    Const(i32),
    /// Multiply one DAG node with another
    Multiply(Node, Node),
    /// Add one DAG node to another
    Add(Node, Node),
}

/*
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Tape(offset) => {
                write!(f, "tape[{}]", offset)?;
            }
            Value::Const(value) => {
                write!(f, "{}", value)?;
            }
            Value::Multiply(ref l, ref r) => {
                write!(f, "({:?} * {:?})", l, r)?;
            }
            Value::Add(ref l, ref r) => {
                write!(f, "({:?} + {:?})", l, r)?;
            }
        }
        Ok(())
    }
}
*/

/// A directed acyclic graph representing operations optimized from Brainfuck.
/// Code consisting only of shifts and adds can be reduced to a graph from
/// tape offsets to tape offsets. Certain loops can also be transformed into a
/// DAG.
// TODO: try adding HashMap<Value, usize> for reverse node lookup;
// see if this helps for efficiency.
#[derive(Clone, Debug)]
pub struct DAG {
    nodes: Vec<Value>,
    terminals: HashMap<i32, Node>,
    // TODO Should this be private?
    pub zeroed: bool,
}

impl std::ops::Index<Node> for DAG {
    type Output = Value;

    fn index(&self, node: Node) -> &Value {
        &self.nodes[node.0]
    }
}

impl DAG {
    pub fn new(zeroed: bool) -> Self {
        Self {
            nodes: Vec::new(),
            terminals: HashMap::new(),
            zeroed,
        }
    }

    fn add_node(&mut self, value: Value) -> Node {
        self.nodes.push(value);
        Node(self.nodes.len() - 1)
    }

    fn default_value(&self, offset: i32) -> Value {
        if self.zeroed {
            Value::Const(0)
        } else {
            Value::Tape(offset)
        }
    }

    pub fn set(&mut self, offset: i32, value: Value) {
        let node = self.add_node(value);
        self.terminals.insert(offset, node);
    }

    pub fn get(&self, offset: i32) -> Value {
        self.terminals
            .get(&offset)
            .map(|x| self[*x])
            .unwrap_or(self.default_value(offset))
    }

    fn get_node(&mut self, offset: i32) -> Node {
        if let Some(node) = self.terminals.get(&offset) {
            *node
        } else {
            let node = self.add_node(self.default_value(offset));
            self.terminals.insert(offset, node);
            node
        }
    }

    pub fn add(&mut self, offset: i32, value: i32) {
        let old_node = self.get_node(offset);
        // Combine with existing add of constant
        if let Value::Add(lhs, rhs) = self[old_node] {
            if let Value::Const(old_value) = self[rhs] {
                let new_node = self.add_node(Value::Const(old_value + value));
                self.set(offset, Value::Add(lhs, new_node));
                return;
            }
        }
        let new_node = self.add_node(Value::Const(value));
        self.set(offset, Value::Add(old_node, new_node));
    }

    pub fn mul(&mut self, offset: i32, value: Value) {
        let old_node = self.get_node(offset);
        let new_node = self.add_node(value);
        self.set(offset, Value::Multiply(old_node, new_node));
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.terminals.clear();
    }

    pub fn shift(&mut self, shift: i32) {
        for i in self.nodes.iter_mut() {
            if let Value::Tape(offset) = *i {
                *i = Value::Tape(offset + shift);
            }
        }
    }

    pub fn topological_sort(&self) -> impl Iterator<Item = Node> {
        // Assumes nodes are never deleted, so numberic order is toplogical
        // TODO: doesn't skip unneeded nodes
        (0..self.nodes.len()).map(Node)
    }

    pub fn terminals<'a>(&'a self) -> impl Iterator<Item = (i32, Node)> + 'a {
        self.terminals.iter().map(|(k, v)| (*k, *v))
    }

    pub fn extend(&mut self, mut expr: DAG) {
        for i in expr.terminals.values_mut() {
            i.0 += self.nodes.len();
        }

        for i in &mut expr.nodes {
            match i {
                Value::Tape(offset) => {
                    if let Some(node) = self.terminals.get(offset) {
                        // XXX
                        *i = self[*node];
                    }
                }
                Value::Const(_) => {}
                Value::Add(lhs, rhs) | Value::Multiply(lhs, rhs) => {
                    lhs.0 += self.nodes.len();
                    rhs.0 += self.nodes.len();
                }
            }
        }

        self.terminals.extend(expr.terminals);
        self.nodes.extend(expr.nodes);
    }

    //fn simplify(&mut self);

    // TODO efficiency
    pub fn dependencies(&self, node: Node) -> HashSet<i32> {
        fn dependencies_iter(dag: &DAG, set: &mut HashSet<i32>, node: Node) {
            match dag[node] {
                Value::Tape(offset) => {
                    set.insert(offset);
                }
                Value::Const(_) => {}
                Value::Multiply(l, r) => {
                    dependencies_iter(dag, set, l);
                    dependencies_iter(dag, set, r);
                }
                Value::Add(l, r) => {
                    dependencies_iter(dag, set, l);
                    dependencies_iter(dag, set, r);
                }
            }
        }
        let mut set = HashSet::new();
        dependencies_iter(self, &mut set, node);
        set
    }
}
