use std::{cmp::Ordering, collections::HashMap, default};

use crate::image_utils::{multiply_color, saturating_add_rgba};

use super::*;


#[derive(macro_utils::TryFrom)]
#[conversion_type(i64)]
enum RotationMode{
    fast,
    precise
}

pub struct RotationNode{
    mode: RotationMode,
    rotating:RgbaImage,
    angle:f64,
    buffer:RgbaImage,
    buffered:bool
}

impl RotationNode{
    pub fn new()->Self{
        RotationNode { mode: RotationMode::fast,rotating: RgbaImage::default(), angle: 0.0, buffered: false, buffer:RgbaImage::default() }
    }

}

impl NodeStatic for RotationNode{
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("fast".to_string());
        presetValues.push("precise".to_string());
        vec![
            NodeInputOptions{name:"mode".to_string(),IOType:NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"rotating".to_string(),IOType: NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true, presetValues:None,subtype:None},
            NodeInputOptions{name: "angle".to_string(),IOType: NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true, presetValues:None,subtype:None}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name: "rotated".to_string(), IOType: NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true}]
    }

    fn get_node_name_static()->String {
        "Rotate".to_string()
    }
}


impl Node for RotationNode{

    fn clear_buffers(&mut self) {
        *self = RotationNode::new();
    }

    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0=> if let NodeIOType::IntType(mode) = value{
                self.mode = match mode.try_into(){
                    Ok(val)=>val,
                    Err(_)=> return Err(NodeError::InvalidInput(Self::get_node_name_static(), value, index))
                };
            },
            1 => if let NodeIOType::BitmapType(image) = value{
                self.rotating = image;
            },
            2 => if let NodeIOType::FloatType(angle) = value{
                self.angle = angle*(std::f64::consts::PI/180.0);
            },

            _ => ()
        }


        NodeResult::Ok(())
    }


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            let mut minX = f64::MAX;
            let mut minY = f64::MAX; 
            let mut maxX = f64::MIN;
            let mut maxY = f64::MIN;
            
            let halfWidth = self.rotating.width() as f64/2.0;
            let halfHeight = self.rotating.height() as f64/2.0;
            for x in [0,self.rotating.width()]{
                for y in [0,self.rotating.height()]{

                    let mut rotX = ((x as f64)-halfWidth)*self.angle.cos()-((y as f64)-halfHeight)*self.angle.sin();
                    let mut rotY = ((x as f64)-halfWidth)*self.angle.sin()+((y as f64)-halfHeight)*self.angle.cos();
                    rotX += halfWidth;
                    rotY += halfHeight;
                    minX = if rotX <  minX {rotX}else{minX};
                    maxX = if rotX >  maxX {rotX}else{maxX};
                    minY = if rotY <  minY {rotY}else{minY};
                    maxY = if rotY >  maxY {rotY}else{maxY};
                }
            }


            
            self.angle = -self.angle;
            match self.mode{
                RotationMode::precise =>{
                    self.buffer = RgbaImage::from_fn((maxX-minX).ceil() as u32, (maxY-minY).ceil() as u32, |x,y|{
                        let mut rotX = ((x as f64)-halfWidth+minX)*self.angle.cos()-((y as f64)-halfHeight+minY)*self.angle.sin();
                        let mut rotY = ((x as f64)-halfWidth+minX)*self.angle.sin()+((y as f64)-halfHeight+minY)*self.angle.cos();
                        rotX += halfWidth;
                        rotY += halfHeight;
                        let mut finalPix = Rgba([0,0,0,0]);
                        //top left
                        {
                            let source = match self.rotating.get_pixel_checked((rotX.floor()) as u32, (rotY.floor()) as u32){
                                Some(val) if rotX>0.0 && rotY >0.0=> val.clone(),
                                Some(_) => Rgba([0,0,0,0]),
                                None => Rgba([0,0,0,0])
                            };
                            finalPix = saturating_add_rgba(&finalPix, &multiply_color(&source, (((rotX.floor()+1.0) - (rotX))*((rotY.floor()+1.0)-(rotY))).abs() as f32))
                        }
                        if rotX.floor() != rotX.ceil(){
                            let source = match self.rotating.get_pixel_checked((rotX.ceil()) as u32, (rotY.floor()) as u32){
                                Some(val) if rotX>0.0 && rotY >0.0=> val.clone(),
                                Some(_) => Rgba([0,0,0,0]),
                                None => Rgba([0,0,0,0])
                            };
                            finalPix = saturating_add_rgba(&finalPix, &multiply_color(&source, (((rotX.floor()+1.0) - (rotX+1.0))*((rotY.floor()+1.0)-(rotY))).abs() as f32))
                        }
                        if rotY.floor() != rotY.ceil(){
                            let source = match self.rotating.get_pixel_checked((rotX.floor()) as u32, (rotY.ceil()) as u32){
                                Some(val) if rotX>0.0 && rotY >0.0=> val.clone(),
                                Some(_) => Rgba([0,0,0,0]),
                                None => Rgba([0,0,0,0])
                            };
                            finalPix = saturating_add_rgba(&finalPix, &multiply_color(&source, (((rotX.floor()+1.0) - (rotX))*((rotY.floor()+1.0)-(rotY+1.0))).abs() as f32))
                        }
                        if rotX.floor() != rotX.ceil() && rotY.floor() != rotY.ceil(){
                            let source = match self.rotating.get_pixel_checked((rotX.ceil()) as u32, (rotY.ceil()) as u32){
                                Some(val) if rotX>0.0 && rotY >0.0=> val.clone(),
                                Some(_) => Rgba([0,0,0,0]),
                                None => Rgba([0,0,0,0])
                            };
                            finalPix = saturating_add_rgba(&finalPix, &multiply_color(&source, (((rotX.floor()+1.0) - (rotX+1.0))*((rotY.floor()+1.0)-(rotY+1.0))).abs() as f32))
                        }
                        finalPix
                    });
                }
                RotationMode::fast => {
                    self.buffer = RgbaImage::from_fn((maxX-minX).ceil() as u32, (maxY-minY).ceil() as u32, |x,y|{
                        let mut rotX = ((x as f64)-halfWidth+minX)*self.angle.cos()-((y as f64)-halfHeight+minY)*self.angle.sin();
                        let mut rotY = ((x as f64)-halfWidth+minX)*self.angle.sin()+((y as f64)-halfHeight+minY)*self.angle.cos();
                        rotX += halfWidth;
                        rotY += halfHeight;
                        
                        match self.rotating.get_pixel_checked((rotX.round()) as u32, (rotY.round()) as u32){
                            Some(val) if rotX>0.0 && rotY >0.0=> val.clone(),
                            Some(_) => Rgba([0,0,0,0]),
                            None => Rgba([0,0,0,0])
                        }
                            
                    });
                }
            }

            self.angle = -self.angle;


            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }


}