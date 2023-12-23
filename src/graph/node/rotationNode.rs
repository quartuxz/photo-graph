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

impl NodeStatic for RotationNode{
    fn get_inputs_static()->Vec<NodeInputOptions>{
        vec![NodeInputOptions{name:"bitmap".to_string(),IOType: NodeIOType::BitmapType(RgbaImage::default()),canAlterDefault:false,hasConnection:true, presetValues:None,subtype:None},
            NodeInputOptions{name: "angle".to_string(),IOType: NodeIOType::FloatType(0.0),canAlterDefault:true,hasConnection:true, presetValues:None,subtype:None}]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name: "rotated".to_string(), IOType: NodeIOType::BitmapType(RgbaImage::default()), hasConnection:true}]
    }

    fn get_node_name_static()->String {
        "Rotate".to_string()
    }
}


impl Node for RotationNode{

    fn clear_buffers(&mut self) {
        self.buffered = false;
    }


}