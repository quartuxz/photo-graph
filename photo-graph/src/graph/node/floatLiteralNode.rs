use super::*;

pub struct FloatLiteralNode{
    floatLiteral : f64
}

impl FloatLiteralNode{
    pub fn new(floatLiteral:f64)->Self{
        FloatLiteralNode { floatLiteral }
    }



}


impl NodeStatic for FloatLiteralNode{
    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::FloatType(f64::default()), hasConnection:true, name:"".to_string(),subtype:None}]
    }

    fn get_node_name_static()->String where Self:Sized {
        "Float literal".to_string()
    }
}

impl Node for FloatLiteralNode{


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::FloatType(self.floatLiteral));
    }
}