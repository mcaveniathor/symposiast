#!/usr/bin/env python3
"""Check a generated static site without fetching external URLs."""

import argparse
from collections import Counter
from dataclasses import dataclass, field
from html.parser import HTMLParser
from pathlib import Path, PurePosixPath
import re
import sys
from urllib.parse import unquote, urlsplit


@dataclass
class Reference:
    source: Path
    line: int
    url: str


@dataclass
class Document:
    ids: set[str] = field(default_factory=set)
    anchors: set[str] = field(default_factory=set)
    references: list[Reference] = field(default_factory=list)
    errors: list[tuple[int, str]] = field(default_factory=list)


CSS_URL = re.compile(r"url\(\s*(?:\"([^\"]*)\"|'([^']*)'|([^\s)]*))\s*\)", re.I)
CSS_IMPORT = re.compile(r"@import\s+(?:\"([^\"]*)\"|'([^']*)')", re.I)
CSS_COMMENT = re.compile(r"/\*.*?\*/", re.S)


def css_references(source: Path, css: str, first_line: int = 1) -> list[Reference]:
    # Preserve offsets and line numbers while discarding commented-out URLs.
    css = CSS_COMMENT.sub(lambda match: re.sub(r"[^\n]", " ", match.group()), css)
    references = []
    for pattern in (CSS_URL, CSS_IMPORT):
        for match in pattern.finditer(css):
            url = next(value for value in match.groups() if value is not None)
            line = first_line + css.count("\n", 0, match.start())
            references.append(Reference(source, line, url))
    return references


class PageParser(HTMLParser):
    def __init__(self, source: Path):
        super().__init__(convert_charrefs=True)
        self.source = source
        self.document = Document()
        self.counts: Counter[str] = Counter()
        self.lang = ""
        self.in_title = False
        self.title = []
        self.in_style = False
        self.style_line = 1
        self.style = []

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        line = self.getpos()[0]
        self.counts[tag] += 1
        if tag == "html":
            self.lang = (attributes.get("lang") or "").strip()
        elif tag == "title":
            self.in_title = True
        elif tag == "style":
            self.in_style = True
            self.style_line = line
            self.style = []
        elif tag == "base":
            self.document.errors.append((line, "<base> is unsupported; use explicit site-relative URLs"))
        identifier = attributes.get("id")
        if identifier is not None:
            if identifier in self.document.ids:
                self.document.errors.append((line, f"duplicate id {identifier!r}"))
            self.document.ids.add(identifier)
            self.document.anchors.add(identifier)
        if tag == "a" and attributes.get("name"):
            self.document.anchors.add(attributes["name"])
        for name in ("href", "src", "poster"):
            if attributes.get(name) is not None:
                self.document.references.append(Reference(self.source, line, attributes[name]))
        if attributes.get("style"):
            self.document.references.extend(css_references(self.source, attributes["style"], line))

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)
        self.handle_endtag(tag)

    def handle_endtag(self, tag):
        if tag == "title":
            self.in_title = False
        elif tag == "style" and self.in_style:
            self.document.references.extend(css_references(self.source, "".join(self.style), self.style_line))
            self.in_style = False

    def handle_data(self, data):
        if self.in_title:
            self.title.append(data)
        if self.in_style:
            self.style.append(data)

    def finish(self):
        self.close()
        for tag in ("html", "title", "main", "h1"):
            if self.counts[tag] != 1:
                self.document.errors.append((1, f"expected one <{tag}>, found {self.counts[tag]}"))
        if not self.lang:
            self.document.errors.append((1, "<html> needs a nonempty lang attribute"))
        if not "".join(self.title).strip():
            self.document.errors.append((1, "<title> must not be empty"))
        return self.document


