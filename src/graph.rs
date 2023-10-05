mod node;

use thiserror::Error;
use std::{collections::HashMap, hash::Hash};
use self::node::{Node, NodeIOType, NodeInputOptions};
use image::RgbaImage;

#[derive(Clone, PartialEq, Debug)]
pub struct Edge{
    inputIndex : u16,
    outputIndex: u16,
    inputNode: usize,
    outputNode: usize
}

//a literal is a node without inputs
//every node's input can have exactly one output that maps to it through an edge
//every node's output can have many edges that map it to many inputs
//every node has a default input node that is a literal for every input
//every literal has one output (edge in defaultInputEdges) that maps to a single input in a non-literal node
//every new Graph starts with a final node and a bitmap literal node.
pub struct Graph{
    nodes: HashMap<usize,Box<dyn node::Node>>,
    edges : Vec<(usize, Edge)>,
    defaultInputEdges: HashMap<usize, Vec<Edge>>,
    commandHistory : String

}

#[derive(Error, Debug, PartialEq)]
pub enum GraphError{
    #[error("The graph has a cycle!")]
    Cycle,
    #[error("The graph does not contain the edge!")]
    EdgeNotFound,
    #[error("The graph does not contain the node!")]
    NodeNotFound,
    #[error("The nodes have mismatched types and/or indices and can not be connected!")]
    mismatchedNodes
}

pub type GraphResult<T> = Result<T, GraphError>;

impl Graph{
    //processes the final bitmap output for a graph.
    pub fn process(&mut self)->RgbaImage{


        let mut includes = true;
        let mut layer = 0;
        while includes {
            includes = false;
            for edge in & mut self.edges{
                if(edge.0 == layer){
                    let val = self.nodes.get_mut(&edge.1.outputNode).unwrap().get(edge.1.outputIndex).unwrap();
                    self.nodes.get_mut(&edge.1.inputNode).unwrap().set(edge.1.inputIndex, val).unwrap();
                    includes = true;
                }

            }
            layer+=1;
        }

        for (key, value) in &mut self.nodes{
            value.clear_buffers();
        }
        
        if let node::NodeIOType::BitmapType(bitmap) =self.nodes.get_mut(&0).unwrap().get(0).unwrap(){
            bitmap
        }else{
            panic!();
        }
    }


    fn recalculate_layers(&mut self)->GraphResult<()>{
        let mut distances : Vec<usize> = vec![];
        for edge in &self.edges {
            let mut distance_from_literal = 0;
            let mut edges_to_check = vec![&edge.1];
            let mut visited_edges: Vec<&Edge> = vec![&edge.1];
            while edges_to_check.len() != 0{
                let mut new_edges_to_check: Vec<&Edge> = vec![];
                for edge2 in &self.edges{
                    //check if the inputting node is in an edge as the receipient(i.e is not a literal node)
                    for edge3 in &edges_to_check {
                        if edge3.outputNode == edge2.1.inputNode {
                            //detecting loops in the graph
                            for edge4 in &visited_edges{
                                if **edge4 == edge2.1{
                                    return GraphResult::Err(GraphError::Cycle);
                                }
                            }
                            
                            new_edges_to_check.push(&edge2.1);
                            visited_edges.push(&edge2.1);
                        }
                    }
                }
                if new_edges_to_check.len() != 0{
                    distance_from_literal += 1;
                }
                edges_to_check = new_edges_to_check;
            }
            distances.push(distance_from_literal);

        }

        for i in 0..self.edges.len() {
            self.edges[i].0 = distances[i]; 
        }

        GraphResult::Ok(())
    }


