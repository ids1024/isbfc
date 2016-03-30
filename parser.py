import collections

OUTPUT=0
INPUT=1
LOOP=2
ENDLOOP=3
MOVE=4
ADD=5
SET=6
MULCOPY=7
SCAN=8
LOADOUT=9
LOADOUTSET=10
IF=11
ENDIF=12

def parse(code):
    code = ''.join(i for i in code if i in '.,[]+-<>')
    tokens = []
    for i in code:
        if i == '+':
            tokens.append((ADD, (0, 1)))
        elif i == '-':
            tokens.append((ADD, (0, -1)))
        elif i == '>':
            tokens.append((MOVE, 1))
        elif i == '<':
            tokens.append((MOVE, -1))
        elif i == '[':
            tokens.append((LOOP, None))
        elif i == ']':
            tokens.append((ENDLOOP, None))
        elif i == ',':
            tokens.append((INPUT, 1))
        if i == '.':
            tokens.append((LOADOUT, (0, 0)))
            tokens.append((OUTPUT, None))
    return tokens

def optimize(tokens):
    add = 0
    move = 0
    newtokens = []
    allzero = True
    for token, value in tokens:
        if add and token != ADD:
            newtokens.append((ADD, (0, add)))
            add = 0
        elif move and token != MOVE:
            newtokens.append((MOVE, move))
            move = 0

        if token == ADD and value[0] == 0:
            #TODO: Optimization could still be extended
            if allzero:
                newtokens.append((SET, value))
            else:
                add += value[1]
        elif token == ADD and allzero:
            newtokens.append((SET, value))
        elif token == LOADOUT and allzero:
            offset, add = value
            newtokens.append((LOADOUTSET, add))
        elif token == MOVE:
            move += value
        else:
            newtokens.append((token, value))

        #NOTE: Must be updated for new tokens
        if token in (ADD, SET, INPUT):
            allzero = False
    if add:
        newtokens.append((ADD, (0, add)))
    elif move:
        newtokens.append((MOVE, move))

    newtokens2 = []

    i = 0
    while i < len(newtokens):
        optimized = False

        # Optimize scan loop
        if (i < len(newtokens)-2 and
             newtokens[i][0] == LOOP and
             newtokens[i+1][0] == MOVE and
             newtokens[i+2][0] == ENDLOOP):

            offset = newtokens[i+1][1]
            newtokens2.append((SCAN, offset))
            optimized = True
            i += 2

        # Optimize out clear loop / multiply move loop
        if not optimized and newtokens[i][0] == LOOP:
            j = i + 1
            adds = {}
            sets = {}
            while j < len(newtokens) and newtokens[j][0] != ENDLOOP:
                if newtokens[j][0] == ADD:
                    offset, add = newtokens[j][1]
                    if offset in sets:
                        sets[offset] += add
                    else:
                        adds[offset] = adds.get(offset, 0) + add
                elif newtokens[j][0] == SET and newtokens[j][1][0] != 0:
                    offset, val = newtokens[j][1]
                    if offset in adds:
                        del adds[offset]
                    sets[offset] = val
                else:
                    break
                j += 1
            else:
                if 0 not in adds:
                    pass
                    # print("Warning: Infinite loop detected.")
                elif len(adds) == 1:
                    if sets:
                        newtokens2.append((IF, 0))
                        for offset, val in sets.items():
                            newtokens2.append((SET, (offset, val)))
                    newtokens2.append((SET, (0, 0)))
                    if sets:
                        newtokens2.append((ENDIF, None))
                    i = j
                    optimized = True
                elif adds[0] == -1:
                    if sets:
                        newtokens2.append((IF, 0))
                        for offset, val in sets.items():
                            newtokens2.append((SET, (offset, val)))
                    for k, v in adds.items():
                        if k != 0:
                            newtokens2.append((MULCOPY, (0, k, v)))
                    if sets:
                        newtokens2.append((ENDIF, None))
                    newtokens2.append((SET, (0, 0)))
                    i = j
                    optimized = True

        # SET + MULCOPY = SET + ADD
        if (not optimized and
             i < len(newtokens)-1 and
             newtokens[i][0] == SET and
             newtokens[i+1][0] == MULCOPY and
             newtokens[i][1][0] == newtokens[i+1][1][0]):

            offset, value = newtokens[i][1]
            j = i+1
            while (j < len(newtokens)-1 and
                   newtokens[j][0] == MULCOPY and
                   newtokens[j][1][0] == offset):
                src, dest, mul = newtokens[j][1]
                newtokens2.append((ADD, (dest, value*mul)))
                j += 1

            newtokens2.append((SET, (offset, value)))
            i = j - 1
            optimized = True

        if (not optimized and
             i < len(newtokens)-2 and
             newtokens[i][0] in (LOADOUT, LOADOUTSET) and
             newtokens[i+1][0] == OUTPUT and
             newtokens[i+2][0] in (LOADOUT, LOADOUTSET)):
           
            newtokens2.append(newtokens[i])
            newtokens2.append(newtokens[i+2])
            i += 2
            optimized = True

        # Optimize ADD/SET/MOVE + OUTPUT + ADD/SET/MOVE
        if (not optimized and
             i < len(newtokens)-2 and
             newtokens[i][0] in (ADD, MOVE, SET)):
            j = i
            outputs = []
            adds = {}
            sets = {}
            shift = 0
            shifted = False
            while j < len(newtokens):
                if newtokens[j][0] == ADD:
                    offset, val = newtokens[j][1]
                    offset += shift
                    adds[offset] = adds.get(offset, 0) + val
                elif newtokens[j][0] == SET:
                    offset, val = newtokens[j][1]
                    offset += shift
                    adds[offset] = 0
                    sets[offset] = val
                elif newtokens[j][0] == LOADOUT:
                    offset, add = newtokens[j][1]
                    offset += shift
                    if offset in sets:
                        val = sets[offset] + adds.get(offset, 0) + add
                        outputs.append((None, None, val))
                    else:
                        outputs.append((offset, adds.get(offset, 0) + add, None))
                elif newtokens[j][0] == LOADOUTSET:
                    value = newtokens[j][1]
                    outputs.append((None, None, value))
                elif newtokens[j][0] == MOVE:
                    shift += newtokens[j][1]
                    shifted = True
                elif newtokens[j][0] == OUTPUT:
                    pass
                else:
                    j -= 1
                    break
                j += 1

            if (adds or shifted or sets) and outputs:
                for offset, add, _set in outputs:
                    if _set is not None:
                        newtokens2.append((LOADOUTSET, _set))
                    else:
                        newtokens2.append((LOADOUT, (offset, add)))
                newtokens2.append((OUTPUT, None))
                for offset, val in sets.items():
                    val = val + adds.get(offset, 0)
                    newtokens2.append((SET, (offset, val)))
                for offset, add in adds.items():
                    if add and (offset not in sets):
                        newtokens2.append((ADD, (offset, add)))
                if shift:
                    newtokens2.append((MOVE, shift))
                i = j
                optimized = True

        if not optimized:
            newtokens2.append(newtokens[i])

        i += 1

    # Optimize various things
    newtokens3 = []
    shift = 0
    # With normal dict, the order sometimes switches
    # in recursion, and the optimized never exits.
    vals = collections.OrderedDict()
    ops = {}
    for token, value in newtokens2:
        if token not in (SET, ADD):
            for k, v in vals.items():
                newtokens3.append((ops[k], (k, v)))
            vals.clear()
            ops.clear()

        if token in (SET, ADD):
            offset, val = value
            offset += shift
            # ADD/SET then SET does nothing; remove it
            if offset in ops and token == SET:
                ops.pop(offset)
                vals.pop(offset)
            if offset not in ops:
                ops[offset] = token # SET or ADD
            vals[offset] = vals.get(offset, 0) + val
        elif token == MULCOPY:
            src, dest, mul = value
            newtokens3.append((MULCOPY, (src+shift, dest+shift, mul)))
        # XXX Deal with shift in if
        elif token == IF:
            offset = shift + value
            newtokens3.append((IF, offset))
        elif token == ENDIF:
            newtokens3.append((ENDIF, None))
        elif token == MOVE:
            shift += value
        elif token == OUTPUT:
            newtokens3.append((token, value))
        elif token in (LOOP, ENDLOOP, INPUT, SCAN, LOADOUT, LOADOUTSET):
            if shift:
                newtokens3.append((MOVE, shift))
                shift = 0
            newtokens3.append((token, value))
        else:
            raise ValueError('What is this ' + str(token) + ' doing here?')

    # Any remaining add/set/shift is ignored, as it would have no effect

    # Optimize recursively
    if newtokens3 != tokens:
        return optimize(newtokens3)

    return newtokens3
