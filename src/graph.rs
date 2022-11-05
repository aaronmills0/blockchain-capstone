use std::io::Write;

type Nd = usize;
type Ed<'a> = &'a (usize, usize);
struct Graph {
    nodes: Vec<usize>,
    edges: Vec<(usize, usize)>,
}

pub fn render_to<W: Write>(output: &mut W, length: usize) {
    let mut nodes: Vec<usize> = Vec::with_capacity(length);
    for i in 0..length {
        nodes.push(i);
    }
    let mut edges: Vec<(usize, usize)> = Vec::with_capacity(length - 1);
    for i in 1..length {
        edges.push((i - 1, i));
    }
    let graph = Graph { nodes, edges };

    dot::render(&graph, output).unwrap()
}

impl<'a> dot::Labeller<'a, Nd, Ed<'a>> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("blockchain").unwrap()
    }

    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n)).unwrap()
    }
    fn node_label<'b>(&'b self, n: &Nd) -> dot::LabelText<'b> {
        dot::LabelText::LabelStr(self.nodes[*n].to_string().into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed<'a>> for Graph {
    fn nodes(&self) -> dot::Nodes<'a, Nd> {
        (0..self.nodes.len()).collect()
    }
    fn edges(&'a self) -> dot::Edges<'a, Ed<'a>> {
        self.edges.iter().collect()
    }
    fn source(&self, e: &Ed) -> Nd {
        e.0
    }
    fn target(&self, e: &Ed) -> Nd {
        e.1
    }
}
