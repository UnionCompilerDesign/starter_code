use std::sync::{Arc, Mutex};

use common::ast::{ast_struct::{ASTNode, AST}, data_type::DataType, syntax_element::SyntaxElement};
use integration::module::{ast_stitch, ModElement, Module};
use ir::core::IRGenerator;

 
use safe_llvm::{memory_management::resource_pools::ResourcePools, utils::utils_struct::Utils};
use symbol_table::symbol_table_struct::{SymbolInfo, SymbolTable, SymbolTableStack, SymbolValue};

pub const DEFAULT_PRIORITY_MODELEMENT: i32 = -1;

fn wrap_in_tle(ast_node: ASTNode) -> AST {
    let mut tle: ASTNode = ASTNode::new(SyntaxElement::TopLevelExpression);
    tle.add_child(ast_node);
    AST::new(tle)
}

#[test]
fn test_function_declaration() {
    /* `
    int testFunction() {}

    ; ModuleID = 'dummy_module'
    source_filename = "dummy_module"

    define i64 @testFunction() {
    entryID0:
    }
    */ 
    
    let mut function_ast = ASTNode::new(SyntaxElement::FunctionDeclaration);

    let fn_id = ASTNode::new(SyntaxElement::Identifier("testFunction".to_string()));
    let fn_type = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let fn_block_exp = ASTNode::new(SyntaxElement::BlockExpression);

    function_ast.add_child(fn_id);
    function_ast.add_child(fn_type);
    function_ast.add_child(fn_block_exp);

    let ast: AST = wrap_in_tle(function_ast);

    let mut symbol_table_stack = SymbolTableStack::new();
    let mut symbol_table_global = SymbolTable::new();
    let fn_value = SymbolValue::FunctionValue{
        parameters: Vec::new(),
    };
    let fn_info = SymbolInfo::new(DataType::Integer, fn_value);
    symbol_table_global.add("testFunction".to_string(), fn_info);
    symbol_table_stack.push(symbol_table_global);
    symbol_table_stack.push(SymbolTable::new());

    let mod_ast: Module = ast_stitch(vec![ModElement::new(ast, symbol_table_stack, DEFAULT_PRIORITY_MODELEMENT)]);

    let mut ir_generator = IRGenerator::new();
    let module_tag = ir_generator.generate_ir(mod_ast);  

    let pools: Arc<Mutex<ResourcePools>> = ir_generator.get_resource_pools();
    let pools = pools.try_lock().expect("Failed to lock resource pool mutex in do while IR!");

    let module = pools.get_module(module_tag).expect("Failed to get module");
    let write_result = Utils::write_to_file(module.clone(), "output_fn_declaration.ll");
    match write_result {
        Ok(_) => eprintln!("FN test file written correctly!"),
        Err(_) => panic!("FN test file failed to write!")
    }
    let get_str = Utils::write_to_string(module);
    let test_str = match get_str {
        Ok(str) => str,
        Err(e) => panic!("{}", e)
    };
    let expected_str = "; ModuleID = 'dummy_module'\nsource_filename = \"dummy_module\"\n\ndefine i64 @testFunction() {\nentryID0:\n}\n";

    assert_eq!(test_str, expected_str)
}

