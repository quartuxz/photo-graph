use image::GenericImageView;

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
    moving : RgbaImage,
    x : f64,
    y: f64,
    buffer : RgbaImage,
    buffered:bool
}



impl MoveNode{
    pub fn new()->Self{
        MoveNode { mode:MoveMode::clamp,moving : RgbaImage::default(),x:0.0,y:0.0, buffer: RgbaImage::default(), buffered: false }
    }


    
}

impl NodeStatic for MoveNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("clamp".to_string());
        presetValues.push("extend".to_string());
        presetValues.push("wrap".to_string());
        vec![NodeInputOptions{name:"mode".to_string(),IOType:NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"moving".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"x".to_string(),IOType:NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"y".to_string(),IOType:NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"moved".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),hasConnection:true}]
    }

    fn get_node_name_static()->String {
        "Move".to_string()
    }
}

impl Node for MoveNode{
    fn clear_buffers(&mut self) {
        *self = MoveNode::new();
    }

    fn clear_inputs(&mut self) {
        self.moving = RgbaImage::default();
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
            1 => if let NodeIOType::BitmapType(image) = value{
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
            let roundedX = self.x.round() as i32;
            let roundedY = self.y.round() as i32;

            match self.mode{
                MoveMode::clamp=>{
                    if self.moving.width() as i32 + roundedX <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.x), 1))
                    }
                    if self.moving.height() as i32 + roundedY <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.y), 2))
                    }
                    self.buffer = RgbaImage::from_fn(self.moving.width(),self.moving.height() , 
                    |x,y|{
                        let imageX = x as i32 - roundedX;
                        let imageY = y as i32 - roundedY;
                        if(imageX < 0 || imageY < 0 || imageX >= self.moving.width() as i32 || imageY >= self.moving.height() as i32){
                            Rgba([0,0,0,0])
                        }else{
                            self.moving.get_pixel(imageX as u32, imageY as u32).clone()
                        }
                    
                    });
                },
                MoveMode::extend=>{
                    if self.moving.width() as i32 + roundedX <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.x), 1))
                    }
                    if self.moving.height() as i32 + roundedY <= 0{
                        return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(self.y), 2))
                    }
                    self.buffer = RgbaImage::from_fn((self.moving.width() as i32 +roundedX) as u32,(self.moving.height()as i32 +roundedY) as u32, 
                    |x,y|{
                        let imageX = x as i32 - roundedX;
                        let imageY = y as i32 - roundedY;
                        if(imageX < 0 || imageY < 0 || imageX >= self.moving.width() as i32 || imageY >= self.moving.height() as i32){
                            Rgba([0,0,0,0])
                        }else{
                            self.moving.get_pixel(imageX as u32, imageY as u32).clone()
                        }
                    
                    });
                },
                MoveMode::wrap=>{
                    self.buffer = RgbaImage::from_fn(self.moving.width(),self.moving.height(), 
                    |x,y|{
                        let imageX = (x as i32 + roundedX).rem_euclid(self.moving.width()as i32) ;
                        let imageY = (y as i32 + roundedY).rem_euclid(self.moving.height()as i32) ;
                        
                        self.moving.get_pixel(imageX as u32, imageY as u32).clone()
                    
                    });
                }
            }


            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}