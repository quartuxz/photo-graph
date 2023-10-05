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

impl Node for MathNode{
    fn get_node_name(&self)->String {
        "Math".to_string()
    }

    fn get_inputs(&self)->Vec<NodeInputOptions> {
        let mut presetValues = HashMap::new();
        presetValues.insert("add".to_string(), 0);
        presetValues.insert("subtract".to_string(), 1);
        presetValues.insert("multiply".to_string(), 2);
        presetValues.insert("divide".to_string(), 3);
        vec![NodeInputOptions{name:"operation".to_string(),IOType: NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues)},
            NodeInputOptions{name:"x".to_string(),IOType:NodeIOType::FloatType(f64::default()),canAlterDefault:true,hasConnection:true,presetValues:None},
            NodeInputOptions{name:"y".to_string(),IOType:NodeIOType::FloatType(f64::default()),canAlterDefault:true,hasConnection:true,presetValues:None},]
            
    }

    fn get_outputs(&self)->Vec<NodeOutputOptions> {
        vec![NodeOutputOptions{name:"result".to_string(),IOType:NodeIOType::FloatType(f64::default()),hasConnection:true}]
    }

    fn clear_buffers(&mut self) {
        self.buffered = false;
    }
}