use crate::image_utils::*;
use super::*;
use std::convert::TryFrom;


enum BlendMode{
    multiply,
    screen,
    darken,
    lighten,
    colorDodge,
    colorBurn
}

impl TryFrom<i64> for BlendMode {
    type Error = ();

    fn try_from(v: i64) -> Result<Self, Self::Error> {
        match v {
            x if x == BlendMode::multiply as i64 => Ok(BlendMode::multiply),
            x if x == BlendMode::screen as i64 => Ok(BlendMode::screen),
            x if x == BlendMode::darken as i64 => Ok(BlendMode::darken),
            x if x == BlendMode::lighten as i64 => Ok(BlendMode::lighten),
            x if x == BlendMode::colorBurn as i64 => Ok(BlendMode::colorBurn),
            x if x == BlendMode::colorDodge as i64 => Ok(BlendMode::colorDodge),
            _ => Err(()),
        }
    }
}
pub struct BlendNode{
    operation:BlendMode,
    buffer : RgbaImage,
    foreground : RgbaImage,
    background : RgbaImage,
    buffered:bool
}



impl BlendNode{
    pub fn new()->Self{
        BlendNode { operation:BlendMode::multiply,foreground: RgbaImage::default(), background: RgbaImage::default(), buffer: RgbaImage::default(), buffered: false }
    }


    
}

impl NodeStatic for BlendNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("multiply".to_string());
        presetValues.push("screen".to_string());
        presetValues.push("darken".to_string());
        presetValues.push("lighten".to_string());
        presetValues.push("color dodge".to_string());
        presetValues.push("color burn".to_string());
        vec![NodeInputOptions{name:"mode".to_string(),IOType: NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"foreground".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"background".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"mixed".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),hasConnection:true}]
    }

    fn get_node_name_static()->String {
        "Blend".to_string()
    }
}

impl Node for BlendNode{
    fn clear_buffers(&mut self) {
        *self = BlendNode::new();
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
    //thanks to https://www.w3.org/TR/compositing-1 for explaining much of this
    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            self.buffer = RgbaImage::from_fn(std::cmp::max(self.foreground.width(),self.background.width()), std::cmp::max(self.foreground.height(),self.background.height()), |_x,_y| {Rgba([0,0,0,0])});
            match self.operation{
                BlendMode::multiply=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};

                    
                    *pixel = blend(&fpix, &bpix, multiply_rgba_by_rgba);
                    pixel.0[3] = fpix.0[3];
                });},
                BlendMode::screen=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};

                    
                    *pixel = blend(&fpix, &bpix, screen_formula);
                    pixel.0[3] = fpix.0[3];
                });},
                BlendMode::darken=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};

                    
                    *pixel = blend(&fpix, &bpix, darken_formula);
                    pixel.0[3] = fpix.0[3];
                });},
                BlendMode::lighten=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};

                    
                    *pixel = blend(&fpix, &bpix, lighten_formula);
                    pixel.0[3] = fpix.0[3];
                });},
                BlendMode::colorDodge =>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};

                    
                    *pixel = blend(&fpix, &bpix, color_dodge_formula);
                    pixel.0[3] = fpix.0[3];
                });},
                BlendMode::colorBurn=>{self.buffer.enumerate_pixels_mut().for_each(|(x,y,pixel)|{
                    let mut fpix = match self.foreground.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};
                    let mut bpix = match self.background.get_pixel_checked(x, y){Some(val)=>val.clone(),None=>Rgba([0,0,0,0])};

                    
                    *pixel = blend(&fpix, &bpix, color_burn_formula);
                    pixel.0[3] = fpix.0[3];
                });}
            }
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}