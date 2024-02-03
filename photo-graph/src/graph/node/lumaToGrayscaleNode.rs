use image::*;

use super::*;

pub struct LumaToGrayscaleNode{
    luma: Luma<u8>,
    width: u32,
    height:u32,
    buffered: bool,
    buffer : Arc<DynamicImage>
}

impl LumaToGrayscaleNode{
    pub fn new()->Self{
        LumaToGrayscaleNode { luma:Luma([255]),width:500,height:500, buffered: false, buffer: Arc::new(DynamicImage::default()) }
    }

}

impl NodeStatic for LumaToGrayscaleNode{

    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{IOType:NodeIOType::LumaType(Luma([255])), canAlterDefault:true,hasConnection:true, name:"luma".to_string(), presetValues:None,subtype:None},
            NodeInputOptions{IOType:NodeIOType::FloatType(500.0), canAlterDefault:true,hasConnection:true, name:"width".to_string(), presetValues:None,subtype:None},
            NodeInputOptions{IOType:NodeIOType::FloatType(500.0), canAlterDefault:true,hasConnection:true, name:"height".to_string(), presetValues:None,subtype:None}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:true, name:"grayscale".to_string(),subtype:Some(NodeIOSubtypes::GrayscaleImage)}]
    }

    fn get_node_name_static()->String{
        "Luma to grayscale".to_string()
    }
}

impl Node for LumaToGrayscaleNode{


    fn clear_buffers(&mut self) {
        *self = LumaToGrayscaleNode::new();
    }


    
    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0 => if let NodeIOType::LumaType(luma) = value{
                self.luma = luma;
            }
            1 => if let NodeIOType::FloatType(mut float) = value{
                if float < 1.0 {
                    float = 1.0;
                }
                self.width = float as u32;
            }
            2 => if let NodeIOType::FloatType(mut float) = value{
                if float < 1.0 {
                    float = 1.0;
                }
                self.height = float as u32;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageLuma8(ImageBuffer::from_fn(self.width, self.height, |_x,_y| {self.luma}));
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::DynamicImageType(self.buffer.clone()))
    }
}