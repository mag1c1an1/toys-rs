# SPDX-FileCopyrightText: LakeSoul Contributors
#
# SPDX-License-Identifier: Apache-2.0
from pathlib import Path


def dirname(start: int, end: int, base: str):
    return [f"{base}-{i}" for i in range(start, end + 1)]


def create_dir(path):
    try:
        Path(path).mkdir(parents=True, exist_ok=True)  # parents=True 表示创建所有父目录
        print(f"Directory '{path}' created successfully.")
    except Exception as e:
        print(f"Error creating directory '{path}': {e}")


if __name__ == "__main__":
    names = dirname(0, 24, "lecture")
    paths = [f"./cs70/{name}" for name in names]
    for path in paths:
        create_dir(path)
