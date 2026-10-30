# Holiday data

`holidays.tsv.gz` holds the holidays of every country and subdivision that [python-holidays](https://github.com/vacanza/holidays) knows, 2006–2051, and is embedded in pChart. It is generated; do not edit it by hand.

- Source: python-holidays 0.107, MIT. India is only available up to 2035.
- Lines: `REGION<TAB>YYYY-MM-DD<TAB>Name`. A country code holds the holidays of the whole country; a subdivision (`DE-BE`) holds only how it differs, and an empty name marks a national holiday it does not keep.
- 721 regions, 212,728 lines, 7.4 MB unpacked.

Rebuild it with the same version, and the file comes out byte for byte the same:

```sh
python -m venv /tmp/holidays-venv
/tmp/holidays-venv/bin/pip install holidays==0.107
/tmp/holidays-venv/bin/python -I tools/holidays/generate.py 2006 2051 /tmp/holidays.tsv
gzip -9 -n -c /tmp/holidays.tsv > data/holidays.tsv.gz
```
