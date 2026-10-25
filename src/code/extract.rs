//! Register read off Rust code by its own grammar (syn): the compiler's
//! parse is the teacher here, and every relation it gives is exact. A name
//! is what it relates to: what a function calls, takes and returns, what a
//! type has and holds, what implements what, what belongs to what.

use phiecs::quest::Row;
use syn::visit::{self, Visit};

/// The last segment of a type's path, through references and slices:
/// `&mut Vec<Row>` is `Vec`.
pub fn type_name(t: &syn::Type) -> Option<String> {
    match t {
        syn::Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        syn::Type::Reference(r) => type_name(&r.elem),
        syn::Type::Slice(s) => type_name(&s.elem),
        syn::Type::Array(a) => type_name(&a.elem),
        _ => None,
    }
}

pub(crate) fn row(about: &str, kind: &str, other: &str) -> Row {
    Row::new(about, kind, other)
}

#[derive(Default)]
pub struct Reader {
    pub rows: Vec<Row>,
    /// Every name declared as a type (struct, enum, trait, type alias).
    pub types: Vec<String>,
    within: Vec<String>,
}

impl Reader {
    fn signature(&mut self, f: &str, sig: &syn::Signature) {
        for input in &sig.inputs {
            if let syn::FnArg::Typed(t) = input {
                if let Some(n) = type_name(&t.ty) {
                    self.rows.push(row(f, "takes", &n));
                }
            }
        }
        if let syn::ReturnType::Type(_, t) = &sig.output {
            if let Some(n) = type_name(t) {
                self.rows.push(row(f, "returns", &n));
            }
        }
    }

    fn function(&mut self, name: String, sig: &syn::Signature, owner: Option<String>, body: impl FnOnce(&mut Self)) {
        self.signature(&name, sig);
        // The words of the name (docs/124): `count_even` has the words `count`
        // and `even`; what a word relates to is then read across functions.
        for w in name.split('_').filter(|w| w.len() > 1) {
            self.rows.push(row(&name, "has-word", w));
        }
        if let Some(o) = owner {
            self.rows.push(row(&name, "part-of", &o));
        }
        self.within.push(name);
        body(self);
        self.within.pop();
    }
}

impl<'a> Visit<'a> for Reader {
    fn visit_item_fn(&mut self, f: &'a syn::ItemFn) {
        self.function(f.sig.ident.to_string(), &f.sig, None, |me| visit::visit_block(me, &f.block));
    }

    fn visit_item_impl(&mut self, i: &'a syn::ItemImpl) {
        let owner = type_name(&i.self_ty);
        if let (Some(o), Some((_, t, _))) = (&owner, &i.trait_) {
            if let Some(tr) = t.segments.last() {
                self.rows.push(row(o, "implements", &tr.ident.to_string()));
            }
        }
        for item in &i.items {
            if let syn::ImplItem::Fn(m) = item {
                self.function(m.sig.ident.to_string(), &m.sig, owner.clone(), |me| visit::visit_block(me, &m.block));
            }
        }
    }

    fn visit_item_struct(&mut self, s: &'a syn::ItemStruct) {
        let name = s.ident.to_string();
        self.types.push(name.clone());
        for field in &s.fields {
            if let Some(id) = &field.ident {
                self.rows.push(row(&name, "has", &id.to_string()));
            }
            if let Some(t) = type_name(&field.ty) {
                self.rows.push(row(&name, "holds", &t));
            }
        }
    }

    fn visit_item_enum(&mut self, e: &'a syn::ItemEnum) {
        let name = e.ident.to_string();
        self.types.push(name.clone());
        for v in &e.variants {
            self.rows.push(row(&name, "has", &v.ident.to_string()));
        }
    }

    fn visit_item_trait(&mut self, t: &'a syn::ItemTrait) {
        let name = t.ident.to_string();
        self.types.push(name.clone());
        for item in &t.items {
            if let syn::TraitItem::Fn(m) = item {
                self.function(m.sig.ident.to_string(), &m.sig, Some(name.clone()), |_| {});
            }
        }
    }

    fn visit_item_type(&mut self, t: &'a syn::ItemType) {
        self.types.push(t.ident.to_string());
    }

    fn visit_expr_call(&mut self, c: &'a syn::ExprCall) {
        if let (Some(f), syn::Expr::Path(p)) = (self.within.last().cloned(), &*c.func) {
            if let Some(s) = p.path.segments.last() {
                self.rows.push(row(&f, "calls", &s.ident.to_string()));
            }
        }
        visit::visit_expr_call(self, c);
    }

    /// The operators a function's body uses (`%`, `>`, `==`): exact from the
    /// parser, and the place where a name's word shows its meaning (docs/124).
    fn visit_expr_binary(&mut self, b: &'a syn::ExprBinary) {
        if let Some(f) = self.within.last().cloned() {
            self.rows.push(row(&f, "uses-op", quote_op(&b.op)));
        }
        visit::visit_expr_binary(self, b);
    }

    /// A number or bool written in the body, as written (`0`, `0.0`, `2`):
    /// what a type exchanges (`i32 : 0 :: f64 : 0.0`).
    fn visit_expr_lit(&mut self, l: &'a syn::ExprLit) {
        let text = match &l.lit {
            syn::Lit::Int(x) => x.to_string(),
            syn::Lit::Float(x) => x.to_string(),
            syn::Lit::Bool(x) => x.value.to_string(),
            _ => return,
        };
        if let Some(f) = self.within.last().cloned() {
            self.rows.push(row(&f, "has-literal", &text));
        }
    }

    fn visit_expr_method_call(&mut self, c: &'a syn::ExprMethodCall) {
        if let Some(f) = self.within.last().cloned() {
            self.rows.push(row(&f, "calls", &c.method.to_string()));
        }
        visit::visit_expr_method_call(self, c);
    }
}

fn quote_op(op: &syn::BinOp) -> &'static str {
    use syn::BinOp::*;
    match op {
        Add(_) => "+", Sub(_) => "-", Mul(_) => "*", Div(_) => "/", Rem(_) => "%",
        And(_) => "&&", Or(_) => "||", BitXor(_) => "^", BitAnd(_) => "&", BitOr(_) => "|",
        Shl(_) => "<<", Shr(_) => ">>", Eq(_) => "==", Lt(_) => "<", Le(_) => "<=", Ne(_) => "!=", Ge(_) => ">=", Gt(_) => ">",
        AddAssign(_) => "+=", SubAssign(_) => "-=", MulAssign(_) => "*=", DivAssign(_) => "/=", RemAssign(_) => "%=",
        _ => "op",
    }
}
