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
    buffer : RgbaImage,
    foreground : RgbaImage,
    background : RgbaImage,
    buffered:bool
}



impl ComposeNode{
    pub fn new()->Self{
        ComposeNode { operation:CompositionType::overlay,foreground: RgbaImage::default(), background: RgbaImage::default(), buffer: RgbaImage::default(), buffered: false }
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
            NodeInputOptions{name:"foreground".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"background".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"mixed".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),hasConnection:true}]
    }

    fn get_node_name_static()->String {
        "Compose".to_string()
    }
}

impl Node for ComposeNode{
    fn clear_buffers(&mut self) {
        *self = ComposeNode::new();
    }
    fn clear_inputs(&mut self) {
        self.background = RgbaImage::default();
        self.foreground = RgbaImage::default();
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
            1 => if let NodeIOType::BitmapType(image) = value{
                self.foreground = image;
            }
            2 => if let NodeIOType::BitmapType(image) = value{
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
            self.buffer = RgbaImage::from_fn(std::cmp::max(self.foreground.width(),self.background.width()), std::cmp::max(self.foreground.height(),self.background.height()), |_x,_y| {Rgba([0,0,0,0])});
            match self.operation{
                CompositionType::mask => {self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let falpha = normalized(fpix.0[3]);
                    let balpha = normalized(bpix.0[3]);
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);

                    *pixel = multiply_color(&fpix, balpha);
                    
                });},
                CompositionType::overlay => {self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let falpha = normalized(fpix.0[3]);
                    let balpha = normalized(bpix.0[3]);
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);
                    bpix = get_relative_color(&bpix, balpha);

                    bpix = multiply_color(&bpix, 1.0-falpha);
                    *pixel = saturating_add_rgba(&fpix, &bpix);
                });},
                CompositionType::inverseMask => {self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let falpha = normalized(fpix.0[3]);
                    let balpha = normalized(bpix.0[3]);
                    //premultiply
                    bpix = get_relative_color(&bpix, balpha);

                    *pixel = multiply_color(&bpix, 1.0-falpha);

                });},
                CompositionType::atop=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    //source
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    //destination
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let falpha = normalized(fpix.0[3]);
                    let balpha = normalized(bpix.0[3]);
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);
                    bpix = get_relative_color(&bpix, balpha);
                    
                    fpix = multiply_color(&fpix, balpha);
                    bpix = multiply_color(&bpix, 1.0-falpha);
                    *pixel = saturating_add_rgba(&fpix, &bpix);
                });},
                CompositionType::neither=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    //source
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    //destination
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let falpha = normalized(fpix.0[3]);
                    let balpha = normalized(bpix.0[3]);
                    //premultiply
                    fpix = get_relative_color(&fpix, falpha);
                    bpix = get_relative_color(&bpix, balpha);
                    
                    fpix = multiply_color(&fpix, 1.0-balpha);
                    bpix = multiply_color(&bpix, 1.0-falpha);
                    *pixel = saturating_add_rgba(&fpix, &bpix);
                });},
                CompositionType::foreground=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    //source
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    *pixel = fpix;
                })

                }

            }
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}