def local_target(root: Path, base: str, reference: Reference) -> tuple[Path, str] | None:
    url = reference.url.strip()
    if not url:
        return None
    parsed = urlsplit(url)
    # External, protocol-relative, mail, data, and other scheme URLs are not fetched.
    if parsed.scheme or parsed.netloc:
        return None
    path = unquote(parsed.path, errors="strict")
    fragment = unquote(parsed.fragment, errors="strict").split(":~:", 1)[0]
    if "\\" in path or "\x00" in path:
        raise ValueError("local URL contains a backslash or NUL")
    if path.startswith("/"):
        if path == base.rstrip("/"):
            relative = ""
        elif path.startswith(base):
            relative = path[len(base):]
        else:
            raise ValueError(f"local URL is outside the configured base {base!r}")
        parts = []
    elif path:
        relative = path
        parts = list(reference.source.relative_to(root).parent.parts)
    else:
        return reference.source, fragment
    for part in PurePosixPath(relative).parts:
        if part in ("", "."):
            continue
        if part == "..":
            if not parts:
                raise ValueError("local URL escapes the output root")
            parts.pop()
        else:
            parts.append(part)
    target = root.joinpath(*parts)
    if not target.resolve().is_relative_to(root):
        raise ValueError("local URL resolves outside the output root")
    if target.is_dir():
        target = target / "index.html"
    if not target.resolve().is_relative_to(root):
        raise ValueError("local URL resolves outside the output root")
    if not target.is_file():
        raise ValueError(f"missing local target {target.relative_to(root).as_posix()!r}")
    return target, fragment


def check(root: Path, base: str) -> int:
    documents = {}
    references = []
    errors = []
    for source in sorted(root.rglob("*")):
        if not source.is_file():
            continue
        suffix = source.suffix.lower()
        if suffix not in (".html", ".htm", ".css"):
            continue
        if not source.resolve().is_relative_to(root):
            errors.append((source, 1, "file resolves outside the output root"))
            continue
        try:
            content = source.read_text(encoding="utf-8")
            if suffix in (".html", ".htm"):
                parser = PageParser(source)
                parser.feed(content)
                document = parser.finish()
                documents[source.resolve()] = document
                references.extend(document.references)
                errors.extend((source, line, message) for line, message in document.errors)
            else:
                references.extend(css_references(source, content))
        except (OSError, UnicodeError, ValueError) as error:
            errors.append((source, 1, f"could not read or parse file: {error}"))
    if not documents:
        errors.append((root, 1, "no HTML pages found"))
    local_count = 0
    for reference in references:
        try:
            result = local_target(root, base, reference)
            if result is None:
                continue
            local_count += 1
            target, fragment = result
            document = documents.get(target.resolve())
            if fragment and document is not None and fragment not in document.anchors:
                raise ValueError(f"missing anchor {fragment!r} in {target.relative_to(root).as_posix()!r}")
        except (OSError, UnicodeError, ValueError) as error:
            errors.append((reference.source, reference.line, f"{reference.url!r}: {error}"))
    for source, line, message in errors:
        label = source.relative_to(root).as_posix()
        print(f"{label}:{line}: {message}", file=sys.stderr)
    if errors:
        print(f"Site checks failed: {len(errors)} error(s).", file=sys.stderr)
        return 1
    print(f"Site checks passed: {len(documents)} HTML pages, {local_count} local references, base {base}")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, help="generated site output directory")
    parser.add_argument("--base", default="/", help="deployment URL path, such as /symposiast/")
    args = parser.parse_args()
    root = args.directory.resolve()
    if not root.is_dir():
        parser.error(f"output directory does not exist: {args.directory}")
    parsed_base = urlsplit(args.base)
    if (not args.base.startswith("/") or args.base.startswith("//") or parsed_base.scheme
            or parsed_base.netloc or parsed_base.query or parsed_base.fragment
            or any(part in (".", "..") for part in args.base.split("/"))):
        parser.error("--base must be an absolute URL path without query, fragment, or dot segments")
    base = args.base.rstrip("/") + "/"
    return check(root, base)


if __name__ == "__main__":
    sys.exit(main())
