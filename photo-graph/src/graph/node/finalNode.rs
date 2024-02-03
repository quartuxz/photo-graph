use crate::image_utils::return_non_empty;

use super::*;

use image::{ImageBuffer};

pub struct FinalNode{
    image : Arc<DynamicImage>
}

impl FinalNode{
    pub fn new()->Self{
        FinalNode { image: Arc::new(DynamicImage::default()) }
    }

}

impl NodeStatic for FinalNode{
    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::ImageRgba8(return_non_empty(&RgbaImage::default())))), canAlterDefault:false,hasConnection:true, name:"image".to_string(), presetValues:None,subtype:None}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:false,name:"".to_string(),subtype:None}]
    }

    fn get_node_name_static()->String {
        "Final".to_string()
    }
}

impl Node for FinalNode{

    fn clear_buffers(&mut self) {
        *self=FinalNode::new();
    }
    

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if self.image.to_rgba8().is_empty(){
            self.image = Arc::new(DynamicImage::ImageRgba8(return_non_empty(&RgbaImage::default())));
        }
        NodeResult::Ok(NodeIOType::DynamicImageType(self.image.clone()))
    }
    fn set(&mut self, index: u16, value:NodeIOType) -> NodeResult<()> {

        self.generate_input_errors(&index, &value)?;

        if let NodeIOType::DynamicImageType(image) = value{
            self.image = image
        }

        NodeResult::Ok(())
    }
}