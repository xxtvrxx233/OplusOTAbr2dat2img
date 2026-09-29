#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
OplusOTAbr2dat2img

把 Oplus（OPPO / 一加 / Realme）OTA 包中的
    <分区>.new.dat.br
    <分区>.transfer.list
还原为可直接刷入的原始分区镜像 out/<分区>.img。

仅支持 A-only 动态分区设备。

镜像还原使用同目录下的 sdat2img.py（xpirt/sdat2img，MIT 协议），
其许可证见 LICENSES/sdat2img-MIT.txt。

设计约束（改动时请勿破坏）：
  * 不重命名 / 修改 / 移动用户的原始输入文件，全程只读。
  * 所有产物仅写入输出目录，该目录顶层只包含 <分区>.img，
    中间文件置于 <outdir>/.tmp/。
"""

import argparse
import contextlib
import ctypes
import io
import locale
import os
import re
import sys
import unicodedata
from concurrent.futures import ThreadPoolExecutor, as_completed

try:
    import brotli
except ImportError:  # 未安装时不应输出 traceback，交由 main() 给出安装指引
    brotli = None

try:
    import sdat2img
except ImportError:  # 缺文件时不应输出 traceback，交由 main() 给出提示
    sdat2img = None

VERSION = "2.0"

BR_SUFFIX = ".new.dat.br"
TL_SUFFIX = ".transfer.list"

# 流式解压的分块大小。不一次性读入整个 .new.dat：system 分区可达数 GB。
CHUNK_SIZE = 1 << 20

# --------------------------------------------------------------------------
# 多语言
# --------------------------------------------------------------------------

MESSAGES = {
    "zh": {
        "banner": "OplusOTAbr2dat2img v{v} — OTA .br 转换为 .img",
        "scanning": "扫描目录：{path}",
        "indir_missing": "错误：目录不存在：{path}",
        "no_sources": "未找到 *{suffix} 文件。\n请确认该目录中已包含 OTA 包的 .br 与 .list 文件。",
        "found": "识别到 {n} 个可转换的分区：",
        "col_idx": "编号",
        "col_part": "分区名",
        "col_dat": "数据文件",
        "col_tl": "transfer.list",
        "overwrite": "其中 {n} 个输出文件已存在，将被覆盖。",
        "unpaired_head": "未配对（跳过）：",
        "unpaired_item": "  - {file}（缺少对应的 {missing}）",
        "conflict_head": "文件名冲突（跳过）：",
        "conflict_group": "  - 分区名 {part} 对应多个文件，无法确定使用哪一个：",
        "conflict_item": "      {file}",
        "outdir": "输出目录：{path}",
        "confirm": "确认开始转换？[y/N] ",
        "aborted": "已取消，未做任何改动。",
        "sdat2img_missing": "错误：未找到 sdat2img.py。\n该文件必须与 br2dat2img.py 位于同一目录。",
        "brotli_missing": "错误：未安装 brotli 模块。\n  Debian / Ubuntu: apt install python3-brotli\n  Termux:          pkg install python-brotli\n  Arch:            pacman -S python-brotli\n  Windows:         pip install brotli",
        "stage1_head": "阶段 1/2：解压（{n} 个分区，并行数 {w}）",
        "stage1_ok": "  [ok] {part}",
        "stage2_head": "阶段 2/2：还原镜像",
        "stage2_item": "  [{i}/{n}] {part}",
        "ok_item": "  [ok] {part} -> {path}",
        "fail_item": "  [!!] {part} 失败：{err}",
        "err_brotli": "解压出错：{msg}",
        "err_sdat2img": "无法解析 transfer.list（详见上方错误信息）",
        "err_empty_img": "sdat2img 未生成有效镜像",
        "summary": "完成：{ok} 成功，{fail} 失败，{skip} 跳过。",
        "fail_head": "转换失败的分区：",
        "tmp_hint": "失败的中间文件保留在 {path}，可用于排查。",
        "all_skipped": "没有分区配对成功，未执行转换。",
        "arg_indir": "OTA 文件所在目录（默认：当前目录）",
        "arg_outdir": "镜像输出目录（默认：./out）",
        "arg_partition": "只转换指定的分区，可给多个；分区名或原始文件名均可（默认：全部）",
        "part_not_found": "错误：未找到以下分区：{names}\n可用分区：{available}",
        "part_none_available": "（无）",
        "part_incomplete": "注意：以下分区的文件不完整，已被跳过：{names}",
        "arg_lang": "界面语言（默认 auto：按环境变量与系统区域自动判定，回退为 en）",
        "arg_jobs": "并行解压的进程数（默认：CPU 核心数）",
        "arg_yes": "跳过确认，直接开始",
        "arg_version": "显示版本号并退出",
    },
    "en": {
        "banner": "OplusOTAbr2dat2img v{v} - OTA .br to .img converter",
        "scanning": "Scanning directory: {path}",
        "indir_missing": "Error: directory does not exist: {path}",
        "no_sources": "No *{suffix} file found.\nMake sure the directory contains the OTA .br and .list files.",
        "found": "Found {n} partition(s) ready to convert:",
        "col_idx": "#",
        "col_part": "Partition",
        "col_dat": "Data file",
        "col_tl": "transfer.list",
        "overwrite": "{n} output file(s) already exist and will be overwritten.",
        "unpaired_head": "Unpaired (skipped):",
        "unpaired_item": "  - {file} (no matching {missing})",
        "conflict_head": "Filename conflicts (skipped):",
        "conflict_group": "  - partition name {part} matches several files, cannot determine which one to use:",
        "conflict_item": "      {file}",
        "outdir": "Output directory: {path}",
        "confirm": "Start converting? [y/N] ",
        "aborted": "Aborted, nothing was changed.",
        "sdat2img_missing": "Error: sdat2img.py was not found.\nIt must be located in the same directory as br2dat2img.py.",
        "brotli_missing": "Error: the brotli module is not installed.\n  Debian / Ubuntu: apt install python3-brotli\n  Termux:          pkg install python-brotli\n  Arch:            pacman -S python-brotli\n  Windows:         pip install brotli",
        "stage1_head": "Stage 1/2: decompressing ({n} partitions, {w} workers)",
        "stage1_ok": "  [ok] {part}",
        "stage2_head": "Stage 2/2: rebuilding images",
        "stage2_item": "  [{i}/{n}] {part}",
        "ok_item": "  [ok] {part} -> {path}",
        "fail_item": "  [!!] {part} failed: {err}",
        "err_brotli": "decompression error: {msg}",
        "err_sdat2img": "cannot parse transfer.list (see the error above)",
        "err_empty_img": "sdat2img produced no valid image",
        "summary": "Done: {ok} succeeded, {fail} failed, {skip} skipped.",
        "fail_head": "Failed partitions:",
        "tmp_hint": "Intermediate files of failed partitions are kept in {path} for inspection.",
        "all_skipped": "No partition could be paired, nothing was converted.",
        "arg_indir": "directory containing the OTA files (default: current directory)",
        "arg_outdir": "directory to write the images to (default: ./out)",
        "arg_partition": "convert only the given partitions, one or more; accepts partition names or original filenames (default: all)",
        "part_not_found": "Error: no such partition: {names}\nAvailable: {available}",
        "part_none_available": "(none)",
        "part_incomplete": "Note: these partitions have incomplete files and were skipped: {names}",
        "arg_lang": "UI language (default auto: detect from environment and system locale, falls back to en)",
        "arg_jobs": "number of parallel decompression workers (default: CPU count)",
        "arg_yes": "skip the confirmation prompt",
        "arg_version": "show version and exit",
    },
}

LANG = "en"

# argparse 自己的框架文案（usage:、options:、错误提示等）由它内部用 gettext 生成，
# 默认只有英文，替换 argparse._ 才能让 --help 和参数报错跟着界面语言走。
ARGPARSE_ZH = {
    "usage: ": "用法: ",
    "options": "选项",
    "positional arguments": "位置参数",
    "subcommands": "子命令",
    "show this help message and exit": "显示此帮助信息并退出",
    " (default: %(default)s)": "（默认：%(default)s）",
    "%(prog)s: error: %(message)s\n": "%(prog)s: 错误: %(message)s\n",
    "%(prog)s: warning: %(message)s\n": "%(prog)s: 警告: %(message)s\n",
    "unrecognized arguments: %s": "无法识别的参数: %s",
    "the following arguments are required: %s": "缺少必需参数: %s",
    "expected one argument": "需要一个参数",
    "expected at most one argument": "最多接受一个参数",
    "expected at least one argument": "至少需要一个参数",
    "invalid choice: %(value)r (choose from %(choices)s)":
        "无效选项: %(value)r（可选值：%(choices)s）",
    "invalid choice: %(value)r, maybe you meant %(closest)r? ":
        "无效选项: %(value)r，你是指 %(closest)r 吗？",
    "invalid %(type)s value: %(value)r": "无效的 %(type)s 值: %(value)r",
    "not allowed with argument %s": "不可与参数 %s 同时使用",
    "one of the arguments %s is required": "需要以下参数之一: %s",
    "ambiguous option: %(option)s could match %(matches)s":
        "选项有歧义: %(option)s 可能是 %(matches)s",
    "argument %(argument_name)s: %(message)s": "参数 %(argument_name)s: %(message)s",
    "ignored explicit argument %r": "忽略了显式参数 %r",
    "unexpected option string: %s": "意外的选项字符串: %s",
    "unknown parser %(parser_name)r (choices: %(choices)s)":
        "未知的子解析器 %(parser_name)r（可选：%(choices)s）",
}

# 这两条 argparse 走 ngettext（单复数），中文不分单复数，取单数形式即可
ARGPARSE_ZH_PLURAL = {
    "expected %s argument": "需要 %s 个参数",
    "conflicting option string: %s": "冲突的选项字符串: %s",
}


def localize_argparse():
    """把 argparse 自身的框架文案换成当前语言。

    argparse 模块级 `from gettext import gettext as _`，其代码运行时查全局名 `_`，
    所以替换 `argparse._` 即可生效。非中文时保持原样。
    """
    if LANG != "zh":
        return
    argparse._ = lambda message: ARGPARSE_ZH.get(message, message)
    argparse.ngettext = lambda singular, plural, n: ARGPARSE_ZH_PLURAL.get(singular, singular)


def t(key, **kw):
    return MESSAGES[LANG][key].format(**kw)


def _is_chinese(tag):
    """zh / zh-CN / zh_CN.UTF-8 / zh-Hans-CN 都算中文，zhx_CN 不算。"""
    if not tag:
        return False
    tag = tag.replace("_", "-").split(".")[0].split("@")[0].lower()
    return tag == "zh" or tag.startswith("zh-")


def detect_lang():
    """参数 > 专用环境变量 > POSIX 标准变量 > 系统区域，回退为英文。"""
    val = os.environ.get("BR2DAT2IMG_LANG")
    if val:
        return "zh" if _is_chinese(val) else "en"

    # POSIX 规定的优先级：LC_ALL > LC_MESSAGES > LANG
    for var in ("LC_ALL", "LC_MESSAGES", "LANG"):
        val = os.environ.get(var)
        if val:
            return "zh" if _is_chinese(val) else "en"

    if sys.platform == "win32":
        # Windows 上 locale 模块返回的东西跟系统显示语言经常对不上，直接问 Win32。
        # LANGID 低 10 位是主语言 ID，0x04 即中文（简繁港澳新都算）。
        try:
            langid = ctypes.windll.kernel32.GetUserDefaultUILanguage()
            return "zh" if (langid & 0x3FF) == 0x04 else "en"
        except Exception:
            return "en"

    try:
        return "zh" if _is_chinese(locale.getlocale()[0]) else "en"
    except Exception:
        return "en"


# --------------------------------------------------------------------------
# 文件名解析
# --------------------------------------------------------------------------

_TRAILING_NUMBER = re.compile(r"\.\d+$")


def clean_partition_name(stem):
    """移除结尾的点分纯数字段（OPPO 添加的 NV 号），可连续多段。

        my_bigball.10010111          -> my_bigball
        my_bigball.10010111.10010112 -> my_bigball
        system_ext                   -> system_ext
        system                       -> system
    """
    while True:
        stripped = _TRAILING_NUMBER.sub("", stem)
        if stripped == stem:
            return stem
        stem = stripped


class Job(object):
    """一个待转换的分区：分区名 + 它的两个原始输入文件名。"""

    __slots__ = ("part", "br_name", "tl_name")

    def __init__(self, part, br_name, tl_name):
        self.part = part
        self.br_name = br_name
        self.tl_name = tl_name


def scan(workdir):
    """扫描目录，按分区名把 .br 和 .transfer.list 配对。

    返回 (可转换的分区列表, 未配对列表, 冲突列表)。
    """
    brs, tls = {}, {}
    for name in sorted(os.listdir(workdir)):
        if not os.path.isfile(os.path.join(workdir, name)):
            continue
        if name.endswith(BR_SUFFIX):
            bucket, stem = brs, name[: -len(BR_SUFFIX)]
        elif name.endswith(TL_SUFFIX):
            bucket, stem = tls, name[: -len(TL_SUFFIX)]
        else:
            continue
        bucket.setdefault(clean_partition_name(stem), []).append(name)

    ready, unpaired, conflicts = [], [], []

    for part in sorted(set(brs) | set(tls)):
        br_list, tl_list = brs.get(part, []), tls.get(part, [])

        # 同一分区名对应多个文件时无法确定使用哪一个，整组跳过
        if len(br_list) > 1 or len(tl_list) > 1:
            conflicts.append((part, br_list + tl_list))
            continue

        if br_list and tl_list:
            ready.append(Job(part, br_list[0], tl_list[0]))
        elif br_list:
            unpaired.append((br_list[0], TL_SUFFIX.lstrip(".")))
        else:
            unpaired.append((tl_list[0], BR_SUFFIX.lstrip(".")))

    return ready, unpaired, conflicts


def partition_of(filename):
    """从原始文件名反推分区名；文件名不认识时返回 None。"""
    for suffix in (BR_SUFFIX, TL_SUFFIX):
        if filename.endswith(suffix):
            return clean_partition_name(filename[: -len(suffix)])
    return None


def match_jobs(tokens, ready):
    """把命令行给出的分区标识换算成 Job。

    标识允许三种写法：分区名、原始数据文件名、原始 transfer.list 文件名。
    后两种是为了让用户能直接从工作目录里复制文件名粘贴过来。

    返回 (选中的 Job 列表, 无法解析的标识列表)，两者都保持用户给出的顺序。
    """
    index = {}
    for job in ready:
        for key in (job.part, job.br_name, job.tl_name):
            index[key] = job

    picked, unknown = [], []
    for token in tokens:
        job = index.get(token)
        if job is None:
            unknown.append(token)
        elif job not in picked:
            picked.append(job)
    return picked, unknown


# --------------------------------------------------------------------------
# 转换
# --------------------------------------------------------------------------

@contextlib.contextmanager
def quiet_stdout():
    """吞掉 sdat2img 的 stdout。

    它会逐 range 打印 'Copying N blocks into position ...'，真实 OTA 下达数千行。
    错误信息走 stderr，此处不作处理，失败原因仍然可见。
    """
    original = sys.stdout
    sys.stdout = io.StringIO()
    try:
        yield
    finally:
        sys.stdout = original


def decompress_one(job, indir, tmpdir):
    """阶段 1：brotli 解压。分块处理，内存占用与分区大小无关。"""
    br_path = os.path.join(indir, job.br_name)
    dat_path = os.path.join(tmpdir, job.part + ".new.dat")
    decompressor = brotli.Decompressor()
    try:
        with open(br_path, "rb") as src, open(dat_path, "wb") as dst:
            while True:
                chunk = src.read(CHUNK_SIZE)
                if not chunk:
                    break
                dst.write(decompressor.process(chunk))
    except (brotli.error, OSError) as exc:
        return job, False, t("err_brotli", msg=exc)
    return job, True, None


def convert_one(job, indir, outdir, tmpdir):
    """阶段 2：sdat2img 还原。

    必须串行：quiet_stdout() 替换的是进程级全局 sys.stdout，多线程并行时
    一个线程退出时会恢复另一个线程的重定向，sdat2img 的输出将泄漏到终端。
    """
    tl_path = os.path.join(indir, job.tl_name)
    dat_path = os.path.join(tmpdir, job.part + ".new.dat")
    img_path = os.path.join(outdir, job.part + ".img")

    try:
        with quiet_stdout():
            sdat2img.main(tl_path, dat_path, img_path)
    except (SystemExit, Exception):
        # sdat2img 解析失败时内部直接调用 sys.exit(1)。SystemExit 继承 BaseException
        # 而非 Exception，若不在此显式捕获，将导致整个进程直接退出。
        return job, False, t("err_sdat2img")

    if not os.path.isfile(img_path) or os.path.getsize(img_path) == 0:
        return job, False, t("err_empty_img")

    # 仅在成功时清理中间文件；失败残留的 .new.dat 保留在 .tmp/ 中以便排查
    try:
        os.remove(dat_path)
    except OSError:
        pass
    return job, True, None


# --------------------------------------------------------------------------
# 界面
# --------------------------------------------------------------------------

def display_width(text):
    """中文等全角字符占两列，需按显示宽度对齐。"""
    return sum(2 if unicodedata.east_asian_width(c) in "WF" else 1 for c in text)


def pad(text, width, align="left"):
    fill = " " * max(0, width - display_width(text))
    return fill + text if align == "right" else text + fill


def print_table(ready, unpaired, conflicts, outdir):
    headers = [t("col_idx"), t("col_part"), t("col_dat"), t("col_tl")]
    rows = [
        (str(i), job.part, job.br_name, job.tl_name)
        for i, job in enumerate(ready, 1)
    ]
    widths = [
        max(display_width(headers[c]), *(display_width(r[c]) for r in rows))
        for c in range(4)
    ]

    print(t("found", n=len(ready)))
    print()
    print("  " + "  ".join(pad(headers[c], widths[c]) for c in range(4)).rstrip())
    for row in rows:
        cells = [pad(row[0], widths[0], "right")] + [pad(row[c], widths[c]) for c in range(1, 4)]
        print("  " + "  ".join(cells).rstrip())
    print()

    existing = sum(
        1 for job in ready if os.path.exists(os.path.join(outdir, job.part + ".img"))
    )
    if existing:
        print(t("overwrite", n=existing))

    if unpaired:
        print(t("unpaired_head"))
        for name, missing in unpaired:
            print(t("unpaired_item", file=name, missing=missing))
        print()

    if conflicts:
        print(t("conflict_head"))
        for part, names in conflicts:
            print(t("conflict_group", part=part))
            for name in names:
                print(t("conflict_item", file=name))
        print()

    print(t("outdir", path=outdir))


def confirm():
    try:
        answer = input(t("confirm")).strip().lower()
    except (EOFError, KeyboardInterrupt):
        print()
        return False
    return answer in ("y", "yes")


# --------------------------------------------------------------------------
# 入口
# --------------------------------------------------------------------------

def _preparse_lang(argv):
    """argparse 的帮助文本同样需要翻译，因此须先于 parser 确定语言。

    这里必须自行校验取值：本函数跑在 argparse 之前，若把无效值（如 --lang xx）
    直接交给 MESSAGES 会 KeyError 抛 traceback，而正确的行为是由 argparse
    在解析阶段报 "invalid choice"。
    """
    for i, arg in enumerate(argv):
        if arg == "--lang" and i + 1 < len(argv):
            value = argv[i + 1]
        elif arg.startswith("--lang="):
            value = arg.split("=", 1)[1]
        else:
            continue
        return value if value in ("zh", "en") else "auto"
    return "auto"


def build_parser():
    localize_argparse()
    parser = argparse.ArgumentParser(
        prog="br2dat2img.py",
        description=t("banner", v=VERSION),
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    parser.add_argument("-i", "--indir", default=".", metavar="DIR", help=t("arg_indir"))
    parser.add_argument("-o", "--outdir", default="out", metavar="DIR", help=t("arg_outdir"))
    parser.add_argument("-p", "--partition", nargs="+", metavar="NAME", help=t("arg_partition"))
    parser.add_argument("--lang", choices=["auto", "zh", "en"], default="auto", help=t("arg_lang"))
    parser.add_argument("-j", "--jobs", type=int, default=0, metavar="N", help=t("arg_jobs"))
    parser.add_argument("-y", "--yes", action="store_true", help=t("arg_yes"))
    parser.add_argument("--version", action="version", version="%(prog)s " + VERSION,
                        help=t("arg_version"))
    return parser


def main(argv=None):
    global LANG

    # stdout 重定向到管道/文件时默认为块缓冲，stderr 则无缓冲，
    # 错误信息会先于 banner 输出。此处强制使用行缓冲。
    try:
        sys.stdout.reconfigure(line_buffering=True)
    except (AttributeError, ValueError):
        pass

    argv = list(sys.argv[1:] if argv is None else argv)
    initial = _preparse_lang(argv)
    LANG = detect_lang() if initial == "auto" else initial

    args = build_parser().parse_args(argv)

    indir = os.path.abspath(args.indir)
    outdir = os.path.abspath(args.outdir)
    tmpdir = os.path.join(outdir, ".tmp")

    print(t("banner", v=VERSION))
    print()

    if not os.path.isdir(indir):
        print(t("indir_missing", path=indir), file=sys.stderr)
        return 2

    # 依赖先于扫描检查：缺少依赖时无需扫描，也避免用户在确认后才发现无法执行
    if sdat2img is None:
        print(t("sdat2img_missing"), file=sys.stderr)
        return 2

    if brotli is None:
        print(t("brotli_missing"), file=sys.stderr)
        return 2

    print(t("scanning", path=indir))
    scanned, unpaired, conflicts = scan(indir)

    # -p 只挑选指定分区；未指定时就是扫到的全部
    ready = scanned
    if args.partition:
        ready, unknown = match_jobs(args.partition, scanned)
        if unknown:
            available = ", ".join(job.part for job in scanned) or t("part_none_available")
            print(t("part_not_found", names=", ".join(unknown), available=available),
                  file=sys.stderr)
            # 用户指定的名字可能对应一个文件不完整的分区，那样他也看不到确认表，
            # 所以这里单独说明，免得他以为是自己名字打错了
            incomplete = []
            for name, _ in unpaired:
                part = partition_of(name)
                if name in unknown or part in unknown:
                    incomplete.append(part or name)
            if incomplete:
                print(t("part_incomplete", names=", ".join(incomplete)), file=sys.stderr)
            return 2

    if not ready:
        if unpaired or conflicts:
            # 有文件但全都没配上对，把原因摊开，别只说一句"没找到"
            print_table([], unpaired, conflicts, outdir)
        print(t("no_sources", suffix=BR_SUFFIX.lstrip(".")))
        return 1

    print()
    print_table(ready, unpaired, conflicts, outdir)
    print()

    if not args.yes and not confirm():
        print(t("aborted"))
        return 1

    # 确认后才写入文件系统，取消时不产生任何改动
    os.makedirs(tmpdir, exist_ok=True)

    workers = args.jobs if args.jobs > 0 else (os.cpu_count() or 1)
    workers = max(1, min(workers, len(ready)))

    succeeded, failed = [], []

    # 阶段 1：并行解压。brotli 是 CPU 密集的独立子进程，并行收益最大。
    print(t("stage1_head", n=len(ready), w=workers))
    decompressed = []
    with ThreadPoolExecutor(max_workers=workers) as pool:
        futures = [pool.submit(decompress_one, job, indir, tmpdir) for job in ready]
        for future in as_completed(futures):
            job, ok, err = future.result()
            if ok:
                decompressed.append(job)
                print(t("stage1_ok", part=job.part))
            else:
                failed.append((job, err))
                print(t("fail_item", part=job.part, err=err))

    # 阶段 2：串行还原，理由见 convert_one 的注释。
    if decompressed:
        print()
        print(t("stage2_head"))
        for i, job in enumerate(decompressed, 1):
            print(t("stage2_item", i=i, n=len(decompressed), part=job.part))
            done_job, ok, err = convert_one(job, indir, outdir, tmpdir)
            if ok:
                succeeded.append(done_job)
                print(t("ok_item", part=done_job.part,
                        path=os.path.relpath(os.path.join(outdir, done_job.part + ".img"))))
            else:
                failed.append((done_job, err))
                print(t("fail_item", part=done_job.part, err=err))

    print()
    print(t("summary", ok=len(succeeded), fail=len(failed), skip=len(unpaired) + len(conflicts)))

    if failed:
        print()
        print(t("fail_head"))
        for job, err in failed:
            print(t("fail_item", part=job.part, err=err))
        print(t("tmp_hint", path=os.path.relpath(tmpdir)))

    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
