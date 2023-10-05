use super::*;

pub struct RotationNode{
    bitmap:RgbaImage,
    angle:f64,
    buffered:bool
}

impl RotationNode{
    pub fn new()->Self{
        RotationNode { bitmap: RgbaImage::default(), angle: 0.0, buffered: false }
    }
}

impl Node for RotationNode{
    fn get_node_name(&self)->String {
        "Rotate".to_string()
    }
    fn clear_buffers(&mut self) {
        self.buffered = false;
    }

    fn get_inputs(&self)->Vec<NodeInputOptions> {
        vec![NodeInputOptions{name:"bitmap".to_string(),IOType: NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true, presetValues:None},
            NodeInputOptions{name: "angle".to_string(),IOType: NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true, presetValues:None}]
    }

    fn get_outputs(&self)->Vec<NodeOutputOptions> {
        vec![NodeOutputOptions{name: "rotated".to_string(), IOType: NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true}]
    }


}