# assemble-logs

Given log files that contain JSON records and are rotated with suffixes using [`file-rotate`](https://crates.io/crates/file-rotate):
- Assemble the contents of log files in chronological order
- Filter records based on `jq`-syntax query
- Format remaining records for terminal viewing

## Log record format
This crate is made to work with the [`file-rotate`](https://crates.io/crates/file-rotate) crate and `slog-json`.
It assumes that slog records are written as JSON, and files rotated with `file-rotate`.
Example log record:
```
{"msg":"Start poll worker","level":"DEBG","ts":"2022-01-31T15:20:20.637","tag":"poller","worker":"http://localhost:8002/","poller":20166}
```
So it has the keys `"msg", "level", "ts", "tag"`. The rest of the keys are key-value pairs from slog (which can obviously also be used in your `jq` filter).

## File suffix format
We currently use a hard-coded suffix scheme:
```
let suffix_scheme = TimestampSuffixScheme::default(FileLimit::Age(Duration::weeks(1)));
```

This implies the default timestamp format which is `%Y%m%dT%H%M%S`. (the `FileLimit` above does not matter at all).

There is no reason this has to be hard-coded and can be parameterized in future work.

# Building

To build, you need to have libjq/jq installed, and set 
```
export JQ_LIB_DIR=/usr/lib/libjq.so
```

# Usage
The main work so far has went into the `assemble` subcommand

```
$ assemble-logs assemble -h
assemble-logs-assemble

USAGE:
    assemble-logs assemble [FLAGS] [OPTIONS] <LOG_PATH> [JQ]

ARGS:
    <LOG_PATH>    The path to the main log file
    <JQ>          JQ query; must return a bool; only used for filtering

FLAGS:
    -c, --compact          Compact - don't print newline on each key-value
    -e, --error-details    Print error details (default is without details)
    -h, --help             Print help information
    -n, --no-format        No formatting: just print json
    -V, --version          Print version information

OPTIONS:
    -a, --after <AFTER>
            Any prefix of a timestamp in the format "%Y%m%dT%H%M%S" The system will already filter
            out files that have a lexically older stimestamp. This will also be used to filter
            records, so you don't necessarily have to use the `jq` filter for that

        --jq-transformation <JQ_TRANSFORMATION>
            Only effective with --no-format
```


# jq examples

Show only logs within a 30 minutes timespan.
```
assemble-logs assemble logs/all.log '.ts > "2021-09-02T22" and .ts < "2021-09-02T22:30"'  | less -r
```

Filter by (short) log level ("CRIT", "ERRO", "WARN", "INFO", "DEBG", "TRCE"):
```
'.level == "WARN" or level == "ERRO"'
```

Show only output with a specific tag

```
'.tag == "poller"'
```

Everything actix-related (e.g. all incoming requests):
```
'.tag | startswith("actix")'
```

# -a

The `-a/--after` argument may seem superfluous since we can do the same with a filter on `.ts` as seen above.
However, it is difficult to extract the `.ts` part of the filter in order to filter out _files_ (as opposed to _records_).
Much time can be saved by filtering out entire files so that they need not be decompressed and their contents processed.
`-a` should thus be used instead of a jq filter when you want to filter out records before the given timestamp.
This will apply the same filter on individual records.


```
$ assemble-logs assemble logs/all.log -a 2022-01-31T12 | less -r
```

This outputs formatted records starting with:
```
Jan 31 12:00:00.151 DEBG [poller] Poll
        poller: 8160
        workers online: 1
Jan 31 12:00:00.151 DEBG [poller] Debug info
        in-flight analysis job submissions: 0
        poller: 8160
```

Future work could include a `-b/--before` argument
