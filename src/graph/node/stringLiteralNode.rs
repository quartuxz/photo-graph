use super::*;

pub struct StringLiteralNode{
    stringLiteral : String
}


impl StringLiteralNode{
    pub fn new(stringLiteral : String)->Self{
        StringLiteralNode { stringLiteral }
    }



}

impl NodeStatic for StringLiteralNode{
    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::StringType(String::default()), hasConnection:true, name:"".to_string()}]
    }
    fn get_node_name_static()->String {
        "String literal".to_string()
    }
}

impl Node for StringLiteralNode{

    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index);

        NodeResult::Ok(NodeIOType::StringType(self.stringLiteral.clone()))
    }
}