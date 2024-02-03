use super::*;

pub struct IntLiteralNode{
    intLiteral : i64
}

impl IntLiteralNode{
    pub fn new(intLiteral:i64)->Self{
        IntLiteralNode { intLiteral }
    }



}

impl NodeStatic for IntLiteralNode{
    fn get_outputs_static()->Vec<NodeOutputOptions>{
        vec![NodeOutputOptions{IOType:NodeIOType::IntType(i64::default()), hasConnection:true, name:"".to_string(),subtype:None}]
    }
    fn get_node_name_static()->String {
        "Integer literal".to_string()
    }
}

impl Node for IntLiteralNode{


    fn get(&mut self, index: u16) -> NodeResult<NodeIOType> {
        self.generate_output_errors(&index)?;

        return NodeResult::Ok(NodeIOType::IntType(self.intLiteral));
    }
}