#[test]
fn test_function_with_if_else() {
    /* 
    int testFunction() {
        if (true) {
            return 1;
        } else {
            return 1;
        }
    }
    
    ; ModuleID = 'dummy_module'
    source_filename = "dummy_module"

    define i64 @testFunction() {
    entryID0:
      br i1 true, label %thenID1, label %elseID1

    thenID1:                                          ; preds = %entryID0
      ret i64 1
      br label %mergeID1

    elseID1:                                          ; preds = %entryID0
      ret i64 1
      br label %mergeID1

    mergeID1:                                         ; preds = %elseID1, %thenID1
    }

    */ 

    let mut if_statement = ASTNode::new(SyntaxElement::IfStatement);

    let mut if_condition = ASTNode::new(SyntaxElement::Condition);

    let if_value = ASTNode::new(SyntaxElement::Literal("true".to_string()));
    if_condition.add_child(if_value);

    let mut then_branch = ASTNode::new(SyntaxElement::BlockExpression);
    let mut return_statement = ASTNode::new(SyntaxElement::Return);
    let mut assigned_value = ASTNode::new(SyntaxElement::AssignedValue);
    let then_ret_value = ASTNode::new(SyntaxElement::Literal("1".to_string()));

    assigned_value.add_child(then_ret_value);

    return_statement.add_child(assigned_value);

    then_branch.add_child(return_statement);

    if_statement.add_child(if_condition);
    if_statement.add_child(then_branch);

    let mut else_branch = ASTNode::new(SyntaxElement::ElseStatement);
    let mut else_block = ASTNode::new(SyntaxElement::BlockExpression);

    let mut return_statement_else = ASTNode::new(SyntaxElement::Return);
    let mut assigned_value = ASTNode::new(SyntaxElement::AssignedValue);
    let return_value = ASTNode::new(SyntaxElement::Literal("1".to_string()));
    assigned_value.add_child(return_value);

    return_statement_else.add_child(assigned_value);

    else_block.add_child(return_statement_else);

    else_branch.add_child(else_block);

    if_statement.add_child(else_branch);

    let mut fn_block = ASTNode::new(SyntaxElement::BlockExpression);

    fn_block.add_child(if_statement);

    let fn_type = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let fn_id = ASTNode::new(SyntaxElement::Identifier("testFunction".to_string()));

    let mut fn_declaration_node = ASTNode::new(SyntaxElement::FunctionDeclaration);
    
    fn_declaration_node.add_child(fn_id);
    fn_declaration_node.add_child(fn_type);
    fn_declaration_node.add_child(fn_block);

    let ast: AST = wrap_in_tle(fn_declaration_node);

    let mut symbol_table_stack = SymbolTableStack::new();
    let mut symbol_table_global = SymbolTable::new();
    let fn_value = SymbolValue::FunctionValue{
        parameters: Vec::new(),
    };
    let fn_info = SymbolInfo::new(DataType::Integer, fn_value);
    symbol_table_global.add("testFunction".to_string(), fn_info);
    symbol_table_stack.push(symbol_table_global);
    symbol_table_stack.push(SymbolTable::new());


    let mod_ast: Module = ast_stitch(vec![ModElement::new(ast, symbol_table_stack, DEFAULT_PRIORITY_MODELEMENT)]);

    let mut ir_generator = IRGenerator::new();

    let module_tag = ir_generator.generate_ir(mod_ast);  

    let pools: Arc<Mutex<ResourcePools>> = ir_generator.get_resource_pools();
    let pools = pools.try_lock().expect("Failed to lock resource pool mutex in do while IR!");

    let module = pools.get_module(module_tag).expect("Failed to get module");
    let write_result = Utils::write_to_file(module.clone(), "output_if_else.ll");
    match write_result {
        Ok(_) => eprintln!("If else test file written correctly!"),
        Err(_) => panic!("If else test file failed to write!")
    }
    let get_str = Utils::write_to_string(module);
    let test_str = match get_str {
        Ok(str) => str,
        Err(e) => panic!("{}", e)
    };
    let expected_str = "; ModuleID = 'dummy_module'\nsource_filename = \"dummy_module\"\n\ndefine i64 @testFunction() {\nentryID0:\n  br i1 true, label %thenID1, label %elseID1\n\nthenID1:                                          ; preds = %entryID0\n  ret i64 1\n  br label %mergeID1\n\nelseID1:                                          ; preds = %entryID0\n  ret i64 1\n  br label %mergeID1\n\nmergeID1:                                         ; preds = %elseID1, %thenID1\n}\n";

    assert_eq!(test_str, expected_str)
}

