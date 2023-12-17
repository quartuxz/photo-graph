use super::*;

pub struct MathNode{
    operation:i64,
    x:f64,
    y:f64,
    buffer:f64,
    buffered:bool
}


impl MathNode{
    pub fn new()->Self{
        MathNode { operation:0,x: 0.0, y: 0.0, buffer: 0.0, buffered: false }
    }


    
}

impl NodeStatic for MathNode{
    
    fn get_inputs_static()->Vec<NodeInputOptions>{
        let mut presetValues = vec![];
        presetValues.push("add".to_string());
        presetValues.push("subtract".to_string());
        presetValues.push("multiply".to_string());
        presetValues.push("divide".to_string());
        vec![NodeInputOptions{name:"operation".to_string(),IOType: NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues)},
            NodeInputOptions{name:"x".to_string(),IOType:NodeIOType::FloatType(f64::default()),canAlterDefault:true,hasConnection:true,presetValues:None},
            NodeInputOptions{name:"y".to_string(),IOType:NodeIOType::FloatType(f64::default()),canAlterDefault:true,hasConnection:true,presetValues:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"result".to_string(),IOType:NodeIOType::FloatType(f64::default()),hasConnection:true}]
    }

    fn get_node_name_static()->String {
        "Math".to_string()
    }
}

impl Node for MathNode{
    fn clear_buffers(&mut self) {
        self.buffered = false;
    }
}