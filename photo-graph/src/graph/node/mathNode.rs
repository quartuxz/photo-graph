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
        vec![NodeInputOptions{name:"operation".to_string(),IOType: NodeIOType::IntType(0),canAlterDefault:true,hasConnection:false,presetValues:Some(presetValues),subtype:None},
            NodeInputOptions{name:"x".to_string(),IOType:NodeIOType::FloatType(f64::default()),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},
            NodeInputOptions{name:"y".to_string(),IOType:NodeIOType::FloatType(f64::default()),canAlterDefault:true,hasConnection:true,presetValues:None,subtype:None},]
    }

    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{name:"result".to_string(),IOType:NodeIOType::FloatType(f64::default()),hasConnection:true,subtype:None}]
    }

    fn get_node_name_static()->String {
        "Math".to_string()
    }
}

impl Node for MathNode{
    fn clear_buffers(&mut self) {
        *self = MathNode::new();
    }
    fn set(&mut self, index: u16, value: NodeIOType) -> NodeResult<()> {
        self.generate_input_errors(&index, &value)?;
        match index {
            0 => if let NodeIOType::IntType(operation) = value{
                self.operation = operation;
            }
            1 => if let NodeIOType::FloatType(float) = value{
                self.x = float;
            }
            2 => if let NodeIOType::FloatType(float) = value{
                self.y = float;
            }
            _ => ()
        }


        NodeResult::Ok(())
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;
        if !self.buffered {
            match self.operation{
                0 => self.buffer = self.x+self.y,
                1 => self.buffer = self.x-self.y,
                2 => self.buffer = self.x*self.y,
                3 => self.buffer = self.x/self.y,
                _=> ()
            }
            self.buffered =true;
        }


        NodeResult::Ok(NodeIOType::FloatType(self.buffer))
    }
}