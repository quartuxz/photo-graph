use crate::image_utils::*;
use super::*;
use std::convert::TryFrom;


#[derive(macro_utils::TryFrom)]
#[conversion_type(i64)]
enum BlendMode{
    multiply,
    screen,
    darken,
    lighten,
    colorDodge,
    colorBurn,
    linearDodge,
    linearBurn
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
        presetValues.push("linear dodge".to_string());
        presetValues.push("linear burn".to_string());
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
    //thanks to https://www.w3.org/TR/compositing-1 for explaining much of this
    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            self.buffer = RgbaImage::from_fn(std::cmp::max(self.foreground.width(),self.background.width()), std::cmp::max(self.foreground.height(),self.background.height()), |_x,_y| {Rgba([0,0,0,0])});
            match self.operation{
                BlendMode::multiply=>{macro_utils::make_blend!(multiply_rgba_by_rgba);},
                BlendMode::screen=>{macro_utils::make_blend!(screen_formula);},
                BlendMode::darken=>{macro_utils::make_blend!(darken_formula);},
                BlendMode::lighten=>{macro_utils::make_blend!(lighten_formula);},
                BlendMode::colorDodge =>{macro_utils::make_blend!(color_dodge_formula);},
                BlendMode::colorBurn=>{macro_utils::make_blend!(color_burn_formula);},
                BlendMode::linearDodge=>{macro_utils::make_blend!(saturating_add_rgba);},
                BlendMode::linearBurn=>{macro_utils::make_blend!(inverse_of_add);}
            }
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}