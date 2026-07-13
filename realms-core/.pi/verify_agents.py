#!/usr/bin/env python3
"""验证 .pi/agents/*.md 模板可被加载。

按 pi agent 格式:
- 顶级 frontmatter (--- 包围)
- 必要字段: name, description, tools (可选 model)
- 正文是 system prompt

零依赖。仅 stdlib。
"""

import re
import sys
from pathlib import Path

REQUIRED_FIELDS = ("name", "description", "tools")
VALID_TOOLS = {
    "read", "grep", "find", "ls", "bash", "edit", "write",
    "glob", "task", "webfetch", "websearch",
}


def parse_frontmatter(path: Path) -> dict:
    """解析 markdown 顶部的 YAML-ish frontmatter。

    简化解析(不依赖 PyYAML):支持 `key: value` 一行格式。
    多行/嵌套留给 pi 自己。
    """
    text = path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        raise ValueError(f"{path.name}: 缺 frontmatter '---' 起头")

    end = text.find("\n---\n", 4)
    if end < 0:
        raise ValueError(f"{path.name}: 缺 frontmatter '---' 收尾")

    block = text[4:end]
    out = {}
    for line in block.splitlines():
        line = line.rstrip()
        if not line or ":" not in line:
            continue
        key, _, val = line.partition(":")
        out[key.strip()] = val.strip()
    return out


def verify_agent(path: Path) -> list[str]:
    """返回错误列表(空 = 通过)。"""
    errors = []
    try:
        meta = parse_frontmatter(path)
    except ValueError as e:
        return [str(e)]

    for field in REQUIRED_FIELDS:
        if field not in meta or not meta[field]:
            errors.append(f"  缺字段: {field}")

    name = meta.get("name", "")
    if not re.match(r"^[a-z0-9][a-z0-9-]{0,40}$", name):
        errors.append(f"  name 格式错: '{name}' (要求小写字母/数字/连字符, ≤ 41 字符)")

    tools_str = meta.get("tools", "")
    tools = [t.strip() for t in tools_str.split(",") if t.strip()]
    for t in tools:
        if t not in VALID_TOOLS:
            errors.append(f"  未知 tool: '{t}'")

    body = path.read_text(encoding="utf-8")
    body = body.split("\n---\n", 1)[1] if "\n---\n" in body else ""
    if len(body.strip()) < 100:
        errors.append(f"  正文 < 100 字符 (实际 {len(body.strip())})")

    return errors


def main() -> int:
    if len(sys.argv) < 2:
        agents_dir = Path(__file__).parent / "agents"
    else:
        agents_dir = Path(sys.argv[1])

    if not agents_dir.is_dir():
        print(f"✗ 目录不存在: {agents_dir}")
        return 1

    md_files = sorted(agents_dir.glob("*.md"))
    if not md_files:
        print(f"✗ {agents_dir} 无 .md 模板")
        return 1

    print(f"=== 验证 {len(md_files)} 个 subagent 模板 ===")
    print(f"目录: {agents_dir}\n")

    all_ok = True
    seen_names = set()
    for path in md_files:
        errors = verify_agent(path)
        if errors:
            all_ok = False
            print(f"✗ {path.name}")
            for e in errors:
                print(e)
        else:
            meta = parse_frontmatter(path)
            tools = meta.get("tools", "")
            name = meta.get("name", "")
            if name in seen_names:
                all_ok = False
                print(f"✗ {path.name}: name 重复 '{name}'")
                continue
            seen_names.add(name)
            print(f"✓ {path.name}  name='{name}'  tools='{tools}'")

    print()
    if all_ok:
        print(f"=== 全部 {len(md_files)} 个模板通过验证 ===")
        return 0
    else:
        print("=== 验证失败 ===")
        return 1


if __name__ == "__main__":
    sys.exit(main())
