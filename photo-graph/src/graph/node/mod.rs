pub mod imageInputNode;
pub mod finalNode;
pub mod bitmapLiteralNode;
pub mod stringLiteralNode;
pub mod intLiteralNode;
pub mod floatLiteralNode;
pub mod colorLiteralNode;
pub mod rotationNode;
pub mod mathNode;
pub mod colorToImageNode;
pub mod composeNode;
pub mod blendNode;
pub mod moveNode;
pub mod resizeNode;
pub mod scaleNode;

use std::{vec, result, fmt};
use serde::{Serialize, Serializer, ser::SerializeStruct};
use thiserror::Error;
use image::{RgbaImage, Rgba};


#[derive(Serialize)]
pub enum NodeIOSubtypes{
    ColorCurves,
    FilePath
}

pub struct NodeInputOptions{
    pub IOType : NodeIOType,
    pub canAlterDefault: bool,
    pub hasConnection:bool,
    pub name : String,
    pub presetValues : Option<Vec<String>>,
    pub subtype: Option<NodeIOSubtypes>
}

pub struct NodeOutputOptions{
    pub IOType : NodeIOType,
    pub hasConnection:bool,
    pub name : String
}


impl Serialize for NodeInputOptions{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NodeInputOptions", 6)?;
        let IOType = match &self.IOType{
            NodeIOType::IntType(val) =>{ state.serialize_field("defaultValue", &val)?; "int"},
            NodeIOType::FloatType(val) => { state.serialize_field("defaultValue", &val)?; "float"},
            NodeIOType::BitmapType(_) => { state.serialize_field("defaultValue", &Option::<()>::None)?; "bitmap"},
            NodeIOType::ColorType(val) => { state.serialize_field("defaultValue", &val.0)?; "color"},
            NodeIOType::StringType(val) => { state.serialize_field("defaultValue", &val)?; "string"},
        
        };
        state.serialize_field("IOType",IOType)?;
        state.serialize_field("canAlterDefault", &self.canAlterDefault)?;
        state.serialize_field("hasConnection", &self.hasConnection)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("presetValues", &self.presetValues)?;
        state.serialize_field("subtype", &self.subtype)?;
        state.end()
    }
}

impl Serialize for NodeOutputOptions{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NodeInputOptions", 3)?;
        state.serialize_field("IOType", match self.IOType{
            NodeIOType::IntType(_) => "int",
            NodeIOType::FloatType(_) => "float",
            NodeIOType::BitmapType(_) => "bitmap",
            NodeIOType::ColorType(_) => "color",
            NodeIOType::StringType(_) => "string",

        })?;
        state.serialize_field("hasConnection", &self.hasConnection)?;
        state.serialize_field("name", &self.name)?;
        state.end()
    }
}

#[derive(Serialize)]
pub struct NodeDescriptor{
    pub inputNodes : Vec<NodeInputOptions>,
    pub outputNodes : Vec<NodeOutputOptions>,
    pub name : String
}


#[derive(Clone, PartialEq)]
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


#[derive(Error, Debug, PartialEq)]
pub enum NodeError{
    #[error("The node '{0}' does not implement input for '{1}' at position '{2}'.")]
    InvalidInput(String,NodeIOType, u16),
    #[error("The node '{0}' does not implement input at position '{1}'.")]
    InvalidInputIndex(String, u16),
    #[error("The node '{0}' does not implement output at position '{1}'.")]
    InvalidOutputIndex(String, u16),
    #[error("The node '{0}' does not implement ouput.")]
    NoOutput(String),
    #[error("The node '{0}' does not implement input.")]
    NoInput(String),
    #[error("the node '{0}' had an IO error.")]
    IOError(String)
}

pub type NodeResult<T> = result::Result<T, NodeError>;


pub trait NodeStatic: Send{
    fn get_inputs_static()->Vec<NodeInputOptions> where
    Self:Sized
    {
        vec![]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions> where
    Self:Sized
    {
        vec![]
    }

    fn get_node_name_static()->String where Self:Sized;

    fn get_node_descriptor()->NodeDescriptor where Self:Sized{
        NodeDescriptor { inputNodes: Self::get_inputs_static(), outputNodes: Self::get_outputs_static(), name: Self::get_node_name_static() }
    }


}


pub trait NodeDefaults: Send{
    fn get_outputs(&self)->Vec<NodeOutputOptions>;

    //return all the node's inputs, the NodeIOType enum must contain the default input for the given index
    fn get_inputs(&self)->Vec<NodeInputOptions>;

    fn get_node_name(&self)->String;
}

//all fields in implementors that are set with the set method are expected to be repopulated after every processing run.
pub trait Node: Send + NodeDefaults + NodeStatic{
    
    fn generate_output_errors(&self, index:&u16)->NodeResult<()>{
        if self.get_outputs().len() < (*index as usize) {
            return NodeResult::Err(NodeError::InvalidOutputIndex(self.get_node_name(), *index));
        }
        return NodeResult::Ok(())
    }

    fn generate_input_errors(&self, index:&u16, value:&NodeIOType)->NodeResult<()>{
        if self.get_inputs().len() < (*index as usize) {
            return NodeResult::Err(NodeError::InvalidInput(self.get_node_name(), value.clone(), *index));
        }
        if std::mem::discriminant(value) != std::mem::discriminant(&self.get_inputs()[*index as usize].IOType){
            return NodeResult::Err(NodeError::InvalidInput(self.get_node_name(), value.clone(), *index));
        }
        return NodeResult::Ok(())
    }



    fn clear_buffers(&mut self){

    }

    fn clear_inputs(&mut self){

    }

    fn get(&mut self, _index: u16) -> NodeResult<NodeIOType>{
        NodeResult::Err(NodeError::NoOutput(self.get_node_name()))
    }

    fn set(&mut self, _index: u16, _value:NodeIOType) -> NodeResult<()>{
        NodeResult::Err(NodeError::NoInput(self.get_node_name()))
    }


}

impl<T: NodeStatic + Sized> NodeDefaults for T{
    fn get_outputs(&self)->Vec<NodeOutputOptions> {
        T::get_outputs_static()
    }
    fn get_inputs(&self)->Vec<NodeInputOptions> {
        T::get_inputs_static()
    }
    fn get_node_name(&self)->String {
        T::get_node_name_static()
    }
}
