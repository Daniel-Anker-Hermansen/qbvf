use z3::{ast::{BV, Bool, exists_const, Ast}, Context, DatatypeBuilder, FuncDecl, Sort, DatatypeVariant};

fn gen_z3<'a>(whites: Vec<(BV, BV)>, blacks: Vec<(BV, BV)>, depth: usize, ctx: &'a Context) -> Bool<'a> {
    if depth == 0 {
        let sort = DatatypeBuilder::new(ctx, "board")
            .variant("white", Vec::new())
            .variant("black", Vec::new())
            .variant("empty", Vec::new())
            .finish();
        let white = sort.variants[0].constructor.apply(&[]);
        let black = sort.variants[1].constructor.apply(&[]);
        let empty = sort.variants[2].constructor.apply(&[]);
        let bv_sort = Sort::bitvector(ctx, 2);
        let f = FuncDecl::new(ctx, "f", &[&bv_sort, &bv_sort], &sort.sort);
        let mut terms = Vec::new();
        terms.extend(whites.iter().map(|(x, y)| f.apply(&[x, y])._eq(&white)));
        Bool::and(ctx, &terms.iter().collect::<Vec<_>>())
    }
    else {
        unimplemented!()
    }
}

fn main() {
    let context = z3::Context::new(&z3::Config::new());
    let solver = z3::Solver::new(&context);
    solver.assert(&gen_z3(Vec::new(), Vec::new(), 0, &context));
    dbg!(solver.check());
    dbg!(solver.get_model());
}
