#!/usr/bin/env python3
import re
import sys
from pathlib import Path

CODE_RE = re.compile(r'//\s*code\s*x([0-9a-fA-F]{2})')
BYTE_RE = re.compile(r'0b([01]{8})')

def pack_u32_le(bs: list[int]) -> int:
    """Pack 4 bytes en little-endian:
    [b0, b1, b2, b3] -> 0xB3B2B1B0
    Exemple: [0x6C, 0xFE, 0xFE, 0xFE] -> 0xFEFEFE6C
    """
    return (bs[0] << 24) | (bs[1] << 16) | (bs[2] <<8) | (bs[3] << 0)

def parse_codes(text: str) -> dict[int, list[int]]:
    lines = text.splitlines()
    i = 0
    result = {}

    while i < len(lines):
        m = CODE_RE.search(lines[i])
        if not m:
            i += 1
            continue

        code = int(m.group(1), 16)
        i += 1

        bytes_ = []
        while i < len(lines) and len(bytes_) < 16:
            bm = BYTE_RE.search(lines[i])
            if bm:
                bytes_.append(int(bm.group(1), 2))
            i += 1

        if len(bytes_) != 16:
            raise ValueError(f"Code x{code:02X}: attendu 16 octets, trouvé {len(bytes_)}")

        result[code] = bytes_

    return result

def generate_c_for_char(code: int, bytes_: list[int]) -> str:
    r1 = pack_u32_le(bytes_[0:4])
    r2 = pack_u32_le(bytes_[4:8])
    r3 = pack_u32_le(bytes_[8:12])
    r4 = pack_u32_le(bytes_[12:16])

    return (
        f"if (char == {code}) {{\n"
        f"    u32 r1 = 0x{r1:08X};\n"
        f"    u32 r2 = 0x{r2:08X};\n"
        f"    u32 r3 = 0x{r3:08X};\n"
        f"    u32 r4 = 0x{r4:08X};\n"
        f"}}"
    )

def generate_c_switch(codes: dict[int, list[int]]) -> str:
    out = []
    for code in sorted(codes):
        bs = codes[code]
        r1 = pack_u32_le(bs[0:4])
        r2 = pack_u32_le(bs[4:8])
        r3 = pack_u32_le(bs[8:12])
        r4 = pack_u32_le(bs[12:16])

        out.append(f"if (char == {code}) {{")
        out.append(f"    r1 = 0x{r1:08X};")
        out.append(f"    r2 = 0x{r2:08X};")
        out.append(f"    r3 = 0x{r3:08X};")
        out.append(f"    r4 = 0x{r4:08X};")
        out.append(f"}}")
    return "\n".join(out)

def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} fichier.c [code_hex]", file=sys.stderr)
        sys.exit(1)

    path = Path(sys.argv[1])
    text = path.read_text(encoding="utf-8")
    codes = parse_codes(text)

    if len(sys.argv) >= 3:
        code = int(sys.argv[2], 0)
        if code not in codes:
            print(f"Code {code} (0x{code:02X}) introuvable", file=sys.stderr)
            sys.exit(1)
        print(generate_c_for_char(code, codes[code]))
    else:
        print(generate_c_switch(codes))

if __name__ == "__main__":
    main()