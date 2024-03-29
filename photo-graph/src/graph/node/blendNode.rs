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
    buffer : Arc<DynamicImage>,
    foreground : Arc<DynamicImage>,
    background : Arc<DynamicImage>,
    buffered:bool
}



impl BlendNode{
    pub fn new()->Self{
        BlendNode { operation:BlendMode::multiply,foreground: Arc::new(DynamicImage::default()), background: Arc::new(DynamicImage::default()), buffer: Arc::new(DynamicImage::default()), buffered: false }
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
            NodeInputOptions{name:"foreground".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"background".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"mixed".to_string(),IOType:NodeIOType::DynamicImageType(Arc::default()),hasConnection:true,subtype:None}]
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
    //thanks to https://www.w3.org/TR/compositing-1 for explaining much of this
    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            let background = self.background.to_rgba8();
            let foreground = self.foreground.to_rgba8();
            *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageRgba8(RgbaImage::from_fn(std::cmp::max(self.foreground.width(),self.background.width()), std::cmp::max(self.foreground.height(),self.background.height()), |_x,_y| {Rgba([0,0,0,0])}));
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


        NodeResult::Ok(NodeIOType::DynamicImageType(self.buffer.clone()))
    }
}