#[test]
fn test_function_with_while_loop() {
    /*
    int testFunctionWithWhileLoop() {
        while (true) {
            return 42;
        }
    }

    ; ModuleID = 'dummy_module'
    source_filename = "dummy_module"

    define i64 @testFunctionWithWhileLoop() {
    entryID0:
      br label %while_condID1

    while_condID1:                                    ; preds = %while_bodyID1, %entryID0
      br i1 true, label %while_bodyID1, label %while_endID1

    while_bodyID1:                                    ; preds = %while_condID1
      ret i64 42
      br label %while_condID1

    while_endID1:                                     ; preds = %while_condID1
    }

    */

    let mut while_condition = ASTNode::new(SyntaxElement::Condition);
    let while_condition_value = ASTNode::new(SyntaxElement::Literal("true".to_string()));
    while_condition.add_child(while_condition_value);

    let mut while_body = ASTNode::new(SyntaxElement::BlockExpression);
    
    let mut return_statement = ASTNode::new(SyntaxElement::Return);
    let mut assigned_value = ASTNode::new(SyntaxElement::AssignedValue);
    let return_value = ASTNode::new(SyntaxElement::Literal("42".to_string()));
    assigned_value.add_child(return_value);

    return_statement.add_child(assigned_value);
    while_body.add_child(return_statement);

    let mut while_statement = ASTNode::new(SyntaxElement::WhileLoop);
    while_statement.add_child(while_condition);
    while_statement.add_child(while_body);

    let mut fn_block = ASTNode::new(SyntaxElement::BlockExpression);
    fn_block.add_child(while_statement);

    let fn_type = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let fn_id = ASTNode::new(SyntaxElement::Identifier("testFunctionWithWhileLoop".to_string()));

    let mut fn_declaration_node = ASTNode::new(SyntaxElement::FunctionDeclaration);
    fn_declaration_node.add_child(fn_id);
    fn_declaration_node.add_child(fn_type);
    fn_declaration_node.add_child(fn_block);

    let ast = wrap_in_tle(fn_declaration_node);

    let mut symbol_table_stack = SymbolTableStack::new();
    let mut symbol_table_global = SymbolTable::new();
    let fn_value = SymbolValue::FunctionValue{
        parameters: Vec::new(),
    };
    let fn_info = SymbolInfo::new(DataType::Integer, fn_value);
    symbol_table_global.add("testFunctionWithWhileLoop".to_string(), fn_info);
    symbol_table_stack.push(symbol_table_global);
    symbol_table_stack.push(SymbolTable::new());

    let mod_ast: Module = ast_stitch(vec![ModElement::new(ast, symbol_table_stack, DEFAULT_PRIORITY_MODELEMENT)]);

    let mut ir_generator = IRGenerator::new();
    let module_tag = ir_generator.generate_ir(mod_ast);  

    let pools: Arc<Mutex<ResourcePools>> = ir_generator.get_resource_pools();
    let pools = pools.try_lock().expect("Failed to lock resource pool mutex in do while IR!");

    let module = pools.get_module(module_tag).expect("Failed to get module");
    let write_result = Utils::write_to_file(module.clone(), "output_while_loop.ll");
    match write_result {
        Ok(_) => eprintln!("While test file written correctly!"),
        Err(_) => panic!("While test file failed to write!")
    }
    let get_str = Utils::write_to_string(module);
    let test_str = match get_str {
        Ok(str) => str,
        Err(e) => panic!("{}", e)
    };
    let expected_str = "; ModuleID = 'dummy_module'\nsource_filename = \"dummy_module\"\n\ndefine i64 @testFunctionWithWhileLoop() {\nentryID0:\n  br label %while_condID1\n\nwhile_condID1:                                    ; preds = %while_bodyID1, %entryID0\n  br i1 true, label %while_bodyID1, label %while_endID1\n\nwhile_bodyID1:                                    ; preds = %while_condID1\n  ret i64 42\n  br label %while_condID1\n\nwhile_endID1:                                     ; preds = %while_condID1\n}\n";

    assert_eq!(test_str, expected_str)
}


