# Changelog

## Unreleased

## [v0.3.0]

This version introduces backwards incompatibility. The ID field is removed
from input and from output. Now the verification and resolution of name-strings
does not include 'Supplied ID'

- Add [#11]: Make finding the name-string field more flexible.
             Remove the supplied ID from input and results.
- Add [#10]: PDF guide for Windows users

## [v0.2.1]

- Add [#9]: docs and tests
- Add [#8]: logging info

## [v0.2.0]

- Add [#7]: Parallell execution of the queries, ability to work
            with filles of any size.

## [v0.1.1]

- Fix: make compatible to comiple on Windows

## [v0.1.0]

- Add [#6]: work with stdin as input if no other input is given.
- Add [#5]: flag for ignoring best match result, leaving preferred matches only.
- Add [#4]: output formats as CSV, JSON pretty, JSON compact.
- Add [#3]: output verification results as JSON.
- Add [#2]: remote verification of names from input or a file.
- Add [#1]: command line interface with clap.

## Footnotes

This document follows [changelog guidelines]

[v0.3.0]: https://github.com/gnames/gnfinder/compare/v0.2.1...v0.3.0
[v0.2.1]: https://github.com/gnames/gnfinder/compare/v0.2.0...v0.2.1
[v0.2.0]: https://github.com/gnames/gnfinder/compare/v0.1.1...v0.2.0
[v0.1.1]: https://github.com/gnames/gnfinder/compare/v0.1.0...v0.1.1
[v0.1.0]: https://github.com/gnames/gnfinder/tree/v0.1.0

[#11]: https://github.com/gnames/gnverify/issues/11
[#10]: https://github.com/gnames/gnverify/issues/10
[#9]: https://github.com/gnames/gnverify/issues/9
[#8]: https://github.com/gnames/gnverify/issues/8
[#7]: https://github.com/gnames/gnverify/issues/7
[#6]: https://github.com/gnames/gnverify/issues/6
[#5]: https://github.com/gnames/gnverify/issues/5
[#4]: https://github.com/gnames/gnverify/issues/4
[#3]: https://github.com/gnames/gnverify/issues/3
[#2]: https://github.com/gnames/gnverify/issues/2
[#1]: https://github.com/gnames/gnverify/issues/1

[changelog guidelines]: https://github.com/olivierlacan/keep-a-changelog
