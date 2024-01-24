use crate::image_utils::bilinear_interpolate;

use super::*;


#[derive(macro_utils::TryFrom)]
#[conversion_type(i64)]
enum ScaleMode{
    fast,
    precise
}

pub struct ScaleNode{
    mode : ScaleMode,
    scaling : RgbaImage,
    x : f64,
    y: f64,
    buffer : RgbaImage,
    buffered:bool
}



impl ScaleNode{
    pub fn new()->Self{
        ScaleNode {mode:ScaleMode::fast,scaling : RgbaImage::default(),x:0.0,y:0.0, buffer: RgbaImage::default(), buffered: false }
    }


    
}

impl NodeStatic for ScaleNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("fast".to_string());
        presetValues.push("precise".to_string());
        vec![
            NodeInputOptions{name:"mode".to_string(),IOType:NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"scaling".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"x".to_string(),IOType:NodeIOType::FloatType(1.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"y".to_string(),IOType:NodeIOType::FloatType(1.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"Scaled".to_string(),IOType:NodeIOType::BitmapType(RgbaImage::default()),hasConnection:true}]
    }

    fn get_node_name_static()->String {
        "Scale".to_string()
    }
}

impl Node for ScaleNode{
    fn clear_buffers(&mut self) {
        *self = ScaleNode::new();
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
                self.scaling = image;
            }
            2 => if let NodeIOType::FloatType(x) = value{
                if x < 0.0{
                    return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(x), index));
                }
                self.x = x;
            }

            3 => if let NodeIOType::FloatType(y) = value{
                if y < 0.0{
                    return Err(NodeError::InvalidInput(Self::get_node_name_static(), NodeIOType::FloatType(y), index));
                }
                self.y = y;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {            
            match self.mode{
                ScaleMode::fast => self.buffer = RgbaImage::from_fn((self.scaling.width() as f64*self.x).ceil() as u32, (self.scaling.height() as f64*self.y).ceil() as u32, |x,y|{
                    let ax = x as f64 * self.x.recip();
                    let ay = y as f64 * self.y.recip();
                    match self.scaling.get_pixel_checked(ax.round() as u32, ay.round() as u32){
                        Some(val)=>val.clone(),
                        None => Rgba([0,0,0,0])
                    }
                }),
                ScaleMode::precise => self.buffer = RgbaImage::from_fn((self.scaling.width() as f64*self.x).ceil() as u32, (self.scaling.height() as f64*self.y).ceil() as u32, |x,y|{
                    let ax = (x as f64+0.5) * self.x.recip();
                    let ay = (y as f64+0.5) * self.y.recip();
                    bilinear_interpolate(&self.scaling, ax, ay)
                })
            }
            
            

            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}