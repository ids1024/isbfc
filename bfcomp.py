import sys
import os
import subprocess

from parser import parse, optimize
from parser import OUTPUT, INPUT, LOOP, ENDLOOP, MOVE
from parser import ADD, SET, MULCOPY, SCAN

BUFSIZE = 8192

def compile(code):
    tokens = parse(code)
    tokens = optimize(tokens)
    output = """.section .bss
    .lcomm strbuff, 256
    .lcomm mem, """ + str(BUFSIZE) + """
    .set startidx, mem + """ + str(int(BUFSIZE/2)) + """
.section .text
.global _start
_start:
    xor %r12, %r12
    movq $startidx, %rbx
"""
    loops = []
    loopnum = 0
    for token, value in tokens:
        if token == ADD:
            offset, value = value
            if offset == 0:
                dest = "%r12"
            else:
                dest = str(offset*8)+"(%rbx)"
            if value == 1 and dest == "%r12":
                output += "    inc " + dest + "\n"
            elif value >= 1:
                output += "    addq $" + str(value) + ", " + dest + "\n"
            elif value == -1 and dest == "%r12":
                output += "    dec " + dest + "\n"
            elif value <= -1:
                output += "    subq $" + str(-value) + ", " + dest + "\n"
        elif token == MULCOPY:
            src, dest, mul = value
            if src == 0:
                src = "%r12"
            else:
                src = str(src*8)+"(%rbx)"
            if dest == 0:
                dest = "%r12"
            else:
                dest = str(dest*8)+"(%rbx)"

            if mul not in (-1, 1):
                output += "    movq " + src + ", %rax\n" \
                          "    movq $" + str(abs(mul)) + ", %rdx\n" \
                          "    mulq %rdx\n"
                src = "%rax"
            if mul > 0:
                output += "    addq " + src + ", " + dest + "\n"
            else:
                output += "    subq " + src + ", " + dest + "\n"
                
        elif token == SET:
                offset, value = value
                if offset == 0 and value == 0:
                    output += "    xor %r12, %r12\n"
                elif offset == 0:
                    output += "    movq $" + str(value) + ", %r12\n"
                else:
                    output += "    movq $" + str(value) + ", "+str(offset*8)+"(%rbx)\n"
        elif token == MOVE:
            if value:
                output += "    movq %r12, (%rbx)\n"
                if value > 0:
                    output += "    addq $" + str(8*value) + ", %rbx\n"
                else:
                    output += "    subq $" + str(-8*value) + ", %rbx\n"
                output += "    movq (%rbx), %r12\n"
        elif token == LOOP:
            loopnum += 1
            loops.append(loopnum)
            output += "    jmp endloop" + str(loopnum) + '\n' \
                      "    loop" + str(loopnum) + ":\n"
        elif token == ENDLOOP:
            loop = str(loops.pop())
            output += "    endloop" + loop + ':\n' \
                      "    test %r12, %r12\n" \
                      "    jnz loop" + loop + '\n'
        elif token == SCAN:
            # Slighly more optimal than normal loop and move
            loopnum += 1

            output += "    movq %r12, (%rbx)\n" \
                      "    jmp endloop" + str(loopnum) + '\n' \
                      "    loop" + str(loopnum) + ":\n"
            if value > 0:
                output += "    addq $" + str(8*value) + ", %rbx\n"
            else:
                output += "    subq $" + str(-8*value) + ", %rbx\n"
            output += "    endloop" + str(loopnum) + ':\n' \
                      "    cmp $0, (%rbx)\n" \
                      "    jnz loop" + str(loopnum) + '\n' \
                      "    movq (%rbx), %r12\n"

        elif token == INPUT:
            output += """
    xor %rax, %rax
    xor %rdi, %rdi
    movq %rbx, %rsi
    movq $1, %rdx
    syscall
    movq (%rbx), %r12

"""
        elif token == OUTPUT:
            times = value
            if times == 1:
                addr = "%rbx"
                output += "    movq %r12, (%rbx)\n"
            else:
                addr = "$strbuff"
                output += "    movq %r12, (strbuff)\n"
                for i in range(1, times):
                    output += "    movq %r12, (strbuff+" + str(i*8) + ")\n"
            output += """
    movq $1, %rax
    movq $1, %rdi
    movq """ + addr + """, %rsi
    movq $1, %rdx
    syscall

"""

    # Exit syscall
    output += """

    movq $60, %rax
    movq $0, %rdi
    syscall
"""

    return output

if __name__ == '__main__':
    print("Compiling...")
    with open(sys.argv[1]) as bffile:
        output = compile(bffile.read())
    name = os.path.splitext(sys.argv[1])[0]
    with open(name + '.s', 'w') as asmfile:
        asmfile.write(output)
    print("Assembling...")
    status = subprocess.call(['as', '-g', name+'.s', '-o', name+'.o'])
    if status == 0:
        print("Linking...")
        subprocess.call(['ld', name+'.o', '-o', name])
