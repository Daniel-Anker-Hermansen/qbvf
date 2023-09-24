from z3 import *

def gen_contains(positions, x, y):
    return Or([And(x == x0, y == y0) for (x0, y0) in positions])

def gen_has_won(positions):
    row_x = BitVec("row_x", 2)
    row_y = BitVec("row_y", 2)
    row_win = Exists(row_y, ForAll(row_x, Implies(ULT(row_x, 3), gen_contains(positions, row_x, row_y))))
    col_x = BitVec("col_x", 2)
    col_y = BitVec("col_y", 2)
    col_win = Exists(col_x, ForAll(col_y, Implies(ULT(col_y, 3), gen_contains(positions, col_x, col_y))))
    diag = BitVec("diag", 2)
    diag_1a = ForAll(diag, Implies(ULT(diag, 3), gen_contains(positions, diag, diag)))
    diag_2a = ForAll(diag, Implies(ULT(diag, 3), gen_contains(positions, 2 - diag, diag)))
    return Or(row_win, col_win, diag_1a, diag_2a)

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
        return Exists([x, y], And(not_duplicate, gen_black_move(whites, blacks, depth - 1)))

def gen_black_move(whites, blacks, depth):
    x = BitVec(f"bx{depth}", 2)
    y = BitVec(f"by{depth}", 2)
    not_duplicate = gen_is_not_duplicate(whites, blacks, x, y)
    blacks.append((x, y))
    body = Implies(not_duplicate, And(Not(gen_has_won(blacks)), gen_white_move(whites, blacks, depth)))
    return ForAll([x, y], body)


whites = [(0, 0), (2, 2)]
blacks = []

solve(gen_black_move(whites, blacks, 1))