    fn add_edge(&mut self, edge:Edge)->GraphResult<()>{
       
        //checks if the nodes have outputs/inputs at given indices
        if(self.nodes[&(edge.inputNode as usize)].get_inputs().len() < (edge.inputIndex as usize) || self.nodes[&(edge.outputNode as usize)].get_outputs().len() < (edge.outputIndex as usize)){
            return GraphResult::Err(GraphError::mismatchedNodes);
        }
        //checks if the nodes can be connected with equivalent types
        if(std::mem::discriminant(&self.nodes[&(edge.inputNode as usize)].get_inputs()[edge.inputIndex as usize].IOType) != std::mem::discriminant(&self.nodes[&(edge.outputNode as usize)].get_outputs()[edge.outputIndex as usize].IOType)){
            return GraphResult::Err(GraphError::mismatchedNodes);
        }

        //removes old edge at input index and node
        for i in 0..self.edges.len(){
            if(self.edges[i].1.inputNode == edge.inputNode && self.edges[i].1.inputIndex == edge.inputIndex){
                self.edges.remove(i);
                break;
            }
        }
        self.edges.push((0, edge));
        return self.recalculate_layers();
    }

    //an edge to remove and a bool indicating wether an expensive layer recalculation is need to update the order in which the nodes are used
    fn remove_edge_and_replace_with_default(&mut self, edge:&Edge, recalculate :bool)->GraphResult<()>{
        for thisEdge in &mut self.edges {
                if(thisEdge.1 == *edge){
                    //replace the removed edge with one that connects the now empty input to the output of the default literal node
                    for i in &self.defaultInputEdges[&edge.inputNode] {
                        if i.inputIndex == edge.inputIndex {
                            thisEdge.1 = i.clone();
                        }
                    }
                    if(recalculate){
                        return self.recalculate_layers();
                    }
                    return GraphResult::Ok(());
                }
        }
        return GraphResult::Err(GraphError::EdgeNotFound);
    }

    fn remove_node(&mut self, index: usize)->GraphResult<()>{

        if !self.nodes.contains_key(&index){
            return GraphResult::Err(GraphError::NodeNotFound);
        }

        self.nodes.remove(&index);
        let mut toRemove:Vec<usize> = vec![];

        for i in 0..self.edges.len(){
            let currentEdge = self.edges[i].1.clone();
            if index == currentEdge.outputNode{
                self.remove_edge_and_replace_with_default(&currentEdge, false)?;
            }
            //guaranteed to be an ascending index list
            if index == currentEdge.inputNode{
                toRemove.push(i);
            }
        }

        let mut removed = 0;

        for removing in toRemove{
            self.edges.remove(removing-removed);
            removed +=1;
        }

        return self.recalculate_layers();
    }

    //add a node and it's literal nodes.
    fn add_node(&mut self, node: Box<dyn node::Node>){
        let nodeKey =self.nodes.len();
        let inputs = node.get_inputs(); 
        self.nodes.insert(nodeKey, node);
        self.defaultInputEdges.insert(nodeKey, vec![]);
        let mut index = 0;
        for input in inputs{
            let defNodeKey = self.nodes.len();
            let defNode:Box<dyn Node> = match input{
                NodeInputOptions {IOType:NodeIOType::BitmapType(bitmap),..} => Box::new(node::bitmapLiteralNode::BitmapLiteralNode::new(bitmap)),
                NodeInputOptions {IOType:NodeIOType::ColorType(color),..} => Box::new(node::colorLiteralNode::ColorLiteralNode::new(color)),
                NodeInputOptions {IOType:NodeIOType::FloatType(floatLiteral),..} => Box::new(node::floatLiteralNode::FloatLiteralNode::new(floatLiteral)),
                NodeInputOptions {IOType:NodeIOType::IntType(intLiteral),..} => Box::new(node::intLiteralNode::IntLiteralNode::new(intLiteral)),
                NodeInputOptions {IOType:NodeIOType::StringType(stringLiteral),..} => Box::new(node::stringLiteralNode::StringLiteralNode::new(stringLiteral))
            };
            self.nodes.insert(defNodeKey, defNode);
            let inputEdge = Edge { inputIndex: index, outputIndex: 0, inputNode: nodeKey, outputNode: defNodeKey };
            self.edges.push((0,inputEdge.clone()));
            self.defaultInputEdges.get_mut(&nodeKey).unwrap().push(inputEdge);
            index+=1;
        }
    }

    pub fn new()->Self{
        let mut graph=Graph { nodes : HashMap::new(), edges : vec![], defaultInputEdges : HashMap::new(), commandHistory: "START;".to_string()};
        graph.add_node(Box::new(node::finalNode::FinalNode::new()));
        graph
    }

