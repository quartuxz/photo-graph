pub mod node;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::{collections::HashMap, fmt};
use crate::graph::node::{NodeStatic};

use self::node::{Node, NodeIOType, NodeInputOptions};
use image::RgbaImage;

#[derive(Clone, PartialEq, Debug)]
pub struct Edge{
    inputIndex : u16,
    outputIndex: u16,
    inputNode: usize,
    outputNode: usize
}


#[derive(Serialize,Deserialize, Clone)]
pub struct Command{
    pub name : String,
    pub args : Vec<String>
}

impl fmt::Display for Command{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut args = String::new();
        for arg in &self.args{
            args.push_str(&arg);
            args.push(' ');
        }
        write!(f, "command: {}; arguments: {}", self.name, args)
    }
}


#[derive(Serialize,Deserialize, Clone)]
pub struct Commands{
    pub commands : Vec<Command>,
    pub graphID : u64
}

impl Commands{
    fn new()->Commands{
        Commands{commands:vec![],graphID:0}
    }
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
    IDCount : usize,
    defaultInputEdges: HashMap<usize, Vec<Edge>>,
    pub commandHistory : Commands

}

#[derive(Error, Debug, PartialEq)]
pub enum GraphError{
    #[error("The graph has a cycle!")]
    Cycle,
    #[error("The graph does not contain the edge!")]
    EdgeNotFound,
    #[error("The graph does not contain the node!")]
    NodeNotFound,
    #[error("The nodes on the edge have mismatched types and/or indices and can not be connected!")]
    MismatchedNodes,
    #[error("The command does not exist!")]
    UnknownCommand,
    #[error("The command is ill-formed")]
    IllFormedCommand
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
                if edge.0 == layer {
                    let val = self.nodes.get_mut(&edge.1.outputNode).unwrap().get(edge.1.outputIndex).unwrap();
                    self.nodes.get_mut(&edge.1.inputNode).unwrap().set(edge.1.inputIndex, val).unwrap();
                    includes = true;
                }

            }
            layer+=1;
        }

        for (_key, value) in &mut self.nodes{
            value.clear_buffers();
        }
        
        if let node::NodeIOType::BitmapType(bitmap) =self.nodes.get_mut(&0).unwrap().get(0).unwrap(){
            bitmap
        }else{
            panic!();
        }
    }


    fn check_for_cycle(&self, edge:&Edge)->bool{
        

        if edge.inputNode == edge.outputNode {
            return true;
        }

        let mut forward_edges_to_check = vec![edge];
        while forward_edges_to_check.len() != 0{
            let mut new_forward_edges_to_check: Vec<&Edge> = vec![];

            for edge2 in &self.edges {
                for edge3 in &forward_edges_to_check{
                    if edge3.inputNode == edge2.1.outputNode {
                        new_forward_edges_to_check.push(&edge2.1);
                    }
                }
            }

            for forward_edge in &new_forward_edges_to_check{
                if **forward_edge == *edge {
                    return true;
                }
            }
            forward_edges_to_check = new_forward_edges_to_check;

        }

        return false;
    }

    fn recalculate_layers(&mut self){
        let mut distances : Vec<usize> = vec![];
        for edge in &self.edges {
            let mut distance_from_literal = 0;
            let mut edges_to_check = vec![&edge.1];
            while edges_to_check.len() != 0{

                let mut new_edges_to_check: Vec<&Edge> = vec![];
                for edge2 in &self.edges{
                    for edge3 in &edges_to_check {
                        //check if the inputting node is in an edge as the receipient(i.e is not a literal node)
                        //check if the edge connects to the same node
                        if edge3.outputNode == edge2.1.inputNode {
                            new_edges_to_check.push(&edge2.1);
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

    }


    fn add_edge(&mut self, edge:Edge)->GraphResult<()>{

        //checks if the nodes have outputs/inputs at given indices
        if self.nodes[&(edge.inputNode as usize)].get_inputs().len() < (edge.inputIndex as usize) || self.nodes[&(edge.outputNode as usize)].get_outputs().len() < (edge.outputIndex as usize) {
            return GraphResult::Err(GraphError::MismatchedNodes);
        }
        //checks if the nodes can be connected with equivalent types
        if std::mem::discriminant(&self.nodes[&(edge.inputNode as usize)].get_inputs()[edge.inputIndex as usize].IOType) != std::mem::discriminant(&self.nodes[&(edge.outputNode as usize)].get_outputs()[edge.outputIndex as usize].IOType) {
            return GraphResult::Err(GraphError::MismatchedNodes);
        }
        let mut removed:Option<Edge> = None;
        //removes old edge at input index and node
        for i in 0..self.edges.len(){
            if self.edges[i].1.inputNode == edge.inputNode && self.edges[i].1.inputIndex == edge.inputIndex {

                let (_,removed_edge) = self.edges.remove(i);
                removed = Some(removed_edge);
                break;
            }
        }
        self.edges.push((0, edge.clone()));

        //undos the adding of the edge to restore to previous state
        return if !self.check_for_cycle(&edge)  {
            self.recalculate_layers();
            GraphResult::Ok(())
        }
        else {
            self.remove_edge_and_replace_with_default(&edge, false)?;
            if let Some(removed_edge) = removed{
                self.add_edge(removed_edge)?;
            }else{
                self.recalculate_layers();
            }
            GraphResult::Err(GraphError::Cycle)
        };
    }

    //an edge to remove and a bool indicating wether an expensive layer recalculation is need to update the order in which the nodes are used
    fn remove_edge_and_replace_with_default(&mut self, edge:&Edge, recalculate :bool)->GraphResult<()>{
        for thisEdge in &mut self.edges {
                if thisEdge.1 == *edge {
                    //replace the removed edge with one that connects the now empty input to the output of the default literal node
                    for i in &self.defaultInputEdges[&edge.inputNode] {
                        if i.inputIndex == edge.inputIndex {
                            thisEdge.1 = i.clone();
                        }
                    }
                    if recalculate {
                        self.recalculate_layers();
                    }
                    return GraphResult::Ok(());
                }
        }
        return GraphResult::Err(GraphError::EdgeNotFound);
    }

    fn remove_node(&mut self, index: usize,recalculate :bool)->GraphResult<()>{

        if !self.nodes.contains_key(&index){
            return GraphResult::Err(GraphError::NodeNotFound);
        }

        let removedNodeO = self.nodes.remove(&index);
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
        if let Some(removedNode) = removedNodeO{
            for i in (index+1)..(removedNode.get_inputs().len()+index+1){
                self.remove_node(i,false);
            }
        }

        if recalculate{
            self.recalculate_layers();
        }

        Ok(())
    }

    //add a node and it's literal nodes.
    fn add_node(&mut self, node: Box<dyn node::Node>){
        let inputs = node.get_inputs(); 
        self.nodes.insert(self.IDCount, node);
        self.defaultInputEdges.insert(self.IDCount, vec![]);
        let mut index = 0;
        for input in inputs{
            let defNodeKey = self.IDCount+index+1;
            let defNode:Box<dyn Node> = match input{
                NodeInputOptions {IOType:NodeIOType::BitmapType(bitmap),..} => Box::new(node::bitmapLiteralNode::BitmapLiteralNode::new(bitmap)),
                NodeInputOptions {IOType:NodeIOType::ColorType(color),..} => Box::new(node::colorLiteralNode::ColorLiteralNode::new(color)),
                NodeInputOptions {IOType:NodeIOType::FloatType(floatLiteral),..} => Box::new(node::floatLiteralNode::FloatLiteralNode::new(floatLiteral)),
                NodeInputOptions {IOType:NodeIOType::IntType(intLiteral),..} => Box::new(node::intLiteralNode::IntLiteralNode::new(intLiteral)),
                NodeInputOptions {IOType:NodeIOType::StringType(stringLiteral),..} => Box::new(node::stringLiteralNode::StringLiteralNode::new(stringLiteral))
            };
            self.nodes.insert(defNodeKey, defNode);
            let inputEdge = Edge { inputIndex: index as u16, outputIndex: 0, inputNode: self.IDCount, outputNode: defNodeKey };
            self.edges.push((0,inputEdge.clone()));
            self.defaultInputEdges.get_mut(&self.IDCount).unwrap().push(inputEdge);
            index+=1;
        }
        self.IDCount += index+1;
    }

    pub fn new()->Self{
        let mut graph=Graph { nodes : HashMap::new(), edges : vec![], defaultInputEdges : HashMap::new(), commandHistory: Commands::new(), IDCount:0};
        graph.add_node(Box::new(node::finalNode::FinalNode::new()));
        //graph.add_node(Box::new(node::imageInputNode::ImageInputNode::new()));
        //graph.add_edge(Edge{inputIndex:0,outputIndex:0,inputNode:0,outputNode:2}).unwrap();
        graph
    }

    pub fn execute_commands(&mut self, mut commands:Commands)->GraphResult<()>{
        for cmd in &commands.commands{
            println!("{}",cmd);
            match cmd.name.as_str(){
                "removeEdge" => self.remove_edge_and_replace_with_default(&Edge {outputNode:cmd.args[0].parse().unwrap(),outputIndex:cmd.args[1].parse().unwrap(),inputNode:cmd.args[2].parse().unwrap(),inputIndex:cmd.args[3].parse().unwrap()}, true)?,
                "addEdge" => self.add_edge(Edge {outputNode:cmd.args[0].parse().unwrap(),outputIndex:cmd.args[1].parse().unwrap(),inputNode:cmd.args[2].parse().unwrap(),inputIndex:cmd.args[3].parse().unwrap()})?,
                "addNode" => 
                if cmd.args[0] == node::imageInputNode::ImageInputNode::get_node_name_static() {
                    self.add_node(Box::new(node::imageInputNode::ImageInputNode::new()));
                }else if cmd.args[0] == node::colorToImageNode::ColorToImageNode::get_node_name_static(){
                    self.add_node(Box::new(node::colorToImageNode::ColorToImageNode::new()));
                }else if cmd.args[0] == node::mathNode::MathNode::get_node_name_static(){
                    self.add_node(Box::new(node::mathNode::MathNode::new()));
                }
                "removeNode" => self.remove_node(cmd.args[0].parse().unwrap(),true)?,
                "moveNode" => (),
                "modifyDefault" => {match self.nodes.get_mut(&cmd.args[0].parse().unwrap()){
                    Some(node) => {
                        let nodeName = node.get_node_name();
                        if nodeName == node::floatLiteralNode::FloatLiteralNode::get_node_name_static(){
                            self.nodes.insert(cmd.args[0].parse().unwrap(), Box::new(node::floatLiteralNode::FloatLiteralNode::new(match cmd.args[2].parse(){
                                Ok(parsed) => parsed,
                                Err(_) => return Err(GraphError::IllFormedCommand)
                            })));
                        }
                        else if nodeName == node::intLiteralNode::IntLiteralNode::get_node_name_static(){
                            self.nodes.insert(cmd.args[0].parse().unwrap(), Box::new(node::intLiteralNode::IntLiteralNode::new(match cmd.args[2].parse(){
                                Ok(parsed) => parsed,
                                Err(_) => return Err(GraphError::IllFormedCommand)
                            })));
                        }
                        else if nodeName == node::stringLiteralNode::StringLiteralNode::get_node_name_static(){
                            self.nodes.insert(cmd.args[0].parse().unwrap(), Box::new(node::stringLiteralNode::StringLiteralNode::new(cmd.args[2].clone())));
                        }
                        else if nodeName == node::colorLiteralNode::ColorLiteralNode::get_node_name_static(){
                            let mut channels : [u8;4] = [0;4];
                            for i in 0..4 {
                                channels[i] = match cmd.args[i+2].parse(){
                                    Ok(parsed) => parsed,
                                    Err(_) => return Err(GraphError::IllFormedCommand)
                                }
                            } 
                            self.nodes.insert(cmd.args[0].parse().unwrap(), Box::new(node::colorLiteralNode::ColorLiteralNode::new(image::Rgba(channels))));
                        }
                    }
                    None => return Err(GraphError::NodeNotFound)
                }; ()}
                _ => return Err(GraphError::UnknownCommand)
            }
        }
        self.commandHistory.commands.append(&mut commands.commands);
        Ok(())
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

        let mut graph = super::Graph::new();
        //2
        graph.add_node(Box::new(super::node::colorToImageNode::ColorToImageNode::new()));
        //6
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        //10
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        //14
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        //18
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));
        //22
        graph.add_node(Box::new(super::node::mathNode::MathNode::new()));

        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:2,outputNode:6}).unwrap();
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:6,outputNode:10}).unwrap();
        graph.add_edge(super::Edge{inputIndex:2, outputIndex:0, inputNode:6,outputNode:14}).unwrap();
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:10,outputNode:22}).unwrap();
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:14,outputNode:18}).unwrap();
        graph.add_edge(super::Edge{inputIndex:1, outputIndex:0, inputNode:18,outputNode:22}).unwrap();

        let res = graph.add_edge(super::Edge{inputIndex:0, outputIndex:0, inputNode:0,outputNode:2});
        assert_eq!(res, Ok(()));
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