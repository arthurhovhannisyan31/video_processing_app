---
name: docs-maintenance
description: Update project modules documentation
argument-hint: Which module documentation should be updated?
---

If the user passed arguments, treat them as target modules for documentation update.

If the user didn't pass arguments update documentation for following modules:

- Project readme: ./README.md
- Project docs: ./docs/*
- Backend: ./modules/backend/README.md
- Frontend: modules/frontend/README.md