use swc_core::common::util::take::Take;
use swc_core::ecma::ast::Pat::Ident;
use swc_core::ecma::ast::{Decl, ModuleDecl, ModuleItem};
use swc_core::ecma::{
    ast::{Program, VarDeclarator},
    transforms::testing::test,
    visit::{as_folder, FoldWith, VisitMut, VisitMutWith},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

pub struct TransformVisitor;

impl VisitMut for TransformVisitor {
    // Implement necessary visit_mut_* methods for actual custom transform.
    // A comprehensive list of possible visitor methods can be found here:
    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html

    fn visit_mut_var_declarator(&mut self, v: &mut VarDeclarator) {
        // This is not required in this example, but you typically need this.
        v.visit_mut_children_with(self);

        // v.name is `Pat`.
        // See https://rustdoc.swc.rs/swc_ecma_ast/enum.Pat.html
        match &v.name {
            // If we want to delete the node, we should return false.
            //
            // Note the `&*` before i.sym.
            // The type of symbol is `JsWord`, which is an interned string.
            Ident(i) => {
                if &*i.sym == "getServerSideProps" {
                    // Take::take() is a helper function, which stores invalid value in the node.
                    // For Pat, it's `Pat::Invalid`.
                    v.name.take();
                }
            }
            _ => {
                // Noop if we don't want to delete the node.
            }
        }
    }

    fn visit_mut_var_declarators(&mut self, vars: &mut Vec<VarDeclarator>) {
        vars.visit_mut_children_with(self);

        vars.retain(|node| {
            // We want to remove the node, so we should return false.
            if node.name.is_invalid() {
                return false;
            }

            // Return true if we want to keep the node.
            true
        });
    }

    fn visit_mut_module_items(&mut self, stmts: &mut Vec<ModuleItem>) {
        stmts.visit_mut_children_with(self);

        // This is also required, because top-level statements are stored in `Vec<ModuleItem>`.
        stmts.retain(|s| {
            match s {
                ModuleItem::ModuleDecl(d) => match d {
                    ModuleDecl::ExportDecl(ed) => match &ed.decl {
                        Decl::Var(v) => {
                            if v.decls.is_empty() {
                                return false;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }

            return true;
        });
    }
}

/// An example plugin function with macro support.
/// `plugin_transform` macro interop pointers into deserialized structs, as well
/// as returning ptr back to host.
///
/// It is possible to opt out from macro by writing transform fn manually
/// if plugin need to handle low-level ptr directly via
/// `__transform_plugin_process_impl(
///     ast_ptr: *const u8, ast_ptr_len: i32,
///     unresolved_mark: u32, should_enable_comments_proxy: i32) ->
///     i32 /*  0 for success, fail otherwise.
///             Note this is only for internal pointer interop result,
///             not actual transform result */`
///
/// This requires manual handling of serialization / deserialization from ptrs.
/// Refer swc_plugin_macro to see how does it work internally.
#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    program.fold_with(&mut as_folder(TransformVisitor))
}

// An example to test plugin transform.
// Recommended strategy to test plugin's transform is verify
// the Visitor's behavior, instead of trying to run `process_transform` with mocks
// unless explicitly required to do so.
// test!(
//     Default::default(),
//     |_| as_folder(TransformVisitor),
//     simple_transform_kdy1,
//     r#"foo === bar;"#,
//     r#"kdy1 === bar;"#
// );

test!(
    Default::default(),
    |_| as_folder(TransformVisitor),
    delete_export_getserversideprops,
    r#"
export const getServerSideProps = async context => {
    return {
        props: {
            a: 1,
        },
    };
};
const Home = () => {
    return null;
};
export default Home;"#,
    r#"
const Home = () => {
    return null;
};
export default Home;"#
);
