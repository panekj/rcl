#!/usr/bin/env python3

# RCL -- A sane configuration language.
# Copyright 2023 Ruud van Asseldonk

# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# A copy of the License has been included in the root of the repository.

from pygments.lexer import RegexLexer, words
from pygments import token


class RclLexer(RegexLexer):
    """
    This lexer should mirror the lexer in src/lexer.rs.
    """

    name = "RCL"
    aliases = ["rcl"]
    filenames = ["*.rcl"]

    tokens = {
        # The root state here corresponds to `Lexer::next` in Rust.
        "root": [
            (r"#!.*?$", token.Comment.Hashbang),
            (r"//.*?$", token.Comment),
            (r'f"""', token.String, "format_triple_open"),
            (r'"""', token.String, "triple_open"),
            (r'f"', token.String, "format_double_open"),
            (r'"', token.String, "double_open"),
            # TODO: Handle the }
            (r"0b[01_]+", token.Number.Bin),
            (r"0x[0-9a-fA-F_]+", token.Number.Hex),
            (r"[0-9_]+(\.[0-9_]+)?([eE][+-]?[0-9_]+)?", token.Number),
            # In the Rust lexer, there is one state for identifiers, and in
            # there we recognize the keywords. We don't recognize builtins at
            # all in the Rust lexer. Here, we do split all those out by token
            # type.
            (
                words(
                    (
                        "and",
                        "else",
                        "false",
                        "for",
                        "if",
                        "in",
                        "let",
                        "not",
                        "null",
                        "or",
                        "then",
                        "true",
                    ),
                    suffix=r"\b",
                ),
                token.Keyword,
            ),
            (
                words(
                    (
                        "contains",
                        "get",
                        "len",
                    ),
                    suffix=r"\b",
                ),
                token.Name.Builtin,
            ),
            (r"[_a-z][_a-z0-9-]*", token.Name),
            (r"\s+", token.Whitespace),
            # In the Rust lexer the punctuation is split out, and then further
            # into digraphs and monographs. Here we instead split them out by
            # token type.
            (r"<=|>=|==|!=|<|>|\+|-|\*|/|\|", token.Operator),
            (r"[)(}{\]\[=,.:;]", token.Token),
            (r"#", token.Error),
        ],
        "format_double_open": [],
        "format_triple_open": [],
        "double_open": [],
        "triple_open": [],
    }
