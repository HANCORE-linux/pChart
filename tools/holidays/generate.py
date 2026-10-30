"""Write the holiday data pChart embeds, from python-holidays.

Lines are REGION<TAB>YYYY-MM-DD<TAB>Name. A country code holds the holidays
of the whole country; a subdivision (DE-BE) holds only how it differs: the
days it adds, and with an empty name the national days it does not keep.
Three-letter aliases (DEU for DE) are skipped.

Usage: python -I generate.py FIRST_YEAR LAST_YEAR OUTPUT [REGION ...]
"""

import sys

import holidays

ENGLISH = ("en_US", "en_GB", "en")


def language(cls):
    for code in ENGLISH:
        if code in getattr(cls, "supported_languages", ()):
            return code
    return None


def days(code, subdiv, years, lang):
    found = holidays.country_holidays(code, subdiv=subdiv, years=years, language=lang)
    return {day: name for day, name in found.items()}


def main():
    first, last, output = int(sys.argv[1]), int(sys.argv[2]), sys.argv[3]
    only = set(sys.argv[4:])
    years = range(first, last + 1)
    lines = []
    skipped = 0
    for code, subdivs in sorted(holidays.list_supported_countries().items()):
        if len(code) != 2:
            continue
        if only and code not in only and not any(r.startswith(code + "-") for r in only):
            continue
        cls = getattr(holidays, code)
        lang = language(cls)
        national = days(code, None, years, lang)
        if not only or code in only:
            lines += [f"{code}\t{day}\t{name}" for day, name in sorted(national.items())]
        for sub in subdivs:
            region = f"{code}-{sub}"
            if only and region not in only:
                continue
            local = days(code, sub, years, lang)
            lines += [f"{region}\t{day}\t{name}" for day, name in sorted(local.items()) if national.get(day) != name]
            gaps = sorted(day for day in national if day not in local)
            lines += [f"{region}\t{day}\t" for day in gaps]
            skipped += len(gaps)
    with open(output, "w", encoding="utf-8") as out:
        out.write("\n".join(lines) + "\n")
    print(f"{len(lines)} lines, {skipped} of them national days a subdivision does not keep", file=sys.stderr)


if __name__ == "__main__":
    main()
