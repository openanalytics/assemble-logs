# assemble_logs

# Building

To build, you need to have libjq/jq installed, and set 
```
export JQ_LIB_DIR=/usr/lib/libjq.so
```

# Usage examples

Show only logs within a 30 minutes timespan.
```
assemble_logs all.log '.ts > "2021-09-02T22" and .ts < "2021-09-02T22:30"'  | less -R
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
