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
    # Optimize various things
    newtokens = []
    shift = 0
    # With normal dict, the order sometimes switches
    # in recursion, and the optimized never exits.
    vals = collections.OrderedDict()
    ops = {}
    do_output = False
    for token, value in tokens:
        if token not in (SET, ADD, MOVE, LOADOUT, LOADOUTSET):
            if do_output:
                newtokens.append((OUTPUT, None))
                do_output = False

            for k, v in vals.items():
                newtokens.append((ops[k], (k, v)))
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
            newtokens.append((MULCOPY, (src+shift, dest+shift, mul)))
        # XXX Deal with shift in if
        elif token == IF:
            offset = shift + value
            newtokens.append((IF, offset))
        elif token == ENDIF:
            newtokens.append((ENDIF, None))
        elif token == MOVE:
            shift += value
        elif token == OUTPUT:
            do_output = True
        elif token == LOADOUT:
            offset, add = value
            offset += shift
            if ops.get(offset) == SET:
                newtokens.append((LOADOUTSET, vals[offset] + add))
            elif ops.get(offset) == ADD:
                newtokens.append((LOADOUT, (offset, vals[offset] + add)))
            else:
                newtokens.append((LOADOUT, (offset, add)))
        elif token == LOADOUTSET:
            newtokens.append((token, value))
        elif token in (LOOP, ENDLOOP, INPUT, SCAN):
            if shift:
                newtokens.append((MOVE, shift))
                shift = 0
            newtokens.append((token, value))
        else:
            raise ValueError('What is this ' + str(token) + ' doing here?')

    # Any remaining add/set/shift is ignored, as it would have no effect
    if do_output:
        newtokens.append((OUTPUT, None))

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

        if not optimized:
            newtokens2.append(newtokens[i])

        i += 1

    # Optimize recursively
    if newtokens2 != tokens:
        return optimize(newtokens2)

    return newtokens2
