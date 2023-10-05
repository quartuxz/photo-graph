use super::*;

pub struct FloatLiteralNode{
    floatLiteral : f64
}

impl FloatLiteralNode{
    pub fn new(floatLiteral:f64)->Self{
        FloatLiteralNode { floatLiteral }
    }
}


impl Node for FloatLiteralNode{
    fn get_node_name(&self)->String {
        "float literal".to_string()
    }

    fn get_outputs(&self)->Vec<NodeOutputOptions> {
        vec![NodeOutputOptions{IOType:NodeIOType::FloatType(f64::default()), hasConnection:true, name:"".to_string()}]
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::FloatType(self.floatLiteral));
    }
}