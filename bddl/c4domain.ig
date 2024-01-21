#blackactions
:action occupyOnTop
:parameters (?x,?y)
:precondition (open(?x,?y) NOT(open(?x,?y+1)))
:effect (black(?x,?y))
:action occupyBottom
:parameters (?x,?y)
:precondition (open(?x,ymax))
:effect (black(?x,ymax))
#whiteactions
:action occupyOnTop
:parameters (?x,?y)
:precondition (open(?x,?y) NOT(open(?x,?y+1)))
:effect (white(?x,?y))
:action occupyBottom
:parameters (?x,?y)
:precondition (open(?x,ymax))
:effect (white(?x,ymax))
