OUTPUT=0
INPUT=1
LOOPSTART=2
LOOPEND=3
MOVE=4
ADD=5
SET=6
MULCOPY=7
SCAN=8

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
            tokens.append((LOOPSTART, None))
        elif i == ']':
            tokens.append((LOOPEND, None))
        elif i == ',':
            tokens.append((INPUT, None))
        if i == '.':
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

    # Optimize out clear loop
    i = 0
    while i < len(newtokens):
        if newtokens[i][0] == LOOPSTART:
            j = i + 1
            while j < len(newtokens) and newtokens[j][0] != LOOPEND:
                if newtokens[j][0] != ADD:
                    break
                j += 1
            else:
                del newtokens[i:j+1]
                # ADD before clear does nothing, so remove it
                if i>0 and newtokens[i-1][0] == ADD:
                    del newtokens[i-1]
                    i -= 1
                value = 0
                # ADD after SET can be simplified to SET
                if (i<len(newtokens) and 
                        newtokens[i][0] == ADD and
                        newtokens[i][1][0] == 0):
                    value = newtokens[i][1][1]
                    del newtokens[i]
                newtokens.insert(i, (SET, (0, value)))
        i += 1

    # Optimize MOVE + SET + MOVE and MOVE + ADD + MOVE
    i = 0
    while i < len(newtokens)-2:
        if (newtokens[i][0] == MOVE and
             newtokens[i+1][0] in (SET, ADD) and
             newtokens[i+2][0] == MOVE):

            opp = newtokens[i+1][0] # SET or ADD
            value = newtokens[i+1][1][1]
            offset = newtokens[i+1][1][0] + newtokens[i][1]
            move = offset + newtokens[i+2][1] - newtokens[i+1][1][0]
            del newtokens[i:i+3]
            newtokens.insert(i, (opp, (offset, value)))
            if move:
                newtokens.insert(i+1, (MOVE, move))

        i += 1

    # Optimize copy/multiplication
    i = 0
    while i < len(newtokens):
        if newtokens[i][0] == LOOPSTART:
            j = i + 1
            adds = {}
            while j < len(newtokens) and newtokens[j][0] != LOOPEND:
                if newtokens[j][0] != ADD:
                    break
                offset, add = newtokens[j][1]
                adds[offset] = adds.get(offset, 0) + add
                j += 1
            else:
                if 0 not in adds:
                    print("Warning: Infinite loop detected.")
                elif adds[0] == -1:
                    del adds[0]
                    del newtokens[i:j+1]
                    for k, v in adds.items():
                        newtokens.insert(i, (MULCOPY, (0, k, v)))
                        i += 1
                    newtokens.insert(i, (SET, (0, 0)))
        i += 1

    # Optimize scan loop
    i = 0
    while i < len(newtokens)-2:
        if (newtokens[i][0] == LOOPSTART and
             newtokens[i+1][0] == MOVE and
             newtokens[i+2][0] == LOOPEND):

            offset = newtokens[i+1][1]
            del newtokens[i:i+3]
            newtokens.insert(i, (SCAN, offset))
        i += 1

    # Optimize recursively
    if newtokens != tokens:
        return optimize(newtokens)

    return newtokens
