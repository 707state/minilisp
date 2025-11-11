from capstone import *

if __name__ == "__main__":
    hex_instructions = [
    "d28007c0",
    "92400400",
    "f1000c1f",
    "9a9f17e0",
    "d37be800",
    "b2400400",
    "d65f03c0",
    ]

    # 转换成 bytes，小端序
    CODE = b''.join(int(x, 16).to_bytes(4, 'little') for x in hex_instructions)

    md = Cs(CS_ARCH_ARM64, CS_MODE_LITTLE_ENDIAN)
    for i in md.disasm(CODE, 0x0):
        print(f"0x{i.address:08x}:\t{i.bytes.hex()}\t{i.mnemonic}\t{i.op_str}")
