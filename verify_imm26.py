#!/usr/bin/env python3
def calc_b_imm26(pc_current, target):
    """
    计算 AArch64 B 指令的 imm26
    pc_current: 当前指令地址（字节）
    target: 目标 label 地址（字节）
    """
    pc = pc_current
    offset_bytes = target - pc if target > pc_current else pc - target
    imm26 = offset_bytes // 4
    if target < pc_current:
        imm26_bin = 0x10000000 - imm26
    else:
        imm26_bin = imm26
    print(f"PC: 0x{pc_current:X}, Target: 0x{target:X}")
    print(f"Offset bytes: {offset_bytes} -> imm26: {imm26}")
    print(f"imm26 26-bit binary (hex) = 0x{imm26_bin:X}")
    return imm26, imm26_bin


calc_b_imm26(int(input(),16),int(input(),16))
