from z3 import *
import time

def gen_contains(positions, x, y):
    return Or([And(x == x0, y == y0) for (x0, y0) in positions])

def gen_has_won(positions):
    rows = Or([And([gen_contains(positions, x, y) for x in range(3)]) for y in range(3)])
    cols = Or([And([gen_contains(positions, x, y) for y in range(3)]) for x in range(3)])
    diag_1 = And([gen_contains(positions, x, x) for x in range(3)])
    diag_2 = And([gen_contains(positions, 3 - 1 - x, x) for x in range(3)])
    return Or(rows, cols, diag_1, diag_2)

def gen_white_has_won(whites, blacks):
    black_won = gen_has_won(blacks)
    white_won = gen_has_won(whites)
    return And(Not(black_won), white_won)

def gen_is_not_duplicate(whites, blacks, x, y):
    white_duplicate = gen_contains(whites, x, y)
    black_duplicates = gen_contains(blacks, x, y)
    return Not(Or(white_duplicate, black_duplicates))

def gen_white_move(whites, blacks, depth):
    x = BitVec(f"wx{depth}", 2)
    y = BitVec(f"wy{depth}", 2)
    not_duplicate = gen_is_not_duplicate(whites, blacks, x, y)
    whites.append((x, y))
    if depth == 0:
        return Exists([x, y], And(not_duplicate, gen_has_won(whites)))
    else:
        return Exists([x, y], And(not_duplicate, Or(gen_has_won(whites), gen_black_move(whites, blacks, depth - 1))))

def gen_black_move(whites, blacks, depth):
    x = BitVec(f"bx{depth}", 2)
    y = BitVec(f"by{depth}", 2)
    not_duplicate = gen_is_not_duplicate(whites, blacks, x, y)
    blacks.append((x, y))
    body = Implies(not_duplicate, And(Not(gen_has_won(blacks)), gen_white_move(whites, blacks, depth)))
    return ForAll([x, y], body)

whites = []
blacks = []

formula = gen_white_move(whites, blacks, 3)

now = time.time()

solver = Then("simplify", "smt").solver()
#solver = Solver()
solver.add(formula)

print(solver.check())

elapsed = time.time() - now

print(elapsed)

