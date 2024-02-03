use image::*;

use super::*;

pub struct SeparateRGBANode{
    buffered: bool,
    separating: Arc<DynamicImage>,
    red : Arc<DynamicImage>,
    green : Arc<DynamicImage>,
    blue: Arc<DynamicImage>,
    alpha : Arc<DynamicImage>
}

impl SeparateRGBANode{
    pub fn new()->Self{
        SeparateRGBANode { 
             buffered: false, 
             red: Arc::new(DynamicImage::default()),
             blue: Arc::new(DynamicImage::default()),
             green: Arc::new(DynamicImage::default()),
             alpha: Arc::new(DynamicImage::default()),
             separating: Arc::new(DynamicImage::default())
             }
    }

}

impl NodeStatic for SeparateRGBANode{

    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())), canAlterDefault:true,hasConnection:true, name:"separating".to_string(), presetValues:None,subtype:Some(NodeIOSubtypes::ColorImage)},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:true, name:"red".to_string(),subtype:Some(NodeIOSubtypes::GrayscaleImage)},
        NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:true, name:"green".to_string(),subtype:Some(NodeIOSubtypes::GrayscaleImage)},
        NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:true, name:"blue".to_string(),subtype:Some(NodeIOSubtypes::GrayscaleImage)},
        NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:true, name:"alpha".to_string(),subtype:Some(NodeIOSubtypes::GrayscaleImage)},
        ]
    }

    fn get_node_name_static()->String{
        "Separate RGBA".to_string()
    }
}

impl Node for SeparateRGBANode{


    fn clear_buffers(&mut self) {
        *self = SeparateRGBANode::new();
    }


    
    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0 => if let NodeIOType::DynamicImageType(image) = value{
                self.separating = image;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            *Arc::get_mut(&mut self.red).unwrap() = DynamicImage::ImageLuma8(ImageBuffer::from_fn(self.separating.width(), self.separating.height(), |x,y| {
                Luma([self.separating.get_pixel(x, y).0[0]])
            }));
            *Arc::get_mut(&mut self.green).unwrap() = DynamicImage::ImageLuma8(ImageBuffer::from_fn(self.separating.width(), self.separating.height(), |x,y| {
                Luma([self.separating.get_pixel(x, y).0[1]])
            }));
            *Arc::get_mut(&mut self.blue).unwrap() = DynamicImage::ImageLuma8(ImageBuffer::from_fn(self.separating.width(), self.separating.height(), |x,y| {
                Luma([self.separating.get_pixel(x, y).0[2]])
            }));
            *Arc::get_mut(&mut self.alpha).unwrap() = DynamicImage::ImageLuma8(ImageBuffer::from_fn(self.separating.width(), self.separating.height(), |x,y| {
                Luma([self.separating.get_pixel(x, y).0[3]])
            }));


            self.buffered =true;
        }


        match index{
            0=>NodeResult::Ok(NodeIOType::DynamicImageType(self.red.clone())),
            1=>NodeResult::Ok(NodeIOType::DynamicImageType(self.green.clone())),
            2=>NodeResult::Ok(NodeIOType::DynamicImageType(self.blue.clone())),
            3=>NodeResult::Ok(NodeIOType::DynamicImageType(self.alpha.clone())),
            _=>Err(NodeError::InvalidOutputIndex(self.get_node_name(), index))
        }

        
    }
}