import sys
import os
import subprocess

def compile(code):
    output = """.section .bss
    .lcomm mem, 8192
    .set startidx, mem + 4096
.section .text
.global _start
_start:
    movq $0, %r12
    movq $startidx, %rbx
"""
    code = ''.join(i for i in code if i in '.,[]+-<>')
    loops = []
    loopnum = 0
    for i in code:
        if i == '+':
            output += "    inc %r12\n"
        elif i == '-':
            output += "    dec %r12\n"
        elif i == '>':
            output += "    movq %r12, (%rbx)\n" \
                      "    add $8, %rbx\n" \
                      "    movq (%rbx), %r12\n"
        elif i == '<':
            output += "    movq %r12, (%rbx)\n" \
                      "    sub $8, %rbx\n" \
                      "    movq (%rbx), %r12\n"
        elif i == '[':
            loopnum += 1
            loops.append(loopnum)
            output += "    loop" + str(loopnum) + ":\n"
        elif i == ']':
            output += "    cmp $0, %r12\n" \
                      "    jnz loop" + str(loops.pop()) + '\n'
        elif i == ',':
            output += """
    movq $0, %rax
    movq $0, %rdi
    movq %rbx, %rsi
    movq $1, %rdx
    syscall
    movq (%rbx), %r12

"""
        elif i == '.':
            output += """
    movq %r12, (%rbx)
    movq $1, %rax
    movq $1, %rdi
    movq %rbx, %rsi
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
