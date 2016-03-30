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
    adds = collections.OrderedDict()
    sets = collections.OrderedDict()
    do_output = False
    for token, value in tokens:
        # Optimize out clear loop / multiply move loop
        if (token == ENDLOOP and newtokens[-1][0] == LOOP and
                not shift and 0 in adds):
            if len(adds) == 1:
                newtokens.pop() # Remove LOOP
                if sets:
                    newtokens.append((IF, 0))
                    for offset, val in sets.items():
                        newtokens.append((SET, (offset, val)))
                newtokens.append((SET, (0, 0)))
                if sets:
                    newtokens.append((ENDIF, None))
                adds.clear()
                sets.clear()
                continue
            elif adds[0] == -1:
                newtokens.pop() # Remove LOOP
                if sets:
                    newtokens.append((IF, 0))
                    for k, v in sets.items():
                        newtokens.append((SET, (k, v)))
                for k, v in adds.items():
                    if k != 0:
                        newtokens.append((MULCOPY, (0, k, v)))
                if sets:
                    newtokens.append((ENDIF, None))
                newtokens.append((SET, (0, 0)))
                adds.clear()
                sets.clear()
                continue

        if token not in (SET, ADD, MOVE, LOADOUT, LOADOUTSET, OUTPUT):
            if do_output:
                newtokens.append((OUTPUT, None))
                do_output = False

            for k, v in sets.items():
                newtokens.append((SET, (k, v)))
            for k, v in adds.items():
                newtokens.append((ADD, (k, v)))
            sets.clear()
            adds.clear()
        if shift and token in (LOOP, INPUT, SCAN):
            newtokens.append((MOVE, shift))
            shift = 0

        if token == SET:
            offset, val = value
            offset += shift
            # ADD then SET does nothing; remove it
            if offset in adds and token == SET:
                adds.pop(offset)
            sets[offset] = val
        elif token == ADD:
            offset, val = value
            offset += shift
            if offset in sets:
                sets[offset] = sets[offset] + val
            else:
                adds[offset] = adds.get(offset, 0) + val
        elif token == MULCOPY:
            src, dest, mul = value
            newtokens.append((MULCOPY, (src+shift, dest+shift, mul)))
        # XXX Deal with shift in if
        elif token == IF:
            offset = shift + value
            newtokens.append((IF, offset))
        elif token == MOVE:
            shift += value
        elif token == OUTPUT:
            do_output = True
        elif token == LOADOUT:
            offset, add = value
            offset += shift
            if offset in sets:
                newtokens.append((LOADOUTSET, sets[offset] + add))
            elif offset in adds:
                newtokens.append((LOADOUT, (offset, adds[offset] + add)))
            else:
                newtokens.append((LOADOUT, (offset, add)))
        elif token == ENDLOOP:
            if newtokens[-1][0] == LOOP and shift and not (sets or adds):
                newtokens.pop() # Remove STARTLOOP
                newtokens.append((SCAN, shift))
            else:
                if shift:
                    newtokens.append((MOVE, shift))
                newtokens.append((ENDLOOP, None))
            shift = 0
        elif token in (ENDIF, LOADOUTSET, LOOP, INPUT, SCAN):
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