#[test]
fn test_function_with_do_while_loop() {
    /*
    int testFunctionWithDoWhileLoop() {
        do {
            return 24;
        } while (true);
    }

    ; ModuleID = 'dummy_module'
    source_filename = "dummy_module"

    define i64 @testFunctionWithDoWhileLoop() {
    entryID0:
      br label %do_bodyID1

    do_bodyID1:                                       ; preds = %do_condID1, %entryID0
      ret i64 24
      br label %do_condID1

    do_condID1:                                       ; preds = %do_bodyID1
      br i1 true, label %do_bodyID1, label %do_endID1

    do_endID1:                                        ; preds = %do_condID1
    }

    */

    let mut do_while_condition = ASTNode::new(SyntaxElement::Condition);
    let do_while_condition_value = ASTNode::new(SyntaxElement::Literal("true".to_string()));
    do_while_condition.add_child(do_while_condition_value);

    let mut return_statement = ASTNode::new(SyntaxElement::Return);
    let mut assigned_value = ASTNode::new(SyntaxElement::AssignedValue);
    let return_value = ASTNode::new(SyntaxElement::Literal("24".to_string()));
    assigned_value.add_child(return_value);
    return_statement.add_child(assigned_value);

    let mut do_while_body = ASTNode::new(SyntaxElement::BlockExpression);
    do_while_body.add_child(return_statement);

    let mut do_while_statement = ASTNode::new(SyntaxElement::DoWhileLoop);
    do_while_statement.add_child(do_while_body);
    do_while_statement.add_child(do_while_condition);


    let fn_type = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let mut fn_block = ASTNode::new(SyntaxElement::BlockExpression);
    fn_block.add_child(do_while_statement);

    let mut function_declaration_node = ASTNode::new(SyntaxElement::FunctionDeclaration);
    let fn_id = ASTNode::new(SyntaxElement::Identifier("testFunctionWithDoWhileLoop".to_string()));
    function_declaration_node.add_child(fn_id);
    function_declaration_node.add_child(fn_type);
    function_declaration_node.add_child(fn_block);

    let ast = wrap_in_tle(function_declaration_node);

    let mut symbol_table_stack = SymbolTableStack::new();
    let mut symbol_table_global = SymbolTable::new();
    let fn_value = SymbolValue::FunctionValue{
        parameters: Vec::new(),
    };
    let fn_info = SymbolInfo::new(DataType::Integer, fn_value);
    symbol_table_global.add("testFunctionWithDoWhileLoop".to_string(), fn_info);
    symbol_table_stack.push(symbol_table_global);
    symbol_table_stack.push(SymbolTable::new());

    let mod_ast: Module = ast_stitch(vec![ModElement::new(ast, symbol_table_stack, DEFAULT_PRIORITY_MODELEMENT)]);

    let mut ir_generator = IRGenerator::new();
    let module_tag = ir_generator.generate_ir(mod_ast);  

    let pools: Arc<Mutex<ResourcePools>> = ir_generator.get_resource_pools();
    let pools = pools.try_lock().expect("Failed to lock resource pool mutex in do while IR!");

    let module = pools.get_module(module_tag).expect("Failed to get module");
    let write_result = Utils::write_to_file(module.clone(), "output_do_while.ll");
    match write_result {
        Ok(_) => eprintln!("Do while test file written correctly!"),
        Err(_) => panic!("Do while test file failed to write!")
    }
    let get_str = Utils::write_to_string(module);
    let test_str = match get_str {
        Ok(str) => str,
        Err(e) => panic!("{}", e)
    };
    let expected_str = "; ModuleID = 'dummy_module'\nsource_filename = \"dummy_module\"\n\ndefine i64 @testFunctionWithDoWhileLoop() {\nentryID0:\n  br label %do_bodyID1\n\ndo_bodyID1:                                       ; preds = %do_condID1, %entryID0\n  ret i64 24\n  br label %do_condID1\n\ndo_condID1:                                       ; preds = %do_bodyID1\n  br i1 true, label %do_bodyID1, label %do_endID1\n\ndo_endID1:                                        ; preds = %do_condID1\n}\n";

    assert_eq!(test_str, expected_str)
}

