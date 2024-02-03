use super::*;

use image::Rgba;

pub struct LumaLiteralNode{
    luma : Luma<u8>
}

impl LumaLiteralNode{
    pub fn new(luma:Luma<u8>)->Self{
        LumaLiteralNode { luma }
    }

}

impl NodeStatic for LumaLiteralNode{

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::LumaType(Luma([255])), hasConnection:true, name:"".to_string(),subtype:None}]
    }
    
    fn get_node_name_static()->String {
        "Luma literal".to_string()
    }
}


impl Node for LumaLiteralNode{


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::LumaType(self.luma.clone()));
    }
}