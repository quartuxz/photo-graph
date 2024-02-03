use super::*;

#[derive(macro_utils::TryFrom)]
#[conversion_type(i64)]
enum ResizeMode{
    clamp,
    resize
}

pub struct ResizeNode{
    mode: ResizeMode,
    resizing : Arc<DynamicImage>,
    x : f64,
    y: f64,
    buffer : Arc<DynamicImage>,
    buffered:bool
}



impl ResizeNode{
    pub fn new()->Self{
        ResizeNode { mode:ResizeMode::clamp,resizing : Arc::new(DynamicImage::default()),x:0.0,y:0.0, buffer: Arc::new(DynamicImage::default()), buffered: false }
    }


    
}

impl NodeStatic for ResizeNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("clamp".to_string());
        presetValues.push("resize".to_string());
        vec![NodeInputOptions{name:"mode".to_string(),IOType:NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"resizing".to_string(),IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())),canAlterDefault:false,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"x".to_string(),IOType:NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"y".to_string(),IOType:NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"resized".to_string(),IOType:NodeIOType::DynamicImageType(Arc::default()),hasConnection:true,subtype:None}]
    }

    fn get_node_name_static()->String {
        "Resize".to_string()
    }
}

impl Node for ResizeNode{
    fn clear_buffers(&mut self) {
        *self = ResizeNode::new();
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
                self.resizing = image;
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

            let resizing = self.resizing.to_rgba8();
            match self.mode{
                //clamps thhe size based on non-transparent pixels, the new size is equal to the bounds of the non-transparent image + xy
                ResizeMode::clamp=>{
                    let mut minX = u32::MAX;
                    let mut minY = u32::MAX; 
                    let mut maxX = u32::MIN;
                    let mut maxY = u32::MIN;


                    for (x,y,pix) in resizing.enumerate_pixels(){
                        if(pix.0[3] != 0){
                            minX = if x <  minX {x}else{minX};
                            maxX = if x >  maxX {x}else{maxX};
                            minY = if y <  minY {y}else{minY};
                            maxY = if y >  maxY {y}else{maxY};
                        }

                    }
                    *Arc::get_mut(&mut self.buffer).unwrap() = DynamicImage::ImageRgba8(RgbaImage::from_fn(((maxX-minX) as i32 +roundedX) as u32, ((maxY-minY) as i32 +roundedY) as u32, |x,y|{
                        let ax = x+minX;
                        let ay = y+minY;
                        resizing.get_pixel_checked(ax, ay).unwrap_or(&Rgba([0,0,0,0])).clone()
                    }))
                },
                //resizes the image, filling the new space with transparency
                ResizeMode::resize=>{
                    *Arc::get_mut(&mut self.buffer).unwrap() =  DynamicImage::ImageRgba8(RgbaImage::from_fn((self.resizing.width() as i32 +roundedX) as u32, (self.resizing.height() as i32 +roundedY) as u32, |x,y|{
                        resizing.get_pixel_checked(x, y).unwrap_or(&Rgba([0,0,0,0])).clone()
                    }))
                },

            }


            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::DynamicImageType(self.buffer.clone()))
    }
}