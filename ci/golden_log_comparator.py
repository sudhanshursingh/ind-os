#!/usr/bin/env python3
"""
Golden Log Comparator for Ind OS Test Harness (Sprint SP-03).
Compares captured serial output against golden references or structured patterns.
Supports:
1. Exact string matches.
2. Ordered regex pattern sequences (e.g., matching git commit hashes, boot timestamps).
3. Loud unified diff generation upon deviation.
"""

import sys
import re
import difflib

class ComparisonResult:
    def __init__(self, passed: bool, diff: str = "", message: str = ""):
        self.passed = passed
        self.diff = diff
        self.message = message

def compare_logs(actual_lines: list[str], expected_patterns: list[str], ordered: bool = True) -> ComparisonResult:
    """
    Compares actual serial log lines against expected patterns.
    Each expected pattern can be:
    - An exact substring/line
    - A regex pattern if prefixed with `re:`
    """
    if not ordered:
        # Unordered check: all expected patterns must be present
        missing = []
        for pat in expected_patterns:
            found = False
            is_regex = pat.startswith("re:")
            regex_str = pat[3:] if is_regex else None

            for line in actual_lines:
                if is_regex and re.search(regex_str, line):
                    found = True
                    break
                elif not is_regex and pat in line:
                    found = True
                    break
            if not found:
                missing.append(pat)

        if missing:
            diff_lines = [f"- Expected pattern not found: {p}" for p in missing]
            return ComparisonResult(
                passed=False,
                diff="\n".join(diff_lines),
                message=f"Missing {len(missing)} expected pattern(s) in serial output."
            )
        return ComparisonResult(passed=True, message="All unordered patterns matched.")

    # Ordered check: patterns must match in the given sequence
    actual_idx = 0
    num_actual = len(actual_lines)
    mismatches = []

    for pat_idx, pat in enumerate(expected_patterns):
        is_regex = pat.startswith("re:")
        regex_str = pat[3:] if is_regex else None
        matched = False

        while actual_idx < num_actual:
            line = actual_lines[actual_idx].strip()
            actual_idx += 1
            if is_regex and re.search(regex_str, line):
                matched = True
                break
            elif not is_regex and pat.strip() in line:
                matched = True
                break

        if not matched:
            mismatches.append((pat_idx, pat))
            break

    if mismatches:
        pat_idx, pat = mismatches[0]
        # Generate unified diff for visibility
        clean_actual = [l.strip() for l in actual_lines]
        clean_expected = [p[3:] if p.startswith("re:") else p for p in expected_patterns]
        diff = "\n".join(difflib.unified_diff(
            clean_expected,
            clean_actual,
            fromfile="expected_golden_log",
            tofile="actual_serial_output",
            lineterm=""
        ))
        return ComparisonResult(
            passed=False,
            diff=diff,
            message=f"Pattern #{pat_idx + 1} '{pat}' not satisfied in serial sequence."
        )

    return ComparisonResult(passed=True, message="All ordered patterns matched perfectly.")

def main():
    if len(sys.argv) < 3:
        print("Usage: golden_log_comparator.py <actual_log_path> <golden_spec_path>")
        sys.exit(2)

    with open(sys.argv[1], "r", encoding="utf-8") as af:
        actual_lines = af.readlines()

    with open(sys.argv[2], "r", encoding="utf-8") as ef:
        expected_patterns = [line.rstrip("\n") for line in ef if line.strip() and not line.startswith("#")]

    res = compare_logs(actual_lines, expected_patterns)
    if res.passed:
        print("✅ PASS: Golden log comparison succeeded.")
        sys.exit(0)
    else:
        print(f"❌ FAIL: {res.message}")
        print("\n--- Unified Diff ---")
        print(res.diff)
        sys.exit(1)

if __name__ == "__main__":
    main()