#[test]
fn test_function_with_assign() {
    /*
    int testFunctionWithAssign() {
        int test_var = 0;
        int test_var_2;
    }

    ; ModuleID = 'dummy_module'
    source_filename = "dummy_module"

    define i64 @testFunctionWithAssign() {
    entryID0:
      %test_var = alloca i64, align 8
      store i64 0, ptr %test_var, align 4
      %test_var_2 = alloca i64, align 8
    }

    */

    let mut assignment_node = ASTNode::new(SyntaxElement::Initialization);

    let id_node = ASTNode::new(SyntaxElement::Identifier("test_var".to_string()));
    let type_node = ASTNode::new(SyntaxElement::Type(DataType::Integer));

    let mut assignment_node_2 = ASTNode::new(SyntaxElement::Initialization);
    let id_node_2 = ASTNode::new(SyntaxElement::Identifier("test_var_2".to_string()));
    let type_node_2 = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let mut var_node_2 = ASTNode::new(SyntaxElement::Variable);
    var_node_2.add_child(id_node_2);
    var_node_2.add_child(type_node_2);
    assignment_node_2.add_child(var_node_2);

    let mut var_node = ASTNode::new(SyntaxElement::Variable);
    var_node.add_child(id_node);
    var_node.add_child(type_node);

    let mut value_node = ASTNode::new(SyntaxElement::AssignedValue);

    let num_node = ASTNode::new(SyntaxElement::Literal("0".to_string()));
    value_node.add_child(num_node);

    assignment_node.add_child(var_node);
    assignment_node.add_child(value_node);


    let mut fn_block = ASTNode::new(SyntaxElement::BlockExpression);
    fn_block.add_child(assignment_node);
    fn_block.add_child(assignment_node_2);

    let fn_type = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let fn_id = ASTNode::new(SyntaxElement::Identifier("testFunctionWithAssign".to_string()));

    let mut fn_declaration_node = ASTNode::new(SyntaxElement::FunctionDeclaration);
    fn_declaration_node.add_child(fn_id);
    fn_declaration_node.add_child(fn_type);
    fn_declaration_node.add_child(fn_block);

    let ast = wrap_in_tle(fn_declaration_node);

    let mut symbol_table_stack = SymbolTableStack::new();
    let mut symbol_table_global = SymbolTable::new();
    let fn_value = SymbolValue::FunctionValue{
        parameters: Vec::new(),
    };
    let fn_info = SymbolInfo::new(DataType::Integer, fn_value);
    symbol_table_global.add("testFunctionWithAssign".to_string(), fn_info);
    symbol_table_stack.push(symbol_table_global);
    symbol_table_stack.push(SymbolTable::new());

    let mod_ast: Module = ast_stitch(vec![ModElement::new(ast, symbol_table_stack, DEFAULT_PRIORITY_MODELEMENT)]);

    let mut ir_generator = IRGenerator::new();
    let module_tag = ir_generator.generate_ir(mod_ast);  

    let pools = ir_generator.get_resource_pools();

    let module = pools.lock().expect("coouldn't unlock pools mutex").get_module(module_tag).expect("No module found!");
    let write_result = Utils::write_to_file(module.clone(), "output_assign.ll");
    match write_result {
        Ok(_) => eprintln!("Assign test file written correctly!"),
        Err(_) => panic!("Assign test file failed to write!")
    }
    let get_str = Utils::write_to_string(module);
    let test_str = match get_str {
        Ok(str) => str,
        Err(e) => panic!("{}", e)
    };
    let expected_str = "; ModuleID = 'dummy_module'\nsource_filename = \"dummy_module\"\n\ndefine i64 @testFunctionWithAssign() {\nentryID0:\n  %test_var = alloca i64, align 8\n  store i64 0, ptr %test_var, align 4\n  %test_var_2 = alloca i64, align 8\n}\n";

    assert_eq!(test_str, expected_str)

}

