use super::*;

use image::{ImageBuffer};

pub struct FinalNode{
    bitmap : RgbaImage
}

impl FinalNode{
    pub fn new()->Self{
        FinalNode { bitmap: RgbaImage::default() }
    }

}

impl NodeStatic for FinalNode{
    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{IOType:NodeIOType::BitmapType(ImageBuffer::from_fn(500, 500, |_x,_y| {Rgba([100,0,50,255])})), canAlterDefault:false,hasConnection:true, name:"bitmap".to_string(), presetValues:None,subtype:None}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::BitmapType(RgbaImage::default()), hasConnection:false,name:"".to_string()}]
    }

    fn get_node_name_static()->String {
        "Final".to_string()
    }
}

impl Node for FinalNode{

    fn clear_buffers(&mut self) {
        
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        NodeResult::Ok(NodeIOType::BitmapType(self.bitmap.clone()))
    }
    fn set(&mut self, index: u16, value:NodeIOType) -> NodeResult<()> {

        self.generate_input_errors(&index, &value)?;

        if let NodeIOType::BitmapType(bitmap) = value{
            self.bitmap = bitmap
        }

        NodeResult::Ok(())
    }
}