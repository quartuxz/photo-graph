pub mod imageInputNode;
pub mod finalNode;
pub mod bitmapLiteralNode;
pub mod stringLiteralNode;
pub mod intLiteralNode;
pub mod floatLiteralNode;
pub mod colorLiteralNode;
pub mod rotationNode;
pub mod mathNode;

use std::{vec, result, fmt, collections::HashMap};
use thiserror::Error;
use image::{RgbaImage, Rgba};


pub struct NodeInputOptions{
    pub IOType : NodeIOType,
    pub canAlterDefault: bool,
    pub hasConnection:bool,
    pub name : String,
    pub presetValues : Option<HashMap<String,i64>>
}

pub struct NodeOutputOptions{
    pub IOType : NodeIOType,
    pub hasConnection:bool,
    pub name : String
}

#[derive(Clone)]
pub enum NodeIOType{
    IntType(i64),
    FloatType(f64),
    BitmapType(RgbaImage),
    ColorType(Rgba<u8>),
    StringType(String)
}

impl fmt::Display for NodeIOType{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        
        match *self{
            NodeIOType::IntType(_) => write!(f, "int"),
            NodeIOType::BitmapType(_) => write!(f, "bitmap"),
            NodeIOType::ColorType(_) => write!(f, "color"),
            NodeIOType::FloatType(_)=> write!(f, "float"),
            NodeIOType::StringType(_)=> write!(f, "string")
        }
    }

}

impl fmt::Debug for NodeIOType{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { 
        
        match *self{
            NodeIOType::IntType(_) => write!(f, "int"),
            NodeIOType::BitmapType(_) => write!(f, "bitmap"),
            NodeIOType::ColorType(_) => write!(f, "color"),
            NodeIOType::FloatType(_)=> write!(f, "float"),
            NodeIOType::StringType(_)=> write!(f, "string")
        }
    }
}


#[derive(Error, Debug)]
pub enum NodeError{
    #[error("The node '{0}' does not implement input for '{1}' at position '{2}'.")]
    InvalidInput(String,NodeIOType, u16),
    #[error("The node '{0}' does not implement input at position '{1}'.")]
    InvalidInputIndex(String, u16),
    #[error("The node '{0}' does not implement output at position '{1}'.")]
    InvalidOutputIndex(String, u16),
    #[error("The node '{0} does not implement ouput.'")]
    NoOutput(String),
    #[error("The node '{0} does not implement input.'")]
    NoInput(String)
}

pub type NodeResult<T> = result::Result<T, NodeError>;



//all fields in implementors that are set with the set_x methods are expected to be repopulated after every processing run.
pub trait Node: Send + Sync{
    
    
    fn get_outputs(&self)->Vec<NodeOutputOptions>{
        vec![]
    }

    //return all the node's inputs, the NodeIOType enum must contain the default input for the given index
    fn get_inputs(&self)->Vec<NodeInputOptions>{
        vec![]
    }

    fn generate_output_errors(&self, index:&u16)->NodeResult<()>{
        if(self.get_outputs().len() < (*index as usize)){
            return NodeResult::Err(NodeError::InvalidOutputIndex(self.get_node_name(), *index));
        }
        return NodeResult::Ok(())
    }

    fn generate_input_errors(&self, index:&u16, value:&NodeIOType)->NodeResult<()>{
        if(self.get_inputs().len() < (*index as usize)){
            return NodeResult::Err(NodeError::InvalidInput(self.get_node_name(), value.clone(), *index));
        }
        if std::mem::discriminant(value) != std::mem::discriminant(&self.get_inputs()[*index as usize].IOType){
            return NodeResult::Err(NodeError::InvalidInput(self.get_node_name(), value.clone(), *index));
        }
        return NodeResult::Ok(())
    }

    fn get_node_name(&self)->String;

    fn clear_buffers(&mut self){

    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType>{
        NodeResult::Err(NodeError::NoOutput(self.get_node_name()))
    }

    fn set(&mut self, index: u16, value:NodeIOType) -> NodeResult<()>{
        NodeResult::Err(NodeError::NoInput(self.get_node_name()))
    }


}