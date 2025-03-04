use hir::{known, HasSource, Name};
use syntax::{
    ast::{self, NameOwner},
    AstNode, TextRange,
};

use crate::{
    assist_context::{AssistContext, Assists},
    AssistId, AssistKind,
};

// Assist: generate_is_empty_from_len
//
// Generates is_empty implementation from the len method.
//
// ```
// struct MyStruct { data: Vec<String> }
//
// impl MyStruct {
//     p$0ub fn len(&self) -> usize {
//         self.data.len()
//     }
// }
// ```
// ->
// ```
// struct MyStruct { data: Vec<String> }
//
// impl MyStruct {
//     pub fn len(&self) -> usize {
//         self.data.len()
//     }
//
//     pub fn is_empty(&self) -> bool {
//         self.len() == 0
//     }
// }
// ```
pub(crate) fn generate_is_empty_from_len(acc: &mut Assists, ctx: &AssistContext) -> Option<()> {
    let fn_node = ctx.find_node_at_offset::<ast::Fn>()?;
    let fn_name = fn_node.name()?;

    if fn_name.text() != "len" {
        cov_mark::hit!(len_function_not_present);
        return None;
    }

    if fn_node.param_list()?.params().next().is_some() {
        cov_mark::hit!(len_function_with_parameters);
        return None;
    }

    let impl_ = fn_node.syntax().ancestors().find_map(ast::Impl::cast)?;
    if get_impl_method(ctx, &impl_, &known::is_empty).is_some() {
        cov_mark::hit!(is_empty_already_implemented);
        return None;
    }

    let range = get_text_range_of_len_function(ctx, &impl_)?;

    acc.add(
        AssistId("generate_is_empty_from_len", AssistKind::Generate),
        "Generate a is_empty impl from a len function",
        range,
        |builder| {
            let code = r#"

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }"#
            .to_string();
            builder.insert(range.end(), code)
        },
    )
}

fn get_impl_method(
    ctx: &AssistContext,
    impl_: &ast::Impl,
    fn_name: &Name,
) -> Option<hir::Function> {
    let db = ctx.sema.db;
    let impl_def: hir::Impl = ctx.sema.to_def(impl_)?;

    let scope = ctx.sema.scope(impl_.syntax());
    let krate = impl_def.module(db).krate();
    let ty = impl_def.target_ty(db);
    let traits_in_scope = scope.traits_in_scope();
    ty.iterate_method_candidates(db, krate, &traits_in_scope, Some(fn_name), |_, func| Some(func))
}

fn get_text_range_of_len_function(ctx: &AssistContext, impl_: &ast::Impl) -> Option<TextRange> {
    let db = ctx.sema.db;
    let func = get_impl_method(ctx, impl_, &known::len)?;
    let node = func.source(db)?;
    Some(node.syntax().value.text_range())
}

#[cfg(test)]
mod tests {
    use crate::tests::{check_assist, check_assist_not_applicable};

    use super::*;

    #[test]
    fn len_function_not_present() {
        cov_mark::check!(len_function_not_present);
        check_assist_not_applicable(
            generate_is_empty_from_len,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    p$0ub fn test(&self) -> usize {
            self.data.len()
        }
    }
"#,
        );
    }

    #[test]
    fn len_function_with_parameters() {
        cov_mark::check!(len_function_with_parameters);
        check_assist_not_applicable(
            generate_is_empty_from_len,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    p$0ub fn len(&self, _i: bool) -> usize {
        self.data.len()
    }
}
"#,
        );
    }

    #[test]
    fn is_empty_already_implemented() {
        cov_mark::check!(is_empty_already_implemented);
        check_assist_not_applicable(
            generate_is_empty_from_len,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    p$0ub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
"#,
        );
    }

    #[test]
    fn generate_is_empty() {
        check_assist(
            generate_is_empty_from_len,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    p$0ub fn len(&self) -> usize {
        self.data.len()
    }
}
"#,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
"#,
        );
    }

    #[test]
    fn multiple_functions_in_impl() {
        check_assist(
            generate_is_empty_from_len,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    pub fn new() -> Self {
        Self { data: 0 }
    }

    p$0ub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn work(&self) -> Option<usize> {

    }
}
"#,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    pub fn new() -> Self {
        Self { data: 0 }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn work(&self) -> Option<usize> {

    }
}
"#,
        );
    }

    #[test]
    fn multiple_impls() {
        check_assist_not_applicable(
            generate_is_empty_from_len,
            r#"
struct MyStruct { data: Vec<String> }

impl MyStruct {
    p$0ub fn len(&self) -> usize {
        self.data.len()
    }
}

impl MyStruct {
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
"#,
        );
    }
}
