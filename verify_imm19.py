#!/usr/bin/env python3
def calc_bcond_imm19(pc_current, target):
    """
    计算 AArch64 B.cond 指令的 imm19

    pc_current: 当前指令地址（字节）
    target: 目标 label 地址（字节）

    返回 imm19 （整数）和机器码的 imm19 字段（19位补码）
    """
    pc = pc_current
    offset_bytes = target - pc if target > pc_current else pc - target
    # 偏移单位是 4 字节
    imm19 = offset_bytes // 2
    # 将 imm19 转换为 19 位补码表示
    if target < pc_current:
        imm19_bin = 0x100000 - imm19
    else:
        imm19_bin = imm19

    # 生成机器码示例（B.cond opcode = 0b01010100 0000..., cond = 0, imm19占19位）
    # 这里只演示 imm19 部分
    print(f"PC: 0x{pc_current:X}, Target: 0x{target:X}")
    print(f"Offset bytes: {offset_bytes} -> imm19: {imm19}")
    print(f"imm19 19-bit binary (hex) = 0x{imm19_bin:X}")
    return imm19, imm19_bin


calc_bcond_imm19(int(input(),16),int(input(),16))
