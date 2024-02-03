use crate::image_utils::*;
use super::*;



pub struct OverlayWithMaskNode{
    buffer : Arc<DynamicImage>,
    top : Arc<DynamicImage>,
    bottom : Arc<DynamicImage>,
    mask : Arc<DynamicImage>,
    buffered:bool
}



impl OverlayWithMaskNode{
    pub fn new()->Self{
        OverlayWithMaskNode { mask: Arc::new(DynamicImage::default()), top: Arc::new(DynamicImage::default()), bottom: Arc::new(DynamicImage::default()), buffer: Arc::new(DynamicImage::default()), buffered: false }
    }


    
}

impl NodeStatic for OverlayWithMaskNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{

        vec![NodeInputOptions{name:"top".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"bottom".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"mask".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:Some(NodeIOSubtypes::GrayscaleImage)}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"overlayed".to_string(),IOType:NodeIOType::DynamicImageType(Arc::default()),hasConnection:true,subtype:None}]
    }

    fn get_node_name_static()->String {
        "Overlay with mask".to_string()
    }
}

impl Node for OverlayWithMaskNode{
    fn clear_buffers(&mut self) {
        *self = OverlayWithMaskNode::new();
    }

    
    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0 => if let NodeIOType::DynamicImageType(image) = value{
                self.top = image;
            }
            1 => if let NodeIOType::DynamicImageType(image) = value{
                self.bottom = image;
            }
            2 => if let NodeIOType::DynamicImageType(image) = value{
                self.mask = image;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            let top = self.top.to_rgba8();
            let bottom = self.bottom.to_rgba8();
            let mask = self.mask.grayscale().to_luma8();
            *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageRgba8(RgbaImage::from_fn(std::cmp::max(top.width(),bottom.width()), std::cmp::max(top.height(),bottom.height()), |_x,_y| {Rgba([0,0,0,0])}));
            Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                let maskPix = normalized(mask.get_pixel_checked(x, y).unwrap_or(&Luma([0])).0[0]);

                let mut tpix = match top.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                let mut bpix = match bottom.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                let talpha = normalized(tpix.0[3]);
                let balpha = normalized(bpix.0[3]);
                //premultiply
                tpix = get_relative_color(&tpix, talpha);
                bpix = get_relative_color(&bpix, balpha);


                tpix = multiply_color(&tpix, maskPix);
                bpix = multiply_color(&bpix, 1.0-maskPix);
                *pixel = saturating_add_rgba(&tpix, &bpix);
            });
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::DynamicImageType(self.buffer.clone()))
    }
}