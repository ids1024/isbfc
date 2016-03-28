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
    for token, value in tokens:
        if add and token != ADD:
            newtokens.append((ADD, (0, add)))
            add = 0
        elif move and token != MOVE:
            newtokens.append((MOVE, move))
            move = 0

        if token == ADD and value[0] == 0:
            add += value[1]
        elif token == MOVE:
            move += value
        else:
            newtokens.append((token, value))
    if add:
        newtokens.append((ADD, (0, add)))
    elif move:
        newtokens.append((MOVE, move))

    i = 0
    while i < len(newtokens):
        # Optimize out clear loop / multiply move loop
        if newtokens[i][0] == LOOP:
            j = i + 1
            adds = {}
            while j < len(newtokens) and newtokens[j][0] != ENDLOOP:
                if newtokens[j][0] != ADD:
                    break
                offset, add = newtokens[j][1]
                adds[offset] = adds.get(offset, 0) + add
                j += 1
            else:
                if 0 not in adds:
                    pass
                    # print("Warning: Infinite loop detected.")
                elif len(adds) == 1:
                    del newtokens[i:j+1]
                    newtokens.insert(i, (SET, (0, 0)))
                elif adds[0] == -1:
                    del adds[0]
                    del newtokens[i:j+1]
                    for k, v in adds.items():
                        newtokens.insert(i, (MULCOPY, (0, k, v)))
                        i += 1
                    newtokens.insert(i, (SET, (0, 0)))

        # SET + ADD = SET
        if (i < len(newtokens)-1 and
             newtokens[i][0] == SET and
             newtokens[i+1][0] == ADD and
             newtokens[i][1][0] == newtokens[i+1][1][0]):

            offset = newtokens[i][1][0]
            value = newtokens[i][1][1] + newtokens[i+1][1][1]
            del newtokens[i:i+2]
            newtokens.insert(i, (SET, (offset, value)))

        # Optimize MOVE + (SET/ADD) + MOVE -> (SET/ADD) + MOVE
        if (i < len(newtokens)-2 and
             newtokens[i][0] == MOVE and
             newtokens[i+1][0] in (SET, ADD)):

            vals = {}
            ops = {}
            j = i + 1
            while j < len(newtokens) and newtokens[j][0] != MOVE:
                if newtokens[j][0] not in (SET, ADD):
                    break
                offset, val = newtokens[j][1]
                # ADD then SET does nothing; remove it
                if ops.get(offset) == ADD and newtokens[j][0] == SET:
                    ops.pop(offset)
                    vals.pop(offset)
                if offset not in ops:
                    ops[offset] = newtokens[j][0] # SET or ADD
                vals[offset] = vals.get(offset, 0) + val
                j += 1
            else:
                offset = newtokens[i][1]
                move = offset + newtokens[j][1]
                del newtokens[i:j+1]
                for k, v in vals.items():
                    opp = ops[k]
                    newtokens.insert(i, (opp, (offset+k, v)))
                    i += 1
                if move:
                    newtokens.insert(i, (MOVE, move))

        # Optimize scan loop
        if (i < len(newtokens)-2 and
             newtokens[i][0] == LOOP and
             newtokens[i+1][0] == MOVE and
             newtokens[i+2][0] == ENDLOOP):

            offset = newtokens[i+1][1]
            del newtokens[i:i+3]
            newtokens.insert(i, (SCAN, offset))

        if (i < len(newtokens)-2 and
             newtokens[i][0] in (LOADOUT, LOADOUTSET) and
             newtokens[i+1][0] == OUTPUT and
             newtokens[i+2][0] in (LOADOUT, LOADOUTSET)):
           
            del newtokens[i+1]

        # Optimize ADD/MOVE + OUTPUT + ADD/MOVE
        if (i < len(newtokens)-2 and
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
                del newtokens[i:j+1]
                for offset, add, _set in outputs:
                    if _set is not None:
                        newtokens.insert(i, (LOADOUTSET, _set))
                    else:
                        newtokens.insert(i, (LOADOUT, (offset, add)))
                    i += 1
                newtokens.insert(i, (OUTPUT, None))
                i += 1
                for offset, val in sets.items():
                    val = val + adds.get(offset, 0)
                    newtokens.insert(i, (SET, (offset, val)))
                    i += 1
                for offset, add in adds.items():
                    if add and (offset not in sets):
                        newtokens.insert(i, (ADD, (offset, add)))
                        i += 1
                if shift:
                    newtokens.insert(i, (MOVE, shift))
                    i += 1

        i += 1

    # Optimize recursively
    if newtokens != tokens:
        return optimize(newtokens)

    return newtokens
