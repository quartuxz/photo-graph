use std::cmp;

use image::GenericImageView;

use crate::image_utils::{bilinear_interpolate, color_f32_to_u8};

use super::*;

#[derive(macro_utils::TryFrom)]
#[conversion_type(i64)]
enum MoveMode{
    clamp,
    extend,
    wrap
}

pub struct MoveNode{
    mode: MoveMode,
    moving : Arc<DynamicImage>,
    x : f64,
    y: f64,
    buffer : Arc<DynamicImage>,
    buffered:bool
}



impl MoveNode{
    pub fn new()->Self{
        MoveNode { mode:MoveMode::clamp,moving : Arc::new(DynamicImage::default()),x:0.0,y:0.0, buffer: Arc::new(DynamicImage::default()), buffered: false }
    }


    
}

impl NodeStatic for MoveNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("clamp".to_string());
        presetValues.push("extend".to_string());
        presetValues.push("wrap".to_string());
        vec![NodeInputOptions{name:"mode".to_string(),IOType:NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"moving".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"x".to_string(),IOType:NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"y".to_string(),IOType:NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"moved".to_string(),IOType:NodeIOType::DynamicImageType(Arc::default()),hasConnection:true,subtype:None}]
    }

    fn get_node_name_static()->String {
        "Move".to_string()
    }
}

impl Node for MoveNode{
    fn clear_buffers(&mut self) {
        *self = MoveNode::new();
    }

    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0 => if let NodeIOType::IntType(operation) = value{
                self.mode = match operation.try_into(){
                    Ok(val)=>val,
                    Err(_)=> return Err(NodeError::InvalidInput(Self::get_node_name_static(), value, index))
                };
            }
            1 => if let NodeIOType::DynamicImageType(image) = value{
                self.moving = image;
            }
            2 => if let NodeIOType::FloatType(x) = value{
                self.x = x;
            }

            3 => if let NodeIOType::FloatType(y) = value{
                self.y = y;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            let roundedX = self.x.ceil() as i32;
            let roundedY = self.y.ceil() as i32;
            let moving = self.moving.to_rgba8();
            match self.mode{
                MoveMode::clamp=>{
                    if self.moving.width() as i32 + roundedX <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.x), 1))
                    }
                    if self.moving.height() as i32 + roundedY <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.y), 2))
                    }
                    *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageRgba8(RgbaImage::from_fn((self.moving.width() as i32+cmp::min(0, roundedX)) as u32,(self.moving.height() as i32+cmp::min(0, roundedY)) as u32 , 
                    |x,y|{
                        let imageX = x as f64 +0.5 - self.x;
                        let imageY = y as f64 +0.5 - self.y;
                        
                        color_f32_to_u8(&bilinear_interpolate(&moving, imageX, imageY))
                    
                    }));
                },
                MoveMode::extend=>{
                    if self.moving.width() as i32 + roundedX <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.x), 1))
                    }
                    if self.moving.height() as i32 + roundedY <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.y), 2))
                    }
                    *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageRgba8(RgbaImage::from_fn((self.moving.width() as i32 +roundedX) as u32,(self.moving.height()as i32 +roundedY) as u32, 
                    |x,y|{
                        let imageX = x as f64 +0.5 - self.x;
                        let imageY = y as f64 +0.5 - self.y;
                        color_f32_to_u8(&bilinear_interpolate(&moving, imageX, imageY))
                    
                    }));
                },
                MoveMode::wrap=>{
                    *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageRgba8(RgbaImage::from_fn(self.moving.width(),self.moving.height(), 
                    |x,y|{
                        let imageX = (x as f64+0.5 - self.x).rem_euclid(self.moving.width()as f64) ;
                        let imageY = (y as f64+0.5 - self.y).rem_euclid(self.moving.height()as f64) ;
                        
                        color_f32_to_u8(&bilinear_interpolate(&moving, imageX, imageY))
                    
                    }));
                }
            }


            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::DynamicImageType(self.buffer.clone()))
    }
}