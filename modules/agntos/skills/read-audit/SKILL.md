# Read audit log

To answer "why was X installed?" or similar:

1. Use `audit` with action `search` and query terms (package name, path, or phrase from the user).
2. Read `prompt` and `summary` fields in results.
3. Cite the audit entry id if the user may want rollback details.
