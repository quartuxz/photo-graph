use super::*;

pub struct BitmapLiteralNode{
    bitmap : RgbaImage
}

impl BitmapLiteralNode{
    pub fn new(bitmap:RgbaImage)->BitmapLiteralNode{
        BitmapLiteralNode { bitmap }
    }


}

impl NodeStatic for BitmapLiteralNode{
    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true, name:"".to_string()}]
    }

    fn get_node_name_static()->String {
        "Bitmap literal".to_string()
    }
}


impl Node for BitmapLiteralNode{

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::BitmapType(self.bitmap.clone()));
    }
}