import z3

# Instance: bool matrix for knots
def crosses_win_possible(instance, size):
    solver = z3.Solver()

    for i in range(size):
        for j in range(size):
            if instance[i][j]:
                solver.add(z3.Bool(f"kx{i}y{j}"))

    for i in range(size):
        for j in range(size):
            crosses = z3.Bool(f"cx{i}y{j}")
            knots = z3.Bool(f"kx{i}y{j}")
            solver.add(crosses != knots)

    triplets = []
    diag1 = []
    diag2 = []
    for i in range(size):
        diag1.append(z3.Bool(f"cx{i}y{i}"))
        k = size - i - 1
        diag2.append(z3.Bool(f"cx{i}y{k}"))
        hori = []
        vert = []
        for j in range(size):
            hori.append(z3.Bool(f"cx{i}y{j}"))
            vert.append(z3.Bool(f"cx{j}y{i}"))
        triplets.append(z3.And(hori))
        triplets.append(z3.And(vert))
    triplets.append(z3.And(diag1))
    triplets.append(z3.And(diag2))
    solver.add(z3.Or(triplets))

    if solver.check() == z3.sat:
        return solver.model()
    else:
        return None

instance = [
    [True, True, False, False],
    [False, True, False, False],
    [True, False, True, True],
    [False, False, True, True]
]

#print(crosses_win_possible(instance, 4))

def white_win_possible_formula(black_positions, size):
    asserts = []

    f = z3.Function("f", z3.BitVecSort(4), z3.BitVecSort(4), z3.BoolSort())
    for x, y in black_positions:
        asserts.append(f(x, y) == False)

    row_y = z3.BitVec("row_y", 4)
    inc_x = z3.BitVec("inc_x", 4)
    row = z3.ForAll(inc_x, z3.Or(z3.UGE(inc_x, size), f(inc_x, row_y) == True))
    asserts.append(z3.ULT(row_y, 3))
    
    col_x = z3.BitVec("col_x", 4)
    inc_y = z3.BitVec("inc_y", 4)
    col = z3.ForAll(inc_y, z3.Or(z3.UGE(inc_y, size), f(col_x, inc_y) == True))
    asserts.append(z3.ULT(col_x, 3))

    diag = z3.BitVec("diag", 4)
    diag_1 = z3.ForAll(diag, z3.Or(z3.UGE(diag, size), f(diag, diag) == True)) 
    diag_2 = z3.ForAll(diag, z3.Or(z3.UGE(diag, size), f(size - 1 - diag, diag) == True)) 

    asserts.append(z3.Or(row, col, diag_1, diag_2))

    return z3.And(asserts), f


def white_win_possible(black_positions, size):
    solver = z3.Solver()

    formula, _ = white_win_possible_formula(black_positions, size)
    solver.add(formula)

    if solver.check() == z3.sat:
        print(solver.model())
    else:
        print("Not possible")



def white_win_possible_for_all_black_moves(black_positions, size):
    def helper(x, y):
        state, f = white_win_possible_formula(black_positions,size)
        return z3.And(state, f(x, y))
    solver = z3.Solver()
    
    new_x = z3.BitVec("new_x", 4)
    new_y = z3.BitVec("new_y", 4)
    state, f = white_win_possible_formula(black_positions,size)
    formula = z3.ForAll(new_x, z3.Or(z3.UGE(new_x, size), z3.ForAll(new_y, z3.Or(z3.UGE(new_y, size), z3.And(state, f(new_x, new_y) == False)))))
    solver.add(formula)

    if solver.check() == z3.sat:
        print(solver.model())
    else:
        print("Not possible")
    

white_win_possible_for_all_black_moves([(0, 0), (1, 2)], 3)
