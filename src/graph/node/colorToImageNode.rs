use super::*;
use image::{DynamicImage, ImageBuffer};

pub struct ColorToImageNode{
    color: Rgba<u8>,
    width: u32,
    height:u32,
    buffered: bool,
    buffer : RgbaImage
}

impl ColorToImageNode{
    pub fn new()->Self{
        ColorToImageNode { color: Rgba([100,100,100,255]),width:500,height:500, buffered: false, buffer: RgbaImage::default() }
    }

}

impl NodeStatic for ColorToImageNode{

    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{IOType:NodeIOType::ColorType(Rgba([100,100,100,255])), canAlterDefault:true,hasConnection:true, name:"color".to_string(), presetValues:None},
            NodeInputOptions{IOType:NodeIOType::FloatType(500.0), canAlterDefault:true,hasConnection:true, name:"width".to_string(), presetValues:None},
            NodeInputOptions{IOType:NodeIOType::FloatType(500.0), canAlterDefault:true,hasConnection:true, name:"height".to_string(), presetValues:None}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true, name:"bitmap".to_string()}]
    }

    fn get_node_name_static()->String{
        "Color to image".to_string()
    }
}

impl Node for ColorToImageNode{


    fn clear_buffers(&mut self) {
        self.buffered = false;
        self.buffer = RgbaImage::default();
        self.color = Rgba([100,100,100,255]);
        self.width = 500;
        self.height = 500;
    }


    
    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0 => if let NodeIOType::ColorType(color) = value{
                self.color = color;
            }
            1 => if let NodeIOType::FloatType(float) = value{
                self.width = float as u32;
            }
            2 => if let NodeIOType::FloatType(float) = value{
                self.height = float as u32;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if(!self.buffered){
            self.buffer = ImageBuffer::from_fn(self.width, self.height, |x,y| {self.color});
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}