use crate::ir::{ArgType, AttributeValue, ElementType, Node, TensorType};
use crate::protos::tensor_proto::DataType;
use protobuf::Enum;

/// Update output type for Cast operations, preserving rank.
pub fn cast_update_outputs(node: &mut Node) {
    if node.inputs.len() != 1 {
        panic!("Cast: multiple inputs are not supported");
    }
    let input = &mut node.inputs[0];
    let output = &mut node.outputs[0];

    let elem_type = match node.attrs.get("to") {
        Some(value) => match &value {
            AttributeValue::Int64(type_id) => match DataType::from_i32(*type_id as i32).unwrap() {
                DataType::FLOAT => ElementType::Float32,
                DataType::FLOAT16 => ElementType::Float16,
                DataType::INT32 => ElementType::Int32,
                DataType::INT64 => ElementType::Int64,
                DataType::DOUBLE => ElementType::Float64,
                DataType::BOOL => ElementType::Bool,
                DataType::STRING => ElementType::String,
                data_type => panic!("Cast: unsupported type {data_type:?}"),
            },
            _ => panic!("'to' attribute must be an Int64"),
        },
        None => panic!("Cast node must have a 'to' attribute"),
    };

    match input.ty.clone() {
        ArgType::Tensor(tensor) => {
            if tensor.rank == 0 {
                // treat 0-dim tensor as scalar
                output.ty = ArgType::Scalar(elem_type);
                input.ty = ArgType::Scalar(tensor.elem_type);
            } else {
                // Cast input and output are the same shape, but possibly different types
                output.ty = ArgType::Tensor(TensorType {
                    elem_type,
                    rank: tensor.rank,
                    static_shape: None,
                });
            }
        }
        ArgType::Scalar(_) => output.ty = ArgType::Scalar(elem_type),
        _ => panic!("Cast: only scalar and tensor inputs are valid"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Argument, NodeType, TensorType};
    use crate::node::test_utils::NodeBuilder;

    fn create_test_node(input_rank: usize, to_type: i64) -> Node {
        NodeBuilder::new(NodeType::Cast, "test_cast")
            .input_tensor_f32("X", input_rank, None)
            .output_tensor_f32("Y", input_rank, None) // Element type will be overwritten
            .attr_int("to", to_type)
            .build()
    }

    // Additional test function to demonstrate scalar inputs
    fn create_scalar_test_node(to_type: i64) -> Node {
        NodeBuilder::new(NodeType::Cast, "test_cast")
            .input_scalar_f32("X")
            .output_scalar_f32("Y") // Element type will be overwritten
            .attr_int("to", to_type)
            .build()
    }

    #[test]
    fn test_cast_float_to_int64() {
        let mut node = create_test_node(2, DataType::INT64.value() as i64);
        cast_update_outputs(&mut node);

        match &node.outputs[0].ty {
            ArgType::Tensor(tensor) => {
                assert_eq!(tensor.elem_type, ElementType::Int64);
                assert_eq!(tensor.rank, 2);
            }
            _ => panic!("Expected tensor output"),
        }
    }

    #[test]
    fn test_cast_scalar_handling() {
        let mut node = create_test_node(0, DataType::BOOL.value() as i64);
        cast_update_outputs(&mut node);

        match &node.outputs[0].ty {
            ArgType::Scalar(elem_type) => {
                assert_eq!(*elem_type, ElementType::Bool);
            }
            _ => panic!("Expected scalar output for 0-rank tensor"),
        }

        match &node.inputs[0].ty {
            ArgType::Scalar(elem_type) => {
                assert_eq!(*elem_type, ElementType::Float32);
            }
            _ => panic!("Input should have been converted to scalar"),
        }
    }

    #[test]
    #[should_panic(expected = "Cast: multiple inputs are not supported")]
    fn test_cast_multiple_inputs() {
        let mut node = create_test_node(2, DataType::INT64.value() as i64);
        node.inputs.push(Argument {
            name: "extra".to_string(),
            ty: ArgType::Tensor(TensorType {
                elem_type: ElementType::Float32,
                rank: 1,
                static_shape: None,
            }),
            value: None,
            passed: true,
        });
        cast_update_outputs(&mut node);
    }

    #[test]
    fn test_cast_scalar_to_bool() {
        let mut node = create_scalar_test_node(DataType::BOOL.value() as i64);
        cast_update_outputs(&mut node);

        match &node.outputs[0].ty {
            ArgType::Scalar(elem_type) => {
                assert_eq!(*elem_type, ElementType::Bool);
            }
            _ => panic!("Expected scalar output"),
        }
    }
}
