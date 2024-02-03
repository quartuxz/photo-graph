

use super::*;


pub struct InvertNode{
    inverting : Arc<DynamicImage>,
    buffer : Arc<DynamicImage>,
    buffered : bool
}

impl InvertNode{
    pub fn new()->Self{
        InvertNode { inverting:Arc::new(DynamicImage::default()),buffer: Arc::new(DynamicImage::default()), buffered:false }
    }

}

impl NodeStatic for InvertNode{
    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{IOType:NodeIOType::DynamicImageType(Arc::new(DynamicImage::default())), canAlterDefault:false,hasConnection:true, name:"inverting".to_string(), presetValues:None,subtype:None}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:true,name:"inverted".to_string(),subtype:None}]
    }

    fn get_node_name_static()->String {
        "Invert".to_string()
    }
}

impl Node for InvertNode{

    fn clear_buffers(&mut self) {
        *self=InvertNode::new();
    }
    

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered{
            let mut inverted = (*self.inverting).clone();
            inverted.invert();
            self.buffer = Arc::new(inverted);
            self.buffered= true;
        }
        NodeResult::Ok(NodeIOType::DynamicImageType(self.buffer.clone()))
    }
    fn set(&mut self, index: u16, value:NodeIOType) -> NodeResult<()> {

        self.generate_input_errors(&index, &value)?;

        if let NodeIOType::DynamicImageType(image) = value{
            self.inverting = image
        }

        NodeResult::Ok(())
    }
}