# SPDX-FileCopyrightText: LakeSoul Contributors
#
# SPDX-License-Identifier: Apache-2.0

from pathlib import Path
import re
import shutil


def list_pdf_files(directory, target_base: str):
    # 使用 pathlib 遍历目录
    path = Path(directory)
    for pdf_file in path.glob("*.pdf"):  # 查找所有 PDF 文件
        # 检查文件名是否包含 "lecture"
        pattern = r"\d+"  # -?表示可选的负号，\.?表示可选的小数点
        numbers = str(int("".join(re.findall(pattern, pdf_file.name))))
        shutil.copy(pdf_file, f"{target_base}-{numbers}")

        print(f"Found PDF file: {pdf_file.name}, {numbers}")


if __name__ == "__main__":
    # 获取当前工作目录
    current_directory = Path.cwd()
    list_pdf_files(current_directory, target_base="./cs70/lecture")