#[test]
fn test_function_with_reassign() {
    /*
    int testFunctionWithReassign() {
        int test_var = 0;
        test_var = 42;
    }

    ; ModuleID = 'dummy_module'
    source_filename = "dummy_module"

    define i64 @testFunctionWithReassign() {
    entryID0:
      %test_var = alloca i64, align 8
      store i64 0, ptr %test_var, align 4
      store i64 42, ptr %test_var, align 4
}

    */

    let mut assignment_node = ASTNode::new(SyntaxElement::Initialization);

    let id_node = ASTNode::new(SyntaxElement::Identifier("test_var".to_string()));
    let type_node = ASTNode::new(SyntaxElement::Type(DataType::Integer));


    let mut var_node = ASTNode::new(SyntaxElement::Variable);
    var_node.add_child(id_node);
    var_node.add_child(type_node);

    let mut value_node = ASTNode::new(SyntaxElement::AssignedValue);

    let num_node = ASTNode::new(SyntaxElement::Literal("0".to_string()));
    value_node.add_child(num_node);

    assignment_node.add_child(var_node.clone());
    assignment_node.add_child(value_node);

    let mut reassignment_node = ASTNode::new(SyntaxElement::Assignment);
    reassignment_node.add_child(var_node);
    let mut value_node = ASTNode::new(SyntaxElement::AssignedValue);
    value_node.add_child(ASTNode::new(SyntaxElement::Literal("42".to_string())));
    reassignment_node.add_child(value_node);


    let mut fn_block = ASTNode::new(SyntaxElement::BlockExpression);
    fn_block.add_child(assignment_node);
    fn_block.add_child(reassignment_node);

    let fn_type = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let fn_id = ASTNode::new(SyntaxElement::Identifier("testFunctionWithReassign".to_string()));

    let mut fn_declaration_node = ASTNode::new(SyntaxElement::FunctionDeclaration);
    fn_declaration_node.add_child(fn_id);
    fn_declaration_node.add_child(fn_type);
    fn_declaration_node.add_child(fn_block);

    let ast = wrap_in_tle(fn_declaration_node);

    let mut symbol_table_stack = SymbolTableStack::new();
    let mut symbol_table_global = SymbolTable::new();
    let fn_value = SymbolValue::FunctionValue{
        parameters: Vec::new(),
    };
    let fn_info = SymbolInfo::new(DataType::Integer, fn_value);
    symbol_table_global.add("testFunctionWithReassign".to_string(), fn_info);
    symbol_table_stack.push(symbol_table_global);
    symbol_table_stack.push(SymbolTable::new());

    let mod_ast: Module = ast_stitch(vec![ModElement::new(ast, symbol_table_stack, DEFAULT_PRIORITY_MODELEMENT)]);

    let mut ir_generator = IRGenerator::new();
    let module_tag = ir_generator.generate_ir(mod_ast);  

    let pools = ir_generator.get_resource_pools();

    let module = pools.lock().expect("coouldn't unlock pools mutex").get_module(module_tag).expect("No module found!");
    let write_result = Utils::write_to_file(module.clone(), "output_reassign.ll");
    match write_result {
        Ok(_) => eprintln!("Reassign test file written correctly!"),
        Err(_) => panic!("Reassign test file failed to write!")
    }
    let get_str = Utils::write_to_string(module);
    let test_str = match get_str {
        Ok(str) => str,
        Err(e) => panic!("{}", e)
    };
    let expected_str = "; ModuleID = 'dummy_module'\nsource_filename = \"dummy_module\"\n\ndefine i64 @testFunctionWithReassign() {\nentryID0:\n  %test_var = alloca i64, align 8\n  store i64 0, ptr %test_var, align 4\n  store i64 42, ptr %test_var, align 4\n}\n";

    assert_eq!(test_str, expected_str)



}

