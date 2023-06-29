module.exports = grammar({
    name: 'p4',

    extras: $ => [/\s/, $.line_comment, $.block_comment],

    externals: $ => [$.block_comment],

    conflicts: $ => [
        [$.type_or_void, $.type_identifier],
        [$.non_type_name, $.type_identifier],
        [$.annotation_token, $.type_identifier],
        [$.non_table_kw_name, $.type_identifier],
        [$.expression],
    ],

    rules: {
        source_file: $ => repeat(
            seq($._declaration, /\s/),
        ),

        _declaration: $ => choice(
            $.constant_declaration,
            $.extern_declaration,
            $.action_declaration,
            $.parser_declaration,
            $.type_declaration,
            $.control_declaration,
            $.instantiation,
            $.error_declaration,
            $.match_kind_declaration,
            $.function_declaration,
            $.preproc_include_declaration,
            $.preproc_define_declaration,
            $.preproc_undef_declaration,
            $.preproc_conditional_declaration
        ),

        non_type_name: $ => choice(
            $.identifier,
            'apply',
            'key',
            'actions',
            'state',
            'entries',
            'type',
        ),

        comment: $ => choice(
            $.line_comment,
            $.block_comment
        ),

        line_comment: $ => token(seq(
            '//', /.*/
        )),


        name: $ => choice(
            $.non_type_name,
            $.type_identifier,
        ),

        preproc_include_declaration: $ => seq(
            field("KeyWord", seq('#','include')),
            '<',
            $.file_name,
            '>'
        ),
        preproc_define_declaration: $ => seq(
            field("KeyWord", seq('#','define')),
            $.name,
            $.expression
        ),
        
        preproc_undef_declaration: $ => seq(
            field("KeyWord", seq('#','undef')),
            $.name
        ),
        preproc_conditional_declaration: $ => prec.left(seq(
            '#',
            field("KeyWord", choice('if', 'ifdef', 'ifndef')),
            $.expression,
            $._declaration,
            repeat($.preproc_conditional_declaration_elif),
            optional($.preproc_conditional_declaration_else),
            '#',
            field("KeyWordEnd", 'endif'))),
        preproc_conditional_declaration_else: $ => prec.left(seq('#', field("KeyWord", 'else'), $._declaration)),
        preproc_conditional_declaration_elif: $ => prec.left(seq('#', field("KeyWord", 'elif'), $.expression, $._declaration,)),

        file_name: $ => /\w*\.\w*/,

        non_table_kw_name: $ => choice(
            $.identifier,
            $.type_identifier,
            'apply',
            'state',
            'type',
        ),

        annotation: $ => choice(
            seq('@', field("name", $.name)),
            seq('@', field("name", $.name), '(', field("body", optional($.annotation_body)), ')'),
            seq('@', field("name", $.name), '[', field("struct", $.structured_annotation_body), ']'),
        ),
        annotation_list: $ => repeat1($.annotation),

        parameter_list: $ => seq(
            repeat(seq($.parameter, ',')), $.parameter
        ),

        parameter: $ => seq(field("annotation", optional($.annotation_list)), field("direction", optional($.direction)), field("type", $.type_ref), field("name", $.name), optional(seq('=', field("value", $.expression)))),

        direction: $ => choice(
            'in',
            'out',
            'inout',
        ),

        package_type_declaration: $ => choice(
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'package'), field('name', $.name), field('parameters_type', optional($.type_parameters))),
            seq(field('parameters', optional($.parameter_list)), ')'),
        ),
        instantiation: $ => choice(
            seq(field("type", $.type_ref), '(', field("args", optional($.argument_list)), ')', field('name', $.name), ';'),
            seq(field("annotation", optional($.annotation_list)), field("type", $.type_ref), '(', field("args", optional($.argument_list)), ')', field('name', $.name), ';'),
            seq(field("annotation", optional($.annotation_list)), field("type", $.type_ref), '(', field("args", optional($.argument_list)), ')', field('name', $.name), '=', field('obj', $.obj_initializer), ';'),
            seq(field("type", $.type_ref), '(', field("args", optional($.argument_list)), ')', field('name', $.name), '=', field('obj', $.obj_initializer), ';'),
        ),

        obj_initializer: $ => seq(
            '{', repeat($._obj_declaration), '}'
        ),

        _obj_declaration: $ => choice(
            $.function_declaration,
            $.instantiation,
        ),

        dot_prefix: $ => seq(
            '.',
        ),

        parser_declaration: $ => seq(
            field("declaration", $.parser_type_declaration),
            field("body", $.parser_body)
        ),

        parser_body: $ => seq(
            '{',
            repeat($._parser_local_element),
            repeat($.parser_state),
            '}'
        ),

        _parser_local_element: $ => choice(
            $.constant_declaration,
            $.variable_declaration,
            $.instantiation,
            $.value_set_declaration,
        ),

        parser_type_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'parser'), field('name', $.name), field('parameters_type', optional($.type_parameters)), '(', field('parameters', optional($.parameter_list)), ')'
        ),
        parser_state: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'state'), field("name", $.name), field("body", $.parser_state_body)
        ),
        parser_state_body: $ => seq(
            '{',
            field("statement", optional($.parser_state_body_statement)),
            field("transition_statement", optional($.transition_statement)),
            '}'
        ),

        parser_state_body_statement: $ => repeat1($._parser_statement),

        _parser_statement: $ => choice(
            $.assignment_or_method_call_statement,
            $.direct_application,
            $.parser_block_statement,
            $.constant_declaration,
            $.variable_declaration,
            $.empty_statement,
            $.conditional_statement,
        ),

        parser_block_statement: $ => seq(
            field("annotation", optional($.annotation_list)), field("body",$.parser_block_statement_body)
        ),

        parser_block_statement_body: $ => seq(
        	'{', repeat($._parser_statement), '}'
        ),

        transition_statement: $ => seq(
            field("KeyWord", 'transition'), $._state_expression
        ),

        _state_expression: $ => choice(
            seq(field("name",$.name), ';'),
            $.select_expression,
        ),

        select_expression: $ => seq(
            field("KeyWord", 'select'), $.select_expression_params, $.select_expression_body
        ),
        select_expression_params: $ => seq('(', optional($.expression_list), ')'),
        select_expression_body: $ => seq('{', optional($.select_case_list), '}'),

        select_case_list: $ => repeat1($.select_case),

        select_case: $ => seq(
            field('type',$._keyset_expression), ':', field('name',$.name), ';'
        ),

        _keyset_expression: $ => choice(
            $.tuple_keyset_expression,
            $.simple_keyset_expression,
        ),

        tuple_keyset_expression: $ => choice(
            seq("(", $.simple_keyset_expression, ",", $.simple_expression_list, ")"),
            seq("(", field("reduce", $.reduced_simple_keyset_expression), ")"),
        ),

        simple_expression_list: $ => seq(repeat(seq($.simple_keyset_expression, ',')), $.simple_keyset_expression),

        reduced_simple_keyset_expression: $ => choice(
            seq(field("value", $.expression), "&&&", field("value2", $.expression)),
            seq(field("value", $.expression), "..", field("value2", $.expression)),
            'default',
            "_",
        ),

        simple_keyset_expression: $ => choice(
            field("value", $.expression),
            'default',
            '_',
            seq(field("value", $.expression), 'mask', field("value2", $.expression)),
            seq(field("value", $.expression), 'range', field("value2", $.expression)),
        ),

        value_set_declaration: $ => seq(
            field("annotation", optional($.annotation_list)),
            field("KeyWord", 'valueset'),
            '<',
            field("type", choice($.base_type, $.tuple_type, $.type_name)),
            '>',
            '(',
            field("expression", $.expression),
            ')',
            field("name", $.name),
            ';'
        ),
        
        control_declaration: $ => seq(
            field("declaration", $.control_type_declaration),
            field("body", $.control_body),
        ),
        
        parser_declaration: $ => seq(
            field("declaration", $.parser_type_declaration),
            field("body", $.parser_body)
        ),

        control_body: $ => seq(
            '{',
            repeat($._control_local_declaration),
            field("KeyWord", 'apply'),
            $.block_statement,
            '}'
        ),

        control_type_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'control'), field("name", $.name), field('parameters_type', optional($.type_parameters)), '(', field("parameters", optional($.parameter_list)), ')'
        ),

        _control_local_declaration: $ => choice(
            $.constant_declaration,
            $.action_declaration,
            $.table_declaration,
            $.instantiation,
            $.variable_declaration,
        ),

        extern_declaration: $ => choice(
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'extern'), field('name', $.non_type_name), field('parameters_type', optional($.type_parameters)), '{', field('method', optional($.method_prototype_list)), '}'),
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'extern'), field('function', $.function_prototype), ';'),
        ),

        function_prototype: $ => seq(
            field("type", $.type_or_void), field("name", $.name), field('parameters_type', optional($.type_parameters)), '(', field("parameters_list", optional($.parameter_list)), ')'
        ),

        method_prototype_list: $ => repeat1($.method_prototype),

        method_prototype: $ => choice(
            seq(field("annotation", optional($.annotation_list)), field('function', $.function_prototype), ';'),
            seq(field("annotation", optional($.annotation_list)), field('type', $.type_identifier), '(', field('parameters', optional($.parameter_list)), ')', ';'),
        ),

        type_ref: $ => choice(
            $.base_type,
            $.type_name,
            $.specialized_type,
            $.header_stack_type,
            $.tuple_type,
        ),

        named_type: $ => choice(
            $.type_name,
            $.specialized_type,
        ),

        prefixed_type: $ => choice(
            $.type_identifier,
            seq($.dot_prefix, $.type_identifier),
        ),

        type_name: $ => seq(
            $.prefixed_type,
        ),

        tuple_type: $ => seq(
            field("KeyWord", 'tuple'), '<', optional($.type_argument_list), '>'
        ),

        header_stack_type: $ => choice(
            seq($.type_name, '[', $.expression, ']'),
            seq($.specialized_type, '[', $.expression, ']'),
        ),

        specialized_type: $ => seq(
            $.prefixed_type, '<', optional($.type_argument_list), '>'
        ),

        base_type: $ => choice(
            'bool',
            'error',
            'match_kind',
            'string',
            'int',
            'bit',
            seq('bit', '<', $.integer, '>'),
            seq('int', '<', $.integer, '>'),
            seq('varbit', '<', $.integer, '>'),
            seq('bit', '<', '(', $.expression, ')', '>'),
            seq('int', '<', '(', $.expression, ')', '>'),
            seq('varbit', '<', '(', $.expression, ')', '>'),
        ),

        type_or_void: $ => prec.left(1, choice(
            $.type_ref,
            'void',
            $.identifier,
        )),

        type_parameters: $ => seq(
            '<', $.type_parameter_list, '>'
        ),

        type_parameter_list: $ => seq(repeat(seq($.name, ',')), $.name),

        real_type_arg: $ => choice(
            '_',
            $.type_ref,
            'void',
        ),

        type_arg: $ => choice(
            '_',
            $.type_ref,
            $.non_type_name,
            'void',
        ),

        // TODO: Check if last real_type_arg should be type_arg
        real_type_argument_list: $ => seq(repeat(seq($.real_type_arg, ',')), $.real_type_arg),

        type_argument_list: $ => seq(repeat(seq($.type_arg, ',')), $.type_arg),

        type_declaration: $ => field("type_kind", choice(
            $._derived_type_declaration,
            $.typedef_declaration,
            seq($.parser_type_declaration, ';'),
            seq($.control_type_declaration, ';'),
            seq($.package_type_declaration, ';'),
        )),

        _derived_type_declaration: $ => choice(
            $.header_type_declaration,
            $.header_union_declaration,
            $.struct_type_declaration,
            $.enum_declaration,
        ),

        header_type_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'header'), field("name", $.name), field('parameters_type', optional($.type_parameters)), '{', field("field_list", optional($.struct_field_list)), '}'
        ),

        header_union_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'header_union'), field("name", $.name), field('parameters_type', optional($.type_parameters)), '{', field("field_list", optional($.struct_field_list)), '}'
        ),
        
        struct_type_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'struct'), field("name", $.name), field('parameters_type', optional($.type_parameters)), '{', field("field_list", optional($.struct_field_list)), '}'
        ),
        
        struct_field_list: $ => repeat1($.struct_field),
        struct_field: $ => seq(
            field("annotation", optional($.annotation_list)), field("type", $.type_ref), field("name", $.name), ';'
        ),
        
        enum_declaration: $ => choice(
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'enum'), field("name", $.name), '{', field("option_list", $.identifier_list), '}'),
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'enum'), field("type", $.type_ref), field("name", $.name), '{', field("option_list", $.specified_identifier_list), '}'),
        ),
        
        error_declaration: $ => seq(
            field("KeyWord", 'error'), '{', field("option_list", $.identifier_list), '}'
        ),
        
        match_kind_declaration: $ => seq(
            field("KeyWord", 'match_kind'), '{', field("option_list", $.identifier_list), '}'
        ),

        identifier_list: $ => seq(repeat(seq($.name, ',')), $.name),

        specified_identifier_list: $ => choice(
            $.specified_identifier,
            seq($.specified_identifier_list, ',', $.specified_identifier),
        ),

        specified_identifier: $ => seq(
            $.name, '=', $.initializer
        ),

        typedef_declaration: $ => choice(
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'typedef'), field("type", $.type_ref), field("name", $.name), ';'),
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'typedef'), field("type", $._derived_type_declaration), field("name", $.name), ';'),
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'type'), field("type", $.type_ref), field("name", $.name), ';'),
            seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'type'), field("type", $._derived_type_declaration), field("name", $.name), ';'),
        ),

        assignment_or_method_call_statement: $ => choice(
            seq(field("name", $.lvalue), '(', field("parameters", optional($.argument_list)), ')', ';'),
            seq(field("name", $.lvalue), '<', field("type", optional($.type_argument_list)), '>', '(', field("parameters", optional($.argument_list)), ')', ';'),
            seq(field("name", $.lvalue), '=', field("expression", $.expression), ';'),
        ),

        empty_statement: $ => seq(
            ';',
        ),

        return_statement: $ => seq(field("KeyWord", 'return'), field("expression", optional($.expression)), ';'),

        exit_statement: $ => seq(
            'exit', ';'
        ),

        conditional_statement: $ => choice(
            prec.left(seq(field("KeyWord", 'if'), '(', field("expression", $.expression), ')', field("bodyIf", $._statement))),
            prec.left(seq(field("KeyWord", 'if'), '(', field("expression", $.expression), ')', field("bodyIf", $._statement), field("KeyWordEnd", 'else'), field("bodyElse", $._statement))),
        ),

        direct_application: $ => choice(
            seq(field("name", $.type_name), '.', field("KeyWord", 'apply'), '(', field("args", optional($.argument_list)), ')', ';'),
            seq(field("specialized", $.specialized_type), '.', field("KeyWord", 'apply'), '(', field("args", optional($.argument_list)), ')', ';'),
        ),

        _statement: $ => choice(
            $.assignment_or_method_call_statement,
            $.direct_application,
            $.conditional_statement,
            $.empty_statement,
            $.block_statement,
            $.exit_statement,
            $.return_statement,
            $.switch_statement,
        ),

        block_statement: $ => seq(
            field("annotation", optional($.annotation_list)), '{', optional($._stat_or_decl_list), '}'
        ),

        _stat_or_decl_list: $ => repeat1($._statement_or_declaration),

        switch_statement: $ => seq(
            field("KeyWord", 'switch'), '(', field("expression",$.expression), ')', '{', field("body", repeat($.switch_case)), '}'
        ),

        switch_case: $ => choice(
            seq(field("name",$.switch_label), ':', field("value",$.block_statement)),
            seq(field("name",$.switch_label), ':'),
        ),

        switch_label: $ => choice(
            'default',
            $.non_brace_expression,
        ),

        _statement_or_declaration: $ => choice(
            $.variable_declaration,
            $.constant_declaration,
            $._statement,
        ),

        table_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'table'), field("name", $.name), '{', field("table", $.table_property_list), '}'
        ),

        table_property_list: $ => repeat1($._table_property),

        _table_property: $ => choice(
            $.keys_table,
            $.action_table,
            $.entries_table,
            $.name_table
        ),

        keys_table: $ => seq(field("KeyWord", 'key'), '=', '{', field("keys", optional($.key_element_list)), '}'),
        action_table: $ => seq(field("KeyWord", 'actions'), '=', '{', field("actions", optional($.action_list)), '}'),
        entries_table: $ => seq(field("annotation", optional($.annotation_list)), field("KeyWord", 'const'), field("KeyWordEnd", 'entries'), '=', '{', field("entries", optional($.entry_list)), '}'),
        name_table: $ => seq(field("annotation", optional($.annotation_list)), field("KeyWord", optional('const')), field("name", $.non_table_kw_name), '=', field("expression", $.initializer), ';'),

        key_element_list: $ => repeat1($.key_element),

        key_element: $ => seq(
            seq(field("expression", $.expression), ':', field("name", $.name), field("annotation", optional($.annotation_list)), ';'),
        ),

        action_list: $ => repeat1($.action),
        action: $ => seq(field("annotation", optional($.annotation_list)), $._action_ref, ';'),
        _action_ref: $ => choice(
            field("name", $.prefixed_non_type_name),
            seq(field("name", $.prefixed_non_type_name), '(', field("args", optional($.argument_list)), ')'),
        ),

        entry_list: $ => repeat1($.entry),

        entry: $ => seq(
            $._keyset_expression, ':', $._action_ref, field("annotation", optional($.annotation_list)), ';'
        ),

        action_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'action'), field("name", $.name), '(', field('parameters', optional($.parameter_list)), ')', field('block', $.block_statement)
        ),

        variable_declaration: $ => seq(field("annotation", optional($.annotation_list)), field("type", $.type_ref), field("name", $.name), '=', field("value", optional($.initializer)), ';'),

        constant_declaration: $ => seq(
            field("annotation", optional($.annotation_list)), field("KeyWord", 'const'), field("type", $.type_ref), field("name", $.name), '=', field("value", $.initializer), ';'
        ),

        initializer: $ => seq(
            $.expression,
        ),

        function_declaration: $ => seq(
            $.function_prototype, $.block_statement
        ),

        argument_list: $ => seq(repeat(seq($.argument, ',')), $.argument),

        argument: $ => choice(
            field("expression", $.expression),
            seq(field("name", $.name), '=', field("expression", $.expression)),
            '_',
            seq(field("name", $.name), '=', '_'),
        ),

        kv_list: $ => seq(repeat(seq($.kv_pair, ',')), $.kv_pair),

        kv_pair: $ => seq(
            $.name, '=', $.expression
        ),

        expression_list: $ => seq(repeat(seq($.expression, ',')), $.expression),

        annotation_body: $ => choice(
            seq(field("body", $.annotation_body), '(', field("body2", optional($.annotation_body)), ')'),
            seq(field("body", $.annotation_body), field("token", $.annotation_token)),
        ),

        structured_annotation_body: $ => choice(
            $.expression_list,
            $.kv_list,
        ),

        annotation_token: $ => choice(
            'abstract',
            'action',
            'actions',
            'apply',
            'bool',
            $.bool,
            'bit',
            'const',
            'control',
            'default',
            'else',
            'entries',
            'enum',
            'error',
            'exit',
            'extern',
            'header',
            'header_union',
            'if',
            'in',
            'inout',
            'int',
            'key',
            'match_kind',
            'type',
            'out',
            'parser',
            'package',
            'pragma',
            'return',
            'select',
            'state',
            'string',
            'struct',
            'switch',
            'table',
            'transition',
            'tuple',
            'typedef',
            'varbit',
            'valueset',
            'void',
            "_",
            $.identifier,
            $.type_identifier,
            $.string,
            $.integer,
            "&&&",
            "..",
            "<<",
            "&&",
            "||",
            "==",
            "!=",
            ">=",
            "<=",
            "++",
            "+",
            "|+|",
            "-",
            "|-|",
            "*",
            "/",
            "%",
            "|",
            "&",
            "^",
            "~",
            "[",
            "]",
            "{",
            "}",
            "<",
            ">",
            "!",
            ":",
            ",",
            "?",
            ".",
            "=",
            ";",
            "@",
            'unknown_token',
        ),

        member: $ => seq(
            $.name,
        ),

        prefixed_non_type_name: $ => choice(
            $.non_type_name,
            seq($.dot_prefix, $.non_type_name),
        ),

        lvalue: $ => choice(
            $.prefixed_non_type_name,
            'this',
            $.lvalue_dot,
            $.lvalue_bra,
            $.lvalue_double_dot,
        ),
        lvalue_dot: $ => seq($.lvalue, '.', $.member),
        lvalue_bra: $ => seq($.lvalue, '[', $.expression, ']'),
        lvalue_double_dot: $ => seq($.lvalue, '[', $.expression, ':', $.expression, ']'),

        bool: $ => choice(
            'true',
            'false'
        ),

        expression: $ => choice(
            $.integer,
            $.bool,
            'this',
            $.string,
            $.non_type_name,
            seq($.dot_prefix, $.non_type_name),
            prec.left(1, seq($.expression, '[', $.expression, ']')),
            prec.left(1, seq($.expression, '[', $.expression, ':', $.expression, ']')),
            prec.left(1, seq('{', optional($.expression_list), '}')),
            prec.left(1, seq('{', $.kv_list, '}')),
            prec.left(1, seq('(', $.expression, ')')),
            prec.right(2, seq('!', $.expression)),
            prec.right(2, seq('~', $.expression)),
            prec.right(2, seq('-', $.expression)),
            prec.right(2, seq('+', $.expression)),
            prec.left(1, seq($.type_name, '.', $.member)),
            prec.left(1, seq('error', '.', $.member)),
            prec.left(1, seq($.expression, '.', $.member)),
            prec.left(3, seq($.expression, '*', $.expression)),
            prec.left(3, seq($.expression, '/', $.expression)),
            prec.left(3, seq($.expression, '%', $.expression)),
            prec.left(4, seq($.expression, '+', $.expression)),
            prec.left(4, seq($.expression, '-', $.expression)),
            prec.left(4, seq($.expression, '|+|', $.expression)),
            prec.left(4, seq($.expression, '|-|', $.expression)),
            prec.left(5, seq($.expression, '<<', $.expression)),
            prec.left(5, seq($.expression, '>>', $.expression)),
            prec.left(11, seq($.expression, '<=', $.expression)), //TODO: Double check precedence of <=,<... and == 
            prec.left(11, seq($.expression, '>=', $.expression)),
            prec.left(11, seq($.expression, '<', $.expression)),
            prec.left(11, seq($.expression, '>', $.expression)),
            prec.left(7, seq($.expression, '!=', $.expression)),
            prec.left(7, seq($.expression, '==', $.expression)),
            prec.left(8, seq($.expression, '&', $.expression)),
            prec.left(9, seq($.expression, '^', $.expression)),
            prec.left(10, seq($.expression, '|', $.expression)),
            prec.left(1, seq($.expression, '++', $.expression)),
            prec.left(12, seq($.expression, '&&', $.expression)),
            prec.left(12, seq($.expression, '||', $.expression)),
            prec.right(13, seq($.expression, '?', $.expression, ':', $.expression)),
            prec.left(1, seq($.expression, '<', $.real_type_argument_list, '>', '(', optional($.argument_list), ')')),
            prec.left(1, seq($.expression, '(', optional($.argument_list), ')')),
            prec.left(1, seq($.named_type, '(', optional($.argument_list), ')')),
            prec.right(2, seq('(', $.type_ref, ')', $.expression)),
        ),

        non_brace_expression: $ => choice(
            $.integer,
            $.string,
            $.bool,
            'this',
            $.non_type_name,
            seq($.dot_prefix, $.non_type_name),
            seq($.non_brace_expression, '[', $.expression, ']'),
            seq($.non_brace_expression, '[', $.expression, ':', $.expression, ']'),
            seq('(', $.expression, ')'),
            seq('!', $.expression),
            seq('~', $.expression),
            seq('-', $.expression),
            seq('+', $.expression),
            seq($.type_name, '.', $.member),
            seq('error', '.', $.member),
            seq($.non_brace_expression, '.', $.member),
            seq($.non_brace_expression, '*', $.expression),
            seq($.non_brace_expression, '/', $.expression),
            seq($.non_brace_expression, '%', $.expression),
            seq($.non_brace_expression, '+', $.expression),
            seq($.non_brace_expression, '-', $.expression),
            seq($.non_brace_expression, '|+|', $.expression),
            seq($.non_brace_expression, '|-|', $.expression),
            seq($.non_brace_expression, '<<', $.expression),
            seq($.non_brace_expression, '>>', $.expression),
            seq($.non_brace_expression, '<=', $.expression),
            seq($.non_brace_expression, '>=', $.expression),
            seq($.non_brace_expression, '<', $.expression),
            seq($.non_brace_expression, '>', $.expression),
            seq($.non_brace_expression, '!=', $.expression),
            seq($.non_brace_expression, '==', $.expression),
            seq($.non_brace_expression, '&', $.expression),
            seq($.non_brace_expression, '^', $.expression),
            seq($.non_brace_expression, '|', $.expression),
            seq($.non_brace_expression, '++', $.expression),
            seq($.non_brace_expression, '&&', $.expression),
            seq($.non_brace_expression, '||', $.expression),
            seq($.non_brace_expression, '?', $.expression, ':', $.expression),
            seq($.non_brace_expression, '<', $.real_type_argument_list, '>', '(', optional($.argument_list), ')'),
            seq($.non_brace_expression, '(', optional($.argument_list), ')'),
            seq($.named_type, '(', optional($.argument_list), ')'),
            seq('(', $.type_ref, ')', $.expression),
        ),

        integer: $ => /(\d+([wWsS]))?\d+(([xX][0-9A-F_]+)|([bB][0-1_]+)|([oO][0-7_]+)|([dD][0-9_]+))?/,

        string: $ => /"(\\.|[^"\\])*"/,


        prec: $ => $.identifier,

        identifier: $ => /[a-zA-Z_]\w*/,

        type_identifier: $ => $.identifier,

    }
});