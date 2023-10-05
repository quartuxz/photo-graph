use super::*;

pub struct StringLiteralNode{
    stringLiteral : String
}


impl StringLiteralNode{
    pub fn new(stringLiteral : String)->Self{
        StringLiteralNode { stringLiteral }
    }
}

impl Node for StringLiteralNode{
    fn get_node_name(&self)->String {
        "string literal".to_string()
    }

    fn get_outputs(&self)->Vec<NodeOutputOptions> {
        vec![NodeOutputOptions{IOType:NodeIOType::StringType(String::default()), hasConnection:true, name:"".to_string()}]
    }

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index);

        NodeResult::Ok(NodeIOType::StringType(self.stringLiteral.clone()))
    }
}