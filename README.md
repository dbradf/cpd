# Copy/Paste detection

This tool is an experiment in copy/paste detection in source code. Instead of
tokenizing the code to attempt to find matching sequences of tokens, it relies
on the fact that most programming languages are line-oriented and provides
analysis based on lines.

The idea is to build up an index of group of lines in all the files being
analyzed. The size of the line groups is based on the minimum number of lines
needed to be considered a match. We then go through the index and any entries
with more than 1 entry are values that appear to be copy/pasted.

We can then post-process the index to do some merging of consecutive lines and
generate a report that provides results file by file.

## Dealing with false positives

The big challenge with this (and other copy/paste detection tools) is dealing
with false positives. There are a lot of patterns is code that tend to get
repeated frequently.

## Configuration

A configuration file can be provided to customize how the detection is performed.
The file should be in json and provide via the `--config-file` parameter.

It takes the following values:

```json
{
    "min_lines": 8,
    "ignored_globs": [
        "**/package.json",
        "**/*.lock"
    ]
}
```

* **min_lines**: Minimum number of lines to consider a block of lines a duplicate.
* **ignored_globs**: List of file globs that should be ignored when performing analysis.

## Usage

```bash
Copy/Paste detection for source code

Usage: cpd [OPTIONS]

Options:
      --report-file <REPORT_FILE>  File to write report to
      --base-dir <BASE_DIR>        Base directory to analyze from [default: .]
      --config-file <CONFIG_FILE>  Path to configuration file
  -h, --help                       Print help
  -V, --version                    Print version
```

## Inspirations

* [jspcd](https://github.com/kucherenko/jscpd) - Copy/paste detection written in javascript.