#[test]
fn test_function_with_for_loop() {
    /*
    int testForLoop() {
        for (int test_var = 0; 1; test_var = 42) {
            break;
        }
    }

    ; ModuleID = 'dummy_module'
    source_filename = "dummy_module"

    define i64 @testForLoop() {
    entryID0:
      %test_var = alloca i64, align 8
      store i64 0, ptr %test_var, align 4
      br label %for_condID1

    for_condID1:                                      ; preds = %for_incID1, %for_bodyID1
      br i1 true, label %for_bodyID1, label %for_endID1

    for_bodyID1:                                      ; preds = %for_condID1
      br label %for_condID1
      br label %for_incID1

    for_incID1:                                       ; preds = %for_bodyID1
      store i64 42, ptr %test_var, align 4
      br label %for_condID1

    for_endID1:                                       ; preds = %for_condID1
    }

    */

    let mut assignment_node = ASTNode::new(SyntaxElement::Initialization);

    let id_node = ASTNode::new(SyntaxElement::Identifier("test_var".to_string()));
    let type_node = ASTNode::new(SyntaxElement::Type(DataType::Integer));

    let mut assignment_node_2 = ASTNode::new(SyntaxElement::Initialization);
    let id_node_2 = ASTNode::new(SyntaxElement::Identifier("test_var_2".to_string()));
    let type_node_2 = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let mut var_node_2 = ASTNode::new(SyntaxElement::Variable);
    var_node_2.add_child(id_node_2);
    var_node_2.add_child(type_node_2);
    assignment_node_2.add_child(var_node_2);

    let mut var_node = ASTNode::new(SyntaxElement::Variable);
    var_node.add_child(id_node);
    var_node.add_child(type_node);

    let mut value_node = ASTNode::new(SyntaxElement::AssignedValue);

    let num_node = ASTNode::new(SyntaxElement::Literal("0".to_string()));
    value_node.add_child(num_node);

    assignment_node.add_child(var_node.clone());
    assignment_node.add_child(value_node);

    let mut loop_node = ASTNode::new(SyntaxElement::ForLoop);

    let mut init_node = ASTNode::new(SyntaxElement::LoopInitializer);
    init_node.add_child(assignment_node);

    loop_node.add_child(init_node);

    let mut for_condition = ASTNode::new(SyntaxElement::Condition);
    let for_condition_value = ASTNode::new(SyntaxElement::Literal("true".to_string()));
    for_condition.add_child(for_condition_value);

    loop_node.add_child(for_condition);

    let mut for_inc = ASTNode::new(SyntaxElement::LoopIncrement);

    let mut reassignment_node = ASTNode::new(SyntaxElement::Assignment);
    reassignment_node.add_child(var_node);
    let mut value_node = ASTNode::new(SyntaxElement::AssignedValue);
    value_node.add_child(ASTNode::new(SyntaxElement::Literal("42".to_string())));
    reassignment_node.add_child(value_node);

    for_inc.add_child(reassignment_node);
    loop_node.add_child(for_inc);

    let mut for_body = ASTNode::new(SyntaxElement::BlockExpression);
    for_body.add_child(ASTNode::new(SyntaxElement::Continue));

    loop_node.add_child(for_body);

    let mut fn_block = ASTNode::new(SyntaxElement::BlockExpression);
    fn_block.add_child(loop_node);
    //fn_block.add_child(assignment_node_2);

    let fn_type = ASTNode::new(SyntaxElement::Type(DataType::Integer));
    let fn_id = ASTNode::new(SyntaxElement::Identifier("testForLoop".to_string()));

    let mut fn_declaration_node = ASTNode::new(SyntaxElement::FunctionDeclaration);
    fn_declaration_node.add_child(fn_id);
    fn_declaration_node.add_child(fn_type);
    fn_declaration_node.add_child(fn_block);

    let ast = wrap_in_tle(fn_declaration_node);

    let mut symbol_table_stack = SymbolTableStack::new();
    let mut symbol_table_global = SymbolTable::new();
    let fn_value = SymbolValue::FunctionValue{
        parameters: Vec::new(),
    };
    let fn_info = SymbolInfo::new(DataType::Integer, fn_value);
    symbol_table_global.add("testForLoop".to_string(), fn_info);
    symbol_table_stack.push(symbol_table_global);
    symbol_table_stack.push(SymbolTable::new());

    let mod_ast: Module = ast_stitch(vec![ModElement::new(ast, symbol_table_stack, DEFAULT_PRIORITY_MODELEMENT)]);

    let mut ir_generator = IRGenerator::new();
    let module_tag = ir_generator.generate_ir(mod_ast);  

    let pools = ir_generator.get_resource_pools();

    let module = pools.lock().expect("coouldn't unlock pools mutex").get_module(module_tag).expect("No module found!");
    let write_result = Utils::write_to_file(module.clone(), "output_for_loop.ll");
    match write_result {
        Ok(_) => eprintln!("For loop test file written correctly!"),
        Err(_) => panic!("For loop test file failed to write!")
    }
    let get_str = Utils::write_to_string(module);
    let test_str = match get_str {
        Ok(str) => str,
        Err(e) => panic!("{}", e)
    };
    let expected_str = "; ModuleID = 'dummy_module'\nsource_filename = \"dummy_module\"\n\ndefine i64 @testForLoop() {\nentryID0:\n  %test_var = alloca i64, align 8\n  store i64 0, ptr %test_var, align 4\n  br label %for_condID1\n\nfor_condID1:                                      ; preds = %for_incID1, %for_bodyID1, %entryID0\n  br i1 true, label %for_bodyID1, label %for_endID1\n\nfor_bodyID1:                                      ; preds = %for_condID1\n  br label %for_condID1\n  br label %for_incID1\n\nfor_incID1:                                       ; preds = %for_bodyID1\n  store i64 42, ptr %test_var, align 4\n  br label %for_condID1\n\nfor_endID1:                                       ; preds = %for_condID1\n}\n";

    assert_eq!(test_str, expected_str)

}