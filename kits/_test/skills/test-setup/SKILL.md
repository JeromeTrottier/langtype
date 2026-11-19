---
name: test-setup
description: Use when setting up a new knowledge base from the minimal test kit. Creates one note and one counter as a draft.
---
1. Call `start_session`, then `describe_kit`.
2. Call `propose_setup` with one `note` ("Welcome") and one `counter` ("visits", value 0).
3. Show the preview to the user and call `apply_draft` when they accept.
4. Call `end_session` with a one-line recap.
