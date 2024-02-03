use super::*;

pub struct DynamicImageLiteralNode{
    image : Arc<DynamicImage>
}

impl DynamicImageLiteralNode{
    pub fn new(bitmap:DynamicImage)->DynamicImageLiteralNode{
        DynamicImageLiteralNode { image: Arc::new(bitmap) }
    }


}

impl NodeStatic for DynamicImageLiteralNode{
    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::DynamicImageType(Arc::default()), hasConnection:true, name:"".to_string(),subtype:None}]
    }

    fn get_node_name_static()->String {
        "Dynamic image literal".to_string()
    }
}


impl Node for DynamicImageLiteralNode{

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::DynamicImageType(self.image.clone()));
    }
}