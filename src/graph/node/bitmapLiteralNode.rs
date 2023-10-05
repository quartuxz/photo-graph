use super::*;

pub struct BitmapLiteralNode{
    bitmap : RgbaImage
}

impl BitmapLiteralNode{
    pub fn new(bitmap:RgbaImage)->BitmapLiteralNode{
        BitmapLiteralNode { bitmap }
    }
}


impl Node for BitmapLiteralNode{
    fn get_node_name(&self)->String {
        "bitmap literal".to_string()
    }

    fn get_outputs(&self)->Vec<NodeOutputOptions> {
        vec![NodeOutputOptions{IOType:NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true, name:"".to_string()}]
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::BitmapType(self.bitmap.clone()));
    }
}