    pub fn execute_commands(commands:String){

    }
}

#[cfg(test)]
mod tests{
    use super::{node::{imageInputNode::ImageInputNode, Node}, Edge, GraphError};

    #[test]
    fn add_node_test(){
        let mut graph = super::Graph::new();
        graph.add_node(Box::new(ImageInputNode::new()));
        assert_eq!(graph.edges[1].1, super::Edge{inputIndex:0,outputIndex:0,inputNode:2,outputNode:3});
    }
    #[test]
    fn simple_add_edge_test(){
        let mut graph = super::Graph::new();
        graph.add_node(Box::new(ImageInputNode::new()));
        graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:0,outputNode:2}).unwrap();
        assert_eq!(graph.edges[1].1, super::Edge{inputIndex:0,outputIndex:0,inputNode:0,outputNode:2});
        assert_eq!(graph.edges[1].0, 1);
    }

    #[test]
    fn remove_edge_test(){
        let mut graph = super::Graph::new();
        graph.add_node(Box::new(ImageInputNode::new()));
        graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:0,outputNode:2}).unwrap();
        graph.remove_edge_and_replace_with_default(&super::Edge{inputIndex:0, outputIndex:0, inputNode:0,outputNode:2}, true).unwrap();
        assert_eq!(graph.edges[1].1, super::Edge{inputIndex:0,outputIndex:0,inputNode:0,outputNode:1});
        assert_eq!(graph.edges[1].0, 0);
    }

    #[test]
    fn simple_loop_check_test(){
        let mut graph = super::Graph::new();
        let res = graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:0,outputNode:0});
        assert_eq!(res, Err(GraphError::Cycle));
    }

    #[test]
    fn loop_check_test(){
        let mut graph = super::Graph::new();
        graph.add_node(Box::new(super::node::rotationNode::RotationNode::new()));
        graph.add_node(Box::new(super::node::rotationNode::RotationNode::new()));
        graph.add_node(Box::new(super::node::rotationNode::RotationNode::new()));
        graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:2,outputNode:5}).unwrap();
        graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:5,outputNode:8}).unwrap();

        let res = graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:8,outputNode:2});
        assert_eq!(res, Err(GraphError::Cycle));
    }

    #[test]
    fn add_remove_edge_test(){
        let mut graph = super::Graph::new();
        //2
        graph.add_node(Box::new(super::node::rotationNode::RotationNode::new()));
        //5
        graph.add_node(Box::new(super::node::rotationNode::RotationNode::new()));
        //8
        graph.add_node(Box::new(super::node::rotationNode::RotationNode::new()));

        graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:2,outputNode:5}).unwrap();
        graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:5,outputNode:8}).unwrap();


        //11
        graph.add_node(Box::new(ImageInputNode::new()));
        graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:8,outputNode:11}).unwrap();


        //13
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:8,outputNode:13}).unwrap();

        //17
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:5,outputNode:17}).unwrap();

        //21
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:17,outputNode:21}).unwrap();

        //25
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:21,outputNode:25}).unwrap();
        
        graph.add_edge(super::Edge{inputIndex:0,outputIndex:0,inputNode:0,outputNode:2}).unwrap();
        assert_eq!(graph.edges.last().unwrap().1, super::Edge{inputIndex:0,outputIndex:0,inputNode:0,outputNode:2});
        assert_eq!(graph.edges.last().unwrap().0, 5);
        assert_eq!(graph.edges[graph.edges.iter().position(|elem| elem.1 == super::Edge{inputIndex:0, outputIndex:0, inputNode:5,outputNode:8}).unwrap()].0, 2);
        assert_eq!(graph.edges[graph.edges.iter().position(|elem| elem.1 == super::Edge{inputIndex:1, outputIndex:0, inputNode:5,outputNode:17}).unwrap()].0, 3);

        graph.remove_edge_and_replace_with_default(&super::Edge{inputIndex:1, outputIndex:0, inputNode:5,outputNode:17}, true).unwrap();
        assert_eq!(graph.edges.last().unwrap().0, 4);
        
    }

}