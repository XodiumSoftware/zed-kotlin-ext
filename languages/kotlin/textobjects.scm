; Function declarations, covering block bodies (`{ ... }`) and single
; expression bodies (`= ...`) since both are wrapped in `function_body`.
(function_declaration
  (function_body) @function.inside) @function.around

; Lambdas, e.g. `list.map { it * 2 }`.
(lambda_literal
  (statements) @function.inside) @function.around

; Property getters and setters.
(getter
  (function_body) @function.inside) @function.around

(setter
  (function_body) @function.inside) @function.around

; Secondary constructors.
(secondary_constructor
  (statements) @function.inside) @function.around

; Classes, objects, and companion objects.
(class_declaration
  (class_body) @class.inside) @class.around

(class_declaration
  (enum_class_body) @class.inside) @class.around

(object_declaration
  (class_body) @class.inside) @class.around

(companion_object
  (class_body) @class.inside) @class.around

; Comments. Zed falls back to the `around` range when no `inside`
; capture matches, and comment nodes have no inner content node.
(line_comment) @comment.around

(multiline_comment) @comment.around
