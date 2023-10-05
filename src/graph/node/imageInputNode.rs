use super::*;
use image::DynamicImage;

pub struct ImageInputNode{
    location : String,
    buffered: bool,
    buffer : RgbaImage
}

impl ImageInputNode{
    fn retrieve_image(&mut self){
        self.buffer = match image::open(self.location.clone()).unwrap(){
            DynamicImage::ImageRgba8(im) => im,
            _ => panic!("unsupported Image")
        };
    }
    pub fn new()->Self{
        ImageInputNode { location: "".to_string(), buffered: false, buffer: RgbaImage::default() }
    }
}

impl Node for ImageInputNode{

    fn get_node_name(&self)->String {
        "Image Input".to_string()
    }
    fn clear_buffers(&mut self) {
        self.buffered = false;
        self.buffer = RgbaImage::default();
        self.location = String::default();
    }

    fn get_outputs(&self)->Vec<NodeOutputOptions> {
        vec![NodeOutputOptions{IOType:NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true, name:"bitmap".to_string()}]
    }

    fn get_inputs(&self)->Vec<NodeInputOptions> {
        vec![NodeInputOptions{IOType:NodeIOType::StringType("dummy.png".to_string()), canAlterDefault:false,hasConnection:false, name:"".to_string(), presetValues:None}]
    }

    
    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        if let NodeIOType::StringType(location) = value{
            self.location = location;
        }

        NodeResult::Ok(())
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if(!self.buffered){
            self.retrieve_image();
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::BitmapType(self.buffer.clone()))
    }
}
