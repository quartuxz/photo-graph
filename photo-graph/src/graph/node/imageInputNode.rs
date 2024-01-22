use super::*;
use image::DynamicImage;

pub struct ImageInputNode{
    filename : String,
    buffered: bool,
    buffer : RgbaImage,
    username: String
}


impl ImageInputNode{
    fn retrieve_image(&mut self)->NodeResult<()>{
        let mut finalFile = self.filename.clone();
        finalFile = crate::util::sanitize(&finalFile,false);
        if(finalFile.is_empty()){
            finalFile="dummy.png".to_owned();
        }
        if finalFile != "dummy.png"{
            finalFile = crate::util::sanitize(&self.username,true) + finalFile.as_str();
            self.buffer = match image::open(crate::util::RESOURCE_PATH.clone()+"/images/"+ finalFile.as_str()){Ok(val)=>val,Err(_)=>return Err(NodeError::IOError(Self::get_node_name_static()))}.into_rgba8();
        }else{
            self.buffer = match image::open(crate::util::RESOURCE_PATH.clone()+"/web/"+ finalFile.as_str()){Ok(val)=>val,Err(_)=>return Err(NodeError::IOError(Self::get_node_name_static()))}.into_rgba8();
        }

        Ok(())
    }
    pub fn new(username:String)->Self{
        ImageInputNode { filename: String::default(), buffered: false, buffer: RgbaImage::default(), username }
    }

}

impl NodeStatic for ImageInputNode{

    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{IOType:NodeIOType::StringType("dummy.png".to_string()), canAlterDefault:true,hasConnection:false, name:"path".to_string(), presetValues:None,subtype:Some(NodeIOSubtypes::FilePath)}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true, name:"bitmap".to_string()}]
    }

    fn get_node_name_static()->String{
        "Image input".to_string()
    }
}

impl Node for ImageInputNode{


    fn clear_buffers(&mut self) {
        *self = ImageInputNode::new(self.username.clone());
    }


    
    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        if let NodeIOType::StringType(location) = value{
            self.filename = location;
        }

        NodeResult::Ok(())
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            self.retrieve_image()?;
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}
