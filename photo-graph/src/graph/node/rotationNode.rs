use std::{cmp::Ordering, collections::HashMap, default};

use crate::image_utils::{multiply_color, saturating_add_rgba};

use super::*;

pub struct RotationNode{
    rotating:RgbaImage,
    angle:f64,
    buffer:RgbaImage,
    buffered:bool
}

impl RotationNode{
    pub fn new()->Self{
        RotationNode { rotating: RgbaImage::default(), angle: 0.0, buffered: false, buffer:RgbaImage::default() }
    }

}

impl NodeStatic for RotationNode{
    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{name:"rotating".to_string(),IOType: NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true, presetValues:None,subtype:None},
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
            0 => if let NodeIOType::BitmapType(image) = value{
                self.rotating = image;
            },
            1 => if let NodeIOType::FloatType(angle) = value{
                self.angle = angle*(std::f64::consts::PI/180.0);
            },

            _ => ()
        }


        NodeResult::Ok(())
    }


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            let mut rotatedPixels:Vec<(f64,f64)> = Vec::new();
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
            self.angle = -self.angle;


            for px in &mut rotatedPixels{
                px.0 = px.0-minX;
                px.1 = px.1-minY;
                

                // {
                //     let dest = self.buffer.get_pixel_mut(px.0.floor() as u32, px.1.floor() as u32);
                //     *dest = saturating_add_rgba(&multiply_color(&px.2, (((px.0.floor()+1.0) - (px.0))*((px.1.floor()+1.0)-(px.1))).abs() as f32),&dest);
                // }
                // if px.0.ceil() != px.0.floor(){
                //     let dest = self.buffer.get_pixel_mut(px.0.ceil() as u32, px.1.floor() as u32);
                //     *dest = saturating_add_rgba(&multiply_color(&px.2, (((px.0.floor()+1.0) - (px.0+1.0))*((px.1.floor()+1.0)-(px.1))).abs() as f32),&dest);
                // }
                // if px.1.ceil() != px.1.floor(){
                //     let dest = self.buffer.get_pixel_mut(px.0.floor() as u32, px.1.ceil() as u32);
                //     *dest = saturating_add_rgba(&multiply_color(&px.2, (((px.0.floor()+1.0) - (px.0))*((px.1.floor()+1.0)-(px.1+1.0))).abs() as f32),&dest);
                // }
                // if px.1.ceil() != px.1.floor() && px.0.ceil() != px.0.floor(){
                //     let dest = self.buffer.get_pixel_mut(px.0.ceil() as u32, px.1.ceil() as u32);
                //     *dest = saturating_add_rgba(&multiply_color(&px.2, (((px.0.floor()+1.0) - (px.0+1.0))*((px.1.floor()+1.0)-(px.1+1.0))).abs() as f32),&dest);
                // }

            }





            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }


}