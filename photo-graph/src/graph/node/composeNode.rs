use crate::image_utils::*;

use super::*;
use std::convert::TryFrom;


#[derive(macro_utils::TryFrom)]
#[conversion_type(i64)]
enum CompositionType{
    overlay,
    mask,
    inverseMask,
    atop,
    neither,
    foreground
}


pub struct ComposeNode{
    operation:CompositionType,
    buffer : Arc<DynamicImage>,
    foreground : Arc<DynamicImage>,
    background : Arc<DynamicImage>,
    buffered:bool
}



impl ComposeNode{
    pub fn new()->Self{
        ComposeNode { operation:CompositionType::overlay,foreground: Arc::new(DynamicImage::default()), background: Arc::new(DynamicImage::default()), buffer: Arc::new(DynamicImage::default()), buffered: false }
    }


    
}

impl NodeStatic for ComposeNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("overlay".to_string());
        presetValues.push("mask".to_string());
        presetValues.push("inverse mask".to_string());
        presetValues.push("atop".to_string());
        presetValues.push("neither".to_string());
        presetValues.push("foreground".to_string());
        vec![NodeInputOptions{name:"mode".to_string(),IOType: NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"foreground".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"background".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"mixed".to_string(),IOType:NodeIOType::DynamicImageType(Arc::default()),hasConnection:true,subtype:None}]
    }

    fn get_node_name_static()->String {
        "Compose".to_string()
    }
}

impl Node for ComposeNode{
    fn clear_buffers(&mut self) {
        *self = ComposeNode::new();
    }


    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0 => if let NodeIOType::IntType(operation) = value{
                self.operation = match operation.try_into(){
                    Ok(val)=>val,
                    Err(_)=> return Err(NodeError::InvalidInput(Self::get_node_name_static(), value, index))
                };
            }
            1 => if let NodeIOType::DynamicImageType(image) = value{
                self.foreground = image;
            }
            2 => if let NodeIOType::DynamicImageType(image) = value{
                self.background = image;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }
    //thanks to https://ciechanow.ski/alpha-compositing/ for explaining much of this
    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            let foreground = self.foreground.to_rgba8();
            let background = self.background.to_rgba8();
            *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageRgba8(RgbaImage::from_fn(std::cmp::max(self.foreground.width(),self.background.width()), std::cmp::max(self.foreground.height(),self.background.height()), |_x,_y| {Rgba([0,0,0,0])}));
            match self.operation{
                CompositionType::mask => {Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = color_u8_to_f32(match foreground.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let bpix = color_u8_to_f32(match background.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let falpha = fpix.0[3];
                    let balpha = bpix.0[3];
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);

                    let result = multiply_color(&fpix, balpha);
                    *pixel = color_f32_to_u8(&get_relative_color(&result, 1.0/result.0[3]));
                    
                });},
                CompositionType::overlay => {Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = color_u8_to_f32(match foreground.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let mut bpix = color_u8_to_f32(match background.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let falpha = fpix.0[3];
                    let balpha = bpix.0[3];
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);
                    bpix = get_relative_color(&bpix, balpha);

                    bpix = multiply_color(&bpix, 1.0-falpha);

                    let result = saturating_add_rgba(&fpix, &bpix);
                    *pixel = color_f32_to_u8(&get_relative_color(&result, 1.0/result.0[3]));
                });},
                CompositionType::inverseMask => {Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = color_u8_to_f32(match foreground.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let mut bpix = color_u8_to_f32(match background.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let falpha = fpix.0[3];
                    let balpha = bpix.0[3];
                    //premultiply
                    bpix = get_relative_color(&bpix, balpha);

                    let result = multiply_color(&bpix, 1.0-falpha);
                    *pixel = color_f32_to_u8(&get_relative_color(&result, 1.0/result.0[3]));

                });},
                CompositionType::atop=>{Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = color_u8_to_f32(match foreground.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let mut bpix = color_u8_to_f32(match background.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let falpha = fpix.0[3];
                    let balpha = bpix.0[3];
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);
                    bpix = get_relative_color(&bpix, balpha);
                    
                    fpix = multiply_color(&fpix, balpha);
                    bpix = multiply_color(&bpix, 1.0-falpha);
                    let result = saturating_add_rgba(&fpix, &bpix);
                    *pixel = color_f32_to_u8(&get_relative_color(&result, 1.0/result.0[3]));
                });},
                CompositionType::neither=>{Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = color_u8_to_f32(match foreground.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let mut bpix = color_u8_to_f32(match background.get_pixel_checked(x, y){Some(val)=>val,None=>&Rgba([0,0,0,0])});
                    let falpha = fpix.0[3];
                    let balpha = bpix.0[3];
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);
                    bpix = get_relative_color(&bpix, balpha);
                    
                    fpix = multiply_color(&fpix, 1.0-balpha);
                    bpix = multiply_color(&bpix, 1.0-falpha);
                    let result = saturating_add_rgba(&fpix, &bpix);
                    *pixel = color_f32_to_u8(&get_relative_color(&result, 1.0/result.0[3]));
                });},
                CompositionType::foreground=>{Arc::get_mut(&mut self.buffer).unwrap().as_mut_rgba8().unwrap().enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    //source
                    let mut fpix = match foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    *pixel = fpix;
                })

                }

            }
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::DynamicImageType(self.buffer.clone()))
    }
}

