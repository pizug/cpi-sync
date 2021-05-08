# Changelog

All notable changes to this project will be documented in this file.

## [Ideas - not implemented]

## [Unreleased]

## [0.3.0] - 2021-05-08

- Add: `download_worker_count` config option, and concurrent downloads feature for faster operation
- Add: Print elapsed time
- Add: Value Mapping Download
- Fix: Relative path normalization
- Add: Using absolute paths for `local_dir` config option.
- Consider: Gzip Support not implemented (since most of the data comes as zip)

## [0.2.1] - 2021-03-12

- Fix: reported path issues
- Add: `prop_comment_removal` option to remove auto-generated timestamp comments in `parameters.prop`
- Add: Remove local package content before sync
- Add: musl libc binary for Alpine docker images

## [0.2.0] - 2021-02-06

- Config change: change package rules, add regex, include/exclude operation
- Config change: zip extraction option, local_dir for all packages
- Config removal: local_dir for a single package removed, will wait for more feedback
- Fix: long path issue on Windows
- Better error reporting: print HTTP status and response body in case of error
- Suggest Package ID if it is not found and Package Name exists.

## [0.1.0] - 2021-01-31

- First release
