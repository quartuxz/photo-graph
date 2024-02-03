use super::*;

use image::Rgba;

pub struct ColorLiteralNode{
    color : Rgba<u8>
}

impl ColorLiteralNode{
    pub fn new(color:Rgba<u8>)->Self{
        ColorLiteralNode { color }
    }

}

impl NodeStatic for ColorLiteralNode{

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::ColorType(Rgba([255,255,255,255])), hasConnection:true, name:"".to_string(),subtype:None}]
    }
    
    fn get_node_name_static()->String {
        "Color literal".to_string()
    }
}


impl Node for ColorLiteralNode{


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::ColorType(self.color.clone()));
    }
}