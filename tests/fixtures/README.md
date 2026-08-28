# Test Fixtures

This directory is designated for synthetic, minimal, and anonymized HEIC/HEIF container fixtures.

## Guidelines for Adding Fixtures

1. **Privacy & Anonymity**:
   - Never commit real personal photos, sensitive metadata, or proprietary/confidential images (e.g., ExamMint production files).
   - Only synthetic or sanitized test samples should be added here.

2. **File Size**:
   - Keep fixture files as small as possible (ideally synthetic headers or cropped minimal test cases under a few kilobytes).

3. **Regression Cases**:
   - When fixing edge-case parser bugs, create a minimal stripped reproducible sample and pair it with a test case in `tests/regression/`.
